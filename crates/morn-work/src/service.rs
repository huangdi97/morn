//! WorkService: work-package lifecycle and acceptance gating.
//! DurableWorkService: checkpoint/pause/resume/recovery/attention.

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{AcceptanceSpecId, WorkPackageId, WorkspaceId};
use morn_kernel::status::WorkStatus;

use crate::acceptance::AcceptanceSpec;
use crate::attention::{AttentionItem, AttentionPriority};
use crate::checkpoint::Checkpoint;
use crate::recovery::RecoveryRecord;
use crate::work_package::WorkPackage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationWitness {
    pub work_package_id: WorkPackageId,
    pub evidence_ref: String,
}

impl VerificationWitness {
    pub fn new(work_package_id: WorkPackageId, evidence_ref: impl Into<String>) -> Self {
        Self {
            work_package_id,
            evidence_ref: evidence_ref.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptanceApprovalWitness {
    pub work_package_id: WorkPackageId,
    pub principal_ref: String,
    pub role: String,
    pub evidence_ref: String,
}

impl AcceptanceApprovalWitness {
    pub fn new(
        work_package_id: WorkPackageId,
        principal_ref: impl Into<String>,
        role: impl Into<String>,
        evidence_ref: impl Into<String>,
    ) -> Self {
        Self {
            work_package_id,
            principal_ref: principal_ref.into(),
            role: role.into(),
            evidence_ref: evidence_ref.into(),
        }
    }
}

/// Evidence submitted when trying to accept a legacy WorkPackage.
///
/// Canonical v11.5 Work uses AcceptanceDecision; this legacy path still
/// requires Work-bound witness references rather than caller-supplied booleans.
#[derive(Debug, Clone, Default)]
pub struct AcceptanceEvidence {
    pub produced_artifacts: Vec<String>,
    pub verification_witnesses: Vec<VerificationWitness>,
    pub approval_witnesses: Vec<AcceptanceApprovalWitness>,
    pub forbidden_condition_hit: Option<String>,
}

#[derive(Debug, Default)]
pub struct WorkService {
    work_packages: HashMap<WorkPackageId, WorkPackage>,
    acceptance_specs: HashMap<AcceptanceSpecId, AcceptanceSpec>,
}

impl WorkService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_acceptance_spec(&mut self, spec: AcceptanceSpec) {
        self.acceptance_specs.insert(spec.id.clone(), spec);
    }

    pub fn acceptance_spec(&self, id: &AcceptanceSpecId) -> Option<&AcceptanceSpec> {
        self.acceptance_specs.get(id)
    }

    pub fn add_work_package(&mut self, wp: WorkPackage) {
        self.work_packages.insert(wp.id.clone(), wp);
    }

    pub fn work_package(&self, id: &WorkPackageId) -> Option<&WorkPackage> {
        self.work_packages.get(id)
    }

    pub fn work_packages(&self) -> Vec<&WorkPackage> {
        self.work_packages.values().collect()
    }

    pub fn update_status(&mut self, id: &WorkPackageId, status: WorkStatus) -> Result<()> {
        let wp = self
            .work_packages
            .get_mut(id)
            .ok_or_else(|| Error::not_found(format!("work package {id}")))?;
        let current = wp.status;
        if current == status {
            return Ok(());
        }
        if matches!(status, WorkStatus::Accepted | WorkStatus::Rejected) {
            return Err(Error::invalid_state(
                "Accepted/Rejected require the dedicated acceptance decision path",
            ));
        }
        if matches!(
            current,
            WorkStatus::Accepted | WorkStatus::Rejected | WorkStatus::Cancelled
        ) {
            return Err(Error::invalid_state(
                "terminal WorkPackage status cannot be reopened",
            ));
        }
        let allowed = matches!(
            (current, status),
            (WorkStatus::Proposed, WorkStatus::Planned)
                | (WorkStatus::Proposed, WorkStatus::InProgress)
                | (WorkStatus::Proposed, WorkStatus::Blocked)
                | (WorkStatus::Proposed, WorkStatus::Cancelled)
                | (WorkStatus::Planned, WorkStatus::InProgress)
                | (WorkStatus::Planned, WorkStatus::Blocked)
                | (WorkStatus::Planned, WorkStatus::Cancelled)
                | (WorkStatus::InProgress, WorkStatus::WaitingApproval)
                | (WorkStatus::InProgress, WorkStatus::Blocked)
                | (WorkStatus::InProgress, WorkStatus::Completed)
                | (WorkStatus::InProgress, WorkStatus::Cancelled)
                | (WorkStatus::WaitingApproval, WorkStatus::InProgress)
                | (WorkStatus::WaitingApproval, WorkStatus::Blocked)
                | (WorkStatus::WaitingApproval, WorkStatus::Completed)
                | (WorkStatus::WaitingApproval, WorkStatus::Cancelled)
                | (WorkStatus::Blocked, WorkStatus::Planned)
                | (WorkStatus::Blocked, WorkStatus::InProgress)
                | (WorkStatus::Blocked, WorkStatus::Cancelled)
                | (WorkStatus::Completed, WorkStatus::WaitingApproval)
                | (WorkStatus::Completed, WorkStatus::Cancelled)
        );
        if !allowed {
            return Err(Error::invalid_state(format!(
                "invalid WorkPackage status transition {current:?} -> {status:?}"
            )));
        }
        wp.status = status;
        Ok(())
    }

    /// Legacy WorkPackage compatibility boundary.
    ///
    /// v11.5 intentionally has one final acceptance truth model:
    /// `AcceptanceDecision` over persisted, source-grounded outcomes plus
    /// reviewer attestation and one-shot review authorization. Caller-built
    /// legacy witnesses therefore cannot promote a WorkPackage to Accepted.
    pub fn attempt_accept(
        &mut self,
        id: &WorkPackageId,
        _evidence: &AcceptanceEvidence,
    ) -> Result<WorkStatus> {
        let wp = self
            .work_packages
            .get(id)
            .ok_or_else(|| Error::not_found(format!("work package {id}")))?;
        if !matches!(
            wp.status,
            WorkStatus::Completed | WorkStatus::WaitingApproval
        ) {
            return Err(Error::invalid_state(format!(
                "work package {id} must be Completed/WaitingApproval before canonical acceptance, got {:?}",
                wp.status
            )));
        }
        if wp.acceptance_spec_id.is_none() {
            return Err(Error::validation(format!(
                "work package {id} has no AcceptanceSpec and cannot enter canonical acceptance"
            )));
        }
        Err(Error::invalid_state(
            "legacy WorkPackage witnesses cannot create Accepted state; use canonical v11.5 AcceptanceDecision with persisted Outcome and review authority",
        ))
    }

}

/// Durable execution support: checkpoints, pause/resume, recovery, attention.
#[derive(Debug, Default)]
pub struct DurableWorkService {
    checkpoints: Vec<Checkpoint>,
    recovery_records: Vec<RecoveryRecord>,
    attention: Vec<AttentionItem>,
}

impl DurableWorkService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn checkpoint(&mut self, checkpoint: Checkpoint) -> Checkpoint {
        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    pub fn latest_checkpoint(&self, work_package_id: &WorkPackageId) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .filter(|c| c.work_package_id == *work_package_id)
            .max_by_key(|c| c.created_at)
    }

    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// Pause a run: persist a checkpoint with run_state = "paused".
    pub fn pause(
        &mut self,
        workspace_id: WorkspaceId,
        work_package_id: WorkPackageId,
        payload: impl Into<String>,
    ) -> Checkpoint {
        self.checkpoint(Checkpoint::new(
            workspace_id,
            work_package_id,
            "paused",
            payload,
        ))
    }

    /// Resume from the latest checkpoint. Returns the persisted payload.
    pub fn resume(&mut self, work_package_id: &WorkPackageId) -> Result<String> {
        let latest = self
            .latest_checkpoint(work_package_id)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("no checkpoint for {work_package_id}")))?;
        self.checkpoints.push(Checkpoint::new(
            latest.workspace_id.clone(),
            latest.work_package_id.clone(),
            "running",
            latest.payload_json.clone(),
        ));
        Ok(latest.payload_json)
    }

    pub fn record_recovery(&mut self, record: RecoveryRecord) -> RecoveryRecord {
        self.recovery_records.push(record.clone());
        record
    }

    pub fn recovery_records(&self) -> &[RecoveryRecord] {
        &self.recovery_records
    }

    pub fn add_attention(&mut self, item: AttentionItem) -> AttentionItem {
        self.attention.push(item.clone());
        item
    }

    pub fn open_attention(&self) -> Vec<&AttentionItem> {
        self.attention
            .iter()
            .filter(|a| a.status == "open")
            .collect()
    }

    pub fn attention_items(&self) -> &[AttentionItem] {
        &self.attention
    }
}

