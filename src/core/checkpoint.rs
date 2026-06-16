//! checkpoint — Saves and restores execution checkpoints for resumable tasks.
use crate::core::error::MornError;
use crate::core::storage::{SaveCheckpointArgs, Storage};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub session_id: String,
    pub step_index: i32,
    pub step_name: String,
    pub state: Value,
    pub metadata: Value,
    pub parent_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    pub session_id: String,
    pub current_step: i32,
    pub phase: String,
    pub context: Value,
    pub memory_snapshot: Value,
}

pub struct CheckpointManager {
    storage: Arc<Storage>,
}

impl CheckpointManager {
    /// Creates a checkpoint manager backed by shared storage.
    pub fn new(storage: Arc<Storage>) -> Self {
        CheckpointManager { storage }
    }

    /// Saves a checkpoint and serializes its state and metadata to storage.
    pub fn save(&self, cp: &Checkpoint) -> Result<(), MornError> {
        let state_json = serde_json::to_string(&cp.state).map_err(|e| MornError::Internal(e.to_string()))?;
        let metadata_json = serde_json::to_string(&cp.metadata).map_err(|e| MornError::Internal(e.to_string()))?;
        self.storage.save_checkpoint_args(SaveCheckpointArgs {
            id: &cp.id,
            session_id: &cp.session_id,
            step_index: cp.step_index,
            step_name: &cp.step_name,
            state_json: &state_json,
            metadata_json: &metadata_json,
            parent_id: cp.parent_id.as_deref(),
        })
    }

    /// Loads the latest checkpoint for a session id and deserializes its stored JSON fields.
    pub fn load_latest(&self, session_id: &str) -> Result<Option<Checkpoint>, MornError> {
        let row = self.storage.load_latest_checkpoint(session_id)?;
        match row {
            Some((id, sid, step_idx, step_name, state_json, metadata_json, parent_id)) => {
                let state: Value = serde_json::from_str(&state_json).map_err(|e| MornError::Internal(e.to_string()))?;
                let metadata: Value =
                    serde_json::from_str(&metadata_json).map_err(|e| MornError::Internal(e.to_string()))?;
                Ok(Some(Checkpoint {
                    id,
                    session_id: sid,
                    step_index: step_idx,
                    step_name,
                    state,
                    metadata,
                    parent_id,
                    created_at: String::new(),
                }))
            }
            None => Ok(None),
        }
    }

    /// Lists checkpoint summaries for a session id.
    pub fn list(&self, session_id: &str) -> Result<Vec<Checkpoint>, MornError> {
        let rows = self.storage.list_checkpoints(session_id)?;
        let mut checkpoints = Vec::new();
        for (id, step_idx, step_name, created_at) in rows {
            checkpoints.push(Checkpoint {
                id,
                session_id: session_id.to_string(),
                step_index: step_idx,
                step_name,
                state: Value::Null,
                metadata: Value::Null,
                parent_id: None,
                created_at,
            });
        }
        Ok(checkpoints)
    }

    /// Restores an agent state from a checkpoint's serialized state value.
    pub fn restore(&self, cp: &Checkpoint) -> Result<AgentState, MornError> {
        let state: AgentState =
            serde_json::from_value(cp.state.clone()).map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(state)
    }

    /// Forks a checkpoint into a new session id and returns the latest checkpoint for that session.
    pub fn fork(&self, cp_id: &str, new_session_id: &str) -> Result<Checkpoint, MornError> {
        let new_id = uuid::Uuid::new_v4().to_string();
        self.storage
            .fork_checkpoint(cp_id, &new_id, new_session_id)?;
        Ok(self.load_latest(new_session_id)?
            .ok_or_else(|| MornError::Internal("Forked checkpoint not found".to_string()))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn setup_manager() -> CheckpointManager {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        CheckpointManager::new(storage)
    }

    #[test]
    fn test_save_and_load_latest() {
        let mgr = setup_manager();
        let cp = Checkpoint {
            id: "cp-1".to_string(),
            session_id: "session-1".to_string(),
            step_index: 1,
            step_name: "test_step".to_string(),
            state: serde_json::json!({"key": "value"}),
            metadata: serde_json::json!({}),
            parent_id: None,
            created_at: String::new(),
        };
        mgr.save(&cp).unwrap();

        let loaded = mgr.load_latest("session-1").unwrap().unwrap();
        assert_eq!(loaded.id, "cp-1");
        assert_eq!(loaded.step_index, 1);
        assert_eq!(loaded.state["key"], "value");
    }

    #[test]
    fn test_list_checkpoints() {
        let mgr = setup_manager();
        for i in 0..3 {
            let cp = Checkpoint {
                id: format!("cp-{}", i),
                session_id: "session-1".to_string(),
                step_index: i,
                step_name: format!("step_{}", i),
                state: serde_json::json!({"idx": i}),
                metadata: serde_json::json!({}),
                parent_id: None,
                created_at: String::new(),
            };
            mgr.save(&cp).unwrap();
        }
        let list = mgr.list("session-1").unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_restore() {
        let mgr = setup_manager();
        let state = AgentState {
            session_id: "s1".to_string(),
            current_step: 5,
            phase: "plan".to_string(),
            context: serde_json::json!({"task": "test"}),
            memory_snapshot: serde_json::json!({}),
        };
        let cp = Checkpoint {
            id: "cp-restore".to_string(),
            session_id: "s1".to_string(),
            step_index: 5,
            step_name: "plan".to_string(),
            state: serde_json::to_value(&state).unwrap(),
            metadata: serde_json::json!({}),
            parent_id: None,
            created_at: String::new(),
        };
        mgr.save(&cp).unwrap();
        let loaded = mgr.load_latest("s1").unwrap().unwrap();
        let restored = mgr.restore(&loaded).unwrap();
        assert_eq!(restored.session_id, "s1");
        assert_eq!(restored.current_step, 5);
        assert_eq!(restored.phase, "plan");
    }

    #[test]
    fn test_fork() {
        let mgr = setup_manager();
        let cp = Checkpoint {
            id: "cp-orig".to_string(),
            session_id: "session-a".to_string(),
            step_index: 3,
            step_name: "mid".to_string(),
            state: serde_json::json!({"progress": 0.5}),
            metadata: serde_json::json!({}),
            parent_id: None,
            created_at: String::new(),
        };
        mgr.save(&cp).unwrap();
        let forked = mgr.fork("cp-orig", "session-b").unwrap();
        assert_eq!(forked.session_id, "session-b");
        assert_eq!(forked.step_index, 3);
    }
}
