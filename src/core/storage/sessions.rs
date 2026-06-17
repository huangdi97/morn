//! sessions — Persists conversation sessions and their message history.
use crate::core::error::MornError;
use rusqlite::params;

use super::Storage;

pub type SessionRow = (String, String, Option<String>, String, String);

impl Storage {
    pub fn save_session(
        &self,
        id: &str,
        user_id: &str,
        agent_id: Option<&str>,
        context_json: &str,
    ) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, user_id, agent_id, status, context_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)",
            params![
                id, user_id, agent_id, context_json,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<Option<SessionRow>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, agent_id, context_json, status FROM sessions WHERE id = ?1",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query(params![id])
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| MornError::Internal(e.to_string()))?
        {
            Ok(Some((
                row.get(0).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(1).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(2).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(3).map_err(|e| MornError::Internal(e.to_string()))?,
                row.get(4).map_err(|e| MornError::Internal(e.to_string()))?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn update_session_context(&self, id: &str, context_json: &str) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE sessions SET context_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![context_json, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionRow>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, user_id, agent_id, context_json, status FROM sessions ORDER BY created_at DESC")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| MornError::Internal(e.to_string()))?);
        }
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_save_get_list_update() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .save_session(
                "session-test-1",
                "user-test-1",
                Some("agent-test-1"),
                r#"{"step":1}"#,
            )
            .unwrap();

        let session = storage.get_session("session-test-1").unwrap().unwrap();
        assert_eq!(session.1, "user-test-1");
        assert_eq!(session.2.as_deref(), Some("agent-test-1"));
        assert_eq!(storage.list_sessions().unwrap().len(), 1);

        storage
            .update_session_context("session-test-1", r#"{"step":2}"#)
            .unwrap();
        assert_eq!(
            storage.get_session("session-test-1").unwrap().unwrap().3,
            r#"{"step":2}"#
        );
    }
}
