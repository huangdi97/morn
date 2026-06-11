//! learning — Learning engine for decision rules and component preference tracking.

mod preferences;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::storage::{DecisionRule, Storage};

use super::DecisionLevel;

const COMPONENT_PREFERENCES_KEY: &str = "learning.component_preferences";

#[derive(Clone)]
pub struct LearningEngine {
    storage: Option<Storage>,
    min_confidence: f64,
    component_preferences: Arc<Mutex<HashMap<String, i64>>>,
}

impl LearningEngine {
    pub fn new(storage: Option<Storage>, min_confidence: f64) -> Self {
        LearningEngine {
            storage,
            min_confidence,
            component_preferences: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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

    pub fn handle_dont_ask_pattern(&self, user_input: &str) -> Result<Option<String>, String> {
        let lower = user_input.to_lowercase();
        let is_dont_ask = lower.contains("以后不用问我")
            || lower.contains("don't ask me")
            || lower.contains("dont ask me")
            || lower.contains("stop asking")
            || lower.contains("自动执行")
            || lower.contains("auto approve");
        if !is_dont_ask {
            return Ok(None);
        }

        let Some(storage) = &self.storage else {
            return Ok(Some("No storage for learning".to_string()));
        };

        let keyword = extract_keyword(user_input).unwrap_or_default();
        let existing = storage
            .get_decision_rules("default", &keyword)
            .unwrap_or_default();

        if let Some(rule) = existing.first() {
            if let Some(rule_id) = rule.id {
                storage.adjust_rule_threshold(rule_id, -20.0)?;
                storage.upsert_decision_rule(&DecisionRule {
                    id: Some(rule_id),
                    user_id: "default".to_string(),
                    keyword: keyword.clone(),
                    level: rule.level.clone(),
                    trust_threshold: (rule.trust_threshold - 20.0).max(0.0),
                    auto_execute: true,
                    source: "learning_engine".to_string(),
                    hit_count: rule.hit_count + 1,
                    last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                    created_at: None,
                })?;
            }
        } else {
            storage.upsert_decision_rule(&DecisionRule {
                id: None,
                user_id: "default".to_string(),
                keyword: keyword.clone(),
                level: DecisionLevel::L2SingleTool.as_str().to_string(),
                trust_threshold: 30.0,
                auto_execute: true,
                source: "learning_engine".to_string(),
                hit_count: 1,
                last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                created_at: None,
            })?;
        }

        Ok(Some(format!(
            "Auto-approved rule set for keyword '{}'",
            keyword
        )))
    }

    pub fn learn_from_correction_history(
        &self,
        corrections: &[(&str, bool)],
    ) -> Result<usize, String> {
        let Some(storage) = &self.storage else {
            return Ok(0);
        };
        let mut updated = 0;
        for (keyword, approved) in corrections {
            let existing = storage
                .get_decision_rules("default", keyword)
                .unwrap_or_default();
            let change = if *approved { -5.0 } else { 10.0 };
            if let Some(rule) = existing.first() {
                if let Some(rule_id) = rule.id {
                    let new_threshold = (rule.trust_threshold + change).clamp(0.0, 100.0);
                    storage.adjust_rule_threshold(rule_id, change)?;
                    storage.upsert_decision_rule(&DecisionRule {
                        id: Some(rule_id),
                        user_id: "default".to_string(),
                        keyword: keyword.to_string(),
                        level: rule.level.clone(),
                        trust_threshold: new_threshold,
                        auto_execute: new_threshold < 60.0,
                        source: "deep_learning".to_string(),
                        hit_count: rule.hit_count + 1,
                        last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                        created_at: None,
                    })?;
                    updated += 1;
                }
            }
        }
        Ok(updated)
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
        engine
            .ingest_decision("search docs", "single_tool", true)
            .unwrap();
        let before = storage.get_decision_rules("default", "docs").unwrap()[0].trust_threshold;

        engine
            .ingest_decision("search docs", "single_tool", false)
            .unwrap();

        let after = storage.get_decision_rules("default", "docs").unwrap()[0].trust_threshold;
        assert!(after > before);
    }

    #[test]
    fn auto_adjust_downgrades_low_success_rule() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine
            .ingest_decision("report build", "workflow", true)
            .unwrap();

        engine.auto_adjust("build", 0.4).unwrap();

        let rules = storage.get_decision_rules("default", "build").unwrap();
        assert_eq!(rules[0].level, "team");
        assert!(!rules[0].auto_execute);
    }

    #[test]
    fn trend_analysis_reports_missing_data() {
        let engine = LearningEngine::new(Some(Storage::new_in_memory().unwrap()), 50.0);

        let trends = engine.trend_analysis();

        assert_eq!(
            trends,
            vec!["not enough learned decisions for trend analysis"]
        );
    }

    #[test]
    fn handle_dont_ask_pattern_creates_auto_approve_rule() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);

        let result = engine
            .handle_dont_ask_pattern("以后不用问我 about reports")
            .unwrap();

        assert!(result.is_some());
        let rules = storage.get_decision_rules("default", "about").unwrap();
        assert_eq!(rules.len(), 1);
        assert!(rules[0].auto_execute);
        assert!(rules[0].trust_threshold < 50.0);
    }

    #[test]
    fn handle_dont_ask_pattern_english() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);

        let result = engine
            .handle_dont_ask_pattern("don't ask me about deploy")
            .unwrap();

        assert!(result.is_some());
        let rules = storage.get_decision_rules("default", "about").unwrap();
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn handle_dont_ask_pattern_no_match() {
        let engine = LearningEngine::new(Some(Storage::new_in_memory().unwrap()), 50.0);
        let result = engine.handle_dont_ask_pattern("normal question").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn ingest_decision_without_storage_is_noop() {
        let engine = LearningEngine::new(None, 50.0);
        assert!(engine
            .ingest_decision("deploy now", "workflow", true)
            .is_ok());
    }

    #[test]
    fn auto_adjust_skipped_when_ratio_above_threshold() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine
            .ingest_decision("search docs", "single_tool", true)
            .unwrap();
        engine.auto_adjust("docs", 0.8).unwrap();
        let rules = storage.get_decision_rules("default", "docs").unwrap();
        assert_eq!(rules[0].level, "single_tool");
    }

    #[test]
    fn learn_from_correction_history_updates_multiple_keywords() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine
            .ingest_decision("deploy report", "single_agent", true)
            .unwrap();
        engine
            .ingest_decision("search tool", "single_tool", true)
            .unwrap();

        let corrections = &[("deploy", false), ("search", true)];
        let updated = engine.learn_from_correction_history(corrections).unwrap();
        assert_eq!(updated, 2);
    }

    #[test]
    fn learn_from_correction_without_storage_returns_zero() {
        let engine = LearningEngine::new(None, 50.0);
        let updated = engine
            .learn_from_correction_history(&[("deploy", false)])
            .unwrap();
        assert_eq!(updated, 0);
    }

    #[test]
    fn trend_analysis_no_storage() {
        let engine = LearningEngine::new(None, 50.0);
        let trends = engine.trend_analysis();
        assert_eq!(trends, vec!["learning disabled: no storage"]);
    }

    #[test]
    fn handle_dont_ask_no_storage() {
        let engine = LearningEngine::new(None, 50.0);
        let result = engine
            .handle_dont_ask_pattern("以后不用问我 about reports")
            .unwrap();
        assert_eq!(result, Some("No storage for learning".to_string()));
    }
}