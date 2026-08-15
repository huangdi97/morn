//! Managed Work / Outcome Delivery: runs Certified Work Capabilities with SLO,
//! human fallback, retry liability, delivery receipts and acceptance that is
//! independent of the executor.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    AcceptanceDecisionId, CertifiedWorkCapabilityId, DeliveryReceiptId, ManagedWorkRunId,
    WorkspaceId,
};
use morn_kernel::time::Timestamp;

use crate::certification::{CertificationStatus, CertifiedWorkCapability};

/// Delivery lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeliveryStatus {
    Requested,
    Accepted,
    Scheduled,
    Running,
    Waiting,
    HumanFallback,
    Verification,
    Delivered,
    AcceptedOutcome,
    Rejected,
    Retrying,
    Escalated,
    Cancelled,
    Closed,
}

impl DeliveryStatus {
    pub fn can_transition_to(self, next: DeliveryStatus) -> bool {
        use DeliveryStatus::*;
        matches!(
            (self, next),
            (Requested, Accepted)
                | (Requested, Cancelled)
                | (Accepted, Scheduled)
                | (Scheduled, Running)
                | (Running, Waiting)
                | (Running, HumanFallback)
                | (Running, Verification)
                | (Running, Retrying)
                | (Running, Escalated)
                | (Waiting, Running)
                | (Waiting, HumanFallback)
                | (HumanFallback, Running)
                | (HumanFallback, Escalated)
                | (Retrying, Running)
                | (Retrying, Escalated)
                | (Escalated, Running)
                | (Escalated, Cancelled)
                | (Verification, Delivered)
                | (Verification, Retrying)
                | (Delivered, AcceptedOutcome)
                | (Delivered, Rejected)
                | (AcceptedOutcome, Closed)
                | (Rejected, Retrying)
                | (Rejected, Escalated)
                | (Rejected, Closed)
                | (Cancelled, Closed)
        )
    }
}

/// SLO / Outcome metric configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SloConfig {
    pub quality_slo: String,
    pub deadline: Option<String>,
    pub acceptance_method: String,
    pub metric: String,
    pub target: String,
    pub retry_liability: String,
    pub human_fallback: bool,
    pub evidence_required: Vec<String>,
    pub billing_basis: String, // fixed | usage | milestone | outcome (schema only)
}

impl SloConfig {
    pub fn new(
        metric: impl Into<String>,
        target: impl Into<String>,
        acceptance_method: impl Into<String>,
    ) -> Self {
        Self {
            quality_slo: "pass acceptance spec".to_string(),
            deadline: None,
            acceptance_method: acceptance_method.into(),
            metric: metric.into(),
            target: target.into(),
            retry_liability: "1 retry included; further retries require approval".to_string(),
            human_fallback: true,
            evidence_required: vec![
                "execution_receipt".to_string(),
                "artifact_version".to_string(),
            ],
            billing_basis: "outcome".to_string(),
        }
    }
}

/// Human fallback configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanFallback {
    pub trigger: String,
    pub required_role: String,
    pub handoff_context: String,
    pub authority: String,
    pub resume_path: String,
}

/// A managed delivery run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedWorkRun {
    pub id: ManagedWorkRunId,
    pub workspace_id: WorkspaceId,
    pub capability_id: CertifiedWorkCapabilityId,
    pub work_contract_ref: String,
    pub requester: String,
    pub executor: String,
    pub slo: SloConfig,
    pub status: DeliveryStatus,
    pub retries: u32,
    pub human_fallbacks: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ManagedWorkRun {
    pub fn new(
        workspace_id: WorkspaceId,
        capability_id: CertifiedWorkCapabilityId,
        work_contract_ref: impl Into<String>,
        requester: impl Into<String>,
        executor: impl Into<String>,
        slo: SloConfig,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: ManagedWorkRunId::generate_with("mwr"),
            workspace_id,
            capability_id,
            work_contract_ref: work_contract_ref.into(),
            requester: requester.into(),
            executor: executor.into(),
            slo,
            status: DeliveryStatus::Requested,
            retries: 0,
            human_fallbacks: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, next: DeliveryStatus) -> Result<()> {
        if !self.status.can_transition_to(next) {
            return Err(Error::invalid_state(format!(
                "managed run {} cannot go from {:?} to {:?}",
                self.id, self.status, next
            )));
        }
        self.status = next;
        self.updated_at = Timestamp::now();
        Ok(())
    }
}

