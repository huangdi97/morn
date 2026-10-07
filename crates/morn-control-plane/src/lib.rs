//! Durable Morn control-plane reference implementation.
//!
//! Controllers reconcile desired Work state with observed conditions. Cordis,
//! harnesses and workflow engines may execute capabilities, but they do not own
//! this business state.

use serde::{Deserialize, Serialize};

use morn_integration::SourceOfTruthBinding;
use morn_kernel::error::Result;
use morn_profile::DomainProfile;
use morn_runtime::{
    reconcile_attempt, ActionAttempt, BindingMigrationDecision, ExecutionBinding, OutcomeReconciler,
    ReconciliationRecord,
};
use morn_store::MornStore;
use morn_work::acceptance_decision::AcceptanceDecision;
use morn_work::control::{ConditionStatus, WorkCondition, WorkPhase, WorkResource};
use morn_work::value::ValueAssessment;
use morn_world::ObservedOutcome;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ControllerInputs {
    pub capability_resolved: bool,
    pub capability_qualified: bool,
    pub authority_satisfied: bool,
    pub source_of_truth_bound: bool,
    pub provenance_ready: bool,
}

#[derive(Debug, Default)]
pub struct WorkController;

impl WorkController {
    pub fn reconcile(
        &self,
        work: &mut WorkResource,
        profile: &DomainProfile,
        inputs: &ControllerInputs,
    ) {
        work.status.phase = WorkPhase::Resolving;

        Self::condition(
            work,
            "CapabilityResolved",
            inputs.capability_resolved,
            "capability resolver",
        );
        if profile.requires("CapabilityQualification") {
            Self::condition(
                work,
                "CapabilityQualified",
                inputs.capability_qualified,
                "qualification/admission",
            );
        }
        if profile.requires("AuthorityBeforeSideEffect") {
            Self::condition(
                work,
                "AuthoritySatisfied",
                inputs.authority_satisfied,
                "authority provider",
            );
        }
        if profile.requires("SourceOfTruthBinding") {
            Self::condition(
                work,
                "SourceOfTruthBound",
                inputs.source_of_truth_bound,
                "source-of-truth binding",
            );
        }
        if profile.requires("Provenance") {
            Self::condition(
                work,
                "ProvenanceReady",
                inputs.provenance_ready,
                "provenance provider",
            );
        }

        work.status.phase = if work.required_conditions_satisfied() {
            WorkPhase::Ready
        } else {
            WorkPhase::Blocked
        };
        work.mark_observed();
    }

