//! learning — Adjusts supervisor decision rules from execution outcomes.
use crate::core::storage::{DecisionRule, Storage};

use super::DecisionLevel;

#[derive(Clone)]
pub struct LearningEngine {
    storage: Option<Storage>,
    min_confidence: f64,
}

impl LearningEngine {
    /// Creates a learning engine backed by optional storage.
    pub fn new(storage: Option<Storage>, min_confidence: f64) -> Self {
        LearningEngine {
            storage,
            min_confidence,
        }
    }

    /// Records a decision outcome and updates the learned decision rule for its main keyword.
    pub fn ingest_decision(
        &self,
        user_input: &str,
        chosen_level: &str,
        success: bool,
    ) -> Result<(), String> {
        let Some(storage) = &self.storage else {
            return Ok(());
        };
        let Some(keyword) = extract_keyword(user_input) else {
            return Ok(());
        };

        let existing = storage
            .get_decision_rules("default", &keyword)
            .unwrap_or_default();
        if let Some(rule) = existing.first() {
            if let Some(rule_id) = rule.id {
                let change = if success { -5.0 } else { 10.0 };
                storage.adjust_rule_threshold(rule_id, change)?;
            }
            return Ok(());
        }

        storage.upsert_decision_rule(&DecisionRule {
            id: None,
            user_id: "default".to_string(),
            keyword,
            level: chosen_level.to_string(),
            trust_threshold: if success {
                self.min_confidence
            } else {
                (self.min_confidence + 20.0).min(100.0)
            },
            auto_execute: success,
            source: "learning_engine".to_string(),
            hit_count: 1,
            last_used_at: Some(chrono::Utc::now().to_rfc3339()),
            created_at: None,
        })
    }

    /// Downgrades unreliable learned rules for the keyword when recent success is low.
    pub fn auto_adjust(&self, keyword: &str, success_ratio: f64) -> Result<(), String> {
        let Some(storage) = &self.storage else {
            return Ok(());
        };
        if success_ratio >= 0.6 {
            return Ok(());
        }

        let rules = storage
            .get_decision_rules("default", keyword)
            .unwrap_or_default();
        let next_level = rules
            .first()
            .map(|rule| downgrade_level(&rule.level))
            .unwrap_or_else(|| DecisionLevel::L2SingleTool.as_str().to_string());

        storage.upsert_decision_rule(&DecisionRule {
            id: None,
            user_id: "default".to_string(),
            keyword: keyword.to_string(),
            level: next_level,
            trust_threshold: 80.0,
            auto_execute: false,
            source: "learning_engine".to_string(),
            hit_count: rules.first().map(|rule| rule.hit_count).unwrap_or(0),
            last_used_at: Some(chrono::Utc::now().to_rfc3339()),
            created_at: None,
        })
    }

    /// Returns simple trend insights from recent learned behavior.
    pub fn trend_analysis(&self) -> Vec<String> {
        let Some(storage) = &self.storage else {
            return vec!["learning disabled: no storage".to_string()];
        };

        let mut trends = Vec::new();
        for keyword in ["deploy", "search", "report", "analysis", "delete"] {
            if let Ok(rules) = storage.get_decision_rules("default", keyword) {
                if let Some(rule) = rules.first() {
                    if rule.hit_count >= 3 {
                        trends.push(format!(
                            "{} appears frequently; keep level {} with threshold {:.1}",
                            rule.keyword, rule.level, rule.trust_threshold
                        ));
                    }
                }
            }
        }

        if trends.is_empty() {
            trends.push("not enough learned decisions for trend analysis".to_string());
        }
        trends
    }
}

fn extract_keyword(user_input: &str) -> Option<String> {
    let mut words: Vec<String> = user_input
        .to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|ch| ch.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|word| word.len() >= 2)
        .collect();
    words.sort();
    words.dedup();
    words.into_iter().next()
}

fn downgrade_level(level: &str) -> String {
    match level {
        "jump_studio" => DecisionLevel::L5Workflow.as_str().to_string(),
        "workflow" => DecisionLevel::L4Team.as_str().to_string(),
        "team" => DecisionLevel::L3SingleAgent.as_str().to_string(),
        "single_agent" => DecisionLevel::L2SingleTool.as_str().to_string(),
        _ => DecisionLevel::L2SingleTool.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_decision_creates_success_rule() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 55.0);

        engine
            .ingest_decision("deploy report", "workflow", true)
            .unwrap();

        let rules = storage.get_decision_rules("default", "deploy").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].level, "workflow");
        assert!(rules[0].auto_execute);
    }

    #[test]
    fn ingest_decision_adjusts_failed_existing_rule() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine.ingest_decision("search docs", "single_tool", true).unwrap();
        let before = storage.get_decision_rules("default", "docs").unwrap()[0].trust_threshold;

        engine.ingest_decision("search docs", "single_tool", false).unwrap();

        let after = storage.get_decision_rules("default", "docs").unwrap()[0].trust_threshold;
        assert!(after > before);
    }

    #[test]
    fn auto_adjust_downgrades_low_success_rule() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine.ingest_decision("report build", "workflow", true).unwrap();

        engine.auto_adjust("build", 0.4).unwrap();

        let rules = storage.get_decision_rules("default", "build").unwrap();
        assert_eq!(rules[0].level, "team");
        assert!(!rules[0].auto_execute);
    }

    #[test]
    fn trend_analysis_reports_missing_data() {
        let engine = LearningEngine::new(Some(Storage::new_in_memory().unwrap()), 50.0);

        let trends = engine.trend_analysis();

        assert_eq!(trends, vec!["not enough learned decisions for trend analysis"]);
    }
}
