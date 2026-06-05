use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::market::{License, Listing, Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub config_json: Option<String>,
    pub status: String,
    pub trust_score: f64,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRecord {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub domain: Option<String>,
    pub actions: String,
    pub description: Option<String>,
    pub trust_score: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingRecord {
    pub id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
    pub binding_type: String,
    pub config_json: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, component_type TEXT NOT NULL,
                config_json TEXT, status TEXT DEFAULT 'inactive',
                trust_score REAL DEFAULT 70.0, created_at TEXT NOT NULL, updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS capabilities (
                id TEXT PRIMARY KEY, agent_id TEXT NOT NULL, name TEXT NOT NULL,
                domain TEXT, actions TEXT NOT NULL, description TEXT,
                trust_score REAL DEFAULT 50.0,
                FOREIGN KEY (agent_id) REFERENCES agents(id)
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY, user_input TEXT NOT NULL, plan_json TEXT NOT NULL,
                status TEXT DEFAULT 'pending', created_at TEXT NOT NULL, completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS subtasks (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL, agent_id TEXT NOT NULL,
                action TEXT NOT NULL, params_json TEXT NOT NULL, status TEXT DEFAULT 'pending',
                result_json TEXT, started_at TEXT, finished_at TEXT,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );

            CREATE TABLE IF NOT EXISTS executions (
                id TEXT PRIMARY KEY, agent_id TEXT NOT NULL, task_id TEXT NOT NULL,
                action TEXT NOT NULL, status TEXT DEFAULT 'pending',
                latency_ms INTEGER, error_msg TEXT, created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL,
                decision_level TEXT NOT NULL,
                action TEXT NOT NULL,
                context_json TEXT,
                approved BOOLEAN DEFAULT TRUE,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );

            CREATE TABLE IF NOT EXISTS bindings (
                id TEXT PRIMARY KEY, source_agent_id TEXT NOT NULL, target_agent_id TEXT NOT NULL,
                source_port TEXT, target_port TEXT,
                binding_type TEXT DEFAULT 'direct',
                config_json TEXT, created_at TEXT NOT NULL,
                FOREIGN KEY (source_agent_id) REFERENCES agents(id),
                FOREIGN KEY (target_agent_id) REFERENCES agents(id)
            );
            ",
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Agents CRUD
    pub fn insert_agent(&self, agent: &AgentRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agents (id, name, component_type, config_json, status, trust_score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                agent.id, agent.name, agent.component_type, agent.config_json,
                agent.status, agent.trust_score, agent.created_at, agent.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_agent(&self, id: &str) -> Result<Option<AgentRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, component_type, config_json, status, trust_score, created_at, updated_at FROM agents WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(AgentRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                component_type: row.get(2).map_err(|e| e.to_string())?,
                config_json: row.get(3).map_err(|e| e.to_string())?,
                status: row.get(4).map_err(|e| e.to_string())?,
                trust_score: row.get(5).map_err(|e| e.to_string())?,
                created_at: row.get(6).map_err(|e| e.to_string())?,
                updated_at: row.get(7).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_agents(&self) -> Result<Vec<AgentRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, component_type, config_json, status, trust_score, created_at, updated_at FROM agents ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AgentRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    component_type: row.get(2)?,
                    config_json: row.get(3)?,
                    status: row.get(4)?,
                    trust_score: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut agents = Vec::new();
        for row in rows {
            agents.push(row.map_err(|e| e.to_string())?);
        }
        Ok(agents)
    }

    pub fn update_agent_status(&self, id: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE agents SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_agent(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Tasks CRUD
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

    // Capabilities CRUD
    pub fn insert_capability(&self, cap: &CapabilityRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO capabilities (id, agent_id, name, domain, actions, description, trust_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                cap.id, cap.agent_id, cap.name, cap.domain,
                cap.actions, cap.description, cap.trust_score
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_capabilities(&self) -> Result<Vec<CapabilityRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, name, domain, actions, description, trust_score FROM capabilities")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CapabilityRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    name: row.get(2)?,
                    domain: row.get(3)?,
                    actions: row.get(4)?,
                    description: row.get(5)?,
                    trust_score: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut caps = Vec::new();
        for row in rows {
            caps.push(row.map_err(|e| e.to_string())?);
        }
        Ok(caps)
    }

    pub fn delete_capability(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM capabilities WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_crud() {
        let storage = Storage::new_in_memory().unwrap();

        let agent = AgentRecord {
            id: "agent-1".to_string(),
            name: "Test Agent".to_string(),
            component_type: "agent".to_string(),
            config_json: Some("{}".to_string()),
            status: "active".to_string(),
            trust_score: 70.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
        };
        storage.insert_agent(&agent).unwrap();
        let got = storage.get_agent("agent-1").unwrap().unwrap();
        assert_eq!(got.name, "Test Agent");
        assert_eq!(got.status, "active");

        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 1);

        storage.update_agent_status("agent-1", "inactive").unwrap();
        let got = storage.get_agent("agent-1").unwrap().unwrap();
        assert_eq!(got.status, "inactive");

        let task = TaskRecord {
            id: "task-1".to_string(),
            user_input: "hello".to_string(),
            plan_json: "{}".to_string(),
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };
        storage.insert_task(&task).unwrap();
        let tasks = storage.list_tasks().unwrap();
        assert_eq!(tasks.len(), 1);

        let subtask = SubtaskRecord {
            id: "sub-1".to_string(),
            task_id: "task-1".to_string(),
            agent_id: "agent-1".to_string(),
            action: "chat".to_string(),
            params_json: "{}".to_string(),
            status: "pending".to_string(),
            result_json: None,
            started_at: None,
            finished_at: None,
        };
        storage.insert_subtask(&subtask).unwrap();
        let subtasks = storage.list_subtasks("task-1").unwrap();
        assert_eq!(subtasks.len(), 1);

        let decision = DecisionRecord {
            id: "dec-1".to_string(),
            task_id: "task-1".to_string(),
            decision_level: "direct_answer".to_string(),
            action: "chat".to_string(),
            context_json: None,
            approved: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        storage.insert_decision(&decision).unwrap();
        let decisions = storage.list_decisions("task-1").unwrap();
        assert_eq!(decisions.len(), 1);

        let exec = ExecutionRecord {
            id: "exec-1".to_string(),
            agent_id: "agent-1".to_string(),
            task_id: "task-1".to_string(),
            action: "chat".to_string(),
            status: "completed".to_string(),
            latency_ms: Some(100),
            error_msg: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        storage.insert_execution(&exec).unwrap();
        let execs = storage.list_executions("task-1").unwrap();
        assert_eq!(execs.len(), 1);

        let cap = CapabilityRecord {
            id: "cap-1".to_string(),
            agent_id: "agent-1".to_string(),
            name: "chat".to_string(),
            domain: Some("general".to_string()),
            actions: r#"["chat","analyze"]"#.to_string(),
            description: Some("General chat".to_string()),
            trust_score: 70.0,
        };
        storage.insert_capability(&cap).unwrap();
        let caps = storage.list_capabilities().unwrap();
        assert_eq!(caps.len(), 1);
    }
}
