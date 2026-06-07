use rusqlite::params;

use super::{DecisionRecord, ExecutionRecord, Storage, SubtaskRecord, TaskRecord};

impl Storage {
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