/// A delivery receipt: complete evidence of one managed delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub id: DeliveryReceiptId,
    pub run_id: ManagedWorkRunId,
    pub work_contract_ref: String,
    pub capability_ref: String,
    pub capability_version: String,
    pub execution_receipt_refs: Vec<String>,
    pub artifacts: Vec<String>,
    pub decisions: Vec<String>,
    pub state_diffs: Vec<String>,
    pub verification: String,
    pub outcome: String,
    pub slo_result: String,
    pub evidence: Vec<String>,
    pub failures_retries: Vec<String>,
    pub human_interventions: u32,
    pub timestamps: Vec<String>,
}

impl DeliveryReceipt {
    pub fn is_complete(&self) -> bool {
        !self.work_contract_ref.is_empty()
            && !self.capability_ref.is_empty()
            && !self.execution_receipt_refs.is_empty()
            && !self.artifacts.is_empty()
            && !self.verification.is_empty()
            && !self.outcome.is_empty()
            && !self.slo_result.is_empty()
    }
}

/// Acceptance source: must be independent of the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AcceptanceSource {
    Automatic,
    IndependentReviewer,
    HumanCustomer,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceDecision {
    pub id: AcceptanceDecisionId,
    pub run_id: ManagedWorkRunId,
    pub decision: String, // accepted | rejected
    pub source: AcceptanceSource,
    pub decided_by: String,
    pub note: String,
    pub created_at: Timestamp,
}

/// Managed Work service: only Certified capabilities may start deliveries.
#[derive(Debug, Default)]
pub struct ManagedWorkService {
    pub runs: Vec<ManagedWorkRun>,
    pub receipts: Vec<DeliveryReceipt>,
    pub acceptances: Vec<AcceptanceDecision>,
}

