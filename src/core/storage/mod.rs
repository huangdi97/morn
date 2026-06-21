//! storage — Provides SQLite-backed persistence for agents, tasks, settings, and sync data.
use crate::core::error::MornError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

mod agents;
mod costs;
mod decision_rules;
mod governance;
mod market;
mod memory;
mod oauth;
mod proactive;
mod sessions;
mod settings;
mod sync;
mod tasks;
mod users;

pub use agents::*;
pub use costs::*;
pub use governance::*;
pub use memory::*;
pub use oauth::*;
pub use proactive::*;
pub use sessions::*;
pub use sync::*;
pub use tasks::*;
pub use users::*;

impl Storage {
    /// Opens or creates the default persistent SQLite database under the OS data directory.
    pub fn new() -> Result<Self, MornError> {
        Self::with_path(default_database_path()?)
    }

    /// Opens or creates a SQLite database at `path` and returns initialized storage.
    pub fn with_path(path: impl AsRef<Path>) -> Result<Self, MornError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create storage directory {}: {}",
                    parent.display(),
                    e
                )
            })?;
        }

        let conn = Connection::open(path).map_err(|e| MornError::Internal(e.to_string()))?;
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    /// Returns a locked connection handle with consistent error mapping.
    pub(crate) fn conn(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>, MornError> {
        self.conn
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))
    }

    /// Creates an in-memory SQLite database and returns initialized storage for tests or ephemeral use.
    pub fn new_in_memory() -> Result<Self, MornError> {
        let conn = Connection::open_in_memory().map_err(|e| MornError::Internal(e.to_string()))?;
        let storage = Storage {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, component_type TEXT NOT NULL,
                config_json TEXT, status TEXT DEFAULT 'inactive',
                trust_score REAL DEFAULT 70.0, created_at TEXT NOT NULL, updated_at TEXT,
                current_version TEXT DEFAULT '0.1.0',
                update_available INTEGER DEFAULT 0
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
                latency_ms INTEGER, error_msg TEXT, token_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );

            CREATE TABLE IF NOT EXISTS daily_costs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                model TEXT NOT NULL,
                token_count INTEGER NOT NULL DEFAULT 0,
                cost_usd REAL NOT NULL DEFAULT 0.0,
                call_count INTEGER NOT NULL DEFAULT 0,
                UNIQUE(date, agent_id, model)
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
                description TEXT NOT NULL, price REAL, author TEXT NOT NULL,
                rating REAL DEFAULT 0.0, downloads INTEGER DEFAULT 0, created_at TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT '1.0.0', screenshots TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT 'general',
                price_model TEXT NOT NULL DEFAULT 'free',
                requires TEXT NOT NULL DEFAULT '',
                verified INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT ''
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

            CREATE TABLE IF NOT EXISTS market_agent_versions (
                id TEXT PRIMARY KEY,
                listing_id TEXT NOT NULL,
                version TEXT NOT NULL,
                data_json TEXT NOT NULL,
                changelog TEXT DEFAULT '',
                created_at TEXT NOT NULL,
                FOREIGN KEY (listing_id) REFERENCES market_listings(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS market_reviews (
                id TEXT PRIMARY KEY,
                listing_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
                comment TEXT NOT NULL,
                created_at TEXT NOT NULL,
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

            CREATE TABLE IF NOT EXISTS proactive_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                trigger_type TEXT NOT NULL,
                trigger_config TEXT NOT NULL,
                action TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_triggered_at INTEGER,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS memory_entries (
                id TEXT PRIMARY KEY,
                layer TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                metadata TEXT DEFAULT '{}',
                priority REAL DEFAULT 0.0,
                expires_at INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory_entries(agent_id);
            CREATE INDEX IF NOT EXISTS idx_memory_layer ON memory_entries(layer);
            ",
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;

        conn.execute_batch(
            "ALTER TABLE executions ADD COLUMN token_count INTEGER DEFAULT 0;",
        )
        .ok();

        Ok(())
    }

    /// Runs a quick health check against the SQLite database.
    ///
    /// Returns `Ok(())` when the database responds to a `PRAGMA quick_check`
    /// with the value `"ok"`, and an error otherwise.
    pub fn check_health(&self) -> Result<(), MornError> {
        let conn = self.conn()?;
        let result: String = conn
            .pragma_query_value(None, "quick_check", |row| row.get(0))
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if result == "ok" {
            Ok(())
        } else {
            Err(MornError::Internal(format!(
                "Database integrity issue: {}",
                result
            )))
        }
    }
}

fn default_database_path() -> Result<PathBuf, MornError> {
    let data_dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("morn");
    Ok(data_dir.join("morn.sqlite3"))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
