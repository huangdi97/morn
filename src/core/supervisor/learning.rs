//! learning — Learning engine for decision rules and component preference tracking.

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
    /// Creates a learning engine backed by optional storage.
    pub fn new(storage: Option<Storage>, min_confidence: f64) -> Self {
        LearningEngine {
            storage,
            min_confidence,
            component_preferences: Arc::new(Mutex::new(HashMap::new())),
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

    /// G23: handles "don't ask me in the future" patterns — lowers threshold for auto-approval
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

    /// P5: Deep learning from user correction history for personalized decision preference
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

    /// Records whether a user added or removed a component and returns ranked future recommendations.
    pub fn learn_component_preference(
        &self,
        component_name: &str,
        added: bool,
    ) -> Result<Vec<String>, String> {
        let component_name = component_name.trim();
        if component_name.is_empty() {
            return Err("component preference name cannot be empty".to_string());
        }

        let mut preferences = self.load_component_preferences()?;
        let delta = if added { 1 } else { -1 };
        *preferences.entry(component_name.to_string()).or_insert(0) += delta;

        self.save_component_preferences(&preferences)?;
        Ok(rank_component_preferences(&preferences))
    }

    /// Returns currently learned component recommendations, highest preference first.
    pub fn component_recommendations(&self) -> Vec<String> {
        self.load_component_preferences()
            .map(|preferences| rank_component_preferences(&preferences))
            .unwrap_or_default()
    }

    fn load_component_preferences(&self) -> Result<HashMap<String, i64>, String> {
        if let Some(storage) = &self.storage {
            let preferences = match storage.get_setting(COMPONENT_PREFERENCES_KEY)? {
                Some(value) => {
                    serde_json::from_str::<HashMap<String, i64>>(&value).unwrap_or_default()
                }
                None => HashMap::new(),
            };
            let mut cache = self
                .component_preferences
                .lock()
                .map_err(|e| e.to_string())?;
            *cache = preferences.clone();
            return Ok(preferences);
        }

        self.component_preferences
            .lock()
            .map(|cache| cache.clone())
            .map_err(|e| e.to_string())
    }

    fn save_component_preferences(&self, preferences: &HashMap<String, i64>) -> Result<(), String> {
        {
            let mut cache = self
                .component_preferences
                .lock()
                .map_err(|e| e.to_string())?;
            *cache = preferences.clone();
        }

        if let Some(storage) = &self.storage {
            let value = serde_json::to_string(preferences).map_err(|e| e.to_string())?;
            storage.set_setting(COMPONENT_PREFERENCES_KEY, &value)?;
        }
        Ok(())
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

fn rank_component_preferences(preferences: &HashMap<String, i64>) -> Vec<String> {
    let mut ranked = preferences
        .iter()
        .filter(|(_, score)| **score > 0)
        .map(|(component, score)| (component.clone(), *score))
        .collect::<Vec<_>>();
    ranked.sort_by(|(left_name, left_score), (right_name, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_name.cmp(right_name))
    });
    ranked
        .into_iter()
        .map(|(component, _score)| component)
        .collect()
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
    fn learn_component_preference_ranks_added_components() {
        let engine = LearningEngine::new(None, 50.0);

        engine
            .learn_component_preference("web_search", true)
            .unwrap();
        let recommendations = engine
            .learn_component_preference("data_analysis", true)
            .unwrap();

        assert_eq!(recommendations, vec!["data_analysis", "web_search"]);
    }

    #[test]
    fn learn_component_preference_removes_from_recommendations() {
        let engine = LearningEngine::new(None, 50.0);

        engine
            .learn_component_preference("web_search", true)
            .unwrap();
        let recommendations = engine
            .learn_component_preference("web_search", false)
            .unwrap();

        assert!(recommendations.is_empty());
    }

    #[test]
    fn learn_component_preference_persists_with_storage() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine.learn_component_preference("chart", true).unwrap();

        let next_engine = LearningEngine::new(Some(storage), 50.0);

        assert_eq!(next_engine.component_recommendations(), vec!["chart"]);
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
