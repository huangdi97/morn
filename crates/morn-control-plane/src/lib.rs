//! Durable Morn control-plane reference implementation.
//!
//! Controllers reconcile desired Work state with observed conditions. Cordis,
//! harnesses and workflow engines may execute capabilities, but they do not own
//! this business state.

use serde::{Deserialize, Serialize};

pub mod capability_gate;
pub mod controller_runtime;
pub mod external_action;
pub mod profile_guard;
pub mod provider_gate;


pub use capability_gate::{
    CapabilityEligibilityBlock, CapabilityEligibilityGate, CapabilityEligibilityReport,
};
pub use external_action::{begin_external_attempt, begin_external_attempt_with_effect};


pub use provider_gate::{ProviderGate, ProviderGateBlock, ProviderGatePolicy, ProviderGateReport};

pub use profile_guard::{
    enforce_profile_action, evaluate_profile_action, issue_external_action_permit,
    issue_external_action_permit_for_work, resolve_external_action_finalizer, ExternalActionMode,
    ExternalActionPermit, ExternalActionPermitId, ProfileActionDecision,
};

use morn_integration::SourceOfTruthBinding;
use morn_kernel::error::{Error, Result};
use morn_kernel::protocol::{
    plan_protocol_migration, ProtocolCompatibility, ProtocolMigrationPlan, ProtocolSnapshot,
};
use morn_profile::{
    plan_profile_migration, DomainProfile, ProfileCompatibility, ProfileMigrationPlan,
};
use morn_runtime::{
    reconcile_attempt, ActionAttempt, BindingMigrationDecision, DurableWorkflowBinding,
    DurableWorkflowEvidence, ExecutionBinding, ExecutionManifest, OutcomeReconciler,
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

        let profile_conditions_satisfied = profile
            .pre_execution_work_conditions()
            .iter()
            .all(|required| work.condition_is_true(required));
        work.status.phase = if work.required_conditions_satisfied() && profile_conditions_satisfied
        {
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

#[derive(Debug, Default)]
pub struct ProtocolMigrationController;

impl ProtocolMigrationController {
    /// Migrate the protocol interpretation of future execution for this Work.
    /// Minor protocol changes require the caller to prove the Work/Profile was
    /// reevaluated. Existing Attempts remain pinned to their historical
    /// ExecutionManifest/Binding.
    pub fn migrate(
        &self,
        work: &mut WorkResource,
        base: &ProtocolSnapshot,
        candidate: &ProtocolSnapshot,
        profile_reevaluated: bool,
    ) -> Result<ProtocolMigrationPlan> {
        if work.spec.protocol_version != base.protocol_version {
            return Err(Error::conflict(format!(
                "Work protocol {} does not match migration base {}",
                work.spec.protocol_version, base.protocol_version
            )));
        }

        let plan = plan_protocol_migration(base, candidate);
        if plan.compatibility == ProtocolCompatibility::Incompatible {
            return Err(Error::conflict(format!(
                "protocol migration {} -> {} is incompatible: {:?}",
                plan.from_version, plan.to_version, plan.reasons
            )));
        }
        if plan.requires_profile_reevaluation && !profile_reevaluated {
            return Err(Error::invalid_state(
                "protocol migration requires explicit Work/Profile reevaluation",
            ));
        }
        if plan.from_version == plan.to_version {
            return Ok(plan);
        }

        let mut spec = work.spec.clone();
        spec.protocol_version = candidate.protocol_version;
        work.replace_spec(spec);
        Ok(plan)
    }
}

#[derive(Debug, Default)]
pub struct ProfileMigrationController;

impl ProfileMigrationController {
    /// Explicitly migrate the desired Work profile. Existing Attempts/Bindings
    /// remain historical and pinned to their old generation; the current Work
    /// projection is reset and must satisfy the new profile gates again.
    pub fn migrate(
        &self,
        work: &mut WorkResource,
        from: &DomainProfile,
        to: &DomainProfile,
    ) -> Result<ProfileMigrationPlan> {
        if work.spec.profile_ref != from.canonical_ref() {
            return Err(Error::conflict(format!(
                "Work profile {} does not match migration base {}",
                work.spec.profile_ref,
                from.canonical_ref()
            )));
        }

        let plan = plan_profile_migration(from, to);
        if plan.compatibility == ProfileCompatibility::Incompatible {
            return Err(Error::conflict(format!(
                "profile migration {} -> {} is incompatible: {:?}",
                plan.from_ref, plan.to_ref, plan.reasons
            )));
        }

        if plan.from_ref == plan.to_ref {
            return Ok(plan);
        }

        let old_profile_conditions = from.pre_execution_work_conditions();
        let mut spec = work.spec.clone();
        spec.profile_ref = to.canonical_ref();
        spec.required_conditions
            .retain(|condition| !old_profile_conditions.contains(condition));
        spec.required_conditions
            .extend(to.pre_execution_work_conditions());
        spec.required_conditions.sort();
        spec.required_conditions.dedup();

        work.replace_spec(spec);
        Ok(plan)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkProgressInputs<'a> {
    pub binding: Option<&'a ExecutionBinding>,
    pub workflow: Option<&'a DurableWorkflowEvidence>,
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
            let binding_matches =
                binding.work_id == work.id && binding.work_generation == work.generation;
            let mut consistent = WorkCondition::new(
                "ExecutionBindingConsistent",
                if binding_matches {
                    ConditionStatus::True
                } else {
                    ConditionStatus::False
                },
            );
            consistent.reason = if binding_matches {
                format!(
                    "binding {} matches Work {} generation {}",
                    binding.id, work.id, work.generation
                )
            } else {
                format!(
                    "binding {} targets Work {} generation {}, expected Work {} generation {}",
                    binding.id, binding.work_id, binding.work_generation, work.id, work.generation
                )
            };
            consistent.evidence_refs = vec![binding.id.to_string()];
            work.set_condition(consistent);
            if !binding_matches {
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            }

            work.record_active_binding(binding.id.clone());
            let mut condition = WorkCondition::new("ProfileVersionPinned", ConditionStatus::True);
            condition.reason = format!("binding {} pins {}", binding.id, binding.profile_ref);
            condition.evidence_refs = vec![binding.id.to_string()];
            work.set_condition(condition);
        }

        if let Some(workflow) = inputs.workflow {
            let Some(binding) = inputs.binding else {
                let mut condition =
                    WorkCondition::new("WorkflowRuntimeObserved", ConditionStatus::False);
                condition.reason = format!(
                    "workflow run {} cannot advance Work without its pinned ExecutionBinding",
                    workflow.workflow_run_ref
                );
                condition.evidence_refs = workflow.evidence_refs.clone();
                work.set_condition(condition);
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            };

            let binding_matches = workflow.execution_binding_id == binding.id;
            let mut condition = WorkCondition::new(
                "WorkflowRuntimeObserved",
                if binding_matches {
                    ConditionStatus::True
                } else {
                    ConditionStatus::False
                },
            );
            condition.reason = if binding_matches {
                format!(
                    "workflow provider {} observed run {} under binding {}",
                    workflow.provider_ref, workflow.workflow_run_ref, binding.id
                )
            } else {
                format!(
                    "workflow run {} is tied to binding {}, but controller supplied {}",
                    workflow.workflow_run_ref, workflow.execution_binding_id, binding.id
                )
            };
            condition.evidence_refs = workflow.evidence_refs.clone();
            work.set_condition(condition);

            if !binding_matches {
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            }

            // Even a completed durable workflow is executor evidence only.
            // Without source-grounded Outcome + independent Acceptance, Work
            // remains Running/Waiting rather than becoming Accepted.
            if workflow.is_executor_terminal() {
                work.status.phase = WorkPhase::Waiting;
                work.mark_observed();
                if inputs.outcome.is_none()
                    && inputs.acceptance.is_none()
                    && inputs.attempt.is_none()
                {
                    return;
                }
            }
        }

        if let Some(acceptance) = inputs.acceptance {
            if acceptance.work_package_id == work.id {
                let spec_matches =
                    work.spec.acceptance_ref.as_deref().is_none_or(|expected| {
                        expected == acceptance.acceptance_spec_id.to_string()
                    });
                let grounded_outcome = inputs.outcome.filter(|outcome| {
                    outcome.work_package_id == work.id
                        && outcome.is_source_grounded()
                        && acceptance.outcome_refs.iter().any(|id| id == &outcome.id)
                });
                let final_acceptance =
                    acceptance.is_final_acceptance() && spec_matches && grounded_outcome.is_some();

                let mut independent =
                    WorkCondition::new("IndependentAcceptance", ConditionStatus::True);
                independent.reason = format!(
                    "acceptance decision {} by role {}",
                    acceptance.id, acceptance.acting_role
                );
                independent.evidence_refs = vec![acceptance.id.to_string()];
                work.set_condition(independent);

                let mut semantics = WorkCondition::new(
                    "AcceptedOutcomeSemantics",
                    if final_acceptance {
                        ConditionStatus::True
                    } else {
                        ConditionStatus::False
                    },
                );
                semantics.reason = if final_acceptance {
                    "acceptance spec and source-grounded outcome reference match".to_string()
                } else {
                    "acceptance cannot close Work until its spec and a source-grounded outcome are explicitly linked"
                        .to_string()
                };
                semantics.evidence_refs = grounded_outcome
                    .map(|outcome| vec![outcome.id.to_string()])
                    .unwrap_or_default();
                work.set_condition(semantics);

                work.status.phase = match acceptance.disposition {
                    AcceptanceDisposition::Accept if final_acceptance => WorkPhase::Accepted,
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
                let mut condition = WorkCondition::new("OutcomeObservation", ConditionStatus::True);
                condition.reason = format!("source-grounded outcome {}", outcome.id);
                condition.evidence_refs = outcome.evidence_refs.clone();
                work.set_condition(condition);
                work.status.phase = WorkPhase::Delivered;
                work.mark_observed();
                return;
            }
        }

        if let Some(attempt) = inputs.attempt {
            let Some(binding) = inputs.binding else {
                let mut consistent =
                    WorkCondition::new("ExecutionBindingConsistent", ConditionStatus::False);
                consistent.reason = format!(
                    "attempt {} cannot advance Work without its pinned ExecutionBinding",
                    attempt.id
                );
                consistent.evidence_refs = vec![attempt.id.to_string()];
                work.set_condition(consistent);
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            };
            if attempt.binding_id != binding.id {
                let mut consistent =
                    WorkCondition::new("ExecutionBindingConsistent", ConditionStatus::False);
                consistent.reason = format!(
                    "attempt {} is pinned to binding {}, but controller supplied {}",
                    attempt.id, attempt.binding_id, binding.id
                );
                consistent.evidence_refs = vec![attempt.id.to_string(), binding.id.to_string()];
                work.set_condition(consistent);
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            }

            if attempt.state == AttemptState::Verified
                && (attempt.external_ref.is_some() || !attempt.evidence_refs.is_empty())
            {
                let mut receipt =
                    WorkCondition::new("ReceiptAfterExternalAction", ConditionStatus::True);
                receipt.reason = format!("verified attempt {}", attempt.id);
                receipt.evidence_refs = attempt.evidence_refs.clone();
                work.set_condition(receipt);

                if attempt.last_error.is_some() {
                    let mut reconciled =
                        WorkCondition::new("ReconciliationOnUnknown", ConditionStatus::True);
                    reconciled.reason =
                        "previously ambiguous attempt reconciled before verification".to_string();
                    reconciled.evidence_refs = attempt.evidence_refs.clone();
                    work.set_condition(reconciled);
                }
            }

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

        work.status.phase =
            if !work.status.active_bindings.is_empty() || work.status.active_binding.is_some() {
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
    /// Legacy projection save retained for v1 compatibility paths.
    fn save_work_resource(&self, work: &WorkResource) -> Result<()>;
    /// Preferred v11.5 durable save. Uses Work.resource_version as an
    /// optimistic-concurrency token and updates it after a successful write.
    fn save_work_resource_cas(&self, work: &mut WorkResource) -> Result<u64>;
    fn save_execution_binding(&self, work: &WorkResource, binding: &ExecutionBinding)
        -> Result<()>;
    fn save_execution_manifest(
        &self,
        work: &WorkResource,
        manifest: &ExecutionManifest,
    ) -> Result<()>;
    fn save_binding_migration(
        &self,
        work: &WorkResource,
        decision: &BindingMigrationDecision,
    ) -> Result<()>;
    fn save_durable_workflow_binding(
        &self,
        work: &WorkResource,
        binding: &DurableWorkflowBinding,
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

    fn save_work_resource_cas(&self, work: &mut WorkResource) -> Result<u64> {
        let expected = work.resource_version;
        let mut next = work.clone();
        next.resource_version = expected.saturating_add(1);
        let revision = self.save_record_cas(
            "work_resource_v115",
            work.id.as_str(),
            work.workspace_id.as_str(),
            work.created_at.millis(),
            expected,
            &next,
        )?;
        work.mark_persisted_revision(revision);
        Ok(revision)
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

    fn save_execution_manifest(
        &self,
        work: &WorkResource,
        manifest: &ExecutionManifest,
    ) -> Result<()> {
        self.save_record_immutable(
            "execution_manifest_v115",
            &manifest.execution_binding_ref,
            work.workspace_id.as_str(),
            manifest.created_at.millis(),
            manifest,
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

    fn save_durable_workflow_binding(
        &self,
        work: &WorkResource,
        binding: &DurableWorkflowBinding,
    ) -> Result<()> {
        self.save_record_immutable(
            "durable_workflow_binding_v115",
            binding.id.as_str(),
            work.workspace_id.as_str(),
            binding.created_at.millis(),
            binding,
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
    fn stale_work_projection_cannot_overwrite_newer_controller_state() {
        let store = MornStore::open_in_memory().unwrap();
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "goal", "morn.lite@1.0.0");
        let mut primary = WorkResource::new(WorkspaceId::generate(), spec);
        assert_eq!(store.save_work_resource_cas(&mut primary).unwrap(), 1);

        let mut stale = primary.clone();
        primary.status.phase = WorkPhase::Ready;
        assert_eq!(store.save_work_resource_cas(&mut primary).unwrap(), 2);

        stale.status.phase = WorkPhase::Blocked;
        assert!(store.save_work_resource_cas(&mut stale).is_err());

        let restored: WorkResource = store
            .load_record("work_resource_v115", primary.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(restored.resource_version, 2);
        assert_eq!(restored.status.phase, WorkPhase::Ready);
    }

    #[test]
    fn completed_durable_workflow_does_not_accept_work() {
        use morn_kernel::ids::{WorkPackageId, WorkflowDefinitionId, WorkspaceId};
        use morn_kernel::version::Version;
        use morn_runtime::{DurableWorkflowBinding, DurableWorkflowEvidence};
        use morn_work::workflow::{RunStatus, WorkflowRun};

        let work_id = WorkPackageId::generate_with("wp");
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(work_id, "workflow-backed review", "morn.lite@1.0.0"),
        );
        work.spec.required_conditions.clear();
        let binding = ExecutionBinding::for_work(&work, "cap", "workflow-provider", "1");
        let mut run = WorkflowRun::new(
            WorkflowDefinitionId::generate_with("wf"),
            Version::v1(),
            work.workspace_id.clone(),
        );
        run.status = RunStatus::Completed;
        let workflow_binding = DurableWorkflowBinding::new(
            &work,
            &binding,
            "legacy-durable-runtime",
            run.id.to_string(),
        )
        .unwrap();
        let evidence = DurableWorkflowEvidence::from_legacy_run(&workflow_binding, &run).unwrap();

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                workflow: Some(&evidence),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Waiting);
        assert!(work.condition_is_true("WorkflowRuntimeObserved"));
        assert!(!work.condition_is_true("IndependentAcceptance"));
        assert!(!evidence.proves_morn_acceptance());
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
        assert!(work.condition_is_true("ProfileVersionPinned"));

        attempt.transition(AttemptState::Reconciling).unwrap();
        attempt.evidence_refs.push("system://receipt".to_string());
        attempt.external_ref = Some("external-1".to_string());
        attempt.transition(AttemptState::Observed).unwrap();
        attempt.transition(AttemptState::Verified).unwrap();
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                attempt: Some(&attempt),
                ..Default::default()
            },
        );
        assert!(work.condition_is_true("ReceiptAfterExternalAction"));
        assert!(work.condition_is_true("ReconciliationOnUnknown"));

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
        assert!(work.condition_is_true("OutcomeObservation"));

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
        assert!(work.condition_is_true("IndependentAcceptance"));
        assert!(work.condition_is_true("AcceptedOutcomeSemantics"));
    }

    #[test]
    fn acceptance_cannot_close_on_unlinked_or_wrong_spec_outcome() {
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};
        use serde_json::json;

        let work_id = WorkPackageId::generate_with("wp");
        let acceptance_spec_id = AcceptanceSpecId::generate_with("acc");
        let mut spec = WorkSpec::new(work_id.clone(), "restore", "morn.enterprise@1.0.0");
        spec.required_conditions = vec![];
        spec.acceptance_ref = Some(acceptance_spec_id.to_string());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work_id.clone(),
            "restored",
            OutcomeSourceKind::ExternalSystem,
            "system://authoritative",
            json!({"restored":true}),
        );
        outcome.evidence_refs.push("system://receipt".to_string());

        let mut wrong_spec = AcceptanceDecision::new(
            work_id.clone(),
            AcceptanceSpecId::generate_with("other"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("owner"),
            "owner",
            "accepted",
        );
        wrong_spec.outcome_refs.push(outcome.id.clone());
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                outcome: Some(&outcome),
                acceptance: Some(&wrong_spec),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Waiting);
        assert!(!work.condition_is_true("AcceptedOutcomeSemantics"));

        let mut unlinked = AcceptanceDecision::new(
            work_id,
            acceptance_spec_id,
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("owner"),
            "owner",
            "accepted",
        );
        unlinked
            .outcome_refs
            .push(morn_kernel::ids::OutcomeRecordId::generate_with(
                "different",
            ));
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                outcome: Some(&outcome),
                acceptance: Some(&unlinked),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Waiting);
        assert!(!work.condition_is_true("AcceptedOutcomeSemantics"));
    }

    #[test]
    fn profile_readiness_gates_cannot_be_omitted_from_work_spec() {
        let profile = DomainProfile::factory_readonly_v1();
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "factory work", profile.canonical_ref());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);

        // WorkSpec requires only CapabilityResolved by default, but Factory
        // semantics still require qualification, authority, source truth and
        // provenance before Ready.
        WorkController.reconcile(
            &mut work,
            &profile,
            &ControllerInputs {
                capability_resolved: true,
                capability_qualified: false,
                authority_satisfied: false,
                source_of_truth_bound: false,
                provenance_ready: false,
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Blocked);

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
    }

    #[test]
    fn attempt_cannot_advance_work_under_the_wrong_binding() {
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "test binding consistency", "morn.lite@1.0.0");
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding_a = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "1");
        let binding_b = ExecutionBinding::for_work(&work, "cap:b", "provider:b", "1");
        let mut attempt = ActionAttempt::new(binding_a.id.clone(), "key", "action");
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding_b),
                attempt: Some(&attempt),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Blocked);
        assert!(!work.condition_is_true("ExecutionBindingConsistent"));
    }

    #[test]
    fn attempt_without_binding_cannot_advance_work() {
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "test missing binding", "morn.lite@1.0.0");
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "1");
        let mut attempt = ActionAttempt::new(binding.id, "key", "action");
        attempt.transition(AttemptState::Authorized).unwrap();

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                attempt: Some(&attempt),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Blocked);
        assert!(!work.condition_is_true("ExecutionBindingConsistent"));
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

