//! intent — Natural-language agent creation and feedback-based learning.
pub mod agent_builder;

use crate::core::storage::DecisionRule;

use crate::core::supervisor::Supervisor;

impl Supervisor {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn create_agent_from_nl_uses_six_step_understanding() {
        let responses = [
            r#"{"domain":"software engineering","name":"Code Review Agent"}"#,
            r#"{"role":"code reviewer","persona":"reviewer","name":"Review Agent"}"#,
            r#"{"capabilities":["inspect diffs","find bugs"],"skills":["code_review","test"]}"#,
            r#"{"tools":["read_file","exec_python"]}"#,
            r#"{"knowledge":["docs"],"memory":["working_memory","semantic_memory"]}"#,
            r#"{
                "model":"deepseek-chat",
                "communication_style":"technical",
                "persona_config":{
                    "parameters":{
                        "temperature":0.2,
                        "style":"technical",
                        "verbosity":0.4,
                        "proactiveness":0.5
                    },
                    "prompt_layers":{
                        "l1_core_identity":"You are a careful code reviewer.",
                        "l2_skill_instructions":"Inspect code for correctness and tests.",
                        "l3_format_template":"Return findings first.",
                        "l4_constraints":"Do not approve unsafe code.",
                        "l5_conversation_style":"Use concise technical language."
                    }
                }
            }"#,
        ];
        let call_count = AtomicUsize::new(0);
        let chat_fn = |prompt: &str, system: &str| {
            let idx = call_count.fetch_add(1, Ordering::SeqCst);
            assert!(system.contains("COO agent configuration planner"));
            assert!(prompt.contains(&format!("Step {}", idx + 1)));
            if idx > 0 {
                assert!(prompt.contains(&format!("Step {} result", idx)));
            }
            Ok(responses[idx].to_string())
        };

        let supervisor = Supervisor::new(None, None);
        let agent = supervisor
            .create_agent_from_nl("Create an agent that reviews Rust code", &chat_fn, None)
            .unwrap();

        assert_eq!(call_count.load(Ordering::SeqCst), 6);
        assert_eq!(agent.name, "Review Agent");
        assert_eq!(agent.persona, "reviewer");
        assert_eq!(agent.model, "deepseek-chat");
        assert_eq!(agent.tools, vec!["read_file", "exec_python"]);
        assert_eq!(agent.knowledge, vec!["docs"]);
        assert_eq!(agent.skills, vec!["code_review", "test"]);
        assert_eq!(agent.memory, vec!["working_memory", "semantic_memory"]);
        assert_eq!(agent.communication_style, "technical");
        assert_eq!(
            agent.persona_config.prompt_layers.l1_core_identity,
            "You are a careful code reviewer."
        );
    }
}