/// Convenience constructor used by callers that want an attention item for a tool failure.
pub fn tool_failure_attention(
    workspace_id: WorkspaceId,
    work_package_id: &WorkPackageId,
    tool: &str,
    error: &str,
) -> AttentionItem {
    AttentionItem::new(
        workspace_id,
        crate::attention::AttentionKind::ToolFailure,
        format!("tool failure in {work_package_id}"),
        format!("tool {tool} failed: {error}"),
        AttentionPriority::High,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acceptance::AcceptanceSpec;
    use crate::attention::AttentionKind;
    use crate::execution_mode::{ExecutionMode, ExecutorType};
    use morn_kernel::ids::{PrincipalId, WorkspaceId};

    #[test]
    fn work_without_acceptance_spec_cannot_be_accepted() {
        let mut svc = WorkService::new();
        let ws = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let wp = WorkPackage::new(ws, "analyze dataset", owner);
        svc.add_work_package(wp);
        let id = svc.work_packages()[0].id.clone();
        let err = svc.attempt_accept(&id, &AcceptanceEvidence::default());
        assert!(err.is_err());
    }

    #[test]
    fn legacy_witnesses_cannot_promote_completed_work_to_accepted() {
        let mut svc = WorkService::new();
        let ws = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let spec = AcceptanceSpec::new("legacy compatibility")
            .with_required_artifacts(vec!["analysis".to_string()])
            .with_human_approval(true);
        let spec_id = spec.id.clone();
        svc.add_acceptance_spec(spec);
        let wp = WorkPackage::new(ws, "dataset to reviewed claim", owner)
            .with_acceptance_spec(spec_id);
        let wp_id = wp.id.clone();
        svc.add_work_package(wp);
        svc.update_status(&wp_id, WorkStatus::InProgress).unwrap();
        svc.update_status(&wp_id, WorkStatus::Completed).unwrap();

        let evidence = AcceptanceEvidence {
            produced_artifacts: vec!["analysis".to_string()],
            verification_witnesses: vec![VerificationWitness::new(
                wp_id.clone(),
                "world-snapshot://fixture",
            )],
            approval_witnesses: vec![AcceptanceApprovalWitness::new(
                wp_id.clone(),
                "principal://reviewer",
                "reviewer",
                "approval://fixture",
            )],
            forbidden_condition_hit: None,
        };
        let err = svc.attempt_accept(&wp_id, &evidence).unwrap_err();
        assert!(err.to_string().contains("AcceptanceDecision"));
        assert_eq!(
            svc.work_package(&wp_id).unwrap().status,
            WorkStatus::Completed
        );
    }

    #[test]
    fn generic_status_update_cannot_bypass_acceptance_or_reopen_terminal_work() {
        let mut svc = WorkService::new();
        let ws = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let wp = WorkPackage::new(ws, "guard transitions", owner);
        let id = wp.id.clone();
        svc.add_work_package(wp);

        assert!(svc.update_status(&id, WorkStatus::Accepted).is_err());
        assert!(svc.update_status(&id, WorkStatus::Rejected).is_err());
        svc.update_status(&id, WorkStatus::InProgress).unwrap();
        assert!(svc.update_status(&id, WorkStatus::Proposed).is_err());
        svc.update_status(&id, WorkStatus::Cancelled).unwrap();
        assert!(svc.update_status(&id, WorkStatus::InProgress).is_err());
    }

    #[test]
    fn checkpoint_resume_roundtrip() {
        let mut durable = DurableWorkService::new();
        let ws = WorkspaceId::generate();
        let wp_id = WorkPackageId::generate();
        durable.pause(ws, wp_id.clone(), r#"{"step":3}"#);
        let payload = durable.resume(&wp_id).unwrap();
        assert_eq!(payload, r#"{"step":3}"#);
        assert_eq!(
            durable.latest_checkpoint(&wp_id).unwrap().run_state,
            "running"
        );
    }

    #[test]
    fn tool_failure_creates_attention_and_recovery() {
        let mut durable = DurableWorkService::new();
        let ws = WorkspaceId::generate();
        let wp_id = WorkPackageId::generate();
        let item = tool_failure_attention(ws.clone(), &wp_id, "qc-pipeline", "exit 1");
        assert_eq!(item.kind, AttentionKind::ToolFailure);
        durable.add_attention(item);
        assert_eq!(durable.open_attention().len(), 1);
        let rec = RecoveryRecord::new(ws, wp_id, "qc-pipeline exit 1");
        durable.record_recovery(rec);
        assert_eq!(durable.recovery_records().len(), 1);
    }
}