#[cfg(test)]
mod v115_profile_migration_tests {
    use super::*;
    use morn_kernel::ids::{RuntimeBindingId, WorkPackageId, WorkspaceId};
    use morn_kernel::version::Version;
    use morn_work::control::{ConditionStatus, WorkCondition, WorkSpec};

    #[test]
    fn profile_migration_resets_current_projection_and_rebinds_guarantees() {
        let base = DomainProfile::factory_readonly_v1();
        let mut next = base.clone();
        next.version = Version::new(1, 1, 0);
        next.requirements
            .push(morn_profile::GuaranteeRequirement::required(
                "RuntimeAttestation",
            ));

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review outage",
            base.canonical_ref(),
        );
        spec.required_conditions = base.pre_execution_work_conditions();
        spec.required_conditions
            .push("CustomerConstraint".to_string());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        work.set_condition(WorkCondition::new(
            "CapabilityResolved",
            ConditionStatus::True,
        ));
        work.record_active_binding(RuntimeBindingId::generate_with("binding"));
        work.status.phase = WorkPhase::Running;
        work.mark_observed();

        let plan = ProfileMigrationController
            .migrate(&mut work, &base, &next)
            .unwrap();

        assert_eq!(
            plan.compatibility,
            morn_profile::ProfileCompatibility::RequiresReevaluation
        );
        assert_eq!(work.generation, 2);
        assert_eq!(work.spec.profile_ref, next.canonical_ref());
        assert!(work
            .spec
            .required_conditions
            .contains(&"CustomerConstraint".to_string()));
        assert!(work.status.conditions.is_empty());
        assert!(work.status.active_bindings.is_empty());
        assert_eq!(work.status.phase, WorkPhase::Proposed);
    }

    #[test]
    fn incompatible_cross_profile_migration_is_rejected() {
        let base = DomainProfile::factory_readonly_v1();
        let target = DomainProfile::enterprise_v1();
        let spec = morn_work::control::WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review outage",
            base.canonical_ref(),
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        assert!(ProfileMigrationController
            .migrate(&mut work, &base, &target)
            .is_err());
        assert_eq!(work.spec.profile_ref, base.canonical_ref());
        assert_eq!(work.generation, 1);
    }
}

