//! store — ControlPlaneStore: durable store for work / conditions / events.
//!
//! First version uses the existing SQLite persistence (Goal §6.1: 第一版优先
//! 利用现有 DB). Tables are prefixed `cp_` to stay separate from the legacy
//! tables; protocol semantics are preserved regardless of physical layout.

use super::ddl::CONTROL_TABLES_SQL;
use crate::core::error::MornError;
use crate::core::storage::Storage;
use crate::work::{now_iso, Condition, ConditionStatus, ConditionType, Work, WorkStatus};
use rusqlite::params;
use rusqlite::OptionalExtension;

/// Durable control plane store over the shared SQLite database.
#[derive(Debug, Clone)]
pub struct ControlPlaneStore {
    storage: Storage,
}

impl ControlPlaneStore {
    pub fn new(storage: Storage) -> Result<Self, MornError> {
        let store = Self { storage };
        store.init_tables()?;
        Ok(store)
    }

    /// Opens (or creates) a file-backed store — the path used in production
    /// and in crash/restart tests.
    pub fn with_path(path: impl AsRef<std::path::Path>) -> Result<Self, MornError> {
        Self::new(Storage::with_path(path)?)
    }

    /// In-memory store for tests. NOTE: the underlying pool reuses one
    /// connection, so an in-memory DB survives as long as the pool lives.
    pub fn in_memory() -> Result<Self, MornError> {
        Self::new(Storage::new_in_memory()?)
    }

    pub fn conn(
        &self,
    ) -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, MornError> {
        self.storage.conn()
    }

    fn init_tables(&self) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute_batch(CONTROL_TABLES_SQL).map_err(|e| {
            MornError::Internal(format!("failed to create control plane tables: {}", e))
        })?;
        Ok(())
    }

    /// Inserts a Work (idempotent by id: an existing Work is never silently
    /// overwritten — callers must use update_status for status changes).
    pub fn insert_work(&self, work: &Work) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_work (id, kind, api_version, generation, spec_json, status_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    work.id,
                    work.kind,
                    work.api_version,
                    work.generation,
                    serde_json::to_string(&work.spec).map_err(|e| MornError::Internal(e.to_string()))?,
                    serde_json::to_string(&work.status).map_err(|e| MornError::Internal(e.to_string()))?,
                    now_iso(),
                    work.status.updated_at
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Fetches a Work with its conditions merged from cp_work_condition.
    pub fn get_work(&self, id: &str) -> Result<Option<Work>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, kind, api_version, generation, spec_json, status_json FROM cp_work WHERE id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let row = stmt
            .query_row(params![id], |row| {
                let spec_json: String = row.get(4)?;
                let status_json: String = row.get(5)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    spec_json,
                    status_json,
                ))
            })
            .optional()
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let Some((id, kind, api_version, generation, spec_json, status_json)) = row else {
            return Ok(None);
        };
        let spec: crate::work::WorkSpec =
            serde_json::from_str(&spec_json).map_err(|e| MornError::Internal(e.to_string()))?;
        let mut status: WorkStatus =
            serde_json::from_str(&status_json).map_err(|e| MornError::Internal(e.to_string()))?;
        status.conditions = self.get_conditions(&id)?;

        Ok(Some(Work {
            id,
            kind,
            api_version,
            generation,
            spec,
            status,
        }))
    }

    /// Persists the current status projection of a Work (current projection is
    /// mutable by contract; historical facts live in cp_semantic_event).
    pub fn update_work_status(&self, id: &str, status: &WorkStatus) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE cp_work SET status_json = ?2, updated_at = ?3 WHERE id = ?1",
            params![
                id,
                serde_json::to_string(status).map_err(|e| MornError::Internal(e.to_string()))?,
                now_iso()
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        for condition in &status.conditions {
            self.upsert_condition(id, condition)?;
        }
        Ok(())
    }

    /// Upserts one condition (semantic gate) for a Work.
    pub fn upsert_condition(&self, work_id: &str, condition: &Condition) -> Result<(), MornError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO cp_work_condition (work_id, type, status, reason, observed_at, source_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(work_id, type) DO UPDATE SET
                status = excluded.status,
                reason = excluded.reason,
                observed_at = excluded.observed_at,
                source_json = excluded.source_json",
            params![
                work_id,
                condition.type_.as_str(),
                serde_json::to_string(&condition.status).map_err(|e| MornError::Internal(e.to_string()))?,
                condition.reason,
                condition.observed_at,
                condition
                    .source_ref
                    .as_ref()
                    .map(|v| v.to_string())
            ],
        )
        .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Loads all conditions for a Work.
    pub fn get_conditions(&self, work_id: &str) -> Result<Vec<Condition>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT type, status, reason, observed_at, source_json FROM cp_work_condition WHERE work_id = ?1")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![work_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let mut conditions = Vec::new();
        for row in rows {
            let (type_str, status_str, reason, observed_at, source_json) = row?;
            let type_ = ConditionType::ALL
                .iter()
                .find(|t| t.as_str() == type_str)
                .copied()
                .ok_or_else(|| {
                    MornError::Internal(format!("unknown condition type: {type_str}"))
                })?;
            let status: ConditionStatus = serde_json::from_str(&status_str)
                .map_err(|e| MornError::Internal(e.to_string()))?;
            conditions.push(Condition {
                type_,
                status,
                reason,
                observed_at,
                source_ref: source_json.and_then(|s| serde_json::from_str(&s).ok()),
            });
        }
        Ok(conditions)
    }

    // ---- Semantic event history ----

    /// Appends a semantic event (idempotent by event id). Historical facts are
    /// never silently rewritten; supersede/retract are explicit fields.
    pub fn append_event(
        &self,
        event: &crate::control::projection::StoredEvent,
    ) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_semantic_event
                 (id, work_id, event_type, payload_json, supersedes, retracts, valid_time, recorded_at, contract_version, schema_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    event.id,
                    event.work_id,
                    event.event_type,
                    event.payload_json,
                    event.supersedes,
                    event.retracts,
                    event.valid_time,
                    event.recorded_at,
                    crate::work::SEMANTIC_CONTRACT_VERSION,
                    crate::control::EVENT_SCHEMA_VERSION
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Loads events for a Work in recorded order (replay input).
    pub fn get_events(
        &self,
        work_id: &str,
    ) -> Result<Vec<crate::control::projection::StoredEvent>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, work_id, event_type, payload_json, supersedes, retracts, valid_time, recorded_at FROM cp_semantic_event WHERE work_id = ?1 ORDER BY recorded_at, id")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![work_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let (
                id,
                work_id,
                event_type,
                payload_json,
                supersedes,
                retracts,
                valid_time,
                recorded_at,
            ) = row?;
            events.push(crate::control::projection::StoredEvent {
                id,
                work_id,
                event_type,
                payload_json,
                supersedes,
                retracts,
                valid_time,
                recorded_at,
            });
        }
        Ok(events)
    }
}
