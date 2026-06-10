//! intent — Natural-language agent creation and feedback-based learning.
use crate::core::registry::Registry;
use crate::core::storage::DecisionRule;

use crate::core::supervisor::NLAgentDef;
use crate::core::supervisor::Supervisor;

impl Supervisor {
    /// Converts a natural-language agent request into an agent definition using the provided chat function.
    /// Queries the Registry for available components if provided.
    pub fn create_agent_from_nl(
        &self,
        nl: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        registry: Option<&Registry>,
    ) -> Result<NLAgentDef, String> {
        let system_prompt = "You are an agent configuration assistant. Analyze the user's natural language description and return a JSON object with the agent definition. Only return valid JSON, no markdown, no explanation.";

        let (persona_list, model_list, tool_list, knowledge_list, skill_list) = if let Some(reg) = registry {
            let templates = reg.list_templates();
            let personas: Vec<&str> = templates.iter().map(|t| t.persona.as_str()).collect();
            let models: Vec<&str> = templates.iter().map(|t| t.model.as_str()).collect();
            let tools: Vec<&str> = templates.iter().flat_map(|t| t.tools.iter().map(|s| s.as_str())).collect();
            let knowledge: Vec<&str> = templates.iter().flat_map(|t| t.knowledge.iter().map(|s| s.as_str())).collect();
            let skills: Vec<&str> = templates.iter().flat_map(|t| t.skills.iter().map(|s| s.as_str())).collect();
            let mut unique_personas: Vec<&str> = personas.clone();
            unique_personas.sort();
            unique_personas.dedup();
            let mut unique_models: Vec<&str> = models.clone();
            unique_models.sort();
            unique_models.dedup();
            let mut unique_tools: Vec<&str> = tools.clone();
            unique_tools.sort();
            unique_tools.dedup();
            let mut unique_knowledge: Vec<&str> = knowledge.clone();
            unique_knowledge.sort();
            unique_knowledge.dedup();
            let mut unique_skills: Vec<&str> = skills.clone();
            unique_skills.sort();
            unique_skills.dedup();
            (unique_personas.join(", "), unique_models.join(", "), unique_tools.join(", "), unique_knowledge.join(", "), unique_skills.join(", "))
        } else {
            (
                ["assistant", "analyst", "researcher", "writer", "coder", "translator", "reviewer"].join(", "),
                ["deepseek-chat", "deepseek-reasoner"].join(", "),
                ["web_search", "read_file", "write_file", "exec_python", "calc", "get_time", "get_kline", "calc_macd", "chart"].join(", "),
                ["docs", "glossary", "data_sources"].join(", "),
                ["summarization", "translation", "code_review", "grammar_check", "format", "style", "proofread", "report_generation", "debug", "test"].join(", "),
            )
        };

        let prompt = format!(
            r#"User wants to create an agent. Analyze this description:
{}
Available personas: {}
Available models: {}
Available tools: {}
Available knowledge: {}
Available skills: {}

Return a JSON object with exactly these fields (all strings or string arrays):
{{
  "name": "short agent name (2-5 words)",
  "persona": "most appropriate persona from the list above",
  "model": "deepseek-chat",
  "tools": ["list", "of", "tool", "names"],
  "knowledge": ["list", "of", "knowledge", "sources"],
  "skills": ["list", "of", "skills"]
}}
Select tools, knowledge, and skills that best match the user's described use case."#,
            nl, persona_list, model_list, tool_list, knowledge_list, skill_list
        );

        let response = chat_fn(&prompt, system_prompt)?;

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str::<NLAgentDef>(cleaned).map_err(|e| {
            format!(
                "Failed to parse LLM response as AgentDef: {}. Raw: {}",
                e, cleaned
            )
        })
    }

    /// Learns a decision rule from user feedback and returns success when storage updates complete.
    pub fn learn_from_feedback(&mut self, user_input: &str, approved: bool) -> Result<(), String> {
        let user_id = self.user_id.as_deref().unwrap_or("default").to_string();
        let keywords = Self::extract_keywords(user_input);
        if keywords.is_empty() {
            return Ok(());
        }
        let keyword = keywords[0].clone();
        let level = self.decide_level(user_input).as_str().to_string();

        if let Some(ref storage) = self.storage {
            let existing = storage
                .get_decision_rules(&user_id, &keyword)
                .unwrap_or_default();
            if let Some(rule) = existing.first() {
                let change = if approved { -10.0 } else { 15.0 };
                if let Some(rule_id) = rule.id {
                    storage.adjust_rule_threshold(rule_id, change)?;
                }
            } else {
                let rule = DecisionRule {
                    id: None,
                    user_id: user_id.clone(),
                    keyword: keyword.clone(),
                    level,
                    trust_threshold: if approved { 50.0 } else { 75.0 },
                    auto_execute: approved,
                    source: "learned".to_string(),
                    hit_count: 1,
                    last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                    created_at: None,
                };
                storage.upsert_decision_rule(&rule)?;
            }
        }
        Ok(())
    }
}