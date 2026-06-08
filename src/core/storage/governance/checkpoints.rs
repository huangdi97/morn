//! Checkpoint storage operations.

use rusqlite::params;

use super::super::Storage;

pub type CheckpointRow = (String, String, i32, String, String, String, Option<String>);

pub struct SaveCheckpointArgs<'a> {
    pub id: &'a str,
    pub session_id: &'a str,
    pub step_index: i32,
    pub step_name: &'a str,
    pub state_json: &'a str,
    pub metadata_json: &'a str,
    pub parent_id: Option<&'a str>,
}

impl Storage {
    /// Saves a checkpoint using grouped arguments and returns success when the row is stored.
    pub fn save_checkpoint_args(&self, args: SaveCheckpointArgs<'_>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                args.id,
                args.session_id,
                args.step_index,
                args.step_name,
                args.state_json,
                args.metadata_json,
                args.parent_id,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] /* 预留：兼容既有 Storage API */
    /// Saves a checkpoint from individual fields and returns success when the row is stored.
    pub fn save_checkpoint(
        &self,
        id: &str,
        session_id: &str,
        step_index: i32,
        step_name: &str,
        state_json: &str,
        metadata_json: &str,
        parent_id: Option<&str>,
    ) -> Result<(), String> {
        self.save_checkpoint_args(SaveCheckpointArgs {
            id,
            session_id,
            step_index,
            step_name,
            state_json,
            metadata_json,
            parent_id,
        })
    }

    /// Loads the latest checkpoint row for a session id, returning `None` when no checkpoint exists.
    pub fn load_latest_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<Option<CheckpointRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, step_index, step_name, state_json, metadata_json, parent_id FROM checkpoints WHERE session_id = ?1 ORDER BY step_index DESC LIMIT 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some((
                row.get(0).map_err(|e| e.to_string())?,
                row.get(1).map_err(|e| e.to_string())?,
                row.get(2).map_err(|e| e.to_string())?,
                row.get(3).map_err(|e| e.to_string())?,
                row.get(4).map_err(|e| e.to_string())?,
                row.get(5).map_err(|e| e.to_string())?,
                row.get(6).map_err(|e| e.to_string())?,
            )))
        } else {
            Ok(None)
        }
    }

    /// Lists checkpoint summaries for a session ordered by step index.
    pub fn list_checkpoints(
        &self,
        session_id: &str,
    ) -> Result<Vec<(String, i32, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, step_index, step_name, created_at FROM checkpoints WHERE session_id = ?1 ORDER BY step_index ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut checkpoints = Vec::new();
        for row in rows {
            checkpoints.push(row.map_err(|e| e.to_string())?);
        }
        Ok(checkpoints)
    }

    /// Forks a checkpoint into a new id and session id, preserving the stored checkpoint state.
    pub fn fork_checkpoint(
        &self,
        cp_id: &str,
        new_id: &str,
        new_session_id: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
             SELECT ?1, ?2, step_index, step_name, state_json, metadata_json, parent_id, ?3 FROM checkpoints WHERE id = ?4",
            params![new_id, new_session_id, chrono::Utc::now().to_rfc3339(), cp_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_save_load_list_and_fork() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .save_checkpoint("cp-test-1", "session-test-1", 1, "step", "{}", "{}", None)
            .unwrap();

        assert_eq!(
            storage
                .load_latest_checkpoint("session-test-1")
                .unwrap()
                .unwrap()
                .0,
            "cp-test-1"
        );
        assert_eq!(storage.list_checkpoints("session-test-1").unwrap().len(), 1);

        storage
            .fork_checkpoint("cp-test-1", "cp-test-2", "session-test-2")
            .unwrap();
        assert!(storage
            .load_latest_checkpoint("session-test-2")
            .unwrap()
            .is_some());
    }
}