    fn condition(work: &mut WorkResource, name: &str, satisfied: bool, reason: &str) {
        let mut condition = WorkCondition::new(
            name,
            if satisfied {
                ConditionStatus::True
            } else {
                ConditionStatus::False
            },
        );
        condition.reason = reason.to_string();
        work.set_condition(condition);
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkProgressInputs<'a> {
    pub binding: Option<&'a ExecutionBinding>,
    pub attempt: Option<&'a ActionAttempt>,
    pub outcome: Option<&'a ObservedOutcome>,
    pub acceptance: Option<&'a AcceptanceDecision>,
}

/// Reconciles execution evidence into the Work phase without letting a harness
/// session or workflow cursor become canonical business state.
#[derive(Debug, Default)]
pub struct WorkProgressController;

impl WorkProgressController {
    pub fn reconcile(&self, work: &mut WorkResource, inputs: &WorkProgressInputs<'_>) {
        use morn_runtime::AttemptState;
        use morn_work::acceptance_decision::AcceptanceDisposition;

        if !work.required_conditions_satisfied() {
            work.status.phase = WorkPhase::Blocked;
            work.mark_observed();
            return;
        }

        if let Some(binding) = inputs.binding {
            if binding.work_id == work.id && binding.work_generation == work.generation {
                work.status.active_binding = Some(binding.id.clone());
            }
        }

        if let Some(acceptance) = inputs.acceptance {
            if acceptance.work_package_id == work.id {
                work.status.phase = match acceptance.disposition {
                    AcceptanceDisposition::Accept if acceptance.is_final_acceptance() => {
                        WorkPhase::Accepted
                    }
                    AcceptanceDisposition::Reject => WorkPhase::Rejected,
                    AcceptanceDisposition::Conditional
                    | AcceptanceDisposition::RequestMoreEvidence
                    | AcceptanceDisposition::Accept => WorkPhase::Waiting,
                };
                work.mark_observed();
                return;
            }
        }

        if let Some(outcome) = inputs.outcome {
            if outcome.work_package_id == work.id && outcome.is_source_grounded() {
                work.status.phase = WorkPhase::Delivered;
                work.mark_observed();
                return;
            }
        }

        if let Some(attempt) = inputs.attempt {
            work.status.phase = match attempt.state {
                AttemptState::OutcomeUnknown | AttemptState::Reconciling => WorkPhase::Reconciling,
                AttemptState::Authorized
                | AttemptState::Dispatched
                | AttemptState::Acknowledged
                | AttemptState::Committed => WorkPhase::Running,
                AttemptState::Observed | AttemptState::Verified => WorkPhase::Waiting,
                AttemptState::Failed => WorkPhase::Blocked,
                AttemptState::Cancelled => WorkPhase::Cancelled,
                AttemptState::Proposed => WorkPhase::Ready,
            };
            work.mark_observed();
            return;
        }

        work.status.phase = if work.status.active_binding.is_some() {
            WorkPhase::Running
        } else {
            WorkPhase::Ready
        };
        work.mark_observed();
    }
}

#[derive(Debug, Default)]
pub struct ReconciliationController;

impl ReconciliationController {
    pub fn reconcile(
        &self,
        attempt: &mut ActionAttempt,
        provider: &dyn OutcomeReconciler,
    ) -> Result<ReconciliationRecord> {
        reconcile_attempt(attempt, provider)
    }
}

/// Persistence boundary for v11.5 control-plane resources. Work/status and
/// attempt state are mutable projections. Bindings and reconciliation records
/// are immutable historical facts once written.
pub trait ControlPlaneStore {
    fn save_work_resource(&self, work: &WorkResource) -> Result<()>;
    fn save_execution_binding(&self, work: &WorkResource, binding: &ExecutionBinding)
        -> Result<()>;
    fn save_binding_migration(
        &self,
        work: &WorkResource,
        decision: &BindingMigrationDecision,
    ) -> Result<()>;
    fn save_action_attempt(&self, work: &WorkResource, attempt: &ActionAttempt) -> Result<()>;
    fn save_reconciliation(&self, work: &WorkResource, record: &ReconciliationRecord)
        -> Result<()>;
    fn save_source_of_truth_binding(
        &self,
        work: &WorkResource,
        binding: &SourceOfTruthBinding,
    ) -> Result<()>;
    fn save_observed_outcome(&self, work: &WorkResource, outcome: &ObservedOutcome) -> Result<()>;
    fn save_acceptance_decision(
        &self,
        work: &WorkResource,
        decision: &AcceptanceDecision,
    ) -> Result<()>;
    fn save_value_assessment(
        &self,
        work: &WorkResource,
        assessment: &ValueAssessment,
    ) -> Result<()>;
}

impl ControlPlaneStore for MornStore {
    fn save_work_resource(&self, work: &WorkResource) -> Result<()> {
        self.save_record(
            "work_resource_v115",
            work.id.as_str(),
            work.workspace_id.as_str(),
            work.created_at.millis(),
            work,
        )
    }

    fn save_execution_binding(
        &self,
        work: &WorkResource,
        binding: &ExecutionBinding,
    ) -> Result<()> {
        self.save_record_immutable(
            "execution_binding_v115",
            binding.id.as_str(),
            work.workspace_id.as_str(),
            binding.created_at.millis(),
            binding,
        )
    }

    fn save_binding_migration(
        &self,
        work: &WorkResource,
        decision: &BindingMigrationDecision,
    ) -> Result<()> {
        self.save_record_immutable(
            "binding_migration_v115",
            decision.id.as_str(),
            work.workspace_id.as_str(),
            decision.created_at.millis(),
            decision,
        )
    }

    fn save_action_attempt(&self, work: &WorkResource, attempt: &ActionAttempt) -> Result<()> {
        self.save_record(
            "action_attempt_v115",
            attempt.id.as_str(),
            work.workspace_id.as_str(),
            attempt.created_at.millis(),
            attempt,
        )
    }

    fn save_reconciliation(
        &self,
        work: &WorkResource,
        record: &ReconciliationRecord,
    ) -> Result<()> {
        self.save_record_immutable(
            "reconciliation_v115",
            record.id.as_str(),
            work.workspace_id.as_str(),
            record.created_at.millis(),
            record,
        )
    }

    fn save_source_of_truth_binding(
        &self,
        work: &WorkResource,
        binding: &SourceOfTruthBinding,
    ) -> Result<()> {
        self.save_record_immutable(
            "source_of_truth_binding_v115",
            binding.id.as_str(),
            work.workspace_id.as_str(),
            binding.created_at.millis(),
            binding,
        )
    }