impl ManagedWorkService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a managed delivery. Rejects uncertified or suspended capabilities.
    pub fn start(
        &mut self,
        capability: &CertifiedWorkCapability,
        run: ManagedWorkRun,
    ) -> Result<ManagedWorkRun> {
        if capability.status != CertificationStatus::Certified
            && capability.status != CertificationStatus::Restricted
        {
            return Err(Error::validation(format!(
                "capability {} is {:?}; only Certified/Restricted capabilities can start managed work",
                capability.id, capability.status
            )));
        }
        if capability.id != run.capability_id {
            return Err(Error::validation(
                "run capability does not match certified capability",
            ));
        }
        let mut run = run;
        run.transition(DeliveryStatus::Accepted)?;
        run.transition(DeliveryStatus::Scheduled)?;
        run.transition(DeliveryStatus::Running)?;
        self.runs.push(run.clone());
        Ok(run)
    }

    pub fn run(&self, id: &ManagedWorkRunId) -> Option<&ManagedWorkRun> {
        self.runs.iter().find(|r| r.id == *id)
    }

    pub fn enter_human_fallback(
        &mut self,
        run_id: &ManagedWorkRunId,
        _fallback: &HumanFallback,
    ) -> Result<()> {
        let run = self
            .runs
            .iter_mut()
            .find(|r| r.id == *run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(DeliveryStatus::HumanFallback)?;
        run.human_fallbacks += 1;
        Ok(())
    }

    pub fn retry(&mut self, run_id: &ManagedWorkRunId, max_retries: u32) -> Result<()> {
        let run = self
            .runs
            .iter_mut()
            .find(|r| r.id == *run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        if run.retries >= max_retries {
            run.transition(DeliveryStatus::Escalated)?;
            return Ok(());
        }
        run.transition(DeliveryStatus::Retrying)?;
        run.retries += 1;
        run.transition(DeliveryStatus::Running)?;
        Ok(())
    }

    pub fn deliver(
        &mut self,
        run_id: &ManagedWorkRunId,
        receipt: DeliveryReceipt,
    ) -> Result<DeliveryReceipt> {
        if !receipt.is_complete() {
            return Err(Error::validation("delivery receipt is incomplete"));
        }
        let run = self
            .runs
            .iter_mut()
            .find(|r| r.id == *run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(DeliveryStatus::Verification)?;
        run.transition(DeliveryStatus::Delivered)?;
        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    /// Accept/reject. `decided_by` must NOT be the executor (independence).
    pub fn decide(
        &mut self,
        run_id: &ManagedWorkRunId,
        decision: &str,
        source: AcceptanceSource,
        decided_by: &str,
        note: &str,
    ) -> Result<AcceptanceDecision> {
        let run = self
            .runs
            .iter()
            .find(|r| r.id == *run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        if run.executor == decided_by {
            return Err(Error::not_authorized(
                "executor cannot self-accept the delivery outcome",
            ));
        }
        if run.status != DeliveryStatus::Delivered {
            return Err(Error::invalid_state(format!(
                "run {run_id} must be Delivered before acceptance (status {:?})",
                run.status
            )));
        }
        let acceptance = AcceptanceDecision {
            id: AcceptanceDecisionId::generate_with("accd"),
            run_id: run_id.clone(),
            decision: decision.to_string(),
            source,
            decided_by: decided_by.to_string(),
            note: note.to_string(),
            created_at: Timestamp::now(),
        };
        let run = self.runs.iter_mut().find(|r| r.id == *run_id).unwrap();
        if decision == "accepted" {
            run.transition(DeliveryStatus::AcceptedOutcome)?;
        } else {
            run.transition(DeliveryStatus::Rejected)?;
        }
        run.transition(DeliveryStatus::Closed)?;
        self.acceptances.push(acceptance.clone());
        Ok(acceptance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certification::{CertificationEvidence, CertificationService, CertificationSpec};
    use crate::evaluation::{EvalStep, EvaluationRunner};
    use crate::replay::ReplayReport;
    use morn_kernel::version::Version;

    fn certified_capability(svc: &mut CertificationService) -> CertifiedWorkCapability {
        let spec = CertificationSpec::new("dataset-to-claim", Version::v1());
        let evidence = CertificationEvidence {
            evaluation_results: vec![{
                let mut r = EvaluationRunner::new();
                r.run(
                    "cert",
                    "v1",
                    &[
                        EvalStep::new("collect", true),
                        EvalStep::new("release", true),
                    ],
                    &[],
                    &[],
                )
            }],
            replay_reports: vec![ReplayReport {
                id: morn_kernel::ids::ReplayReportId::generate(),
                run_id: morn_kernel::ids::ReplayRunId::generate(),
                scenario_id: morn_kernel::ids::ReplayScenarioId::generate(),
                reproduced: true,
                deviations: vec![],
                step_results: vec![],
                state_diff_refs: vec![],
                outcome: "reproduced".to_string(),
                created_at: Timestamp::now(),
            }],
            ..Default::default()
        };
        let run = svc.start_run(spec.id.clone(), evidence).unwrap();
        let decision = svc.evaluate(&run, &spec).unwrap();
        svc.certify(&run.id, &decision, &spec, "pi").unwrap()
    }

    fn run(cap: &CertifiedWorkCapability, ws: &WorkspaceId) -> ManagedWorkRun {
        ManagedWorkRun::new(
            ws.clone(),
            cap.id.clone(),
            "wc-1",
            "customer",
            "analyst-actor",
            SloConfig::new("reproducibility", "1.0", "independent review"),
        )
    }

    #[test]
    fn only_certified_capability_can_start() {
        let mut svc = ManagedWorkService::new();
        let ws = WorkspaceId::generate();
        let cap = CertifiedWorkCapability {
            id: CertifiedWorkCapabilityId::generate(),
            status: CertificationStatus::Draft,
            version: Version::v1(),
            name: "x".to_string(),
            work_package_template: String::new(),
            workcell_blueprint: String::new(),
            role_harness_bindings: vec![],
            workflow_ref: String::new(),
            acceptance_spec_ref: String::new(),
            outcome_contract_ref: String::new(),
            evaluation_pack_ref: String::new(),
            historical_case_refs: vec![],
            deployment_profile: String::new(),
            context_of_use: vec![],
            certification_decision_id: morn_kernel::ids::CertificationDecisionId::generate(),
            created_at: Timestamp::now(),
        };
        assert!(svc.start(&cap, run(&cap, &ws)).is_err());
    }

    #[test]
    fn full_delivery_lifecycle_with_receipt() {
        let mut cert = CertificationService::new();
        let cap = certified_capability(&mut cert);
        let mut mws = ManagedWorkService::new();
        let ws = WorkspaceId::generate();
        let started = mws.start(&cap, run(&cap, &ws)).unwrap();
        assert_eq!(started.status, DeliveryStatus::Running);

        let receipt = DeliveryReceipt {
            id: DeliveryReceiptId::generate(),
            run_id: started.id.clone(),
            work_contract_ref: "wc-1".to_string(),
            capability_ref: cap.id.to_string(),
            capability_version: cap.version.to_string(),
            execution_receipt_refs: vec!["rcpt-1".to_string()],
            artifacts: vec!["art-1".to_string()],
            decisions: vec!["dec-1".to_string()],
            state_diffs: vec!["diff-1".to_string()],
            verification: "passed".to_string(),
            outcome: "claim released".to_string(),
            slo_result: "target met".to_string(),
            evidence: vec!["execution_receipt".to_string()],
            failures_retries: vec![],
            human_interventions: 1,
            timestamps: vec!["t0".to_string()],
        };
        mws.deliver(&started.id, receipt).unwrap();

        let acc = mws
            .decide(
                &started.id,
                "accepted",
                AcceptanceSource::IndependentReviewer,
                "pi",
                "ok",
            )
            .unwrap();
        assert_eq!(acc.decision, "accepted");
        assert_eq!(mws.run(&started.id).unwrap().status, DeliveryStatus::Closed);
    }

    #[test]
    fn executor_cannot_self_accept() {
        let mut cert = CertificationService::new();
        let cap = certified_capability(&mut cert);
        let mut mws = ManagedWorkService::new();
        let ws = WorkspaceId::generate();
        let started = mws.start(&cap, run(&cap, &ws)).unwrap();
        let receipt = DeliveryReceipt {
            id: DeliveryReceiptId::generate(),
            run_id: started.id.clone(),
            work_contract_ref: "wc-1".to_string(),
            capability_ref: cap.id.to_string(),
            capability_version: cap.version.to_string(),
            execution_receipt_refs: vec!["r".to_string()],
            artifacts: vec!["a".to_string()],
            decisions: vec![],
            state_diffs: vec![],
            verification: "passed".to_string(),
            outcome: "ok".to_string(),
            slo_result: "met".to_string(),
            evidence: vec!["e".to_string()],
            failures_retries: vec![],
            human_interventions: 0,
            timestamps: vec![],
        };
        mws.deliver(&started.id, receipt).unwrap();
        // The executor ("analyst-actor") must not be able to accept its own delivery.
        assert!(mws
            .decide(
                &started.id,
                "accepted",
                AcceptanceSource::HumanCustomer,
                "analyst-actor",
                "self"
            )
            .is_err());
    }

    #[test]
    fn retry_liability_exhausts_to_escalated() {
        let mut cert = CertificationService::new();
        let cap = certified_capability(&mut cert);
        let mut mws = ManagedWorkService::new();
        let ws = WorkspaceId::generate();
        let started = mws.start(&cap, run(&cap, &ws)).unwrap();
        mws.retry(&started.id, 1).unwrap();
        assert_eq!(mws.run(&started.id).unwrap().retries, 1);
        mws.retry(&started.id, 1).unwrap();
        assert_eq!(
            mws.run(&started.id).unwrap().status,
            DeliveryStatus::Escalated
        );
    }
}
