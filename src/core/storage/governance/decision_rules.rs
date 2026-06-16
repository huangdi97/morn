//! Decision rule storage operations.

use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRule {
    pub id: Option<i64>,
    pub user_id: String,
    pub keyword: String,
    pub level: String,
    pub trust_threshold: f64,
    pub auto_execute: bool,
    pub source: String,
    pub hit_count: i64,
    pub last_used_at: Option<String>,
    pub created_at: Option<String>,
}

impl Storage {
    /// Fetches decision rules for a user and keyword, returning all matching records.
    pub fn get_decision_rules(
        &self,
        user_id: &str,
        keyword: &str,
    ) -> Result<Vec<DecisionRule>, MornError> {
        let conn = self.conn.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at, created_at FROM decision_rules WHERE user_id = ?1 AND keyword = ?2")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![user_id, keyword], |row| {
                let auto_execute_int: i32 = row.get(5)?;
                Ok(DecisionRule {
                    id: Some(row.get(0)?),
                    user_id: row.get(1)?,
                    keyword: row.get(2)?,
                    level: row.get(3)?,
                    trust_threshold: row.get(4)?,
                    auto_execute: auto_execute_int != 0,
                    source: row.get(6)?,
                    hit_count: row.get(7)?,
                    last_used_at: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(rules)
    }

    /// Inserts or updates a decision rule keyed by user id and keyword.
    pub fn upsert_decision_rule(&self, rule: &DecisionRule) -> Result<(), MornError> {
        let conn = self.conn.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        conn.execute(
            "INSERT INTO decision_rules (user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(user_id, keyword) DO UPDATE SET
                level = COALESCE(?3, level),
                trust_threshold = COALESCE(?4, trust_threshold),
                auto_execute = COALESCE(?5, auto_execute),
                source = COALESCE(?6, source),
                hit_count = COALESCE(?7, hit_count),
                last_used_at = COALESCE(?8, last_used_at)",
            params![
                rule.user_id, rule.keyword, rule.level, rule.trust_threshold,
                rule.auto_execute, rule.source, rule.hit_count, rule.last_used_at
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Increments a decision rule hit count and updates its last-used timestamp.
    pub fn increment_rule_hit(&self, rule_id: i64) -> Result<(), MornError> {
        let conn = self.conn.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        conn.execute(
            "UPDATE decision_rules SET hit_count = hit_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), rule_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Adjusts a decision rule trust threshold by `change`, clamped to the 0-100 range.
    pub fn adjust_rule_threshold(&self, rule_id: i64, change: f64) -> Result<(), MornError> {
        let conn = self.conn.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        conn.execute(
            "UPDATE decision_rules SET trust_threshold = MAX(0.0, MIN(100.0, trust_threshold + ?1)) WHERE id = ?2",
            params![change, rule_id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_rule_upsert_get_and_update() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .upsert_decision_rule(&DecisionRule {
                id: None,
                user_id: "user-test-1".to_string(),
                keyword: "deploy".to_string(),
                level: "L4".to_string(),
                trust_threshold: 60.0,
                auto_execute: false,
                source: "test".to_string(),
                hit_count: 0,
                last_used_at: None,
                created_at: None,
            })
            .unwrap();

        let rule = storage
            .get_decision_rules("user-test-1", "deploy")
            .unwrap()
            .remove(0);
        assert_eq!(rule.level, "L4");
        assert_eq!(rule.hit_count, 0);

        let id = rule.id.unwrap();
        storage.increment_rule_hit(id).unwrap();
        storage.adjust_rule_threshold(id, 5.0).unwrap();
        let updated = storage.get_decision_rules("user-test-1", "deploy").unwrap();
        assert_eq!(updated[0].hit_count, 1);
        assert_eq!(updated[0].trust_threshold, 65.0);
    }
}
