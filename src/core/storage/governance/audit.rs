//! Audit log storage operations.

use crate::core::error::MornError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRecord {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub details_json: Option<String>,
    pub created_at: String,
}

impl Storage {
    /// Inserts an audit log record and returns success when the row is stored.
    pub fn insert_audit_log(&self, log: &AuditLogRecord) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO audit_log (id, user_id, action, target_type, target_id, details_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![log.id, log.user_id, log.action, log.target_type, log.target_id, log.details_json, log.created_at],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Queries audit logs by optional user and action filters, limited to the requested count.
    pub fn query_audit_log(
        &self,
        user_id: Option<&str>,
        action_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<AuditLogRecord>, MornError> {
        let conn = self.conn()?;
        let sql = match (user_id, action_type) {
            (Some(_), Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 AND action = ?2 ORDER BY created_at DESC LIMIT ?3",
            (Some(_), None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE action = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log ORDER BY created_at DESC LIMIT ?1",
        };

        let mut stmt = conn.prepare(sql).map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = match (user_id, action_type) {
            (Some(uid), Some(act)) => stmt.query_map(params![uid, act, limit], map_audit_row),
            (Some(uid), None) => stmt.query_map(params![uid, limit], map_audit_row),
            (None, Some(act)) => stmt.query_map(params![act, limit], map_audit_row),
            (None, None) => stmt.query_map(params![limit], map_audit_row),
        }
        .map_err(|e| MornError::Internal(e.to_string()))?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(logs)
    }
}

fn map_audit_row(row: &rusqlite::Row) -> rusqlite::Result<AuditLogRecord> {
    Ok(AuditLogRecord {
        id: row.get(0)?,
        user_id: row.get(1)?,
        action: row.get(2)?,
        target_type: row.get(3)?,
        target_id: row.get(4)?,
        details_json: row.get(5)?,
        created_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_log_insert_and_query() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_audit_log(&AuditLogRecord {
                id: "audit-test-1".to_string(),
                user_id: "user-test-1".to_string(),
                action: "create".to_string(),
                target_type: "task".to_string(),
                target_id: "task-test-1".to_string(),
                details_json: Some("{}".to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap();

        let logs = storage
            .query_audit_log(Some("user-test-1"), Some("create"), 10)
            .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].target_id, "task-test-1");
        assert_eq!(logs[0].details_json.as_deref(), Some("{}"));
    }
}