    fn save_observed_outcome(&self, work: &WorkResource, outcome: &ObservedOutcome) -> Result<()> {
        self.save_record_immutable(
            "observed_outcome_v115",
            outcome.id.as_str(),
            work.workspace_id.as_str(),
            outcome.observed_at.millis(),
            outcome,
        )
    }

    fn save_acceptance_decision(
        &self,
        work: &WorkResource,
        decision: &AcceptanceDecision,
    ) -> Result<()> {
        self.save_record_immutable(
            "acceptance_decision_v115",
            decision.id.as_str(),
            work.workspace_id.as_str(),
            decision.decided_at.millis(),
            decision,
        )
    }

    fn save_value_assessment(
        &self,
        work: &WorkResource,
        assessment: &ValueAssessment,
    ) -> Result<()> {
        self.save_record_immutable(
            "value_assessment_v115",
            assessment.id.as_str(),
            work.workspace_id.as_str(),
            assessment.assessed_at.millis(),
            assessment,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_runtime::{AttemptState, ReconciliationObservation};
    use morn_work::control::WorkSpec;

    struct CmmsLookup;

    impl OutcomeReconciler for CmmsLookup {
        fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
            Ok(ReconciliationObservation {
                business_key: attempt.business_key.clone(),
                committed: Some(true),
                observed: Some(true),
                external_ref: Some("MO-88273".into()),
                evidence_refs: vec!["cmms://orders/MO-88273".into()],
            })
        }
    }

    #[test]
    fn work_progress_distinguishes_unknown_outcome_delivery_and_acceptance() {
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};
        use serde_json::json;

        let work_id = WorkPackageId::generate_with("wp");
        let mut spec = WorkSpec::new(
            work_id.clone(),
            "restore service",
            "morn.factory.readonly@1.0.0",
        );
        spec.required_conditions = vec![];
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "cap", "provider", "1");
        let mut attempt = ActionAttempt::new(binding.id.clone(), "key", "action");
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt.mark_outcome_unknown("timeout").unwrap();

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                attempt: Some(&attempt),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Reconciling);

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work_id.clone(),
            "service restored",
            OutcomeSourceKind::ExternalSystem,
            "system://source",
            json!({"restored":true}),
        );
        outcome.evidence_refs.push("system://receipt".to_string());
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                outcome: Some(&outcome),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Delivered);

        let mut acceptance = AcceptanceDecision::new(
            work_id,
            AcceptanceSpecId::generate_with("acc"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("owner"),
            "owner",
            "accepted",
        );
        acceptance.outcome_refs.push(outcome.id.clone());
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                outcome: Some(&outcome),
                acceptance: Some(&acceptance),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Accepted);
    }

    #[test]
    fn factory_work_reconciles_timeout_without_runtime_becoming_truth() {
        let work_id = WorkPackageId::generate_with("wp");
        let mut spec = WorkSpec::new(
            work_id,
            "review machine outage capacity and delivery impact",
            "morn.factory.readonly@1.0.0",
        );
        spec.required_conditions = vec![
            "CapabilityResolved".into(),
            "CapabilityQualified".into(),
            "AuthoritySatisfied".into(),
            "SourceOfTruthBound".into(),
            "ProvenanceReady".into(),
        ];
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let profile = DomainProfile::factory_readonly_v1();
        WorkController.reconcile(
            &mut work,
            &profile,
            &ControllerInputs {
                capability_resolved: true,
                capability_qualified: true,
                authority_satisfied: true,
                source_of_truth_bound: true,
                provenance_ready: true,
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Ready);

        let binding = ExecutionBinding::for_work(&work, "cap@sha256:abc", "dsh", "sdk");
        let mut attempt = ActionAttempt::new(
            binding.id.clone(),
            "work-1042:maintenance",
            "create-maintenance-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt
            .mark_outcome_unknown("client timed out after remote commit")
            .unwrap();

        let reconciliation = ReconciliationController
            .reconcile(&mut attempt, &CmmsLookup)
            .unwrap();
        assert_eq!(attempt.state, AttemptState::Observed);
        assert_eq!(attempt.external_ref.as_deref(), Some("MO-88273"));
        assert_eq!(reconciliation.after, AttemptState::Observed);

        let store = MornStore::open_in_memory().unwrap();
        store.save_work_resource(&work).unwrap();
        store.save_execution_binding(&work, &binding).unwrap();
        store.save_action_attempt(&work, &attempt).unwrap();
        store.save_reconciliation(&work, &reconciliation).unwrap();

        let loaded: WorkResource = store
            .load_record("work_resource_v115", work.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(loaded.status.phase, WorkPhase::Ready);
    }
}
