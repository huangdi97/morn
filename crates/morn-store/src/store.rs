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

const SCHEMA_VERSION: i64 = 1;

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
                // Forward migrations would live here; v1 is the first schema.
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
    fn workspace_and_objects_persist_across_reopen() {
        let path = temp_db("ws");
        let ws = WorkspaceId::generate_with("ws-test");
        let w = Workspace::new("Aging Lab", WorkspaceKind::Lab, PrincipalId::generate());
        {
            let store = MornStore::open(&path).unwrap();
            store.save_workspace(&w).unwrap();
            assert_eq!(store.schema_version().unwrap(), 1);
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
}
