//! decision — Evaluates supervisor decisions and required oversight levels.
mod context;
mod level;
mod weighted;

use super::{DecisionLevel, Supervisor};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Intent {
    pub intent_type: String,
    #[serde(default = "default_intent_complexity")]
    pub complexity: u8,
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub target_agent: String,
}

/// Parses user intent with an LLM and falls back to supervisor heuristics when the LLM is unavailable.
pub fn parse_with_llm(text: &str, llm: &dyn Fn(&str, &str) -> Result<String, String>) -> Intent {
    let system_prompt = "You are a COO intent parser. Return only valid JSON with fields: intent_type, complexity, required_tools, target_agent. intent_type must be one of direct_answer, single_tool, single_agent, team, workflow, jump_studio. complexity is an integer from 1 to 10. required_tools is an array of strings. target_agent is a short routing id.";
    let prompt = format!(
        r#"Parse this user request into routing intent:

{}

Return JSON only:
{{"intent_type":"single_agent","complexity":5,"required_tools":[],"target_agent":"general_agent"}}"#,
        text
    );

    match llm(&prompt, system_prompt) {
        Ok(response) => parse_intent_json(&response).unwrap_or_else(|| parse_intent_fallback(text)),
        Err(_) => parse_intent_fallback(text),
    }
}

impl Supervisor {
    /// Parses user intent using the provided LLM function, falling back to local routing rules.
    pub fn parse_with_llm(
        &self,
        text: &str,
        llm: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Intent {
        parse_with_llm(text, llm)
    }
}

fn default_intent_complexity() -> u8 {
    5
}

fn parse_intent_json(response: &str) -> Option<Intent> {
    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let mut intent = serde_json::from_str::<Intent>(cleaned).ok()?;
    intent.intent_type = normalize_intent_type(&intent.intent_type);
    intent.complexity = intent.complexity.clamp(1, 10);
    intent.required_tools = normalized_list(&intent.required_tools);
    intent.target_agent = intent.target_agent.trim().to_string();
    if intent.target_agent.is_empty() {
        intent.target_agent = target_agent_for_intent(&intent.intent_type).to_string();
    }
    Some(intent)
}

fn parse_intent_fallback(text: &str) -> Intent {
    let level = {
        let supervisor = Supervisor::new(None, None);
        supervisor.decide_level(text)
    };
    let intent_type = level.as_str().to_string();
    Intent {
        complexity: complexity_for_level(&level),
        required_tools: infer_required_tools(text),
        target_agent: target_agent_for_intent(&intent_type).to_string(),
        intent_type,
    }
}

pub(crate) fn normalize_intent_type(intent_type: &str) -> String {
    match intent_type.trim().to_lowercase().as_str() {
        "directanswer" | "direct_answer" | "answer" => "direct_answer",
        "toolcall" | "tool_call" | "single_tool" | "tool" => "single_tool",
        "agenttask" | "agent_task" | "single_agent" | "agent" => "single_agent",
        "teamtask" | "team_task" | "team" => "team",
        "workflowtemplate" | "workflow_template" | "workflow" => "workflow",
        "jumptostudio" | "jump_to_studio" | "jump_studio" | "studio" => "jump_studio",
        _ => "single_agent",
    }
    .to_string()
}

fn normalized_list(values: &[String]) -> Vec<String> {
    let mut normalized = values
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn infer_required_tools(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tools = Vec::new();
    for (keyword, tool) in [
        ("search", "web_search"),
        ("look up", "web_search"),
        ("find ", "web_search"),
        ("calculate", "calc"),
        ("compute", "calc"),
        ("convert", "converter"),
        ("translate", "translator"),
        ("read file", "read_file"),
        ("write file", "write_file"),
    ] {
        if lower.contains(keyword) {
            tools.push(tool.to_string());
        }
    }
    tools.sort();
    tools.dedup();
    tools
}

pub(crate) fn complexity_for_level(level: &DecisionLevel) -> u8 {
    match level {
        DecisionLevel::L1DirectAnswer => 1,
        DecisionLevel::L2SingleTool => 2,
        DecisionLevel::L3SingleAgent => 5,
        DecisionLevel::L4Team => 8,
        DecisionLevel::L5Workflow => 6,
        DecisionLevel::L6JumpToStudio => 9,
    }
}

fn target_agent_for_intent(intent_type: &str) -> &'static str {
    match intent_type {
        "direct_answer" => "assistant",
        "single_tool" => "tool_executor",
        "single_agent" => "general_agent",
        "team" => "agent_team",
        "workflow" => "workflow_runner",
        "jump_studio" => "studio",
        _ => "general_agent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::supervisor::OverrideScope;

    #[test]
    fn decide_level_classifies_direct_answer() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level("hello");

        assert_eq!(level, DecisionLevel::L1DirectAnswer);
    }

    #[test]
    fn decide_level_classifies_single_tool() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level("please search the web");

        assert_eq!(level, DecisionLevel::L2SingleTool);
    }

