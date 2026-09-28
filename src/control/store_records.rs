//! store_records — durable ops for receipt / reconciliation / outcome /
//! acceptance / capability-release / qualification / profile-release.

use super::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{
    now_iso, AcceptanceDecision, CapabilityRelease, ObservedOutcome, Profile, QualificationRecord,
    Receipt, ReconciliationRecord,
};
use rusqlite::params;

impl ControlPlaneStore {
    // ---- Receipt ----

    /// Records a Receipt (idempotent by id).
    pub fn insert_receipt(&self, receipt: &Receipt) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_receipt
                 (id, attempt_ref, external_ref, source, integrity_hash, recorded_at, provenance_json, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    receipt.id,
                    receipt.attempt_ref,
                    receipt.external_ref,
                    receipt.source,
                    receipt.integrity_hash,
                    receipt.recorded_at,
                    receipt.provenance.as_ref().map(|v| v.to_string()),
                    receipt.payload_summary.as_ref().map(|v| v.to_string())
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Lists receipts for an Attempt (audit timeline).
    pub fn get_receipts(&self, attempt_ref: &str) -> Result<Vec<Receipt>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, attempt_ref, external_ref, source, recorded_at FROM cp_receipt WHERE attempt_ref = ?1 ORDER BY recorded_at")
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![attempt_ref], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;

        let mut receipts = Vec::new();
        for row in rows {
            let (id, attempt_id, external_ref, source, recorded_at) = row?;
            receipts.push(Receipt {
                id,
                attempt_ref: attempt_id,
                external_ref,
                source,
                integrity_hash: None,
                recorded_at,
                provenance: None,
                payload_summary: None,
            });
        }
        Ok(receipts)
    }

    // ---- ReconciliationRecord ----

    /// Records a ReconciliationRecord (idempotent by id).
    pub fn insert_reconciliation(&self, record: &ReconciliationRecord) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_reconciliation
                 (id, attempt_ref, query_json, found, external_ref_found, result, decided_by, recorded_at, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    record.id,
                    record.attempt_ref,
                    record.query.as_ref().map(|v| v.to_string()),
                    record.found.map(|f| serde_json::to_string(&f).unwrap_or_default()),
                    record.external_ref_found,
                    serde_json::to_string(&record.result).map_err(|e| MornError::Internal(e.to_string()))?,
                    record.decided_by,
                    record.recorded_at,
                    record.notes
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    // ---- ObservedOutcome / Acceptance ----

    /// Records an ObservedOutcome from an identifiable source (idempotent by id).
    pub fn insert_outcome(&self, outcome: &ObservedOutcome) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_observed_outcome
                 (id, work_ref, source, source_ref, valid_time, recorded_at, metrics_json, notes, supersedes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    outcome.id,
                    outcome.work_ref,
                    serde_json::to_string(&outcome.source).map_err(|e| MornError::Internal(e.to_string()))?,
                    outcome.source_ref,
                    outcome.valid_time,
                    outcome.recorded_at,
                    serde_json::to_string(&outcome.metrics).map_err(|e| MornError::Internal(e.to_string()))?,
                    outcome.notes,
                    outcome.supersedes
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Records an AcceptanceDecision (idempotent by id).
    pub fn insert_acceptance(&self, acceptance: &AcceptanceDecision) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_acceptance
                 (id, work_ref, decision, criteria_refs_json, outcome_refs_json, actor, principal, reason, decided_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    acceptance.id,
                    acceptance.work_ref,
                    serde_json::to_string(&acceptance.decision).map_err(|e| MornError::Internal(e.to_string()))?,
                    serde_json::to_string(&acceptance.criteria_refs).map_err(|e| MornError::Internal(e.to_string()))?,
                    serde_json::to_string(&acceptance.outcome_refs).map_err(|e| MornError::Internal(e.to_string()))?,
                    acceptance.actor,
                    acceptance.principal,
                    acceptance.reason,
                    acceptance.decided_at
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    // ---- Capability supply chain ----

    /// Records a CapabilityRelease (idempotent by name+version).
    pub fn insert_capability_release(
        &self,
        release: &CapabilityRelease,
    ) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_capability_release
                 (name, version, digest, manifest_ref, released_at, provenance_json, deprecated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    release.name,
                    release.version,
                    release.digest,
                    release.manifest_ref,
                    release.released_at,
                    release.provenance.as_ref().map(|v| v.to_string()),
                    release.deprecated as i64
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Records a QualificationRecord (idempotent by id).
    pub fn insert_qualification(&self, record: &QualificationRecord) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_qualification
                 (id, capability_digest, runtime_provider, execution_environment, environment_digest,
                  status, version, qualified_at, evidence_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    record.id,
                    record.capability_digest,
                    record.runtime_provider,
                    record.execution_environment,
                    record.environment_digest,
                    serde_json::to_string(&record.status).map_err(|e| MornError::Internal(e.to_string()))?,
                    record.version,
                    record.qualified_at,
                    serde_json::json!({
                        "input_scope": record.input_scope,
                        "expected_outputs": record.expected_outputs,
                        "failure_modes": record.failure_modes,
                        "cost_evidence": record.cost_evidence,
                        "evaluator_identity": record.evaluator_identity,
                        "limitations": record.limitations
                    })
                    .to_string()
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }

    /// Returns the latest qualification for a capability digest (by
    /// qualified_at, then rowid). Source for the strict-profile binding gate.
    pub fn get_latest_qualification(
        &self,
        capability_digest: &str,
    ) -> Result<Option<(String, String)>, MornError> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT status, qualified_at FROM cp_qualification
                 WHERE capability_digest = ?1
                 ORDER BY qualified_at DESC, rowid DESC LIMIT 1",
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![capability_digest], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })
            .map_err(|e| MornError::Internal(e.to_string()))?;
        match rows.next() {
            Some(Ok(pair)) => Ok(Some(pair)),
            Some(Err(e)) => Err(MornError::Internal(e.to_string())),
            None => Ok(None),
        }
    }

    /// Records a Profile release (idempotent by name+version).
    pub fn insert_profile_release(&self, profile: &Profile) -> Result<bool, MornError> {
        let conn = self.conn()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO cp_profile_release (name, version, digest, requirements_json, released_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    profile.name,
                    profile.version,
                    profile.digest.clone().unwrap_or_default(),
                    serde_json::to_string(&profile.requirements).map_err(|e| MornError::Internal(e.to_string()))?,
                    now_iso()
                ],
            )
            .map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(inserted > 0)
    }
}
