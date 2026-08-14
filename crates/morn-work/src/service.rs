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

/// Evidence submitted when trying to accept a WorkPackage.
#[derive(Debug, Clone, Default)]
pub struct AcceptanceEvidence {
    pub produced_artifacts: Vec<String>,
    pub verification_passed: bool,
    pub required_approvals: Vec<String>,
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
        wp.status = status;
        Ok(())
    }

    /// Attempt to accept a WorkPackage. Without an AcceptanceSpec this always fails.
    pub fn attempt_accept(
        &mut self,
        id: &WorkPackageId,
        evidence: &AcceptanceEvidence,
    ) -> Result<WorkStatus> {
        let wp = self
            .work_packages
            .get(id)
            .ok_or_else(|| Error::not_found(format!("work package {id}")))?;
        let spec_id = wp.acceptance_spec_id.clone().ok_or_else(|| {
            Error::validation(format!(
                "work package {id} has no AcceptanceSpec and cannot be accepted"
            ))
        })?;
        let spec = self
            .acceptance_specs
            .get(&spec_id)
            .ok_or_else(|| Error::not_found(format!("acceptance spec {spec_id}")))?;

        // Every required artifact must have been produced.
        for required in &spec.required_artifacts {
            if !evidence.produced_artifacts.iter().any(|a| a == required) {
                return Err(Error::validation(format!(
                    "acceptance requires artifact {required:?} which was not produced"
                )));
            }
        }
        if !evidence.verification_passed {
            return Err(Error::validation(
                "acceptance requires verification to pass",
            ));
        }
        if spec.human_approval_required && evidence.required_approvals.is_empty() {
            return Err(Error::validation(
                "acceptance requires human approval which is missing",
            ));
        }
        if let Some(forbidden) = &evidence.forbidden_condition_hit {
            return Err(Error::validation(format!(
                "forbidden condition hit during work: {forbidden}"
            )));
        }

        let wp = self
            .work_packages
            .get_mut(id)
            .ok_or_else(|| Error::not_found(format!("work package {id}")))?;
        wp.status = WorkStatus::Accepted;
        Ok(wp.status)
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
    fn work_with_satisfied_acceptance_can_be_accepted() {
        let mut svc = WorkService::new();
        let ws = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let spec = AcceptanceSpec::new("biolab claim")
            .with_required_artifacts(vec!["analysis".to_string()])
            .with_human_approval(true);
        let spec_id = spec.id.clone();
        svc.add_acceptance_spec(spec);
        let wp = WorkPackage::new(ws, "dataset to reviewed claim", owner)
            .with_acceptance_spec(spec_id)
            .with_execution_mode(ExecutionMode::hybrid(
                vec![crate::execution_mode::WorkNature::Probabilistic],
                vec![ExecutorType::Actor, ExecutorType::Human],
            ));
        let wp_id = wp.id.clone();
        svc.add_work_package(wp);
        let status = svc
            .attempt_accept(
                &wp_id,
                &AcceptanceEvidence {
                    produced_artifacts: vec!["analysis".to_string()],
                    verification_passed: true,
                    required_approvals: vec!["pi".to_string()],
                    forbidden_condition_hit: None,
                },
            )
            .unwrap();
        assert_eq!(status, WorkStatus::Accepted);
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
