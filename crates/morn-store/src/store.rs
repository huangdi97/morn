//! MornStore: SQLite-backed persistence for the core Morn records.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ArtifactId, ArtifactVersionId, WorkspaceId};
use morn_kernel::ledger::LedgerEntry;

/// The Morn SQLite store. All records are stored as versioned, typed JSON rows.
pub struct MornStore {
    conn: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEvent {
    pub event_id: String,
    pub workspace_id: String,
    pub event_type: String,
    pub payload: String,
    pub created_at: i64,
    pub dispatched_at: Option<i64>,
}

const SCHEMA_VERSION: i64 = 3;

impl MornStore {
    /// Open (creating if needed) a store at `path` and run migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| Error::internal(e.to_string()))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| Error::internal(e.to_string()))?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| Error::internal(e.to_string()))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory store (for tests).
    pub fn open_in_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = &self.conn;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS morn_records (
                kind TEXT NOT NULL,
                id TEXT NOT NULL,
                workspace_id TEXT NOT NULL DEFAULT '',
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                immutable INTEGER NOT NULL DEFAULT 0,
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                UNIQUE(kind, id)
            );
            CREATE INDEX IF NOT EXISTS idx_records_workspace ON morn_records(kind, workspace_id);
            CREATE TABLE IF NOT EXISTS ledger_entries (
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id TEXT NOT NULL,
                entry_id TEXT NOT NULL UNIQUE,
                event_type TEXT NOT NULL,
                subject TEXT NOT NULL,
                summary TEXT NOT NULL,
                principal TEXT NOT NULL,
                payload_hash TEXT,
                refs TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_ledger_workspace ON ledger_entries(workspace_id);

            CREATE TABLE IF NOT EXISTS control_event_inbox (
                event_id TEXT NOT NULL PRIMARY KEY,
                source TEXT NOT NULL,
                recorded_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS control_event_outbox (
                event_id TEXT NOT NULL PRIMARY KEY,
                workspace_id TEXT NOT NULL DEFAULT '',
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                dispatched_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_outbox_pending
                ON control_event_outbox(dispatched_at, created_at);
            "#,
        )
        .map_err(|e| Error::internal(e.to_string()))?;

        let current: Option<i64> = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| Error::internal(e.to_string()))?;
        match current {
            None => {
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    params![SCHEMA_VERSION],
                )
                .map_err(|e| Error::internal(e.to_string()))?;
            }
            Some(v) if v < SCHEMA_VERSION => {
                // v1/v2 -> v3: immutable-history column is idempotent; inbox/outbox
                // tables are created above with IF NOT EXISTS.
                let _ = conn.execute(
                    "ALTER TABLE morn_records ADD COLUMN immutable INTEGER NOT NULL DEFAULT 0",
                    [],
                );
                conn.execute(
                    "UPDATE schema_version SET version = ?1 WHERE version = ?2",
                    params![SCHEMA_VERSION, v],
                )
                .map_err(|e| Error::internal(e.to_string()))?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| {
                r.get(0)
            })
            .map_err(|e| Error::internal(e.to_string()))
    }

    // ---- durable controller inbox/outbox ----

    /// Atomically claim an inbound event id. Returns true only to the first
    /// observer; duplicate/redelivered events are safe to ignore.
    pub fn claim_inbound_event(
        &self,
        event_id: &str,
        source: &str,
        recorded_at: i64,
    ) -> Result<bool> {
        if event_id.trim().is_empty() || source.trim().is_empty() {
            return Err(Error::validation(
                "inbound event claim requires non-empty id and source",
            ));
        }
        let inserted = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO control_event_inbox (event_id, source, recorded_at) VALUES (?1, ?2, ?3)",
                params![event_id, source, recorded_at],
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(inserted == 1)
    }

