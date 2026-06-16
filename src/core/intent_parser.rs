//! 用户意图解析器 — NL 指令转结构化任务描述
use crate::core::error::MornError;
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IntentType {
    DirectAnswer,
    ToolCall,
    AgentTask,
    TeamTask,
    WorkflowTemplate,
    JumpToStudio,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Intent {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub entities: Vec<String>,
    pub target_level: String,
    #[serde(default)]
    pub raw_input: String,
}

pub struct IntentParser;

impl IntentParser {
    /// Parses the user input using an LLM, with rule-based matching as fallback.
    pub fn parse_with_llm(
        input: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
    ) -> Intent {
        let system_prompt = "You are an intent classifier. Given a user input, determine the intent type. Return a JSON object with fields: intent_type (one of: DirectAnswer, ToolCall, AgentTask, TeamTask, WorkflowTemplate, JumpToStudio), confidence (0.0-1.0), entities (array of key phrases), target_level (one of: direct_answer, single_tool, single_agent, team, workflow, jump_studio). Only return valid JSON, no markdown, no explanation.";

        let prompt = format!(
            r#"Classify this user input: "{}"

Return JSON: {{"intent_type": "...", "confidence": 0.0, "entities": [], "target_level": "..."}}"#,
            input
        );

        match chat_fn(&prompt, system_prompt) {
            Ok(response) => {
                let cleaned = response
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                match serde_json::from_str::<Intent>(cleaned) {
                    Ok(intent) => intent,
                    Err(_) => Self::parse(input),
                }
            }
            Err(_) => Self::parse(input),
        }
    }

    /// Rule-based intent parsing fallback.
    pub fn parse(input: &str) -> Intent {
        let lower = input.to_lowercase();

        let tool_keywords = [
            "search",
            "calculate",
            "look up",
            "find ",
            "convert",
            "translate",
        ];
        let studio_keywords = [
            "create an agent",
            "build an agent",
            "customize",
            "configure agent",
            "create workflow",
        ];
        let workflow_keywords = [
            "report", "analysis", "research", "compare", "plan", "strategy",
        ];
        let team_keywords = [
            "complex",
            "multi-step",
            "multiple agents",
            "team",
            "collaborate",
        ];
        let greet_keywords = ["hello", "hi", "thanks", "bye", "who are you"];

        let entities = Self::extract_entities(input);

        let (intent_type, confidence, target_level) =
            if studio_keywords.iter().any(|k| lower.contains(k)) {
                (IntentType::JumpToStudio, 0.9, "jump_studio")
            } else if team_keywords.iter().any(|k| lower.contains(k)) {
                (IntentType::TeamTask, 0.75, "team")
            } else if workflow_keywords.iter().any(|k| lower.contains(k)) {
                (IntentType::WorkflowTemplate, 0.8, "workflow")
            } else if tool_keywords.iter().any(|k| lower.contains(k)) {
                (IntentType::ToolCall, 0.85, "single_tool")
            } else if greet_keywords.iter().any(|k| lower.contains(k)) {
                (IntentType::DirectAnswer, 0.95, "direct_answer")
            } else if !entities.is_empty() {
                (IntentType::AgentTask, 0.7, "single_agent")
            } else {
                (IntentType::DirectAnswer, 0.5, "direct_answer")
            };

        Intent {
            intent_type,
            confidence,
            entities,
            target_level: target_level.to_string(),
            raw_input: input.to_string(),
        }
    }

    fn extract_entities(input: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let words: Vec<&str> = input.split_whitespace().collect();
        let common_words = [
            "the", "a", "an", "is", "are", "was", "were", "in", "on", "at", "to", "for", "of",
            "with", "and", "or", "but",
        ];
        for chunk in words.windows(2) {
            if !common_words.contains(&chunk[0].to_lowercase().as_str())
                && !common_words.contains(&chunk[1].to_lowercase().as_str())
            {
                entities.push(format!("{} {}", chunk[0], chunk[1]));
            }
        }
        entities.truncate(5);
        entities
    }

    pub fn detect_override(input: &str) -> Option<IntentType> {
        let lower = input.to_lowercase();
        if lower.contains("用数据团队") || lower.contains("use data team") {
            Some(IntentType::TeamTask)
        } else if lower.contains("直接说")
            || lower.contains("just answer")
            || lower.contains("直接回答")
        {
            Some(IntentType::DirectAnswer)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_direct_answer_greeting() {
        let intent = IntentParser::parse("hello");
        assert_eq!(intent.intent_type, IntentType::DirectAnswer);
        assert!(intent.confidence > 0.9);
    }

    #[test]
    fn parse_tool_call() {
        let intent = IntentParser::parse("search for AI news");
        assert_eq!(intent.intent_type, IntentType::ToolCall);
    }

    #[test]
    fn parse_workflow_template() {
        let intent = IntentParser::parse("create a report for this quarter");
        assert_eq!(intent.intent_type, IntentType::WorkflowTemplate);
    }

    #[test]
    fn parse_team_task() {
        let intent = IntentParser::parse("complex multi-step analysis with multiple agents");
        assert_eq!(intent.intent_type, IntentType::TeamTask);
    }

    #[test]
    fn parse_jump_to_studio() {
        let intent = IntentParser::parse("create an agent for data analysis");
        assert_eq!(intent.intent_type, IntentType::JumpToStudio);
    }

    #[test]
    fn detect_override_team() {
        let result = IntentParser::detect_override("use data team to analyze this");
        assert_eq!(result, Some(IntentType::TeamTask));
    }

    #[test]
    fn detect_override_direct() {
        let result = IntentParser::detect_override("直接回答这个问题");
        assert_eq!(result, Some(IntentType::DirectAnswer));
    }

    #[test]
    fn detect_override_none() {
        let result = IntentParser::detect_override("what is the weather");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_agent_task_with_entities() {
        let intent = IntentParser::parse("analyze customer data");
        assert_eq!(intent.intent_type, IntentType::AgentTask);
        assert!(!intent.entities.is_empty());
    }

    #[test]
    fn parse_default_fallback_direct_answer() {
        let intent = IntentParser::parse("singular");
        // single word with no keyword match produces no entities → DirectAnswer fallback
        assert_eq!(intent.intent_type, IntentType::DirectAnswer);
        assert!((intent.confidence - 0.5).abs() < 0.01);
    }

    #[test]
    fn parse_confidence_ranges() {
        let cases = vec![
            ("hello", 0.95),
            ("search for something", 0.85),
            ("create a report", 0.8),
            ("complex multi-step task", 0.75),
            ("customize my agent", 0.9),
        ];
        for (input, expected_conf) in cases {
            let intent = IntentParser::parse(input);
            assert!(
                (intent.confidence - expected_conf).abs() < 0.01,
                "input '{}': expected conf {}, got {}",
                input,
                expected_conf,
                intent.confidence
            );
        }
    }

    #[test]
    fn parse_empty_input() {
        let intent = IntentParser::parse("");
        assert_eq!(intent.intent_type, IntentType::DirectAnswer);
    }

    #[test]
    fn parse_mixed_keywords_prefers_higher_priority() {
        let intent = IntentParser::parse("customize agent to search for data");
        // JumpToStudio has higher priority than ToolCall
        assert_eq!(intent.intent_type, IntentType::JumpToStudio);
    }

    #[test]
    fn parse_entity_extraction_limits() {
        let intent =
            IntentParser::parse("the quick brown fox jumps over the lazy dog near the river");
        assert!(intent.entities.len() <= 5);
    }

    #[test]
    fn parse_with_llm_success() {
        let chat_fn = |_prompt: &str, _system: &str| {
            Ok(r#"{"intent_type": "ToolCall", "confidence": 0.92, "entities": ["stock price"], "target_level": "single_tool"}"#.to_string())
        };
        let intent = IntentParser::parse_with_llm("get me the stock price of AAPL", &chat_fn);
        assert_eq!(intent.intent_type, IntentType::ToolCall);
        assert!((intent.confidence - 0.92).abs() < 0.01);
        assert!(intent.entities.contains(&"stock price".to_string()));
    }

    #[test]
    fn parse_with_llm_invalid_json_falls_back_to_rule() {
        let chat_fn = |_prompt: &str, _system: &str| Ok("not valid json".to_string());
        let intent = IntentParser::parse_with_llm("hello", &chat_fn);
        assert_eq!(intent.intent_type, IntentType::DirectAnswer);
    }

    #[test]
    fn parse_with_llm_error_falls_back_to_rule() {
        let chat_fn = |_prompt: &str, _system: &str| Err(MornError::Internal("LLM unavailable".to_string()));
        let intent = IntentParser::parse_with_llm("search for AI news", &chat_fn);
        assert_eq!(intent.intent_type, IntentType::ToolCall);
    }
}
