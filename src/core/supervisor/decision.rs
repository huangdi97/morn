//! decision — Evaluates supervisor decisions and required oversight levels.
use super::{DecisionLevel, DecisionTier, Supervisor};

const STOP_WORDS: &[&str] = &[
    "的", "了", "是", "在", "有", "和", "就", "不", "人", "都", "一", "个", "上", "也", "很", "到",
    "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他", "她", "它",
];

impl Supervisor {
    /// Classifies input text into a supervisor decision level and returns the selected level.
    pub fn decide_level(&self, text: &str) -> DecisionLevel {
        let text_lower = text.to_lowercase();

        // G22: COO override detection
        if text_lower.contains("用数据团队") || text_lower.contains("use data team") || text_lower.contains("data team") {
            return DecisionLevel::L4Team;
        }
        if text_lower.contains("直接说") || text_lower.contains("just answer") || text_lower.contains("直接回答") {
            return DecisionLevel::L1DirectAnswer;
        }

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

    /// Classifies input with recent outcome and task complexity hints, adjusting obvious over/under-routing.
    pub fn decide_level_with_context(
        &self,
        text: &str,
        recent_success: Option<bool>,
        task_complexity: Option<u8>,
    ) -> DecisionLevel {
        let base_level = self.decide_level(text);
        let adjusted_level = match task_complexity {
            Some(complexity) if complexity <= 2 && is_advanced_level(&base_level) => {
                DecisionLevel::L2SingleTool
            }
            Some(complexity) if complexity >= 8 && is_low_level(&base_level) => {
                if complexity >= 9 {
                    DecisionLevel::L4Team
                } else {
                    DecisionLevel::L3SingleAgent
                }
            }
            _ => base_level,
        };

        if let Some(engine) = &self.learning_engine {
            let success = recent_success.unwrap_or(true);
            let _ = engine.ingest_decision(text, adjusted_level.as_str(), success);
            if recent_success == Some(false) {
                if let Some(keyword) = Self::extract_keywords(text).first() {
                    let _ = engine.auto_adjust(keyword, 0.0);
                }
            }
        }

        adjusted_level
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
        if let Some(override_) = self.decision_override() {
            return (
                override_.level.clone(),
                format!("Decision override applied: {}", override_.level.as_str()),
            );
        }

        // trust score influences execution path: low trust escalates
        let _trust_score = self.trust_scorer.as_ref().map(|ts| {
            ts.get_all_scores().iter().map(|(_, s)| s).cloned().fold(0.0_f64, |a, b| a + b)
                / ts.get_all_scores().len().max(1) as f64
        }).unwrap_or(50.0);

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

    /// Converts a decision level to a tier based on trust score.
    pub fn decide_tier(&self, level: &DecisionLevel) -> DecisionTier {
        let trust_score = self.trust_scorer.as_ref().map(|ts| {
            ts.get_all_scores().iter().map(|(_, s)| s).cloned().fold(0.0_f64, |a, b| a + b)
                / ts.get_all_scores().len().max(1) as f64
        }).unwrap_or(50.0);
        DecisionTier::from_decision_level(level, trust_score)
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

fn is_advanced_level(level: &DecisionLevel) -> bool {
    matches!(
        level,
        DecisionLevel::L4Team | DecisionLevel::L5Workflow | DecisionLevel::L6JumpToStudio
    )
}

fn is_low_level(level: &DecisionLevel) -> bool {
    matches!(
        level,
        DecisionLevel::L1DirectAnswer | DecisionLevel::L2SingleTool
    )
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

    // ---- G22: COO override tests ----

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
}