#[cfg(test)]
mod v115_protocol_migration_tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_kernel::protocol::SemanticInvariant;
    use morn_kernel::version::Version;
    use morn_work::control::WorkSpec;

    #[test]
    fn protocol_minor_migration_requires_profile_reevaluation() {
        let base = ProtocolSnapshot::v11_5();
        let mut next = base.clone();
        next.protocol_version = Version::new(11, 6, 0);
        next.invariants.push(SemanticInvariant::required(
            "new-law",
            "new Work semantic law",
        ));

        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review",
            "morn.factory.readonly@1.0.0",
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);

        assert!(ProtocolMigrationController
            .migrate(&mut work, &base, &next, false)
            .is_err());
        assert_eq!(work.generation, 1);

        let plan = ProtocolMigrationController
            .migrate(&mut work, &base, &next, true)
            .unwrap();
        assert_eq!(
            plan.compatibility,
            ProtocolCompatibility::RequiresReevaluation
        );
        assert_eq!(work.spec.protocol_version, Version::new(11, 6, 0));
        assert_eq!(work.generation, 2);
        assert_eq!(work.status.phase, WorkPhase::Proposed);
    }

    #[test]
    fn same_version_semantic_mutation_cannot_reinterpret_work() {
        let base = ProtocolSnapshot::v11_5();
        let mut mutated = base.clone();
        mutated.semantic_slots.push("Settlement".to_string());
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review",
            "morn.lite@1.0.0",
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        assert!(ProtocolMigrationController
            .migrate(&mut work, &base, &mutated, true)
            .is_err());
        assert_eq!(work.generation, 1);
    }
}
