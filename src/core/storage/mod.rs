//! storage — Provides SQLite-backed persistence for agents, tasks, settings, and sync data.
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

mod agents;
mod decision_rules;
mod governance;
mod market;
mod oauth;
mod sessions;
mod settings;
mod sync;
mod tasks;
mod users;

pub use agents::*;
pub use governance::*;
pub use oauth::*;
pub use sessions::*;
pub use sync::*;
pub use tasks::*;
pub use users::*;

impl Storage {
    /// Opens or creates a SQLite database at `path` and returns initialized storage.
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    /// Creates an in-memory SQLite database and returns initialized storage for tests or ephemeral use.
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

            CREATE TABLE IF NOT EXISTS settings (
                key             TEXT PRIMARY KEY,
                value           TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{License, Listing, Transaction};

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
