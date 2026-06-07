//! tasks — Persists task records, workflow state, and checkpoints.
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub user_input: String,
    pub plan_json: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskRecord {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    pub action: String,
    pub params_json: String,
    pub status: String,
    pub result_json: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub agent_id: String,
    pub task_id: String,
    pub action: String,
    pub status: String,
    pub latency_ms: Option<i64>,
    pub error_msg: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub task_id: String,
    pub decision_level: String,
    pub action: String,
    pub context_json: Option<String>,
    pub approved: bool,
    pub created_at: String,
}

impl Storage {
    /// Inserts a task record and returns success when the row is stored.
    pub fn insert_task(&self, task: &TaskRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tasks (id, user_input, plan_json, status, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task.id,
                task.user_input,
                task.plan_json,
                task.status,
                task.created_at,
                task.completed_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Fetches a task by id and returns `None` when no row exists.
    pub fn get_task(&self, id: &str) -> Result<Option<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, user_input, plan_json, status, created_at, completed_at FROM tasks WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(TaskRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                user_input: row.get(1).map_err(|e| e.to_string())?,
                plan_json: row.get(2).map_err(|e| e.to_string())?,
                status: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
                completed_at: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists task records ordered by newest creation time first.
    pub fn list_tasks(&self) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, user_input, plan_json, status, created_at, completed_at FROM tasks ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    user_input: row.get(1)?,
                    plan_json: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    completed_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| e.to_string())?);
        }
        Ok(tasks)
    }

    /// Updates a task status by id and sets completion time for terminal statuses.
    pub fn update_task_status(&self, id: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let completed_at = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        conn.execute(
            "UPDATE tasks SET status = ?1, completed_at = ?2 WHERE id = ?3",
            params![status, completed_at, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Subtasks CRUD
    /// Inserts a subtask record and returns success when the row is stored.
    pub fn insert_subtask(&self, subtask: &SubtaskRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO subtasks (id, task_id, agent_id, action, params_json, status, result_json, started_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                subtask.id, subtask.task_id, subtask.agent_id, subtask.action,
                subtask.params_json, subtask.status, subtask.result_json,
                subtask.started_at, subtask.finished_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Lists subtasks for a task id and returns the matching records.
    pub fn list_subtasks(&self, task_id: &str) -> Result<Vec<SubtaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, task_id, agent_id, action, params_json, status, result_json, started_at, finished_at FROM subtasks WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(SubtaskRecord {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    action: row.get(3)?,
                    params_json: row.get(4)?,
                    status: row.get(5)?,
                    result_json: row.get(6)?,
                    started_at: row.get(7)?,
                    finished_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut subtasks = Vec::new();
        for row in rows {
            subtasks.push(row.map_err(|e| e.to_string())?);
        }
        Ok(subtasks)
    }

    /// Updates a subtask status and optional result JSON by subtask id.
    pub fn update_subtask_status(
        &self,
        id: &str,
        status: &str,
        result_json: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let finished_at = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };
        conn.execute(
            "UPDATE subtasks SET status = ?1, result_json = ?2, finished_at = ?3 WHERE id = ?4",
            params![status, result_json, finished_at, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Executions CRUD
    /// Inserts an execution record and returns success when the row is stored.
    pub fn insert_execution(&self, exec: &ExecutionRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO executions (id, agent_id, task_id, action, status, latency_ms, error_msg, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                exec.id, exec.agent_id, exec.task_id, exec.action,
                exec.status, exec.latency_ms, exec.error_msg, exec.created_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Lists execution records associated with a task id.
    pub fn list_executions(&self, task_id: &str) -> Result<Vec<ExecutionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, task_id, action, status, latency_ms, error_msg, created_at FROM executions WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(ExecutionRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    task_id: row.get(2)?,
                    action: row.get(3)?,
                    status: row.get(4)?,
                    latency_ms: row.get(5)?,
                    error_msg: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut executions = Vec::new();
        for row in rows {
            executions.push(row.map_err(|e| e.to_string())?);
        }
        Ok(executions)
    }

    // Decisions CRUD
    /// Inserts a decision record and returns success when the row is stored.
    pub fn insert_decision(&self, decision: &DecisionRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO decisions (id, task_id, decision_level, action, context_json, approved, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                decision.id, decision.task_id, decision.decision_level,
                decision.action, decision.context_json, decision.approved, decision.created_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Lists decision records associated with a task id.
    pub fn list_decisions(&self, task_id: &str) -> Result<Vec<DecisionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, task_id, decision_level, action, context_json, approved, created_at FROM decisions WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok(DecisionRecord {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    decision_level: row.get(2)?,
                    action: row.get(3)?,
                    context_json: row.get(4)?,
                    approved: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut decisions = Vec::new();
        for row in rows {
            decisions.push(row.map_err(|e| e.to_string())?);
        }
        Ok(decisions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_task() -> TaskRecord {
        TaskRecord {
            id: "task-test-1".to_string(),
            user_input: "hello".to_string(),
            plan_json: "{}".to_string(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    fn storage_with_task() -> Storage {
        let storage = Storage::new_in_memory().unwrap();
        storage.insert_task(&test_task()).unwrap();
        storage
    }

    #[test]
    fn task_insert_get_list_update() {
        let storage = storage_with_task();

        assert_eq!(
            storage.get_task("task-test-1").unwrap().unwrap().user_input,
            "hello"
        );
        assert_eq!(storage.list_tasks().unwrap().len(), 1);

        storage.update_task_status("task-test-1", "completed").unwrap();
        let updated = storage.get_task("task-test-1").unwrap().unwrap();
        assert_eq!(updated.status, "completed");
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn subtask_insert_list_update() {
        let storage = storage_with_task();
        storage
            .insert_subtask(&SubtaskRecord {
                id: "subtask-test-1".to_string(),
                task_id: "task-test-1".to_string(),
                agent_id: "agent-test-1".to_string(),
                action: "chat".to_string(),
                params_json: "{}".to_string(),
                status: "pending".to_string(),
                result_json: None,
                started_at: None,
                finished_at: None,
            })
            .unwrap();

        assert_eq!(storage.list_subtasks("task-test-1").unwrap().len(), 1);

        storage
            .update_subtask_status("subtask-test-1", "completed", Some("{}"))
            .unwrap();
        let updated = storage.list_subtasks("task-test-1").unwrap();
        assert_eq!(updated[0].status, "completed");
        assert_eq!(updated[0].result_json.as_deref(), Some("{}"));
    }

    #[test]
    fn execution_insert_and_list() {
        let storage = storage_with_task();
        storage
            .insert_execution(&ExecutionRecord {
                id: "exec-test-1".to_string(),
                agent_id: "agent-test-1".to_string(),
                task_id: "task-test-1".to_string(),
                action: "chat".to_string(),
                status: "completed".to_string(),
                latency_ms: Some(42),
                error_msg: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap();

        let executions = storage.list_executions("task-test-1").unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].id, "exec-test-1");
        assert_eq!(executions[0].latency_ms, Some(42));
    }

    #[test]
    fn decision_insert_and_list() {
        let storage = storage_with_task();
        storage
            .insert_decision(&DecisionRecord {
                id: "decision-test-1".to_string(),
                task_id: "task-test-1".to_string(),
                decision_level: "direct_answer".to_string(),
                action: "chat".to_string(),
                context_json: Some("{}".to_string()),
                approved: true,
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .unwrap();

        let decisions = storage.list_decisions("task-test-1").unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].action, "chat");
        assert!(decisions[0].approved);
    }
}
