use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::market::{License, Listing, Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberRecord {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissionRecord {
    pub id: String,
    pub agent_id: String,
    pub user_id: String,
    pub team_id: Option<String>,
    pub permission: String,
    pub granted_at: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub data_json: String,
    pub timestamp: String,
    pub device_id: String,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub id: String,
    pub name: String,
    pub last_seen: String,
    pub public_key: String,
}

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

            CREATE TABLE IF NOT EXISTS market_listings (
                id TEXT PRIMARY KEY, item_type TEXT NOT NULL, name TEXT NOT NULL,
                description TEXT NOT NULL, price REAL NOT NULL, author TEXT NOT NULL,
                rating REAL DEFAULT 0.0, downloads INTEGER DEFAULT 0, created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS market_transactions (
                id TEXT PRIMARY KEY, listing_id TEXT NOT NULL, buyer TEXT NOT NULL,
                amount REAL NOT NULL, timestamp TEXT NOT NULL,
                FOREIGN KEY (listing_id) REFERENCES market_listings(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS market_licenses (
                id TEXT PRIMARY KEY, listing_id TEXT NOT NULL, user_id TEXT NOT NULL,
                granted_at TEXT NOT NULL, expires_at TEXT,
                FOREIGN KEY (listing_id) REFERENCES market_listings(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY, username TEXT UNIQUE NOT NULL, display_name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user', created_at TEXT NOT NULL, last_login TEXT
            );

            CREATE TABLE IF NOT EXISTS teams (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL,
                owner_id TEXT NOT NULL, created_at TEXT NOT NULL,
                FOREIGN KEY (owner_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS team_members (
                id TEXT PRIMARY KEY, team_id TEXT NOT NULL, user_id TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member', joined_at TEXT NOT NULL,
                FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(team_id, user_id)
            );

            CREATE TABLE IF NOT EXISTS agent_permissions (
                id TEXT PRIMARY KEY, agent_id TEXT NOT NULL, user_id TEXT NOT NULL,
                team_id TEXT, permission TEXT NOT NULL DEFAULT 'read', granted_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY, user_id TEXT NOT NULL, action TEXT NOT NULL,
                target_type TEXT NOT NULL, target_id TEXT NOT NULL,
                details_json TEXT, created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_events (
                id TEXT PRIMARY KEY, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
                action TEXT NOT NULL, data_json TEXT NOT NULL,
                timestamp TEXT NOT NULL, device_id TEXT NOT NULL, synced INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY, name TEXT NOT NULL,
                last_seen TEXT NOT NULL, public_key TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS decision_rules (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id         TEXT NOT NULL DEFAULT 'default',
                keyword         TEXT NOT NULL,
                level           TEXT NOT NULL,
                trust_threshold REAL DEFAULT 60.0,
                auto_execute    INTEGER DEFAULT 0,
                source          TEXT DEFAULT 'learned',
                hit_count       INTEGER DEFAULT 0,
                last_used_at    TEXT,
                created_at      TEXT DEFAULT (datetime('now')),
                UNIQUE(user_id, keyword)
            );

            CREATE TABLE IF NOT EXISTS checkpoints (
                id              TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL,
                step_index      INTEGER NOT NULL,
                step_name       TEXT NOT NULL DEFAULT '',
                state_json      TEXT NOT NULL,
                metadata_json   TEXT DEFAULT '{}',
                parent_id       TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS approval_requests (
                id              TEXT PRIMARY KEY,
                action          TEXT NOT NULL,
                level           TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'pending',
                context_json    TEXT,
                requested_by    TEXT,
                responded_at    TEXT,
                response        TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS oauth_tokens (
                id              TEXT PRIMARY KEY,
                provider        TEXT NOT NULL,
                user_id         TEXT NOT NULL,
                access_token    TEXT NOT NULL,
                refresh_token   TEXT,
                expires_at      TEXT,
                scope           TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(provider, user_id)
            );

            CREATE TABLE IF NOT EXISTS privacy_rules (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern         TEXT NOT NULL,
                sensitivity     TEXT NOT NULL DEFAULT 'public',
                action          TEXT NOT NULL DEFAULT 'allow',
                created_at      TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id              TEXT PRIMARY KEY,
                user_id         TEXT NOT NULL DEFAULT 'default',
                agent_id        TEXT,
                status          TEXT DEFAULT 'active',
                context_json    TEXT DEFAULT '{}',
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT
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

    // Market listings CRUD
    pub fn save_listing(&self, listing: &Listing) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO market_listings (id, item_type, name, description, price, author, rating, downloads, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                listing.id, listing.item_type, listing.name, listing.description,
                listing.price, listing.author, listing.rating, listing.downloads, listing.created_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_listings(&self, filter: Option<&str>) -> Result<Vec<Listing>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let sql = match filter {
            Some(_) => "SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings WHERE item_type = ?1 ORDER BY created_at DESC",
            None => "SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings ORDER BY created_at DESC",
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = if let Some(f) = filter {
            stmt.query_map(params![f], map_listing_row)
                .map_err(|e| e.to_string())?
        } else {
            stmt.query_map([], map_listing_row)
                .map_err(|e| e.to_string())?
        };
        let mut listings = Vec::new();
        for row in rows {
            listings.push(row.map_err(|e| e.to_string())?);
        }
        Ok(listings)
    }

    pub fn get_listing(&self, id: &str) -> Result<Option<Listing>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, item_type, name, description, price, author, rating, downloads, created_at FROM market_listings WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(listing_from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn save_transaction(&self, tx: &Transaction) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO market_transactions (id, listing_id, buyer, amount, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tx.id, tx.listing_id, tx.buyer, tx.amount, tx.timestamp],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_license(&self, lic: &License) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO market_licenses (id, listing_id, user_id, granted_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                lic.id,
                lic.listing_id,
                lic.user_id,
                lic.granted_at,
                lic.expires_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_user_licenses(&self, user_id: &str) -> Result<Vec<License>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, listing_id, user_id, granted_at, expires_at FROM market_licenses WHERE user_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id], |row| {
                Ok(License {
                    id: row.get(0)?,
                    listing_id: row.get(1)?,
                    user_id: row.get(2)?,
                    granted_at: row.get(3)?,
                    expires_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut licenses = Vec::new();
        for row in rows {
            licenses.push(row.map_err(|e| e.to_string())?);
        }
        Ok(licenses)
    }

    pub fn update_listing_rating(
        &self,
        id: &str,
        rating: f64,
        downloads: u64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE market_listings SET rating = ?1, downloads = ?2 WHERE id = ?3",
            params![rating, downloads, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_listing(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM market_listings WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Users CRUD
    pub fn insert_user(&self, user: &UserRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO users (id, username, display_name, role, created_at, last_login)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user.id,
                user.username,
                user.display_name,
                user.role,
                user.created_at,
                user.last_login
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_user(&self, id: &str) -> Result<Option<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                username: row.get(1).map_err(|e| e.to_string())?,
                display_name: row.get(2).map_err(|e| e.to_string())?,
                role: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
                last_login: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users WHERE username = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![username]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(UserRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                username: row.get(1).map_err(|e| e.to_string())?,
                display_name: row.get(2).map_err(|e| e.to_string())?,
                role: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
                last_login: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_users(&self) -> Result<Vec<UserRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, username, display_name, role, created_at, last_login FROM users ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    role: row.get(3)?,
                    created_at: row.get(4)?,
                    last_login: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut users = Vec::new();
        for row in rows {
            users.push(row.map_err(|e| e.to_string())?);
        }
        Ok(users)
    }

    pub fn update_user_login(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE users SET last_login = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_user(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Teams CRUD
    pub fn insert_team(&self, team: &TeamRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO teams (id, name, description, owner_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                team.id,
                team.name,
                team.description,
                team.owner_id,
                team.created_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_team(&self, id: &str) -> Result<Option<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(TeamRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                description: row.get(2).map_err(|e| e.to_string())?,
                owner_id: row.get(3).map_err(|e| e.to_string())?,
                created_at: row.get(4).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_teams(&self) -> Result<Vec<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, owner_id, created_at FROM teams ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TeamRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| e.to_string())?);
        }
        Ok(teams)
    }

    pub fn list_teams_for_user(&self, user_id: &str) -> Result<Vec<TeamRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.description, t.owner_id, t.created_at
                 FROM teams t
                 INNER JOIN team_members tm ON t.id = tm.team_id
                 WHERE tm.user_id = ?1
                 ORDER BY t.created_at DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![user_id], |row| {
                Ok(TeamRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    owner_id: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut teams = Vec::new();
        for row in rows {
            teams.push(row.map_err(|e| e.to_string())?);
        }
        Ok(teams)
    }

    pub fn update_team_owner(&self, id: &str, new_owner_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE teams SET owner_id = ?1 WHERE id = ?2",
            params![new_owner_id, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_team(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM teams WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Team Members CRUD
    pub fn insert_team_member(&self, member: &TeamMemberRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO team_members (id, team_id, user_id, role, joined_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                member.id,
                member.team_id,
                member.user_id,
                member.role,
                member.joined_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMemberRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, team_id, user_id, role, joined_at FROM team_members WHERE team_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![team_id], |row| {
                Ok(TeamMemberRecord {
                    id: row.get(0)?,
                    team_id: row.get(1)?,
                    user_id: row.get(2)?,
                    role: row.get(3)?,
                    joined_at: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut members = Vec::new();
        for row in rows {
            members.push(row.map_err(|e| e.to_string())?);
        }
        Ok(members)
    }

    pub fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
            params![team_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_team_member_role(
        &self,
        team_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE team_members SET role = ?1 WHERE team_id = ?2 AND user_id = ?3",
            params![role, team_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Agent Permissions CRUD
    pub fn insert_agent_permission(&self, perm: &AgentPermissionRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agent_permissions (id, agent_id, user_id, team_id, permission, granted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                perm.id,
                perm.agent_id,
                perm.user_id,
                perm.team_id,
                perm.permission,
                perm.granted_at
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_agent_permission(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<Option<AgentPermissionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![agent_id, user_id])
            .map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some(AgentPermissionRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                agent_id: row.get(1).map_err(|e| e.to_string())?,
                user_id: row.get(2).map_err(|e| e.to_string())?,
                team_id: row.get(3).map_err(|e| e.to_string())?,
                permission: row.get(4).map_err(|e| e.to_string())?,
                granted_at: row.get(5).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_agent_permissions(
        &self,
        agent_id: &str,
    ) -> Result<Vec<AgentPermissionRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, agent_id, user_id, team_id, permission, granted_at FROM agent_permissions WHERE agent_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![agent_id], |row| {
                Ok(AgentPermissionRecord {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    user_id: row.get(2)?,
                    team_id: row.get(3)?,
                    permission: row.get(4)?,
                    granted_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut perms = Vec::new();
        for row in rows {
            perms.push(row.map_err(|e| e.to_string())?);
        }
        Ok(perms)
    }

    pub fn delete_agent_permission(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM agent_permissions WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_agent_permissions_for_user(
        &self,
        agent_id: &str,
        user_id: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM agent_permissions WHERE agent_id = ?1 AND user_id = ?2",
            params![agent_id, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Sync Events CRUD
    pub fn insert_sync_event(&self, event: &SyncEventRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO sync_events (id, entity_type, entity_id, action, data_json, timestamp, device_id, synced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id, event.entity_type, event.entity_id, event.action,
                event.data_json, event.timestamp, event.device_id, event.synced as i32
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_unsynced_events(&self) -> Result<Vec<SyncEventRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, entity_type, entity_id, action, data_json, timestamp, device_id, synced FROM sync_events WHERE synced = 0 ORDER BY timestamp ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let synced_int: i32 = row.get(7)?;
                Ok(SyncEventRecord {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    data_json: row.get(4)?,
                    timestamp: row.get(5)?,
                    device_id: row.get(6)?,
                    synced: synced_int != 0,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| e.to_string())?);
        }
        Ok(events)
    }

    pub fn mark_event_synced(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE sync_events SET synced = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_synced_events(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM sync_events WHERE synced = 1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Devices CRUD
    pub fn upsert_device(&self, device: &DeviceRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO devices (id, name, last_seen, public_key)
             VALUES (?1, ?2, ?3, ?4)",
            params![device.id, device.name, device.last_seen, device.public_key],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, name, last_seen, public_key FROM devices ORDER BY last_seen DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(DeviceRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    last_seen: row.get(2)?,
                    public_key: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut devices = Vec::new();
        for row in rows {
            devices.push(row.map_err(|e| e.to_string())?);
        }
        Ok(devices)
    }

    pub fn delete_device(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM devices WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Decision Rules CRUD
    pub fn get_decision_rules(
        &self,
        user_id: &str,
        keyword: &str,
    ) -> Result<Vec<DecisionRule>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, user_id, keyword, level, trust_threshold, auto_execute, source, hit_count, last_used_at, created_at FROM decision_rules WHERE user_id = ?1 AND keyword = ?2")
            .map_err(|e| e.to_string())?;
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
            .map_err(|e| e.to_string())?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| e.to_string())?);
        }
        Ok(rules)
    }

    pub fn upsert_decision_rule(&self, rule: &DecisionRule) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn increment_rule_hit(&self, rule_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE decision_rules SET hit_count = hit_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), rule_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn adjust_rule_threshold(&self, rule_id: i64, change: f64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE decision_rules SET trust_threshold = MAX(0.0, MIN(100.0, trust_threshold + ?1)) WHERE id = ?2",
            params![change, rule_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Audit Log CRUD
    pub fn insert_audit_log(&self, log: &AuditLogRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO audit_log (id, user_id, action, target_type, target_id, details_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![log.id, log.user_id, log.action, log.target_type, log.target_id, log.details_json, log.created_at],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn query_audit_log(
        &self,
        user_id: Option<&str>,
        action_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<AuditLogRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let sql = match (user_id, action_type) {
            (Some(_), Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 AND action = ?2 ORDER BY created_at DESC LIMIT ?3",
            (Some(_), None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, Some(_)) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log WHERE action = ?1 ORDER BY created_at DESC LIMIT ?2",
            (None, None) => "SELECT id, user_id, action, target_type, target_id, details_json, created_at FROM audit_log ORDER BY created_at DESC LIMIT ?1",
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = match (user_id, action_type) {
            (Some(uid), Some(act)) => stmt.query_map(params![uid, act, limit], map_audit_row),
            (Some(uid), None) => stmt.query_map(params![uid, limit], map_audit_row),
            (None, Some(act)) => stmt.query_map(params![act, limit], map_audit_row),
            (None, None) => stmt.query_map(params![limit], map_audit_row),
        }
        .map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(logs)
    }

    // Checkpoints CRUD
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
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO checkpoints (id, session_id, step_index, step_name, state_json, metadata_json, parent_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id, session_id, step_index, step_name, state_json, metadata_json,
                parent_id, chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_latest_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<Option<(String, String, i32, String, String, String, Option<String>)>, String> {
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

    // Approval requests CRUD
    pub fn save_approval_request(
        &self,
        id: &str,
        action: &str,
        level: &str,
        context_json: Option<&str>,
        requested_by: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO approval_requests (id, action, level, status, context_json, requested_by, created_at)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)",
            params![
                id, action, level, context_json, requested_by,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_approval_response(
        &self,
        id: &str,
        status: &str,
        response: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE approval_requests SET status = ?1, response = ?2, responded_at = ?3 WHERE id = ?4",
            params![status, response, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_approval_request(
        &self,
        id: &str,
    ) -> Result<
        Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )>,
        String,
    > {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, action, level, status, context_json, requested_by, responded_at, response FROM approval_requests WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some((
                row.get(0).map_err(|e| e.to_string())?,
                row.get(1).map_err(|e| e.to_string())?,
                row.get(2).map_err(|e| e.to_string())?,
                row.get(3).map_err(|e| e.to_string())?,
                row.get(4).map_err(|e| e.to_string())?,
                row.get(5).map_err(|e| e.to_string())?,
                row.get(6).map_err(|e| e.to_string())?,
                row.get(7).map_err(|e| e.to_string())?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn list_pending_approvals(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id FROM approval_requests WHERE status = 'pending' ORDER BY created_at ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    // OAuth tokens CRUD
    pub fn save_oauth_token(
        &self,
        id: &str,
        provider: &str,
        user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens (id, provider, user_id, access_token, refresh_token, expires_at, scope, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id, provider, user_id, access_token, refresh_token, expires_at, scope,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_oauth_token(
        &self,
        provider: &str,
        user_id: &str,
    ) -> Result<
        Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        )>,
        String,
    > {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, provider, user_id, access_token, refresh_token, expires_at, scope FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![provider, user_id])
            .map_err(|e| e.to_string())?;
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

    pub fn delete_oauth_token(&self, provider: &str, user_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM oauth_tokens WHERE provider = ?1 AND user_id = ?2",
            params![provider, user_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Privacy rules CRUD
    pub fn save_privacy_rule(
        &self,
        pattern: &str,
        sensitivity: &str,
        action: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO privacy_rules (pattern, sensitivity, action) VALUES (?1, ?2, ?3)",
            params![pattern, sensitivity, action],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_privacy_rules(&self) -> Result<Vec<(i64, String, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, pattern, sensitivity, action FROM privacy_rules")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(row.map_err(|e| e.to_string())?);
        }
        Ok(rules)
    }

    // Sessions CRUD
    pub fn save_session(
        &self,
        id: &str,
        user_id: &str,
        agent_id: Option<&str>,
        context_json: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, user_id, agent_id, status, context_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)",
            params![
                id, user_id, agent_id, context_json,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_session(
        &self,
        id: &str,
    ) -> Result<Option<(String, String, Option<String>, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, agent_id, context_json, status FROM sessions WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            Ok(Some((
                row.get(0).map_err(|e| e.to_string())?,
                row.get(1).map_err(|e| e.to_string())?,
                row.get(2).map_err(|e| e.to_string())?,
                row.get(3).map_err(|e| e.to_string())?,
                row.get(4).map_err(|e| e.to_string())?,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn update_session_context(&self, id: &str, context_json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE sessions SET context_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![context_json, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
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

fn map_listing_row(row: &rusqlite::Row) -> rusqlite::Result<Listing> {
    Ok(Listing {
        id: row.get(0)?,
        item_type: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        price: row.get(4)?,
        author: row.get(5)?,
        rating: row.get(6)?,
        downloads: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn listing_from_row(row: &rusqlite::Row) -> Result<Listing, String> {
    Ok(Listing {
        id: row.get(0).map_err(|e| e.to_string())?,
        item_type: row.get(1).map_err(|e| e.to_string())?,
        name: row.get(2).map_err(|e| e.to_string())?,
        description: row.get(3).map_err(|e| e.to_string())?,
        price: row.get(4).map_err(|e| e.to_string())?,
        author: row.get(5).map_err(|e| e.to_string())?,
        rating: row.get(6).map_err(|e| e.to_string())?,
        downloads: row.get(7).map_err(|e| e.to_string())?,
        created_at: row.get(8).map_err(|e| e.to_string())?,
    })
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

    #[test]
    fn test_market_storage() {
        let storage = Storage::new_in_memory().unwrap();

        let listing = Listing {
            id: "listing-test-1".to_string(),
            item_type: "tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            price: 0.5,
            author: "tester".to_string(),
            rating: 4.0,
            downloads: 100,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        storage.save_listing(&listing).unwrap();

        let got = storage.get_listing("listing-test-1").unwrap().unwrap();
        assert_eq!(got.name, "Test Tool");
        assert_eq!(got.price, 0.5);

        let listings = storage.list_listings(None).unwrap();
        assert_eq!(listings.len(), 1);

        let filtered = storage.list_listings(Some("tool")).unwrap();
        assert_eq!(filtered.len(), 1);

        let filtered_empty = storage.list_listings(Some("knowledge")).unwrap();
        assert_eq!(filtered_empty.len(), 0);

        storage
            .update_listing_rating("listing-test-1", 4.5, 101)
            .unwrap();
        let updated = storage.get_listing("listing-test-1").unwrap().unwrap();
        assert_eq!(updated.rating, 4.5);
        assert_eq!(updated.downloads, 101);

        let tx = Transaction {
            id: "tx-test-1".to_string(),
            listing_id: "listing-test-1".to_string(),
            buyer: "user-1".to_string(),
            amount: 0.5,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        storage.save_transaction(&tx).unwrap();

        let lic = License {
            id: "lic-test-1".to_string(),
            listing_id: "listing-test-1".to_string(),
            user_id: "user-1".to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
            expires_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        storage.save_license(&lic).unwrap();

        let user_lics = storage.get_user_licenses("user-1").unwrap();
        assert_eq!(user_lics.len(), 1);
        assert_eq!(user_lics[0].listing_id, "listing-test-1");

        let no_lics = storage.get_user_licenses("unknown").unwrap();
        assert_eq!(no_lics.len(), 0);

        storage.delete_listing("listing-test-1").unwrap();
        assert!(storage.get_listing("listing-test-1").unwrap().is_none());
    }
}
