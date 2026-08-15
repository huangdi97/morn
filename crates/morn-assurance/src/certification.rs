//! Certification: Certified Work Capability model.
//!
//! A Certified Work Capability is NOT an agent: it bundles a WorkPackageTemplate,
//! Workcell blueprint, role/harness bindings, workflow, AcceptanceSpec,
//! OutcomeContract, EvaluationPack, historical cases, deployment profile and
//! certification evidence. One demo pass is never enough to certify.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    CapabilityReleaseId, CertificationDecisionId, CertificationRunId, CertificationSpecId,
    CertifiedWorkCapabilityId, EvaluationRunId, ShadowRunId,
};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::evaluation::EvaluationResult;
use crate::replay::ReplayReport;
use crate::shadow::ShadowRun;

/// Lifecycle state of certification / a certified capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CertificationStatus {
    Draft,
    EvaluationPending,
    Conditional,
    Certified,
    Restricted,
    Failed,
    Suspended,
    Deprecated,
    Retired,
}

impl CertificationStatus {
    pub fn can_transition_to(self, next: CertificationStatus) -> bool {
        use CertificationStatus::*;
        matches!(
            (self, next),
            (Draft, EvaluationPending)
                | (EvaluationPending, Conditional)
                | (EvaluationPending, Certified)
                | (EvaluationPending, Failed)
                | (Conditional, Certified)
                | (Conditional, Failed)
                | (Certified, Restricted)
                | (Certified, Suspended)
                | (Certified, Deprecated)
                | (Restricted, Suspended)
                | (Restricted, Certified)
                | (Restricted, Deprecated)
                | (Suspended, Certified)
                | (Suspended, Restricted)
                | (Deprecated, Retired)
                | (Deprecated, Suspended)
        )
    }
}

/// Certification specification: what must be proven before certifying.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationSpec {
    pub id: CertificationSpecId,
    pub capability_name: String,
    pub capability_version: Version,
    pub context_of_use: Vec<String>,
    pub required_evaluation_suites: Vec<String>,
    pub minimum_acceptance_rate: f64,
    pub policy_required: bool,
    pub recovery_required: bool,
    pub reproducibility_required: bool,
    pub human_gate_required: bool,
    pub provenance_required: bool,
    pub known_failure_coverage_min: f64,
    pub compatibility_requirements: Vec<String>,
    pub created_at: Timestamp,
}

impl CertificationSpec {
    pub fn new(capability_name: impl Into<String>, capability_version: Version) -> Self {
        Self {
            id: CertificationSpecId::generate_with("cspec"),
            capability_name: capability_name.into(),
            capability_version,
            context_of_use: Vec::new(),
            required_evaluation_suites: Vec::new(),
            minimum_acceptance_rate: 1.0,
            policy_required: true,
            recovery_required: true,
            reproducibility_required: true,
            human_gate_required: false,
            provenance_required: true,
            known_failure_coverage_min: 0.0,
            compatibility_requirements: Vec::new(),
            created_at: Timestamp::now(),
        }
    }

    pub fn with_human_gate(mut self, required: bool) -> Self {
        self.human_gate_required = required;
        self
    }
}

/// Evidence inputs to one certification run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CertificationEvidence {
    pub evaluation_results: Vec<EvaluationResult>,
    pub replay_reports: Vec<ReplayReport>,
    pub shadow_runs: Vec<ShadowRun>,
    pub known_failure_cases: Vec<String>,
    pub historical_case_refs: Vec<String>,
}