    /// Enqueue an outbound event exactly once by event id. The caller commits
    /// canonical state first, then records a delivery intent; dispatcher retry
    /// never requires fabricating a new event id.
    pub fn enqueue_outbox_event(
        &self,
        event_id: &str,
        workspace_id: &str,
        event_type: &str,
        payload: &str,
        created_at: i64,
    ) -> Result<bool> {
        if event_id.trim().is_empty() || event_type.trim().is_empty() {
            return Err(Error::validation(
                "outbox event requires non-empty event id and type",
            ));
        }
        let inserted = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO control_event_outbox
                 (event_id, workspace_id, event_type, payload, created_at, dispatched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                params![event_id, workspace_id, event_type, payload, created_at],
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(inserted == 1)
    }

    pub fn pending_outbox_events(&self, limit: usize) -> Result<Vec<OutboxEvent>> {
        let limit = i64::try_from(limit)
            .map_err(|_| Error::validation("outbox limit is too large"))?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT event_id, workspace_id, event_type, payload, created_at, dispatched_at
                 FROM control_event_outbox
                 WHERE dispatched_at IS NULL
                 ORDER BY created_at, event_id
                 LIMIT ?1",
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok(OutboxEvent {
                    event_id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    event_type: row.get(2)?,
                    payload: row.get(3)?,
                    created_at: row.get(4)?,
                    dispatched_at: row.get(5)?,
                })
            })
            .map_err(|e| Error::internal(e.to_string()))?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| Error::internal(e.to_string()))?);
        }
        Ok(events)
    }

    pub fn mark_outbox_dispatched(&self, event_id: &str, dispatched_at: i64) -> Result<()> {
        let updated = self
            .conn
            .execute(
                "UPDATE control_event_outbox
                 SET dispatched_at = ?2
                 WHERE event_id = ?1 AND dispatched_at IS NULL",
                params![event_id, dispatched_at],
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        if updated == 0 {
            return Err(Error::not_found(format!(
                "pending outbox event {event_id}"
            )));
        }
        Ok(())
    }

    // ---- generic typed records ----

    pub fn save_record<T: serde::Serialize>(
        &self,
        kind: &str,
        id: &str,
        workspace_id: &str,
        created_at: i64,
        record: &T,
    ) -> Result<()> {
        let payload = serde_json::to_string(record).map_err(|e| Error::internal(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO morn_records (kind, id, workspace_id, payload, created_at) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(kind, id) DO UPDATE SET payload = excluded.payload, workspace_id = excluded.workspace_id",
                params![kind, id, workspace_id, payload, created_at],
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }

    /// Save an immutable record (receipts, release history, decisions).
    /// Re-saving the same (kind, id) is rejected instead of silently overwritten.
    pub fn save_record_immutable<T: serde::Serialize>(
        &self,
        kind: &str,
        id: &str,
        workspace_id: &str,
        created_at: i64,
        record: &T,
    ) -> Result<()> {
        let payload = serde_json::to_string(record).map_err(|e| Error::internal(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO morn_records (kind, id, workspace_id, payload, created_at, immutable) VALUES (?1, ?2, ?3, ?4, ?5, 1)",
                params![kind, id, workspace_id, payload, created_at],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    Error::conflict(format!("immutable record {kind}/{id} already exists"))
                } else {
                    Error::internal(e.to_string())
                }
            })?;
        Ok(())
    }
    pub fn load_record<T: serde::de::DeserializeOwned>(
        &self,
        kind: &str,
        id: &str,
    ) -> Result<Option<T>> {
        let row: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM morn_records WHERE kind = ?1 AND id = ?2",
                params![kind, id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| Error::internal(e.to_string()))?;
        match row {
            Some(payload) => {
                let record =
                    serde_json::from_str(&payload).map_err(|e| Error::internal(e.to_string()))?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    pub fn load_records<T: serde::de::DeserializeOwned>(&self, kind: &str) -> Result<Vec<T>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM morn_records WHERE kind = ?1 ORDER BY seq")
            .map_err(|e| Error::internal(e.to_string()))?;
        let rows = stmt
            .query_map([kind], |r| r.get::<_, String>(0))
            .map_err(|e| Error::internal(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            let payload = row.map_err(|e| Error::internal(e.to_string()))?;
            let record =
                serde_json::from_str(&payload).map_err(|e| Error::internal(e.to_string()))?;
            out.push(record);
        }
        Ok(out)
    }

    pub fn load_records_in_workspace<T: serde::de::DeserializeOwned>(
        &self,
        kind: &str,
        workspace_id: &str,
    ) -> Result<Vec<T>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM morn_records WHERE kind = ?1 AND workspace_id = ?2 ORDER BY seq")
            .map_err(|e| Error::internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![kind, workspace_id], |r| r.get::<_, String>(0))
            .map_err(|e| Error::internal(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            let payload = row.map_err(|e| Error::internal(e.to_string()))?;
            let record =
                serde_json::from_str(&payload).map_err(|e| Error::internal(e.to_string()))?;
            out.push(record);
        }
        Ok(out)
    }

    // ---- ledger (append-only, monotonic) ----

    pub fn append_ledger_entry(&self, entry: &LedgerEntry) -> Result<()> {
        let refs =
            serde_json::to_string(&entry.refs).map_err(|e| Error::internal(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO ledger_entries (workspace_id, entry_id, event_type, subject, summary, principal, payload_hash, refs, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.workspace_id.to_string(),
                    entry.id.to_string(),
                    entry.event_type,
                    entry.subject,
                    entry.summary,
                    entry.principal,
                    entry.payload_hash,
                    refs,
                    entry.created_at.millis()
                ],
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }

    pub fn load_ledger(&self, workspace_id: &WorkspaceId) -> Result<Vec<LedgerEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT workspace_id, entry_id, event_type, subject, summary, principal, payload_hash, refs, created_at, seq
                 FROM ledger_entries WHERE workspace_id = ?1 ORDER BY seq",
            )
            .map_err(|e| Error::internal(e.to_string()))?;
        let rows = stmt
            .query_map([workspace_id.to_string()], |r| {
                let refs_json: String = r.get(7)?;
                Ok(LedgerEntry {
                    workspace_id: WorkspaceId::new(r.get::<_, String>(0)?),
                    id: morn_kernel::ids::LedgerEntryId::new(r.get::<_, String>(1)?),
                    seq: r.get::<_, i64>(9)? as u64,
                    event_type: r.get(2)?,
                    subject: r.get(3)?,
                    summary: r.get(4)?,
                    principal: r.get(5)?,
                    payload_hash: r.get(6)?,
                    refs: serde_json::from_str(&refs_json).unwrap_or_default(),
                    created_at: morn_kernel::time::Timestamp::from_millis(r.get(8)?),
                })
            })
            .map_err(|e| Error::internal(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| Error::internal(e.to_string()))?);
        }
        Ok(out)
    }

    pub fn count_ledger(&self) -> Result<u64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM ledger_entries", [], |r| r.get(0))
            .map_err(|e| Error::internal(e.to_string()))
    }

    // ---- typed helpers for the most important records ----

    pub fn save_workspace(&self, ws: &morn_kernel::workspace::Workspace) -> Result<()> {
        self.save_record(
            "workspace",
            ws.id.as_str(),
            ws.id.as_str(),
            ws.created_at.millis(),
            ws,
        )
    }

    pub fn load_workspace(
        &self,
        id: &WorkspaceId,
    ) -> Result<Option<morn_kernel::workspace::Workspace>> {
        self.load_record("workspace", id.as_str())
    }

    pub fn list_workspaces(&self) -> Result<Vec<morn_kernel::workspace::Workspace>> {
        self.load_records("workspace")
    }

    pub fn save_object(&self, obj: &morn_world::object::Object) -> Result<()> {
        self.save_record(
            "object",
            obj.id.as_str(),
            obj.workspace_id.as_str(),
            obj.created_at.millis(),
            obj,
        )
    }

    pub fn load_object(
        &self,
        id: &morn_kernel::ids::ObjectId,
    ) -> Result<Option<morn_world::object::Object>> {
        self.load_record("object", id.as_str())
    }

    pub fn list_objects(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_world::object::Object>> {
        self.load_records_in_workspace("object", workspace_id.as_str())
    }

    pub fn save_snapshot(&self, snap: &morn_world::state::StateSnapshot) -> Result<()> {
        self.save_record(
            "snapshot",
            snap.id.as_str(),
            snap.workspace_id.as_str(),
            snap.created_at.millis(),
            snap,
        )
    }

    pub fn load_snapshots_for(
        &self,
        object_id: &morn_kernel::ids::ObjectId,
    ) -> Result<Vec<morn_world::state::StateSnapshot>> {
        let all: Vec<morn_world::state::StateSnapshot> = self.load_records("snapshot")?;
        Ok(all
            .into_iter()
            .filter(|s| s.object_id == *object_id)
            .collect())
    }

    pub fn save_artifact(&self, artifact: &morn_artifact::artifact::Artifact) -> Result<()> {
        self.save_record(
            "artifact",
            artifact.id.as_str(),
            artifact.workspace_id.as_str(),
            artifact.created_at.millis(),
            artifact,
        )
    }

    pub fn save_artifact_version(
        &self,
        version: &morn_artifact::artifact::ArtifactVersion,
    ) -> Result<()> {
        self.save_record(
            "artifact_version",
            version.id.as_str(),
            "",
            version.created_at.millis(),
            version,
        )
    }

    pub fn load_artifact_versions(
        &self,
        artifact_id: &ArtifactId,
    ) -> Result<Vec<morn_artifact::artifact::ArtifactVersion>> {
        let all: Vec<morn_artifact::artifact::ArtifactVersion> =
            self.load_records("artifact_version")?;
        Ok(all
            .into_iter()
            .filter(|v| v.artifact_id == *artifact_id)
            .collect())
    }

    pub fn load_artifact_version(
        &self,
        id: &ArtifactVersionId,
    ) -> Result<Option<morn_artifact::artifact::ArtifactVersion>> {
        self.load_record("artifact_version", id.as_str())
    }

    pub fn save_work_package(&self, wp: &morn_work::work_package::WorkPackage) -> Result<()> {
        self.save_record(
            "work_package",
            wp.id.as_str(),
            wp.workspace_id.as_str(),
            wp.created_at.millis(),
            wp,
        )
    }

    pub fn load_work_package(
        &self,
        id: &morn_kernel::ids::WorkPackageId,
    ) -> Result<Option<morn_work::work_package::WorkPackage>> {
        self.load_record("work_package", id.as_str())
    }

    pub fn list_work_packages(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_work::work_package::WorkPackage>> {
        self.load_records_in_workspace("work_package", workspace_id.as_str())
    }

    pub fn save_checkpoint(&self, cp: &morn_work::checkpoint::Checkpoint) -> Result<()> {
        self.save_record(
            "checkpoint",
            cp.id.as_str(),
            cp.workspace_id.as_str(),
            cp.created_at.millis(),
            cp,
        )
    }

    pub fn load_checkpoints(
        &self,
        work_package_id: &morn_kernel::ids::WorkPackageId,
    ) -> Result<Vec<morn_work::checkpoint::Checkpoint>> {
        let all: Vec<morn_work::checkpoint::Checkpoint> = self.load_records("checkpoint")?;
        Ok(all
            .into_iter()
            .filter(|c| c.work_package_id == *work_package_id)
            .collect())
    }

    // ---- Goal 4: Goal 3 production persistence repositories ----

    pub fn save_certification_spec(
        &self,
        s: &morn_assurance::certification::CertificationSpec,
    ) -> Result<()> {
        self.save_record(
            "certification_spec",
            s.id.as_str(),
            "",
            s.created_at.millis(),
            s,
        )
    }

    pub fn load_certification_specs(
        &self,
    ) -> Result<Vec<morn_assurance::certification::CertificationSpec>> {
        self.load_records("certification_spec")
    }

    pub fn save_certification_run(
        &self,
        r: &morn_assurance::certification::CertificationRun,
    ) -> Result<()> {
        self.save_record(
            "certification_run",
            r.id.as_str(),
            "",
            r.created_at.millis(),
            r,
        )
    }

    pub fn load_certification_runs(
        &self,
    ) -> Result<Vec<morn_assurance::certification::CertificationRun>> {
        self.load_records("certification_run")
    }

    pub fn save_certification_decision(
        &self,
        d: &morn_assurance::certification::CertificationDecision,
    ) -> Result<()> {
        self.save_record_immutable(
            "certification_decision",
            d.id.as_str(),
            "",
            d.created_at.millis(),
            d,
        )
    }

    pub fn load_certification_decisions(
        &self,
    ) -> Result<Vec<morn_assurance::certification::CertificationDecision>> {
        self.load_records("certification_decision")
    }

    pub fn save_certified_capability(
        &self,
        c: &morn_assurance::certification::CertifiedWorkCapability,
    ) -> Result<()> {
        self.save_record(
            "certified_capability",
            c.id.as_str(),
            "",
            c.created_at.millis(),
            c,
        )
    }

    pub fn load_certified_capabilities(
        &self,
    ) -> Result<Vec<morn_assurance::certification::CertifiedWorkCapability>> {
        self.load_records("certified_capability")
    }

    pub fn save_capability_release(
        &self,
        r: &morn_assurance::certification::CapabilityRelease,
    ) -> Result<()> {
        self.save_record_immutable(
            "capability_release",
            r.id.as_str(),
            "",
            r.released_at.millis(),
            r,
        )
    }

    pub fn load_capability_releases(
        &self,
    ) -> Result<Vec<morn_assurance::certification::CapabilityRelease>> {
        self.load_records("capability_release")
    }

    pub fn save_managed_run(&self, r: &morn_assurance::managed_work::ManagedWorkRun) -> Result<()> {
        self.save_record(
            "managed_run",
            r.id.as_str(),
            r.workspace_id.as_str(),
            r.created_at.millis(),
            r,
        )
    }

    pub fn load_managed_runs(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_assurance::managed_work::ManagedWorkRun>> {
        self.load_records_in_workspace("managed_run", workspace_id.as_str())
    }

    pub fn save_delivery_receipt(
        &self,
        r: &morn_assurance::managed_work::DeliveryReceipt,
    ) -> Result<()> {
        self.save_record_immutable("delivery_receipt", r.id.as_str(), "", 0, r)
    }

    pub fn load_delivery_receipts(
        &self,
    ) -> Result<Vec<morn_assurance::managed_work::DeliveryReceipt>> {
        self.load_records("delivery_receipt")
    }

    pub fn save_acceptance_decision(
        &self,
        a: &morn_assurance::managed_work::AcceptanceDecision,
    ) -> Result<()> {
        self.save_record_immutable(
            "acceptance_decision",
            a.id.as_str(),
            "",
            a.created_at.millis(),
            a,
        )
    }

    pub fn load_acceptance_decisions(
        &self,
    ) -> Result<Vec<morn_assurance::managed_work::AcceptanceDecision>> {
        self.load_records("acceptance_decision")
    }

    pub fn save_replacement_comparison(
        &self,
        c: &morn_assurance::replacement::ReplacementComparison,
    ) -> Result<()> {
        self.save_record("replacement_comparison", c.id.as_str(), "", 0, c)
    }

    pub fn load_replacement_comparisons(
        &self,
    ) -> Result<Vec<morn_assurance::replacement::ReplacementComparison>> {
        self.load_records("replacement_comparison")
    }

    pub fn save_replacement_record(
        &self,
        r: &morn_assurance::replacement::ReplacementRecord,
    ) -> Result<()> {
        self.save_record_immutable(
            "replacement_record",
            r.id.as_str(),
            "",
            r.created_at.millis(),
            r,
        )
    }

    pub fn load_replacement_records(
        &self,
    ) -> Result<Vec<morn_assurance::replacement::ReplacementRecord>> {
        self.load_records("replacement_record")
    }

    pub fn save_r4_candidate(
        &self,
        c: &morn_assurance::replacement::PartialReplaceCandidate,
    ) -> Result<()> {
        self.save_record("r4_candidate", c.id.as_str(), "", c.created_at.millis(), c)
    }

    pub fn load_r4_candidates(
        &self,
    ) -> Result<Vec<morn_assurance::replacement::PartialReplaceCandidate>> {
        self.load_records("r4_candidate")
    }

    pub fn save_flywheel_pattern(&self, p: &morn_evolution::flywheel::Pattern) -> Result<()> {
        self.save_record(
            "flywheel_pattern",
            p.id.as_str(),
            "",
            p.created_at.millis(),
            p,
        )
    }

    pub fn load_flywheel_patterns(&self) -> Result<Vec<morn_evolution::flywheel::Pattern>> {
        self.load_records("flywheel_pattern")
    }

    pub fn save_flywheel_candidate(
        &self,
        c: &morn_evolution::flywheel::FlywheelCandidate,
    ) -> Result<()> {
        self.save_record(
            "flywheel_candidate",
            c.id.as_str(),
            c.workspace_id.as_str(),
            c.created_at.millis(),
            c,
        )
    }

    pub fn load_flywheel_candidates(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_evolution::flywheel::FlywheelCandidate>> {
        self.load_records_in_workspace("flywheel_candidate", workspace_id.as_str())
    }

    pub fn save_distillation_candidate(
        &self,
        c: &morn_evolution::distillation::DistillationCandidate,
    ) -> Result<()> {
        self.save_record("distillation_candidate", c.id.as_str(), "", 0, c)
    }

    pub fn load_distillation_candidates(
        &self,
    ) -> Result<Vec<morn_evolution::distillation::DistillationCandidate>> {
        self.load_records("distillation_candidate")
    }

    pub fn save_opint_episode(&self, e: &morn_opint::episode::OperationalEpisode) -> Result<()> {
        self.save_record(
            "opint_episode",
            e.id.as_str(),
            e.workspace_id.as_str(),
            e.started_at.millis(),
            e,
        )
    }

    pub fn load_opint_episodes(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_opint::episode::OperationalEpisode>> {
        self.load_records_in_workspace("opint_episode", workspace_id.as_str())
    }

    pub fn save_predictor_states(
        &self,
        states: &[morn_opint::predictor::PredictorSnapshot],
    ) -> Result<()> {
        for s in states {
            self.save_record(
                "predictor_state",
                s.spec.id.as_str(),
                "",
                s.spec.created_at.millis(),
                s,
            )?;
        }
        Ok(())
    }

    pub fn load_predictor_states(&self) -> Result<Vec<morn_opint::predictor::PredictorSnapshot>> {
        self.load_records("predictor_state")
    }

    pub fn save_prediction(&self, p: &morn_opint::predictor::Prediction) -> Result<()> {
        self.save_record("prediction", p.id.as_str(), "", p.generated_at.millis(), p)
    }

    pub fn load_predictions(&self) -> Result<Vec<morn_opint::predictor::Prediction>> {
        self.load_records("prediction")
    }

    pub fn save_rollback_request(
        &self,
        r: &morn_assurance::rollback::RollbackRequest,
    ) -> Result<()> {
        self.save_record(
            "rollback_request",
            r.id.as_str(),
            r.workspace_id.as_str(),
            r.created_at.millis(),
            r,
        )
    }

    pub fn load_rollback_requests(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_assurance::rollback::RollbackRequest>> {
        self.load_records_in_workspace("rollback_request", workspace_id.as_str())
    }

    pub fn save_rollback_receipt(
        &self,
        r: &morn_assurance::rollback::RollbackReceipt,
    ) -> Result<()> {
        self.save_record_immutable(
            "rollback_receipt",
            r.id.as_str(),
            "",
            r.created_at.millis(),
            r,
        )
    }

    pub fn load_rollback_receipts(&self) -> Result<Vec<morn_assurance::rollback::RollbackReceipt>> {
        self.load_records("rollback_receipt")
    }

    pub fn save_trace_record(&self, t: &morn_evolution::flywheel::TraceRecord) -> Result<()> {
        self.save_record(
            "trace_record",
            t.id.as_str(),
            t.workspace_id.as_str(),
            t.created_at.millis(),
            t,
        )
    }

    pub fn load_trace_records(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_evolution::flywheel::TraceRecord>> {
        self.load_records_in_workspace("trace_record", workspace_id.as_str())
    }

    pub fn save_workflow_definition(
        &self,
        def: &morn_work::workflow::WorkflowDefinition,
    ) -> Result<()> {
        self.save_record(
            "workflow_definition",
            def.id.as_str(),
            def.workspace_id.as_str(),
            def.created_at.millis(),
            def,
        )
    }

    pub fn load_workflow_definitions(
        &self,
    ) -> Result<Vec<morn_work::workflow::WorkflowDefinition>> {
        self.load_records("workflow_definition")
    }

    pub fn save_workflow_run(&self, run: &morn_work::workflow::WorkflowRun) -> Result<()> {
        self.save_record(
            "workflow_run",
            run.id.as_str(),
            run.workspace_id.as_str(),
            run.created_at.millis(),
            run,
        )
    }

    pub fn load_workflow_runs(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_work::workflow::WorkflowRun>> {
        self.load_records_in_workspace("workflow_run", workspace_id.as_str())
    }

    pub fn save_signal(&self, signal: &morn_work::durable::Signal) -> Result<()> {
        self.save_record(
            "signal",
            signal.id.as_str(),
            "",
            signal.created_at.millis(),
            signal,
        )
    }

    pub fn load_signals(&self) -> Result<Vec<morn_work::durable::Signal>> {
        self.load_records("signal")
    }

    pub fn save_evolution_candidate(
        &self,
        c: &morn_evolution::candidate::EvolutionCandidate,
    ) -> Result<()> {
        self.save_record(
            "evolution_candidate",
            c.id.as_str(),
            c.workspace_id.as_str(),
            c.created_at.millis(),
            c,
        )
    }

    pub fn load_evolution_candidates(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<morn_evolution::candidate::EvolutionCandidate>> {
        self.load_records_in_workspace("evolution_candidate", workspace_id.as_str())
    }

    pub fn save_review(&self, review: &morn_artifact::review::Review) -> Result<()> {
        self.save_record(
            "review",
            review.id.as_str(),
            "",
            review.created_at.millis(),
            review,
        )
    }

    pub fn load_reviews(&self) -> Result<Vec<morn_artifact::review::Review>> {
        self.load_records("review")
    }

    pub fn save_artifact_approval(
        &self,
        approval: &morn_artifact::approval::ArtifactApproval,
    ) -> Result<()> {
        self.save_record(
            "artifact_approval",
            approval.id.as_str(),
            "",
            approval.created_at.millis(),
            approval,
        )
    }

    pub fn load_artifact_approvals(
        &self,
    ) -> Result<Vec<morn_artifact::approval::ArtifactApproval>> {
        self.load_records("artifact_approval")
    }

    pub fn save_evolution_branch(&self, b: &morn_evolution::branch::EvolutionBranch) -> Result<()> {
        self.save_record(
            "evolution_branch",
            b.id.as_str(),
            "",
            b.created_at.millis(),
            b,
        )
    }

    pub fn load_evolution_branches(&self) -> Result<Vec<morn_evolution::branch::EvolutionBranch>> {
        self.load_records("evolution_branch")
    }

    pub fn save_evolution_evaluation(
        &self,
        e: &morn_evolution::evaluation::EvolutionEvaluation,
    ) -> Result<()> {
        self.save_record(
            "evolution_evaluation",
            e.id.as_str(),
            "",
            e.created_at.millis(),
            e,
        )
    }

    pub fn load_evolution_evaluations(
        &self,
    ) -> Result<Vec<morn_evolution::evaluation::EvolutionEvaluation>> {
        self.load_records("evolution_evaluation")
    }

    pub fn save_promotion_decision(
        &self,
        d: &morn_evolution::promotion::PromotionDecision,
    ) -> Result<()> {
        self.save_record(
            "promotion_decision",
            d.id.as_str(),
            "",
            d.created_at.millis(),
            d,
        )
    }

    pub fn load_promotion_decisions(
        &self,
    ) -> Result<Vec<morn_evolution::promotion::PromotionDecision>> {
        self.load_records("promotion_decision")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_artifact::service::ArtifactService;
    use morn_kernel::ids::{PrincipalId, WorkspaceId};
    use morn_kernel::status::ArtifactStatus;
    use morn_kernel::workspace::{Workspace, WorkspaceKind};
    use morn_work::workflow::RunStatus;
    use serde_json::json;

    fn temp_db(name: &str) -> String {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "morn_store_test_{name}_{}.db",
            uuid::Uuid::new_v4()
        ));
        path.to_string_lossy().to_string()
    }

    #[test]
    fn fresh_migration_is_v2() {
        let store = MornStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), 3);
    }

    #[test]
    fn upgrade_from_v1_preserves_rows_and_immutable_column() {
        let path = temp_db("upgrade");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE schema_version (version INTEGER NOT NULL);
                 INSERT INTO schema_version (version) VALUES (1);
                 CREATE TABLE morn_records (
                   kind TEXT NOT NULL, id TEXT NOT NULL, workspace_id TEXT NOT NULL DEFAULT '',
                   payload TEXT NOT NULL, created_at INTEGER NOT NULL,
                   seq INTEGER PRIMARY KEY AUTOINCREMENT, UNIQUE(kind, id));
                 INSERT INTO morn_records (kind, id, workspace_id, payload, created_at)
                   VALUES ('legacy', 'row-1', 'ws-1', '{}', 1);",
            )
            .unwrap();
        }
        let store = MornStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), 3);
        let legacy = store
            .load_record::<serde_json::Value>("legacy", "row-1")
            .unwrap();
        assert!(legacy.is_some(), "v1 rows must survive the upgrade");
        store
            .save_record_immutable("test_imm", "i1", "ws-1", 1, &serde_json::json!({"x": 1}))
            .unwrap();
    }

    #[test]
    fn immutable_records_reject_overwrite() {
        let store = MornStore::open_in_memory().unwrap();
        store
            .save_record_immutable(
                "delivery_receipt",
                "rcpt-1",
                "ws-1",
                1,
                &serde_json::json!({"v": 1}),
            )
            .unwrap();
        assert!(store
            .save_record_immutable(
                "delivery_receipt",
                "rcpt-1",
                "ws-1",
                2,
                &serde_json::json!({"v": 2})
            )
            .is_err());
        store
            .save_record("mutable", "m1", "ws-1", 1, &serde_json::json!({"v": 1}))
            .unwrap();
        store
            .save_record("mutable", "m1", "ws-1", 2, &serde_json::json!({"v": 2}))
            .unwrap();
    }

    #[test]
    fn goal3_capability_restart_hydration() {
        let path = temp_db("g3persist");
        let cap = morn_assurance::certification::CertifiedWorkCapability {
            id: morn_kernel::ids::CertifiedWorkCapabilityId::generate_with("cwc"),
            name: "dataset-to-claim".to_string(),
            version: morn_kernel::version::Version::v1(),
            work_package_template: "wp".to_string(),
            workcell_blueprint: "wc".to_string(),
            role_harness_bindings: vec![],
            workflow_ref: "wf".to_string(),
            acceptance_spec_ref: "acc".to_string(),
            outcome_contract_ref: "oc".to_string(),
            evaluation_pack_ref: "ev".to_string(),
            historical_case_refs: vec![],
            deployment_profile: "local".to_string(),
            context_of_use: vec!["biolab".to_string()],
            status: morn_assurance::certification::CertificationStatus::Certified,
            certification_decision_id: morn_kernel::ids::CertificationDecisionId::generate(),
            created_at: morn_kernel::time::Timestamp::now(),
        };
        {
            let store = MornStore::open(&path).unwrap();
            store.save_certified_capability(&cap).unwrap();
        }
        let store = MornStore::open(&path).unwrap();
        let loaded = store.load_certified_capabilities().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, cap.id);
        assert_eq!(
            loaded[0].status,
            morn_assurance::certification::CertificationStatus::Certified
        );
    }
    #[test]
    fn workspace_and_objects_persist_across_reopen() {
        let path = temp_db("ws");
        let ws = WorkspaceId::generate_with("ws-test");
        let w = Workspace::new("Aging Lab", WorkspaceKind::Lab, PrincipalId::generate());
        {
            let store = MornStore::open(&path).unwrap();
            store.save_workspace(&w).unwrap();
            assert_eq!(store.schema_version().unwrap(), 3);
        }
        let store = MornStore::open(&path).unwrap();
        let loaded = store.load_workspace(&w.id).unwrap().expect("workspace");
        assert_eq!(loaded.name, "Aging Lab");
        assert_eq!(loaded.id, w.id);
        let _ = ws;
    }

    #[test]
    fn ledger_is_append_only_across_reopen() {
        let path = temp_db("ledger");
        let ws = WorkspaceId::generate();
        let mut ledger = morn_kernel::ledger::Ledger::new();
        let e1 = ledger
            .append(ws.clone(), "a", "x", "first", "p", None, vec![])
            .unwrap();
        {
            let store = MornStore::open(&path).unwrap();
            store.append_ledger_entry(&e1).unwrap();
        }
        let e2 = {
            let mut ledger2 = morn_kernel::ledger::Ledger::new();
            ledger2
                .append(ws.clone(), "b", "y", "second", "p", None, vec![])
                .unwrap()
        };
        {
            let store = MornStore::open(&path).unwrap();
            store.append_ledger_entry(&e2).unwrap();
        }
        let store = MornStore::open(&path).unwrap();
        let entries = store.load_ledger(&ws).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].subject, "x");
        assert_eq!(entries[1].subject, "y");
    }

    #[test]
    fn artifact_old_version_remains_after_restart() {
        let path = temp_db("artifact");
        let ws = WorkspaceId::generate();
        let author = PrincipalId::generate();
        let mut svc = ArtifactService::new();
        let (artifact, v1) = svc
            .create(
                "analysis",
                "s@1",
                ws.clone(),
                author.clone(),
                "ref1",
                json!({"x":1}),
                "c1",
            )
            .unwrap();
        let v2 = svc
            .supersede(&v1.id, "ref2", json!({"x":2}), "c2", author.clone())
            .unwrap();
        let v1_superseded = svc.version(&v1.id).unwrap().clone();
        {
            let store = MornStore::open(&path).unwrap();
            store.save_artifact(&artifact).unwrap();
            store.save_artifact_version(&v1_superseded).unwrap();
            store.save_artifact_version(&v2).unwrap();
        }
        let store = MornStore::open(&path).unwrap();
        let versions = store.load_artifact_versions(&artifact.id).unwrap();
        assert_eq!(versions.len(), 2);
        let v1_loaded = versions.iter().find(|v| v.version_no == 1).unwrap();
        assert_eq!(v1_loaded.status, ArtifactStatus::Superseded);
        assert_eq!(v1_loaded.structured_content, json!({"x":1}));
        // readable after restart
        assert_eq!(
            store.load_artifact_version(&v1.id).unwrap().unwrap().id,
            v1.id
        );
    }

    #[test]
    fn evolution_and_review_records_persist() {
        let path = temp_db("evolution");
        let ws = WorkspaceId::generate();
        let by = PrincipalId::generate();
        let mut engine = morn_evolution::engine::EvolutionEngine::new();
        let cand = morn_evolution::candidate::EvolutionCandidate::new(
            ws.clone(),
            morn_evolution::candidate::CandidateType::Workflow,
            morn_kernel::version::Version::v1(),
            "workflow v2",
            by.clone(),
        );
        let cand_id = cand.id.clone();
        engine.add_candidate(cand);
        let branch = engine.branch(&cand_id).unwrap();
        engine
            .record_evaluation(morn_evolution::evaluation::EvolutionEvaluation::new(
                branch.id.clone(),
                true,
                true,
                true,
                "all green",
            ))
            .unwrap();
        let decision = engine
            .promote(&branch.id, by, &["governance".to_string()])
            .unwrap();
        {
            let store = MornStore::open(&path).unwrap();
            store.save_evolution_branch(&branch).unwrap();
            store
                .save_evolution_evaluation(&engine.evaluations()[0].clone())
                .unwrap();
            store.save_promotion_decision(&decision).unwrap();
        }
        let store = MornStore::open(&path).unwrap();
        assert_eq!(store.load_evolution_branches().unwrap().len(), 1);
        assert_eq!(store.load_evolution_evaluations().unwrap().len(), 1);
        let decisions = store.load_promotion_decisions().unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(
            decisions[0].new_version,
            Some(morn_kernel::version::Version::new(1, 1, 0))
        );
    }

    #[test]
    fn durable_run_restart_resumes_via_sqlite() {
        use morn_work::durable::{DurableRuntime, Signal, SignalKind};
        use morn_work::workflow::{WorkflowDefinition, WorkflowStep, WorkflowStepKind};

        let path = temp_db("durable");
        let ws = WorkspaceId::generate();

        // Process A: register workflow, start run, wait for human signal, checkpoint.
        let def = WorkflowDefinition::new(ws.clone(), "dataset-to-claim")
            .add_step(WorkflowStep::new("analyze", WorkflowStepKind::Auto))
            .add_step(WorkflowStep::new("review", WorkflowStepKind::SignalWait))
            .add_step(WorkflowStep::new("release", WorkflowStepKind::Auto));
        let def_id = def.id.clone();
        let mut rt_a = DurableRuntime::new();
        rt_a.register_workflow(def.clone());
        let run = rt_a.start_run(&def_id, None).unwrap();
        let run_id = run.id.clone();
        rt_a.wait_for_signal(&run_id, SignalKind::HumanApproval, 3600)
            .unwrap();
        let cp = rt_a.checkpoint(&run_id).unwrap();

        let run_waited = rt_a.run(&run_id).unwrap().clone();
        {
            let store = MornStore::open(&path).unwrap();
            store.save_workflow_definition(&def).unwrap();
            store.save_workflow_run(&run_waited).unwrap();
            store.save_checkpoint(&cp).unwrap();
        }

        // Process restart: fresh runtime + fresh store read; drift check then resume.
        let store = MornStore::open(&path).unwrap();
        let def_loaded = store
            .load_workflow_definitions()
            .unwrap()
            .into_iter()
            .find(|d| d.id == def_id)
            .expect("workflow definition persisted");
        let run_loaded = store
            .load_workflow_runs(&ws)
            .unwrap()
            .into_iter()
            .find(|r| r.id == run_id)
            .expect("run persisted");
        let cps = store
            .load_checkpoints(&morn_kernel::ids::WorkPackageId::new(run_id.to_string()))
            .unwrap();
        assert!(!cps.is_empty(), "checkpoint persisted");

        let mut rt_b = DurableRuntime::new();
        rt_b.register_workflow(def_loaded);
        rt_b.restore_run(run_loaded);
        // No world drift -> resumes to Running.
        rt_b.load_and_resume(&run_id, "world-v1", "world-v1", true)
            .unwrap();
        assert_eq!(rt_b.run(&run_id).unwrap().status, RunStatus::Running);
        // Deliver the human approval signal after restart.
        rt_b.deliver_signal(Signal::new(
            run_id.clone(),
            SignalKind::HumanApproval,
            "approved",
            "pi-1",
            "pi",
        ))
        .unwrap();
        assert_eq!(rt_b.run(&run_id).unwrap().status, RunStatus::Running);
    }

    #[test]
    fn workspace_isolation_filters_by_workspace() {
        let store = MornStore::open_in_memory().unwrap();
        let ws_a = WorkspaceId::generate();
        let ws_b = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let wa = Workspace::new("A", WorkspaceKind::Project, owner.clone());
        let wb = Workspace::new("B", WorkspaceKind::Project, owner);
        store.save_workspace(&wa).unwrap();
        store.save_workspace(&wb).unwrap();
        // list workspaces returns both but each object query is scoped
        let objs_a: Vec<morn_world::object::Object> = store
            .load_records_in_workspace("object", ws_a.as_str())
            .unwrap();
        assert!(objs_a.is_empty());
        assert_eq!(store.list_workspaces().unwrap().len(), 2);
        let _ = ws_b;
    }

    #[test]
    fn inbox_deduplicates_and_outbox_retries_keep_event_identity() {
        let store = MornStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), 3);

        assert!(store
            .claim_inbound_event("evt-in-1", "cmms://plant-a", 10)
            .unwrap());
        assert!(!store
            .claim_inbound_event("evt-in-1", "cmms://plant-a", 11)
            .unwrap());

        assert!(store
            .enqueue_outbox_event(
                "evt-out-1",
                "workspace-1",
                "io.morn.work.changed.v1",
                "{\"work\":\"work-1\"}",
                20,
            )
            .unwrap());
        assert!(!store
            .enqueue_outbox_event(
                "evt-out-1",
                "workspace-1",
                "io.morn.work.changed.v1",
                "{\"work\":\"work-1\"}",
                21,
            )
            .unwrap());

        let pending = store.pending_outbox_events(10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].event_id, "evt-out-1");
        assert!(pending[0].dispatched_at.is_none());

        store.mark_outbox_dispatched("evt-out-1", 30).unwrap();
        assert!(store.pending_outbox_events(10).unwrap().is_empty());
        assert!(store.mark_outbox_dispatched("evt-out-1", 31).is_err());
    }

}