    #[test]
    fn decide_level_uses_default_single_agent() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level("explain this design");

        assert_eq!(level, DecisionLevel::L3SingleAgent);
    }

    #[test]
    fn decide_returns_reason_for_workflow() {
        let supervisor = Supervisor::new(None, None);

        let (level, reason) = supervisor.decide("create a research report");

        assert_eq!(level, DecisionLevel::L2SingleTool);
        assert!(reason.contains("tool"));
    }

    #[test]
    fn decide_with_rules_without_storage_falls_back_to_heuristics() {
        let supervisor = Supervisor::new(None, None);

        let (level, reason) = supervisor.decide_with_rules("calculate 1 + 1");

        assert_eq!(level, DecisionLevel::L2SingleTool);
        assert_eq!(reason, "Single tool operation needed");
    }

    #[test]
    fn decide_with_rules_prefers_session_override() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.override_decision(DecisionLevel::L5Workflow, OverrideScope::Session);

        let (level, reason) = supervisor.decide_with_rules("hello");

        assert_eq!(level, DecisionLevel::L5Workflow);
        assert!(reason.contains("override"));
    }

    #[test]
    fn extract_keywords_filters_stop_words_and_duplicates() {
        let keywords = Supervisor::extract_keywords("hello hello 的 deploy!");

        assert_eq!(keywords, vec!["deploy".to_string(), "hello".to_string()]);
    }

    #[test]
    fn decide_level_with_context_downgrades_simple_advanced_task() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level_with_context("create a report", Some(true), Some(1));

        assert_eq!(level, DecisionLevel::L2SingleTool);
    }

    #[test]
    fn decide_level_with_context_upgrades_complex_low_task() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level_with_context("hello", Some(true), Some(9));

        assert_eq!(level, DecisionLevel::L4Team);
    }

    #[test]
    fn decide_level_with_context_keeps_mid_complexity_base() {
        let supervisor = Supervisor::new(None, None);

        let level = supervisor.decide_level_with_context("explain this design", None, Some(5));

        assert_eq!(level, DecisionLevel::L3SingleAgent);
    }

    #[test]
    fn decide_level_with_context_records_learning_outcome() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage.clone()), None);

        let level = supervisor.decide_level_with_context("hello deploy", Some(true), Some(9));

        assert_eq!(level, DecisionLevel::L4Team);
        let rules = storage.get_decision_rules("default", "deploy").unwrap();
        assert_eq!(rules[0].level, "team");
    }

    #[test]
    fn test_coo_override_data_team() {
        let supervisor = Supervisor::new(None, None);
        let level = supervisor.decide_level("use data team to analyze this");
        assert_eq!(level, DecisionLevel::L4Team);
    }

    #[test]
    fn test_coo_override_direct_answer() {
        let supervisor = Supervisor::new(None, None);
        let level = supervisor.decide_level("直接回答这个问题");
        assert_eq!(level, DecisionLevel::L1DirectAnswer);
    }

    #[test]
    fn test_coo_override_just_answer() {
        let supervisor = Supervisor::new(None, None);
        let level = supervisor.decide_level("just answer the question");
        assert_eq!(level, DecisionLevel::L1DirectAnswer);
    }

    #[test]
    fn parse_with_llm_returns_structured_intent() {
        let llm = |_prompt: &str, _system: &str| {
            Ok(r#"{"intent_type":"ToolCall","complexity":3,"required_tools":["web_search","web_search"],"target_agent":"researcher"}"#.to_string())
        };

        let intent = parse_with_llm("look up rust releases", &llm);

        assert_eq!(intent.intent_type, "single_tool");
        assert_eq!(intent.complexity, 3);
        assert_eq!(intent.required_tools, vec!["web_search"]);
        assert_eq!(intent.target_agent, "researcher");
    }

    #[test]
    fn parse_with_llm_falls_back_to_rules() {
        let llm = |_prompt: &str, _system: &str| Err("offline".to_string());

        let intent = parse_with_llm("calculate 2 + 2", &llm);

        assert_eq!(intent.intent_type, "single_tool");
        assert_eq!(intent.complexity, 2);
        assert_eq!(intent.required_tools, vec!["calc"]);
        assert_eq!(intent.target_agent, "tool_executor");
    }
}