//! Decision rules for configurable COO override behavior.

use crate::core::supervisor::DecisionLevel;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionRule {
    pub id: String,
    pub action: String,
    pub level: DecisionLevel,
    pub condition: String,
    pub effect: String,
    pub created_at: String,
}

pub fn parse_decision_level(s: &str) -> Option<DecisionLevel> {
    match s.trim().to_lowercase().as_str() {
        "l1" | "direct_answer" | "直接回答" => Some(DecisionLevel::L1DirectAnswer),
        "l2" | "single_tool" | "工具" => Some(DecisionLevel::L2SingleTool),
        "l3" | "single_agent" | "单agent" | "单 agent" => Some(DecisionLevel::L3SingleAgent),
        "l4" | "team" | "团队" => Some(DecisionLevel::L4Team),
        "l5" | "workflow" | "工作流" => Some(DecisionLevel::L5Workflow),
        "l6" | "jump_studio" | "创作台" => Some(DecisionLevel::L6JumpToStudio),
        _ => None,
    }
}

pub trait DecisionRuleStore {
    fn add_rule(&self, rule: DecisionRule) -> Result<(), String>;
    fn remove_rule(&self, id: &str) -> Result<(), String>;
    fn list_rules(&self) -> Result<Vec<DecisionRule>, String>;
    fn find_rule(&self, action: &str) -> Result<Option<DecisionRule>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::supervisor::DecisionLevel;

    #[test]
    fn test_parse_decision_level_l1() {
        assert_eq!(
            parse_decision_level("L1"),
            Some(DecisionLevel::L1DirectAnswer)
        );
        assert_eq!(
            parse_decision_level("direct_answer"),
            Some(DecisionLevel::L1DirectAnswer)
        );
        assert_eq!(
            parse_decision_level("直接回答"),
            Some(DecisionLevel::L1DirectAnswer)
        );
    }

    #[test]
    fn test_parse_decision_level_all_levels() {
        assert_eq!(
            parse_decision_level("l1"),
            Some(DecisionLevel::L1DirectAnswer)
        );
        assert_eq!(
            parse_decision_level("l2"),
            Some(DecisionLevel::L2SingleTool)
        );
        assert_eq!(
            parse_decision_level("l3"),
            Some(DecisionLevel::L3SingleAgent)
        );
        assert_eq!(parse_decision_level("l4"), Some(DecisionLevel::L4Team));
        assert_eq!(parse_decision_level("l5"), Some(DecisionLevel::L5Workflow));
        assert_eq!(
            parse_decision_level("l6"),
            Some(DecisionLevel::L6JumpToStudio)
        );
    }

    #[test]
    fn test_parse_decision_level_alias() {
        assert_eq!(
            parse_decision_level("single_tool"),
            Some(DecisionLevel::L2SingleTool)
        );
        assert_eq!(parse_decision_level("team"), Some(DecisionLevel::L4Team));
        assert_eq!(
            parse_decision_level("workflow"),
            Some(DecisionLevel::L5Workflow)
        );
        assert_eq!(
            parse_decision_level("jump_studio"),
            Some(DecisionLevel::L6JumpToStudio)
        );
    }

    #[test]
    fn test_parse_decision_level_case_insensitive() {
        assert_eq!(parse_decision_level("L4"), Some(DecisionLevel::L4Team));
        assert_eq!(
            parse_decision_level("Single_Agent"),
            Some(DecisionLevel::L3SingleAgent)
        );
    }

    #[test]
    fn test_parse_decision_level_invalid() {
        assert_eq!(parse_decision_level("invalid"), None);
        assert_eq!(parse_decision_level(""), None);
    }

    #[test]
    fn test_decision_rule_serialize_deserialize() {
        let rule = DecisionRule {
            id: "test-1".into(),
            action: "deploy".into(),
            level: DecisionLevel::L4Team,
            condition: "contains 'deploy'".into(),
            effect: "require_approval".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: DecisionRule = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.level, DecisionLevel::L4Team);
        assert_eq!(deserialized.action, "deploy");
    }
}