impl CertificationEvidence {
    pub fn is_empty(&self) -> bool {
        self.evaluation_results.is_empty()
            && self.replay_reports.is_empty()
            && self.shadow_runs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationRun {
    pub id: CertificationRunId,
    pub spec_id: CertificationSpecId,
    pub status: CertificationStatus,
    pub evidence: CertificationEvidence,
    pub evaluation_run_ids: Vec<EvaluationRunId>,
    pub shadow_run_ids: Vec<ShadowRunId>,
    pub created_at: Timestamp,
}

impl CertificationRun {
    pub fn new(spec_id: CertificationSpecId) -> Self {
        Self {
            id: CertificationRunId::generate_with("crun"),
            spec_id,
            status: CertificationStatus::Draft,
            evidence: CertificationEvidence::default(),
            evaluation_run_ids: Vec::new(),
            shadow_run_ids: Vec::new(),
            created_at: Timestamp::now(),
        }
    }

    pub fn attach_evidence(&mut self, evidence: CertificationEvidence) {
        self.evidence = evidence;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationDecision {
    pub id: CertificationDecisionId,
    pub run_id: CertificationRunId,
    pub status: CertificationStatus,
    pub reasons: Vec<String>,
    pub approved_by: Option<String>,
    pub created_at: Timestamp,
}

/// A certified work capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertifiedWorkCapability {
    pub id: CertifiedWorkCapabilityId,
    pub name: String,
    pub version: Version,
    pub work_package_template: String,
    pub workcell_blueprint: String,
    pub role_harness_bindings: Vec<String>,
    pub workflow_ref: String,
    pub acceptance_spec_ref: String,
    pub outcome_contract_ref: String,
    pub evaluation_pack_ref: String,
    pub historical_case_refs: Vec<String>,
    pub deployment_profile: String,
    pub context_of_use: Vec<String>,
    pub status: CertificationStatus,
    pub certification_decision_id: CertificationDecisionId,
    pub created_at: Timestamp,
}

/// Release of a capability version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRelease {
    pub id: CapabilityReleaseId,
    pub capability_id: CertifiedWorkCapabilityId,
    pub version: Version,
    pub release_notes: String,
    pub compatibility: String, // patch_compatible | requires_reevaluation | full_recertification
    pub released_at: Timestamp,
}

/// Certification service.
#[derive(Debug, Default)]
pub struct CertificationService {
    pub runs: Vec<CertificationRun>,
    pub decisions: Vec<CertificationDecision>,
    pub capabilities: Vec<CertifiedWorkCapability>,
    pub releases: Vec<CapabilityRelease>,
}

impl CertificationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(
        &mut self,
        spec_id: CertificationSpecId,
        evidence: CertificationEvidence,
    ) -> Result<CertificationRun> {
        if evidence.is_empty() {
            return Err(Error::validation(
                "cannot certify: no evidence provided (evaluation/replay/shadow)",
            ));
        }
        let mut run = CertificationRun::new(spec_id);
        run.attach_evidence(evidence);
        run.status = CertificationStatus::EvaluationPending;
        self.runs.push(run.clone());
        Ok(run)
    }

    /// Evaluate a run against its spec. Returns a decision; never auto-certifies
    /// when evidence is insufficient, policy fails, or a human gate is required.
    pub fn evaluate(
        &self,
        run: &CertificationRun,
        spec: &CertificationSpec,
    ) -> Result<CertificationDecision> {
        let ev = &run.evidence;
        let mut reasons = Vec::new();

        if ev.is_empty() {
            return Ok(CertificationDecision {
                id: CertificationDecisionId::generate_with("cdec"),
                run_id: run.id.clone(),
                status: CertificationStatus::Failed,
                reasons: vec!["insufficient evidence".to_string()],
                approved_by: None,
                created_at: Timestamp::now(),
            });
        }

        let n = ev.evaluation_results.len() as f64;
        let acceptance_rate = ev
            .evaluation_results
            .iter()
            .filter(|e| e.acceptance)
            .count() as f64
            / n;
        if acceptance_rate < spec.minimum_acceptance_rate {
            reasons.push(format!(
                "acceptance rate {:.2} below minimum {:.2}",
                acceptance_rate, spec.minimum_acceptance_rate
            ));
        }

        if spec.policy_required && ev.evaluation_results.iter().any(|e| !e.policy) {
            reasons.push("critical policy check failed".to_string());
        }
        if spec.recovery_required
            && ev
                .evaluation_results
                .iter()
                .any(|e| !e.failures.is_empty() && e.recovery <= 0.0)
        {
            reasons.push("recovery requirement not met".to_string());
        }
        if spec.reproducibility_required && ev.replay_reports.iter().any(|r| !r.reproduced) {
            reasons.push("reproducibility failed in replay".to_string());
        }
        if spec.provenance_required
            && ev
                .evaluation_results
                .iter()
                .any(|e| e.evidence_refs.is_empty())
        {
            reasons.push("provenance/evidence missing".to_string());
        }
        let coverage = ev.known_failure_cases.len() as f64;
        if coverage < spec.known_failure_coverage_min {
            reasons.push("known failure coverage below minimum".to_string());
        }

        let status = if !reasons.is_empty() {
            CertificationStatus::Failed
        } else if spec.human_gate_required {
            CertificationStatus::Conditional
        } else {
            CertificationStatus::Certified
        };
        Ok(CertificationDecision {
            id: CertificationDecisionId::generate_with("cdec"),
            run_id: run.id.clone(),
            status,
            reasons,
            approved_by: None,
            created_at: Timestamp::now(),
        })
    }

    /// Human approval finalizes a Conditional decision (or confirms Certified).
    pub fn certify(
        &mut self,
        run_id: &CertificationRunId,
        decision: &CertificationDecision,
        spec: &CertificationSpec,
        approved_by: &str,
    ) -> Result<CertifiedWorkCapability> {
        if decision.run_id != *run_id {
            return Err(Error::validation(
                "decision does not match the certification run",
            ));
        }
        if decision.status == CertificationStatus::Failed {
            return Err(Error::validation("cannot certify a failed decision"));
        }
        if spec.human_gate_required && decision.status != CertificationStatus::Conditional {
            return Err(Error::validation(
                "human gate required: decision must be conditional pending approval",
            ));
        }
        let final_status = if decision.status == CertificationStatus::Conditional {
            CertificationStatus::Certified
        } else {
            decision.status
        };
        let mut decision = decision.clone();
        decision.approved_by = Some(approved_by.to_string());
        self.decisions.push(decision.clone());
        let capability = CertifiedWorkCapability {
            id: CertifiedWorkCapabilityId::generate_with("cwc"),
            name: spec.capability_name.clone(),
            version: spec.capability_version,
            work_package_template: format!("wp-template:{}", spec.capability_name),
            workcell_blueprint: format!("workcell:{}", spec.capability_name),
            role_harness_bindings: spec.compatibility_requirements.clone(),
            workflow_ref: format!("workflow:{}", spec.capability_name),
            acceptance_spec_ref: format!("acceptance:{}", spec.capability_name),
            outcome_contract_ref: format!("outcome-contract:{}", spec.capability_name),
            evaluation_pack_ref: spec
                .required_evaluation_suites
                .first()
                .cloned()
                .unwrap_or_default(),
            historical_case_refs: Vec::new(),
            deployment_profile: "local-desktop".to_string(),
            context_of_use: spec.context_of_use.clone(),
            status: final_status,
            certification_decision_id: decision.id.clone(),
            created_at: Timestamp::now(),
        };
        self.capabilities.push(capability.clone());
        Ok(capability)
    }

    pub fn release(
        &mut self,
        capability_id: &CertifiedWorkCapabilityId,
        version: Version,
        notes: &str,
        compatibility: &str,
    ) -> Result<CapabilityRelease> {
        let cap = self
            .capabilities
            .iter()
            .find(|c| c.id == *capability_id)
            .ok_or_else(|| Error::not_found(format!("capability {capability_id}")))?;
        if cap.status != CertificationStatus::Certified
            && cap.status != CertificationStatus::Restricted
        {
            return Err(Error::invalid_state(format!(
                "capability is {:?}; only Certified/Restricted can be released",
                cap.status
            )));
        }
        let release = CapabilityRelease {
            id: CapabilityReleaseId::generate_with("crel"),
            capability_id: capability_id.clone(),
            version,
            release_notes: notes.to_string(),
            compatibility: compatibility.to_string(),
            released_at: Timestamp::now(),
        };
        self.releases.push(release.clone());
        Ok(release)
    }

    pub fn transition(
        &mut self,
        capability_id: &CertifiedWorkCapabilityId,
        next: CertificationStatus,
    ) -> Result<()> {
        let cap = self
            .capabilities
            .iter_mut()
            .find(|c| c.id == *capability_id)
            .ok_or_else(|| Error::not_found(format!("capability {capability_id}")))?;
        if !cap.status.can_transition_to(next) {
            return Err(Error::invalid_state(format!(
                "capability {} cannot go from {:?} to {:?}",
                cap.id, cap.status, next
            )));
        }
        cap.status = next;
        Ok(())
    }

    /// Context-of-use enforcement: a Restricted capability only runs in allowed contexts.
    pub fn context_allows(&self, capability: &CertifiedWorkCapability, context: &str) -> bool {
        if capability.context_of_use.is_empty() {
            return true;
        }
        capability.context_of_use.iter().any(|c| c == context)
    }

    /// Version change rules: patch-compatible, re-evaluation, or full recertification.
    pub fn version_compatibility(&self, old: &Version, new: &Version) -> &'static str {
        if old.major != new.major {
            "full_recertification"
        } else if old.minor != new.minor {
            "requires_reevaluation"
        } else {
            "patch_compatible"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::{EvalStep, EvaluationDecision, EvaluationRunner};

    fn eval_runner(decision: EvaluationDecision) -> EvaluationResult {
        let mut runner = EvaluationRunner::new();
        let steps = vec![
            EvalStep::new("collect", true),
            EvalStep::new("release", true),
        ];
        let faults = if decision == EvaluationDecision::Fail {
            vec![crate::simulation::FaultInjection::new(
                crate::simulation::FaultKind::PermissionDenied,
                "release",
                "denied",
            )]
        } else {
            vec![]
        };
        runner.run("cert", "v1", &steps, &faults, &[])
    }

    fn spec(human_gate: bool) -> CertificationSpec {
        CertificationSpec::new("dataset-to-claim", Version::v1()).with_human_gate(human_gate)
    }

    #[test]
    fn insufficient_evidence_cannot_certify() {
        let mut svc = CertificationService::new();
        let s = spec(false);
        assert!(svc
            .start_run(s.id.clone(), CertificationEvidence::default())
            .is_err());
    }

    #[test]
    fn failed_critical_policy_cannot_certify() {
        let mut svc = CertificationService::new();
        let s = spec(false);
        let evidence = CertificationEvidence {
            evaluation_results: vec![eval_runner(EvaluationDecision::Fail)],
            ..Default::default()
        };
        let run = svc.start_run(s.id.clone(), evidence).unwrap();
        let decision = svc.evaluate(&run, &s).unwrap();
        assert_eq!(decision.status, CertificationStatus::Failed);
        assert!(decision.reasons.iter().any(|r| r.contains("policy")));
        assert!(svc.certify(&run.id, &decision, &s, "pi").is_err());
    }

    #[test]
    fn human_gate_requires_approval() {
        let mut svc = CertificationService::new();
        let s = spec(true);
        let evidence = CertificationEvidence {
            evaluation_results: vec![eval_runner(EvaluationDecision::Pass)],
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
        let run = svc.start_run(s.id.clone(), evidence).unwrap();
        let decision = svc.evaluate(&run, &s).unwrap();
        assert_eq!(decision.status, CertificationStatus::Conditional);
        let cap = svc.certify(&run.id, &decision, &s, "pi").unwrap();
        assert_eq!(cap.status, CertificationStatus::Certified);
        assert_eq!(
            svc.decisions.last().unwrap().approved_by.as_deref(),
            Some("pi")
        );
    }

    #[test]
    fn version_change_requires_recertification_when_major_changes() {
        let svc = CertificationService::new();
        assert_eq!(
            svc.version_compatibility(&Version::new(1, 2, 0), &Version::new(1, 2, 1)),
            "patch_compatible"
        );
        assert_eq!(
            svc.version_compatibility(&Version::new(1, 2, 0), &Version::new(1, 3, 0)),
            "requires_reevaluation"
        );
        assert_eq!(
            svc.version_compatibility(&Version::new(1, 2, 0), &Version::new(2, 0, 0)),
            "full_recertification"
        );
    }

    #[test]
    fn restricted_capability_obeys_context_of_use() {
        let mut svc = CertificationService::new();
        let mut s = spec(false);
        s.context_of_use = vec!["biolab".to_string()];
        let evidence = CertificationEvidence {
            evaluation_results: vec![eval_runner(EvaluationDecision::Pass)],
            ..Default::default()
        };
        let run = svc.start_run(s.id.clone(), evidence).unwrap();
        let decision = svc.evaluate(&run, &s).unwrap();
        let cap = svc.certify(&run.id, &decision, &s, "pi").unwrap();
        assert!(svc.context_allows(&cap, "biolab"));
        assert!(!svc.context_allows(&cap, "pharma"));
    }
}
