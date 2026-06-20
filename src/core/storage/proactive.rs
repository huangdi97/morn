//! proactive — Persists proactive rule definitions.
use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveRule {
    pub id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_config: String,
    pub action: String,
    pub enabled: bool,
    pub last_triggered_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Storage {
    pub fn list_proactive_rules(&self) -> Result<Vec<ProactiveRule>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, trigger_type, trigger_config, action, enabled, \
                 last_triggered_at, created_at, updated_at FROM proactive_rules",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProactiveRule {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    trigger_type: row.get(2)?,
                    trigger_config: row.get(3)?,
                    action: row.get(4)?,
                    enabled: row.get::<_, i32>(5)? != 0,
                    last_triggered_at: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(rules)
    }

    pub fn get_proactive_rule(&self, id: &str) -> Result<Option<ProactiveRule>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, trigger_type, trigger_config, action, enabled, \
                 last_triggered_at, created_at, updated_at FROM proactive_rules WHERE id = ?1",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| MornError::Internal(e.to_string()))? {
            Ok(Some(ProactiveRule {
                id: row.get(0)?,
                name: row.get(1)?,
                trigger_type: row.get(2)?,
                trigger_config: row.get(3)?,
                action: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                last_triggered_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn create_proactive_rule(&self, rule: &ProactiveRule) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO proactive_rules (id, name, trigger_type, trigger_config, action, enabled, \
             last_triggered_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                rule.id,
                rule.name,
                rule.trigger_type,
                rule.trigger_config,
                rule.action,
                rule.enabled as i32,
                rule.last_triggered_at,
                rule.created_at,
                rule.updated_at,
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn update_proactive_rule(&self, rule: &ProactiveRule) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE proactive_rules SET name = ?1, trigger_type = ?2, trigger_config = ?3, \
             action = ?4, enabled = ?5, last_triggered_at = ?6, updated_at = ?7 WHERE id = ?8",
            params![
                rule.name,
                rule.trigger_type,
                rule.trigger_config,
                rule.action,
                rule.enabled as i32,
                rule.last_triggered_at,
                rule.updated_at,
                rule.id,
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn delete_proactive_rule(&self, id: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM proactive_rules WHERE id = ?1", params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn toggle_proactive_rule(&self, id: &str, enabled: bool) -> Result<(), MornError> {
        let conn = self.conn()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        conn.execute(
            "UPDATE proactive_rules SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i32, now, id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }
}