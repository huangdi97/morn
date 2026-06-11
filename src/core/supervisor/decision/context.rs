use crate::core::supervisor::{DecisionLevel, DecisionTier, Supervisor};
use tracing;

const STOP_WORDS: &[&str] = &[
    "的", "了", "是", "在", "有", "和", "就", "不", "人", "都", "一", "个", "上", "也", "很", "到",
    "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他", "她", "它",
];

impl Supervisor {
    pub fn decide_level_with_context(
        &self,
        text: &str,
        recent_success: Option<bool>,
        task_complexity: Option<u8>,
    ) -> DecisionLevel {
        let base_level = self.decide_level(text);
        let adjusted_level = match task_complexity {
            Some(complexity)
                if complexity <= 2 && super::weighted::is_advanced_level(&base_level) =>
            {
                DecisionLevel::L2SingleTool
            }
            Some(complexity) if complexity >= 8 && super::weighted::is_low_level(&base_level) => {
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
            if let Err(e) = engine.ingest_decision(text, adjusted_level.as_str(), success) {
                tracing::warn!("Failed to ingest decision: {}", e);
            }
            if recent_success == Some(false) {
                if let Some(keyword) = Self::extract_keywords(text).first() {
                    if let Err(e) = engine.auto_adjust(keyword, 0.0) {
                        tracing::warn!("Failed to auto adjust: {}", e);
                    }
                }
            }
        }

        adjusted_level
    }

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

    pub fn decide_with_rules(&self, text: &str) -> (DecisionLevel, String) {
        if let Some(override_) = self.decision_override() {
            return (
                override_.level.clone(),
                format!("Decision override applied: {}", override_.level.as_str()),
            );
        }

        let _trust_score = self
            .trust_scorer
            .as_ref()
            .map(|ts| {
                ts.get_all_scores()
                    .iter()
                    .map(|(_, s)| s)
                    .cloned()
                    .fold(0.0_f64, |a, b| a + b)
                    / ts.get_all_scores().len().max(1) as f64
            })
            .unwrap_or(50.0);

        if let Some(ref storage) = self.storage {
            let user_id = self.user_id.as_deref().unwrap_or("default");
            let keywords = Self::extract_keywords(text);
            for kw in &keywords {
                let rules = storage.get_decision_rules(user_id, kw).unwrap_or_default();
                if let Some(rule) = rules.first() {
                    if let Some(ref storage) = self.storage {
                        if let Err(e) = storage.increment_rule_hit(rule.id.unwrap_or(0)) {
                            tracing::warn!("Failed to increment rule hit: {}", e);
                        }
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

    pub fn decide_tier(&self, level: &DecisionLevel) -> DecisionTier {
        let trust_score = self
            .trust_scorer
            .as_ref()
            .map(|ts| {
                ts.get_all_scores()
                    .iter()
                    .map(|(_, s)| s)
                    .cloned()
                    .fold(0.0_f64, |a, b| a + b)
                    / ts.get_all_scores().len().max(1) as f64
            })
            .unwrap_or(50.0);
        DecisionTier::from_decision_level(level, trust_score)
    }

    pub(crate) fn extract_keywords(text: &str) -> Vec<String> {
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
