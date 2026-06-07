//! decision — Evaluates supervisor decisions and required oversight levels.
use super::{DecisionLevel, Supervisor};

const STOP_WORDS: &[&str] = &[
    "的", "了", "是", "在", "有", "和", "就", "不", "人", "都", "一", "个", "上", "也", "很", "到",
    "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他", "她", "它",
];

impl Supervisor {
    /// Classifies input text into a supervisor decision level and returns the selected level.
    pub fn decide_level(&self, text: &str) -> DecisionLevel {
        let text_lower = text.to_lowercase();

        let simple_indicators = [
            "hello",
            "hi ",
            "thanks",
            "bye",
            "who are you",
            "what is your name",
            "good morning",
        ];
        if simple_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower == s.trim())
        {
            return DecisionLevel::L1DirectAnswer;
        }

        let tool_indicators = [
            "search",
            "look up",
            "find ",
            "calculate",
            "compute",
            "convert",
            "translate",
            "what time",
            "what's the time",
        ];
        if tool_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L2SingleTool;
        }

        let studio_indicators = [
            "create an agent",
            "create a agent",
            "build an agent",
            "make an agent",
            "design a agent",
            "customize",
            "configure",
            "create workflow",
        ];
        if studio_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower.starts_with(s.trim()))
        {
            return DecisionLevel::L6JumpToStudio;
        }

        let workflow_indicators = [
            "report",
            "analysis",
            "research",
            "investigate",
            "compare",
            "plan",
            "strategy",
            "create a",
        ];
        if workflow_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L5Workflow;
        }

        let team_indicators = [
            "complex",
            "multi-step",
            "multiple",
            "various",
            "comprehensive",
            "full",
            "end-to-end",
        ];
        if team_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L4Team;
        }

        DecisionLevel::L3SingleAgent
    }

    /// Classifies input text and returns both the decision level and a short reasoning string.
    pub fn decide(&self, text: &str) -> (DecisionLevel, String) {
        let level = self.decide_level(text);
        let reasoning = match level {
            DecisionLevel::L1DirectAnswer => "Simple greeting or knowledge query".into(),
            DecisionLevel::L2SingleTool => "Single tool operation needed".into(),
            DecisionLevel::L3SingleAgent => "Requires single agent analysis".into(),
            DecisionLevel::L4Team => "Complex multi-dimensional task".into(),
            DecisionLevel::L5Workflow => "Standard workflow available".into(),
            DecisionLevel::L6JumpToStudio => "User wants to create/modify components".into(),
        };
        (level, reasoning)
    }

    /// Applies stored decision rules for the input text and falls back to heuristic classification.
    pub fn decide_with_rules(&self, text: &str) -> (DecisionLevel, String) {
        if let Some(ref storage) = self.storage {
            let user_id = self.user_id.as_deref().unwrap_or("default");
            let keywords = Self::extract_keywords(text);
            for kw in &keywords {
                let rules = storage.get_decision_rules(user_id, kw).unwrap_or_default();
                if let Some(rule) = rules.first() {
                    if let Some(ref storage) = self.storage {
                        let _ = storage.increment_rule_hit(rule.id.unwrap_or(0));
                    }
                    if rule.auto_execute {
                        return (
                            DecisionLevel::L1DirectAnswer,
                            format!("Rule auto-execute matched keyword '{}'", kw),
                        );
                    }
                    let level = match rule.level.as_str() {
                        "direct_answer" => DecisionLevel::L1DirectAnswer,
                        "single_tool" => DecisionLevel::L2SingleTool,
                        "single_agent" => DecisionLevel::L3SingleAgent,
                        "team" => DecisionLevel::L4Team,
                        "workflow" => DecisionLevel::L5Workflow,
                        "jump_studio" => DecisionLevel::L6JumpToStudio,
                        _ => DecisionLevel::L3SingleAgent,
                    };
                    return (
                        level,
                        format!("Rule matched keyword '{}' with level {}", kw, rule.level),
                    );
                }
            }
        }
        self.decide(text)
    }

    pub(super) fn extract_keywords(text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let raw: Vec<&str> = text_lower.split_whitespace().collect();
        let mut keywords: Vec<String> = Vec::new();
        let stop_set: std::collections::HashSet<&str> = STOP_WORDS.iter().copied().collect();
        for word in raw {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if !cleaned.is_empty() && !stop_set.contains(cleaned.as_str()) && cleaned.len() >= 2 {
                keywords.push(cleaned);
            }
        }
        keywords.sort();
        keywords.dedup();
        keywords
    }
}
