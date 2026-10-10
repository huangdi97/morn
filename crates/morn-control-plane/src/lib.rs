//! Durable Morn control-plane reference implementation.
//!
//! Controllers reconcile desired Work state with observed conditions. Cordis,
//! harnesses and workflow engines may execute capabilities, but they do not own
//! this business state.

use serde::{Deserialize, Serialize};

pub mod binding_guard;
pub mod capability_gate;
pub mod condition_evidence;
pub mod controller_runtime;
pub mod external_action;
pub mod profile_guard;
pub mod provider_gate;
pub mod readiness;

pub use binding_guard::{
    BindingOperationalGate, BindingValidityDecision, BindingValidityDecisionId,
};
pub use capability_gate::{
    CapabilityEligibilityBlock, CapabilityEligibilityGate, CapabilityEligibilityReport,
};
pub use condition_evidence::{derive_controller_inputs, ConditionEvidence, ConditionEvidenceId};
pub use controller_runtime::{ControllerTickResult, DurableWorkControllerRuntime};
pub use external_action::{begin_external_attempt, begin_external_attempt_with_effect};

pub use provider_gate::{ProviderGate, ProviderGateBlock, ProviderGatePolicy, ProviderGateReport};
pub use readiness::{
    authority_condition_evidence, capability_qualification_evidence,
    capability_resolution_evidence, provenance_condition_evidence,
    source_of_truth_condition_evidence, workcell_qualification_evidence,
    CapabilityResolutionDecision, CapabilityResolutionDecisionId,
};

pub use profile_guard::{
    enforce_profile_action, evaluate_profile_action, issue_external_action_permit,
    issue_external_action_permit_for_work, resolve_external_action_finalizer, ExternalActionMode,
    ExternalActionPermit, ExternalActionPermitId, ProfileActionDecision,
};

use morn_assurance::CustomerValueAttestation;
use morn_harness::{ExecutionEvent, ExecutionReceipt};
use morn_integration::{GovernedExternalTaskObservation, InteropBinding, SourceOfTruthBinding};
use morn_kernel::error::{Error, Result};
use morn_kernel::protocol::{
    plan_protocol_migration, ProtocolCompatibility, ProtocolMigrationPlan, ProtocolSnapshot,
};
use morn_profile::{
    plan_profile_migration, DomainProfile, ProfileCompatibility, ProfileMigrationPlan,
};
use morn_runtime::{
    reconcile_attempt, ActionAttempt, AttemptState, BindingMigrationDecision,
    DurableWorkflowBinding, DurableWorkflowEvidence, ExecutionBinding, ExecutionManifest,
    OutcomeReconciler, ReconciliationRecord,
};
use morn_store::MornStore;
use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
use morn_work::control::{ConditionStatus, WorkCondition, WorkPhase, WorkResource};
use morn_work::value::{ValueAssessment, ValueEvidenceClass};
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
        if work.status.phase.is_terminal() {
            return;
        }
        work.status.phase = WorkPhase::Resolving;

        Self::condition(
            work,
            "CapabilityResolved",
            inputs.capability_resolved,
            "capability resolver",
        );
        // Authorization readiness is evidence-derived, distinct from a final
        // per-action Authority decision. Only Works that explicitly require this
        // Condition gate on it; no side effect may bypass Authority enforcement.
        Self::condition(
            work,
            "AuthoritySatisfied",
            inputs.authority_satisfied,
            "authority evidence",
        );
        if profile.requires("CapabilityQualification") {
            Self::condition(
                work,
                "CapabilityQualified",
                inputs.capability_qualified,
                "qualification/admission",
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
    pub receipt: Option<&'a ExecutionReceipt>,
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

        if work.status.phase.is_terminal() {
            return;
        }
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

        if let Some(receipt) = inputs.receipt {
            let Some(binding) = inputs.binding else {
                let mut condition =
                    WorkCondition::new("ExecutionReceiptObserved", ConditionStatus::False);
                condition.reason = format!(
                    "execution receipt {} cannot advance Work without its pinned ExecutionBinding",
                    receipt.id
                );
                condition.evidence_refs = vec![receipt.id.to_string()];
                work.set_condition(condition);
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            };
            let matches = receipt.is_pinned_work_evidence()
                && receipt.workspace_id == work.workspace_id
                && receipt.work_package_id.as_ref() == Some(&work.id)
                && receipt.work_generation == Some(work.generation)
                && receipt.execution_binding_ref.as_ref() == Some(&binding.id)
                && receipt.provider_ref.as_deref() == Some(binding.provider_ref.as_str())
                && receipt.execution_environment_ref == binding.execution_environment_ref;
            let mut condition = WorkCondition::new(
                "ExecutionReceiptObserved",
                if matches {
                    ConditionStatus::True
                } else {
                    ConditionStatus::False
                },
            );
            condition.reason = if matches {
                format!(
                    "executor receipt {} is pinned to Work {} generation {} binding {}",
                    receipt.id, work.id, work.generation, binding.id
                )
            } else {
                "executor receipt identity does not match canonical Work/binding".to_string()
            };
            condition.evidence_refs = std::iter::once(receipt.id.to_string())
                .chain(receipt.trace_refs.iter().cloned())
                .collect();
            work.set_condition(condition);
            if !matches {
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            }
            if receipt.outcome == "outcome-unknown" {
                let mut uncertain =
                    WorkCondition::new("ExecutorOutcomeKnown", ConditionStatus::False);
                uncertain.reason = format!(
                    "executor receipt {} lost definitive settlement; reconciliation is required before any new execution",
                    receipt.id
                );
                uncertain.evidence_refs = std::iter::once(receipt.id.to_string())
                    .chain(receipt.trace_refs.iter().cloned())
                    .collect();
                work.set_condition(uncertain);
                work.status.phase = WorkPhase::Reconciling;
                work.mark_observed();
                return;
            }
            if receipt.ended_at.is_some() && receipt.outcome != "completed" {
                let mut known = WorkCondition::new("ExecutorOutcomeKnown", ConditionStatus::True);
                known.reason = format!(
                    "executor receipt {} settled with failure outcome {}; no business outcome may be inferred",
                    receipt.id, receipt.outcome
                );
                known.evidence_refs = std::iter::once(receipt.id.to_string())
                    .chain(receipt.trace_refs.iter().cloned())
                    .collect();
                work.set_condition(known);

                let mut succeeded =
                    WorkCondition::new("ExecutorExecutionSucceeded", ConditionStatus::False);
                succeeded.reason = format!(
                    "executor receipt {} did not complete successfully ({})",
                    receipt.id, receipt.outcome
                );
                succeeded.evidence_refs = vec![receipt.id.to_string()];
                work.set_condition(succeeded);
                work.status.phase = WorkPhase::Blocked;
                work.mark_observed();
                return;
            }
            if receipt.ended_at.is_some()
                && receipt.outcome == "completed"
                && inputs.workflow.is_none()
                && inputs.outcome.is_none()
                && inputs.acceptance.is_none()
                && inputs.attempt.is_none()
            {
                let mut known = WorkCondition::new("ExecutorOutcomeKnown", ConditionStatus::True);
                known.reason = format!(
                    "executor receipt {} settled as completed; business outcome still requires independent observation",
                    receipt.id
                );
                known.evidence_refs = vec![receipt.id.to_string()];
                work.set_condition(known);
                // Harness completion is execution evidence, not a business outcome.
                work.status.phase = WorkPhase::Waiting;
                work.mark_observed();
                return;
            }
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
            if acceptance.work_package_id == work.id
                && acceptance.work_generation == work.generation
            {
                let spec_matches =
                    work.spec.acceptance_ref.as_deref().is_none_or(|expected| {
                        expected == acceptance.acceptance_spec_id.to_string()
                    });
                let grounded_outcome = inputs.outcome.filter(|outcome| {
                    outcome.work_package_id == work.id
                        && outcome.work_generation == work.generation
                        && outcome.is_source_grounded()
                        && acceptance.outcome_refs.iter().any(|id| id == &outcome.id)
                });
                let independently_witnessed = !acceptance.evidence_refs.is_empty()
                    && !acceptance.acting_role.trim().is_empty()
                    && !acceptance.reason.trim().is_empty();
                let final_acceptance = acceptance.is_final_acceptance()
                    && spec_matches
                    && grounded_outcome.is_some()
                    && independently_witnessed;
                let final_rejection = acceptance.disposition == AcceptanceDisposition::Reject
                    && spec_matches
                    && grounded_outcome.is_some()
                    && independently_witnessed;

                let mut independent = WorkCondition::new(
                    "IndependentAcceptance",
                    if independently_witnessed {
                        ConditionStatus::True
                    } else {
                        ConditionStatus::False
                    },
                );
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
                    AcceptanceDisposition::Reject if final_rejection => WorkPhase::Rejected,
                    AcceptanceDisposition::Conditional
                    | AcceptanceDisposition::RequestMoreEvidence
                    | AcceptanceDisposition::Accept
                    | AcceptanceDisposition::Reject => WorkPhase::Waiting,
                };
                work.mark_observed();
                return;
            }
        }

        if let Some(outcome) = inputs.outcome {
            if outcome.work_package_id == work.id
                && outcome.work_generation == work.generation
                && outcome.is_source_grounded()
            {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkExecutionEvent {
    pub work_package_id: morn_kernel::ids::WorkPackageId,
    pub work_generation: u64,
    #[serde(flatten)]
    pub event: ExecutionEvent,
}

impl WorkExecutionEvent {
    fn from_work(work: &WorkResource, event: &ExecutionEvent) -> Self {
        Self {
            work_package_id: work.id.clone(),
            work_generation: work.generation,
            event: event.clone(),
        }
    }
}

pub trait ControlPlaneStore {
    /// Legacy projection save retained for v1 compatibility paths.
    fn save_work_resource(&self, work: &WorkResource) -> Result<()>;
    /// Preferred v11.5 durable save. Uses Work.resource_version as an
    /// optimistic-concurrency token and updates it after a successful write.
    fn save_work_resource_cas(&self, work: &mut WorkResource) -> Result<u64>;
    fn save_execution_binding(&self, work: &WorkResource, binding: &ExecutionBinding)
        -> Result<()>;
    fn save_condition_evidence(
        &self,
        work: &WorkResource,
        evidence: &ConditionEvidence,
    ) -> Result<()>;
    fn save_capability_resolution(
        &self,
        work: &WorkResource,
        decision: &CapabilityResolutionDecision,
    ) -> Result<()>;
    fn save_execution_manifest(
        &self,
        work: &WorkResource,
        manifest: &ExecutionManifest,
    ) -> Result<()>;
    fn save_execution_event(&self, work: &WorkResource, event: &ExecutionEvent) -> Result<()>;
    fn save_execution_receipt(&self, work: &WorkResource, receipt: &ExecutionReceipt)
        -> Result<()>;
    fn save_interop_binding(&self, work: &WorkResource, binding: &InteropBinding) -> Result<()>;
    fn save_external_task_observation(
        &self,
        work: &WorkResource,
        observation: &GovernedExternalTaskObservation,
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

/// The Work's persisted workspace is authoritative. A caller must not clone a
/// Work ID, change its workspace and attach unrelated execution or acceptance
/// evidence to a different tenant's projection.
fn require_canonical_work_workspace(store: &MornStore, work: &WorkResource) -> Result<()> {
    let canonical: WorkResource = store
        .load_record("work_resource_v115", work.id.as_str())?
        .ok_or_else(|| Error::not_found("canonical Work resource"))?;
    if canonical.workspace_id != work.workspace_id {
        return Err(Error::validation(
            "Work workspace differs from the canonical persisted owner",
        ));
    }
    Ok(())
}

fn source_ref_within_authority(binding_root: &str, observed_ref: &str) -> bool {
    let root = binding_root.trim_end_matches('/');
    observed_ref == root
        || observed_ref.strip_prefix(root).is_some_and(|suffix| {
            suffix.starts_with('/') || suffix.starts_with('#') || suffix.starts_with('?')
        })
}

/// Verify that attested source provenance is not merely syntactically present
/// but points back to the exact durable authority and one-shot observation
/// consumption record that created this Outcome.
fn require_attested_outcome_provenance(
    store: &MornStore,
    work: &WorkResource,
    outcome: &ObservedOutcome,
) -> Result<()> {
    if !outcome.is_attested_source_grounded() {
        return Err(Error::validation(
            "observed outcome lacks deployment-attested source provenance",
        ));
    }
    let binding_ref = outcome
        .source_binding_ref
        .as_deref()
        .ok_or_else(|| Error::validation("attested outcome missing source binding"))?;
    let binding: SourceOfTruthBinding = store
        .load_record("source_of_truth_binding_v115", binding_ref)?
        .ok_or_else(|| Error::not_found("attested outcome source binding"))?;
    binding.validate()?;
    let fact_type = outcome
        .fact_type
        .as_deref()
        .ok_or_else(|| Error::validation("attested outcome missing fact type"))?;
    if binding.site_ref != work.spec.site_ref
        || !binding.authoritative_for(fact_type)
        || !source_ref_within_authority(&binding.source_ref, &outcome.source_ref)
    {
        return Err(Error::validation(
            "attested outcome is outside its persisted source-of-truth authority",
        ));
    }

    let attestation_ref = outcome
        .source_attestation_ref
        .as_deref()
        .ok_or_else(|| Error::validation("attested outcome missing observation attestation"))?;
    let consumed: serde_json::Value = store
        .load_record(
            "source_observation_attestation_consumed_v115",
            attestation_ref,
        )?
        .ok_or_else(|| Error::not_found("consumed source observation attestation"))?;
    let exact = consumed
        .get("attestation_id")
        .and_then(serde_json::Value::as_str)
        == Some(attestation_ref)
        && consumed
            .get("work_id")
            .and_then(serde_json::Value::as_str)
            == Some(work.id.as_str())
        && consumed
            .get("source_binding_id")
            .and_then(serde_json::Value::as_str)
            == Some(binding.id.as_str())
        && consumed
            .get("fact_type")
            .and_then(serde_json::Value::as_str)
            == Some(fact_type)
        && consumed
            .get("outcome_id")
            .and_then(serde_json::Value::as_str)
            == Some(outcome.id.as_str());
    if !exact {
        return Err(Error::validation(
            "attested outcome provenance does not match its consumed observation record",
        ));
    }
    Ok(())
}

/// Work status is a projection of durable evidence. Persisting the projection
/// must never create Delivered/Accepted/Rejected truth ahead of its evidence.
fn require_persisted_work_projection_evidence(
    store: &MornStore,
    work: &WorkResource,
) -> Result<()> {
    let receipt_required = work.condition_is_true("ExecutionReceiptObserved");
    if receipt_required {
        let receipts = store.load_records_in_workspace::<ExecutionReceipt>(
            "execution_receipt_v115",
            work.workspace_id.as_str(),
        )?;
        let has_receipt = receipts.iter().any(|receipt| {
            receipt.work_package_id.as_ref() == Some(&work.id)
                && receipt.work_generation == Some(work.generation)
                && receipt.workspace_id == work.workspace_id
        });
        if !has_receipt {
            return Err(Error::invalid_state(
                "Work projection cannot claim ExecutionReceiptObserved without a persisted receipt for this generation",
            ));
        }
    }

    let acceptance_required = work.condition_is_true("IndependentAcceptance")
        || work.condition_is_true("AcceptedOutcomeSemantics")
        || matches!(work.status.phase, WorkPhase::Accepted | WorkPhase::Rejected);
    let outcome_required = work.condition_is_true("OutcomeObservation")
        || matches!(
            work.status.phase,
            WorkPhase::Delivered | WorkPhase::Accepted | WorkPhase::Rejected
        )
        || acceptance_required;

    if !outcome_required {
        return Ok(());
    }

    let outcomes = store
        .load_records_in_workspace::<ObservedOutcome>(
            "observed_outcome_v115",
            work.workspace_id.as_str(),
        )?
        .into_iter()
        .filter(|outcome| {
            outcome.work_package_id == work.id
                && outcome.work_generation == work.generation
                && outcome.workspace_id == work.workspace_id
                && outcome.is_source_grounded()
        })
        .collect::<Vec<_>>();
    if outcomes.is_empty() {
        return Err(Error::invalid_state(
            "Work projection cannot claim observed/delivered outcome without persisted source-grounded outcome evidence",
        ));
    }
    if !acceptance_required {
        return Ok(());
    }

    let decisions = store.load_records_in_workspace::<AcceptanceDecision>(
        "acceptance_decision_v115",
        work.workspace_id.as_str(),
    )?;
    let decision_matches = decisions.iter().any(|decision| {
        if decision.work_package_id != work.id
            || decision.work_generation != work.generation
            || decision.evidence_refs.is_empty()
            || decision.acting_role.trim().is_empty()
            || decision.reason.trim().is_empty()
            || !decision
                .outcome_refs
                .iter()
                .any(|id| outcomes.iter().any(|outcome| &outcome.id == id))
        {
            return false;
        }
        match work.status.phase {
            WorkPhase::Accepted => decision.is_final_acceptance(),
            WorkPhase::Rejected => decision.disposition == AcceptanceDisposition::Reject,
            _ => true,
        }
    });
    if !decision_matches {
        return Err(Error::invalid_state(
            "Work acceptance projection requires a persisted independently witnessed decision over a grounded outcome",
        ));
    }
    Ok(())
}

/// A persisted action snapshot may skip intermediate in-memory transitions,
/// but must never regress to an earlier, redispatchable state.
fn persisted_attempt_state_reachable(from: AttemptState, target: AttemptState) -> bool {
    let states = [
        AttemptState::Proposed,
        AttemptState::Authorized,
        AttemptState::Dispatched,
        AttemptState::Acknowledged,
        AttemptState::Committed,
        AttemptState::OutcomeUnknown,
        AttemptState::Reconciling,
        AttemptState::Observed,
        AttemptState::Verified,
        AttemptState::Failed,
        AttemptState::Cancelled,
    ];
    let mut pending = vec![from];
    let mut visited = Vec::new();
    while let Some(state) = pending.pop() {
        if state == target {
            return true;
        }
        if visited.contains(&state) {
            continue;
        }
        visited.push(state);
        for candidate in states {
            if state.can_transition_to(candidate) {
                pending.push(candidate);
            }
        }
    }
    false
}

impl ControlPlaneStore for MornStore {
    fn save_work_resource(&self, work: &WorkResource) -> Result<()> {
        // Legacy callers must not bypass optimistic concurrency or the
        // generation/terminal-phase invariants through last-writer-wins.
        self.save_work_resource_cas(&mut work.clone())?;
        Ok(())
    }

    fn save_work_resource_cas(&self, work: &mut WorkResource) -> Result<u64> {
        let expected = work.resource_version;
        if work.generation == 0
            || work.spec.work_package_id != work.id
            || work.status.observed_generation > work.generation
        {
            return Err(Error::validation(
                "invalid canonical Work identity or generation",
            ));
        }
        if let Some(persisted) =
            self.load_record::<WorkResource>("work_resource_v115", work.id.as_str())?
        {
            if persisted.workspace_id != work.workspace_id
                || persisted.created_at != work.created_at
            {
                return Err(Error::validation(
                    "canonical Work owner and creation are immutable",
                ));
            }
            if work.generation < persisted.generation
                || work.generation > persisted.generation.saturating_add(1)
                || (work.generation == persisted.generation && work.spec != persisted.spec)
            {
                return Err(Error::validation(
                    "desired Work spec must change through an explicit next generation",
                ));
            }
            if work.generation == persisted.generation.saturating_add(1)
                && persisted.status.conditions.iter().any(|condition| {
                    condition.condition_type == "ExecutorOutcomeKnown"
                        && condition.status == ConditionStatus::False
                })
            {
                return Err(Error::invalid_state(
                    "cannot create a new Work generation while the previous executor outcome is unknown; reconcile it first",
                ));
            }
            if work.generation == persisted.generation
                && persisted.status.phase.is_terminal()
                && work.status.phase != persisted.status.phase
            {
                return Err(Error::validation(
                    "terminal Work phase cannot be reopened within the same generation",
                ));
            }
            if persisted.termination_requested_at.is_some()
                && persisted.termination_requested_at != work.termination_requested_at
            {
                return Err(Error::validation(
                    "Work termination request cannot be erased or rewritten",
                ));
            }
        }
        require_persisted_work_projection_evidence(self, work)?;
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

    fn save_condition_evidence(
        &self,
        work: &WorkResource,
        evidence: &ConditionEvidence,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if evidence.condition_type.trim().is_empty()
            || evidence.producer_ref.trim().is_empty()
            || (evidence.satisfied && evidence.evidence_refs.is_empty())
        {
            return Err(Error::validation(
                "stored readiness evidence requires a condition, producer and positive witness",
            ));
        }
        if evidence.work_ref != work.id.to_string() || evidence.work_generation != work.generation {
            return Err(Error::validation(
                "condition evidence must match the exact Work generation",
            ));
        }
        self.save_record_immutable(
            "condition_evidence_v115",
            evidence.id.as_str(),
            work.workspace_id.as_str(),
            evidence.observed_at.millis(),
            evidence,
        )
    }

    fn save_capability_resolution(
        &self,
        work: &WorkResource,
        decision: &CapabilityResolutionDecision,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if !decision.matches_work(work) {
            return Err(Error::validation(
                "capability resolution decision must match the exact Work generation and source solution",
            ));
        }
        self.save_record_immutable(
            "capability_resolution_v115",
            decision.id.as_str(),
            work.workspace_id.as_str(),
            decision.created_at.millis(),
            decision,
        )
    }

    fn save_execution_binding(
        &self,
        work: &WorkResource,
        binding: &ExecutionBinding,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if !binding.matches_work_generation(work)
            || binding.profile_ref != work.spec.profile_ref
            || binding.site_ref != work.spec.site_ref
            || binding.autonomy_posture != work.spec.autonomy_posture
        {
            return Err(Error::validation(
                "execution binding must match the exact Work, generation, profile and site",
            ));
        }
        if !binding.environment_identity_consistent() {
            return Err(Error::validation(
                "execution binding environment identity is partial or inconsistent",
            ));
        }
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
        require_canonical_work_workspace(self, work)?;
        let binding: ExecutionBinding = self
            .load_record("execution_binding_v115", &manifest.execution_binding_ref)?
            .ok_or_else(|| Error::not_found("execution binding for manifest"))?;
        if !binding.matches_work_generation(work) || !manifest.validates_against(work, &binding) {
            return Err(Error::validation(
                "execution manifest must match its persisted binding and exact Work generation",
            ));
        }
        self.save_record_immutable(
            "execution_manifest_v115",
            &manifest.execution_binding_ref,
            work.workspace_id.as_str(),
            manifest.created_at.millis(),
            manifest,
        )
    }

    fn save_execution_event(&self, work: &WorkResource, event: &ExecutionEvent) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        let now = morn_kernel::time::Timestamp::now();
        if event.workspace_id != work.workspace_id
            || event.session_id.trim().is_empty()
            || event.summary.trim().is_empty()
            || event.created_at < work.created_at
            || event.created_at > now
        {
            return Err(Error::validation(
                "execution event must be a current, non-empty event in the canonical Work workspace",
            ));
        }
        let durable = WorkExecutionEvent::from_work(work, event);
        self.save_record_immutable(
            "execution_event_v115",
            event.id.as_str(),
            work.workspace_id.as_str(),
            event.created_at.millis(),
            &durable,
        )
    }

    fn save_execution_receipt(
        &self,
        work: &WorkResource,
        receipt: &ExecutionReceipt,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if !receipt.is_pinned_work_evidence()
            || receipt.workspace_id != work.workspace_id
            || receipt.work_package_id.as_ref() != Some(&work.id)
            || receipt.work_generation != Some(work.generation)
            || receipt.session_id.trim().is_empty()
            || receipt.outcome.trim().is_empty()
            || receipt
                .ended_at
                .is_some_and(|ended_at| ended_at < receipt.started_at)
        {
            return Err(Error::validation(
                "execution receipt must be complete and pinned to the exact Work generation",
            ));
        }
        let binding_id = receipt
            .execution_binding_ref
            .as_ref()
            .ok_or_else(|| Error::validation("execution receipt binding is required"))?;
        let binding: ExecutionBinding = self
            .load_record("execution_binding_v115", binding_id.as_str())?
            .ok_or_else(|| Error::not_found("execution receipt binding"))?;
        if !binding.matches_work_generation(work)
            || receipt.provider_ref.as_deref() != Some(binding.provider_ref.as_str())
            || receipt.execution_environment_ref != binding.execution_environment_ref
            || receipt
                .runtime_version
                .as_deref()
                .is_some_and(|version| version != binding.provider_version)
            || binding.provider_digest.as_deref() != receipt.runtime_digest.as_deref()
        {
            return Err(Error::validation(
                "execution receipt provider/version/digest/environment does not match its persisted Work binding",
            ));
        }
        if receipt.ended_at.is_some()
            && (receipt.event_ids.is_empty() || receipt.trace_refs.is_empty())
        {
            return Err(Error::validation(
                "settled execution receipt requires durable event and trace references",
            ));
        }
        for event_id in &receipt.event_ids {
            if !receipt
                .trace_refs
                .iter()
                .any(|reference| reference == event_id)
            {
                return Err(Error::validation(
                    "execution receipt trace references must include every durable event id",
                ));
            }
            let durable: WorkExecutionEvent =
                self.load_record("execution_event_v115", event_id)?
                    .ok_or_else(|| Error::not_found("execution receipt event"))?;
            if durable.work_package_id != work.id
                || durable.work_generation != work.generation
                || durable.event.workspace_id != work.workspace_id
                || durable.event.session_id != receipt.session_id
                || durable.event.created_at < receipt.started_at
                || receipt
                    .ended_at
                    .is_some_and(|ended_at| durable.event.created_at > ended_at)
            {
                return Err(Error::validation(
                    "execution receipt references an event outside its workspace/session/time boundary",
                ));
            }
        }
        self.save_record_immutable(
            "execution_receipt_v115",
            receipt.id.as_str(),
            work.workspace_id.as_str(),
            receipt.started_at.millis(),
            receipt,
        )
    }

    fn save_interop_binding(&self, work: &WorkResource, binding: &InteropBinding) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        let execution: ExecutionBinding = self
            .load_record("execution_binding_v115", &binding.execution_binding_ref)?
            .ok_or_else(|| Error::not_found("interop execution binding"))?;
        if !execution.matches_work_generation(work) {
            return Err(Error::validation(
                "interop binding execution binding belongs to another Work generation",
            ));
        }
        binding.validate_against_execution_binding(&execution)?;
        self.save_record_immutable(
            "interop_binding_v115",
            execution.id.as_str(),
            work.workspace_id.as_str(),
            execution.created_at.millis(),
            binding,
        )
    }

    fn save_external_task_observation(
        &self,
        work: &WorkResource,
        observation: &GovernedExternalTaskObservation,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if observation.work_id != work.id || observation.work_generation != work.generation {
            return Err(Error::validation(
                "external task observation belongs to another Work generation",
            ));
        }
        let binding: ExecutionBinding = self
            .load_record(
                "execution_binding_v115",
                observation.execution_binding_ref.as_str(),
            )?
            .ok_or_else(|| Error::not_found("external task execution binding"))?;
        observation.validate_against_binding(&binding)?;
        let now = morn_kernel::time::Timestamp::now();
        if observation.observed_at < binding.created_at || observation.observed_at > now {
            return Err(Error::validation(
                "external task observation timestamp must be within the bound execution lifetime",
            ));
        }
        let interop: InteropBinding = self
            .load_record(
                "interop_binding_v115",
                observation.execution_binding_ref.as_str(),
            )?
            .ok_or_else(|| Error::not_found("governed interop endpoint binding"))?;
        interop.validate_against_execution_binding(&binding)?;
        if interop.endpoint != observation.endpoint {
            return Err(Error::validation(
                "external task observation endpoint differs from the persisted interop binding",
            ));
        }
        let previous = self
            .load_records_in_workspace::<GovernedExternalTaskObservation>(
                "external_task_observation_v115",
                work.workspace_id.as_str(),
            )?
            .into_iter()
            .filter(|item| {
                item.work_id == work.id
                    && item.work_generation == work.generation
                    && item.execution_binding_ref == observation.execution_binding_ref
                    && item.endpoint == observation.endpoint
                    && item.snapshot.task_id() == observation.snapshot.task_id()
            })
            .max_by_key(|item| item.observed_at.millis());
        if let Some(previous) = previous {
            if observation.observed_at < previous.observed_at
                || (observation.observed_at == previous.observed_at
                    && observation.snapshot != previous.snapshot)
            {
                return Err(Error::conflict(
                    "external task observation is stale or ambiguously ordered",
                ));
            }
            if !previous
                .snapshot
                .terminal_compatible_with(&observation.snapshot)
            {
                return Err(Error::conflict(
                    "terminal external task cannot return to a non-terminal or different terminal state",
                ));
            }
        }
        self.save_record_immutable(
            "external_task_observation_v115",
            observation.id.as_str(),
            work.workspace_id.as_str(),
            observation.observed_at.millis(),
            observation,
        )
    }

    fn save_binding_migration(
        &self,
        work: &WorkResource,
        decision: &BindingMigrationDecision,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if decision.work_id != work.id {
            return Err(Error::validation(
                "binding migration decision belongs to another Work",
            ));
        }
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
        require_canonical_work_workspace(self, work)?;
        if binding.work_id != work.id || binding.work_generation != work.generation {
            return Err(Error::validation(
                "durable workflow binding must match the Work generation",
            ));
        }
        let execution: ExecutionBinding = self
            .load_record(
                "execution_binding_v115",
                binding.execution_binding_id.as_str(),
            )?
            .ok_or_else(|| Error::not_found("durable workflow execution binding"))?;
        if !execution.matches_work_generation(work) {
            return Err(Error::validation(
                "durable workflow references another Work execution binding",
            ));
        }
        self.save_record_immutable(
            "durable_workflow_binding_v115",
            binding.id.as_str(),
            work.workspace_id.as_str(),
            binding.created_at.millis(),
            binding,
        )
    }

    fn save_action_attempt(&self, work: &WorkResource, attempt: &ActionAttempt) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        let binding: ExecutionBinding = self
            .load_record("execution_binding_v115", attempt.binding_id.as_str())?
            .ok_or_else(|| Error::not_found("action attempt execution binding"))?;
        if binding.work_id != work.id || attempt.business_key.trim().is_empty() {
            return Err(Error::validation(
                "action attempt must have a business key and binding owned by this Work",
            ));
        }
        if matches!(
            attempt.state,
            AttemptState::Observed | AttemptState::Verified
        ) && (attempt
            .external_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
            || attempt.evidence_refs.is_empty())
        {
            return Err(Error::validation(
                "observed external attempt requires an external reference and evidence",
            ));
        }
        let kind = "action_attempt_v115";
        // Capture the revision before inspecting the snapshot. Re-reading it
        // after validation could let a competing writer advance the row and
        // accidentally supply our stale projection with that newer revision.
        let expected = self
            .record_revision(kind, attempt.id.as_str())?
            .unwrap_or(0);
        let previous: Option<ActionAttempt> = self.load_record(kind, attempt.id.as_str())?;
        if let Some(previous) = &previous {
            if previous.binding_id != attempt.binding_id
                || previous.business_key != attempt.business_key
                || previous.action != attempt.action
                || previous.created_at != attempt.created_at
                || (previous.state != AttemptState::Proposed
                    && (previous.resource_ref != attempt.resource_ref
                        || previous.site_ref != attempt.site_ref
                        || previous.authority_decision_ref != attempt.authority_decision_ref
                        || previous.external_action_permit_ref
                            != attempt.external_action_permit_ref
                        || previous.effect_contract != attempt.effect_contract))
            {
                return Err(Error::validation(
                    "persisted attempt identity and authorized effect are immutable",
                ));
            }
            if !persisted_attempt_state_reachable(previous.state, attempt.state)
                || attempt.updated_at < previous.updated_at
                || previous
                    .external_ref
                    .as_ref()
                    .is_some_and(|reference| attempt.external_ref.as_ref() != Some(reference))
                || !previous
                    .evidence_refs
                    .iter()
                    .all(|reference| attempt.evidence_refs.contains(reference))
            {
                return Err(Error::conflict(
                    "action attempt state or external evidence regressed",
                ));
            }
            if previous == attempt {
                return Ok(());
            }
        }
        // CAS rejects a concurrent update after our revision snapshot.
        self.save_record_cas(
            kind,
            attempt.id.as_str(),
            work.workspace_id.as_str(),
            attempt.created_at.millis(),
            expected,
            attempt,
        )?;
        Ok(())
    }

    fn save_reconciliation(
        &self,
        work: &WorkResource,
        record: &ReconciliationRecord,
    ) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        let attempt: ActionAttempt = self
            .load_record("action_attempt_v115", record.attempt_id.as_str())?
            .ok_or_else(|| Error::not_found("reconciliation action attempt"))?;
        let binding: ExecutionBinding = self
            .load_record("execution_binding_v115", attempt.binding_id.as_str())?
            .ok_or_else(|| Error::not_found("reconciliation execution binding"))?;
        if binding.work_id != work.id || record.observation.business_key != attempt.business_key {
            return Err(Error::validation(
                "reconciliation must match an attempt and business key owned by this Work",
            ));
        }
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
        require_canonical_work_workspace(self, work)?;
        binding.validate()?;
        if binding.site_ref != work.spec.site_ref {
            return Err(Error::validation(
                "source-of-truth binding site must match the Work site",
            ));
        }
        self.save_record_immutable(
            "source_of_truth_binding_v115",
            binding.id.as_str(),
            work.workspace_id.as_str(),
            binding.created_at.millis(),
            binding,
        )
    }

    fn save_observed_outcome(&self, work: &WorkResource, outcome: &ObservedOutcome) -> Result<()> {
        require_canonical_work_workspace(self, work)?;
        if outcome.work_package_id != work.id
            || outcome.workspace_id != work.workspace_id
            || outcome.work_generation != work.generation
        {
            return Err(Error::validation(
                "observed outcome must belong to the exact Work, workspace and generation",
            ));
        }
        if outcome.has_source_provenance() {
            require_attested_outcome_provenance(self, work, outcome)?;
        }
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
        require_canonical_work_workspace(self, work)?;
        if decision.work_package_id != work.id || decision.work_generation != work.generation {
            return Err(Error::validation(
                "acceptance decision belongs to another Work generation",
            ));
        }
        let is_accept = decision.disposition == AcceptanceDisposition::Accept;
        let is_reject = decision.disposition == AcceptanceDisposition::Reject;
        let is_terminal_decision = is_accept || is_reject;
        if is_terminal_decision
            && work
                .spec
                .acceptance_ref
                .as_deref()
                .is_some_and(|expected| expected != decision.acceptance_spec_id.as_str())
        {
            return Err(Error::validation(
                "terminal acceptance/rejection must match the Work's pinned AcceptanceSpec",
            ));
        }
        if is_terminal_decision
            && (decision.outcome_refs.is_empty()
                || decision.evidence_refs.is_empty()
                || decision.acting_role.trim().is_empty()
                || decision.reason.trim().is_empty())
        {
            return Err(Error::validation(
                "terminal acceptance/rejection requires linked outcomes, independent reviewer, rationale and evidence",
            ));
        }
        for outcome_id in &decision.outcome_refs {
            let outcome: ObservedOutcome = self
                .load_record("observed_outcome_v115", outcome_id.as_str())?
                .ok_or_else(|| Error::not_found("referenced observed outcome"))?;
            if outcome.work_package_id != work.id
                || outcome.workspace_id != work.workspace_id
                || outcome.work_generation != work.generation
            {
                return Err(Error::validation(
                    "acceptance must reference outcomes from the same Work generation and workspace",
                ));
            }
            if is_terminal_decision && !outcome.is_source_grounded() {
                return Err(Error::validation(
                    "terminal acceptance/rejection cannot cite an ungrounded observed outcome",
                ));
            }
            if is_accept {
                require_attested_outcome_provenance(self, work, &outcome)?;
            }
        }
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
        require_canonical_work_workspace(self, work)?;
        if assessment.work_package_id != work.id || assessment.work_generation != work.generation {
            return Err(Error::validation(
                "value assessment belongs to another Work generation",
            ));
        }
        let outcome: ObservedOutcome = self
            .load_record("observed_outcome_v115", assessment.outcome_ref.as_str())?
            .ok_or_else(|| Error::not_found("value assessment outcome"))?;
        if outcome.work_package_id != work.id
            || outcome.workspace_id != work.workspace_id
            || outcome.work_generation != work.generation
        {
            return Err(Error::validation(
                "value assessment must reference an outcome from the same Work generation and workspace",
            ));
        }
        if assessment.evidence_class == ValueEvidenceClass::CustomerValidated {
            require_attested_outcome_provenance(self, work, &outcome)?;
            if !assessment.is_customer_value_claim() {
                return Err(Error::validation(
                    "customer-validated value requires an attested source-grounded outcome, acceptance and evidence",
                ));
            }
            let acceptance_id = assessment
                .acceptance_ref
                .as_ref()
                .ok_or_else(|| Error::validation("customer value requires acceptance reference"))?;
            let acceptance: AcceptanceDecision = self
                .load_record("acceptance_decision_v115", acceptance_id.as_str())?
                .ok_or_else(|| Error::not_found("customer value acceptance decision"))?;
            if acceptance.work_package_id != work.id
                || acceptance.work_generation != work.generation
                || !acceptance.is_final_acceptance()
                || !acceptance.outcome_refs.contains(&assessment.outcome_ref)
            {
                return Err(Error::validation(
                    "customer-validated value requires final acceptance of that exact outcome",
                ));
            }

            let attestation_ref = assessment
                .customer_value_attestation_ref
                .as_deref()
                .ok_or_else(|| {
                    Error::validation("customer value requires attestation reference")
                })?;
            let attestation: CustomerValueAttestation = self
                .load_record("customer_value_attestation_v115", attestation_ref)?
                .ok_or_else(|| Error::not_found("customer value deployment attestation"))?;
            let attested_kpis: Vec<(String, f64)> = attestation
                .kpis
                .iter()
                .map(|metric| (metric.name.clone(), metric.value))
                .collect();
            if attestation.attestation_id != attestation_ref
                || !attestation.active_for(
                    &work.workspace_id,
                    (&work.id, work.generation),
                    &assessment.outcome_ref,
                    acceptance_id,
                    assessment.assessed_at,
                )
                || assessment.baseline_ref != attestation.baseline_ref
                || assessment.kpis != attested_kpis
                || !attestation
                    .evidence_refs
                    .iter()
                    .all(|reference| assessment.evidence_refs.contains(reference))
            {
                return Err(Error::validation(
                    "customer-validated value must exactly match a persisted deployment attestation",
                ));
            }
        }
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
        acceptance
            .evidence_refs
            .push("review://independent-owner-review".to_string());
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

#[cfg(test)]
mod control_plane_persistence_scope_tests {
    use super::*;
    use morn_kernel::ids::{AcceptanceSpecId, PrincipalId, WorkPackageId, WorkspaceId};
    use morn_runtime::CompositionRuntimeRef;
    use morn_work::acceptance_decision::AcceptanceDisposition;
    use morn_work::control::WorkSpec;
    use morn_work::value::ValueEvidenceClass;
    use morn_world::OutcomeSourceKind;
    use serde_json::json;

    fn fixture_work() -> WorkResource {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review external maintenance order",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        WorkResource::new(WorkspaceId::generate(), spec)
    }

    fn attest_test_outcome(
        store: &MornStore,
        work: &WorkResource,
        outcome: &mut ObservedOutcome,
    ) {
        let binding = morn_integration::SourceOfTruthBinding {
            id: morn_integration::SourceOfTruthBindingId::generate_with("sot"),
            site_ref: work.spec.site_ref.clone(),
            source_ref: outcome.source_ref.clone(),
            authority_kind: morn_integration::TruthAuthorityKind::SystemOfRecord,
            authoritative_fact_types: vec!["test.fact".to_string()],
            key_mapping_ref: "mapping://test@1".to_string(),
            query_capability_ref: "capability://test.read@1".to_string(),
            freshness_sla_ms: None,
            conflict_policy: morn_integration::ConflictPolicy::ReconcileBeforeUse,
            version_ref: "binding:test-v1".to_string(),
            created_at: morn_kernel::time::Timestamp::now(),
        };
        store.save_source_of_truth_binding(work, &binding).unwrap();
        let attestation_id = format!("observation-attestation:{}", outcome.id);
        outcome
            .pin_source_provenance(binding.id.to_string(), attestation_id.clone(), "test.fact")
            .unwrap();
        store
            .save_record_immutable(
                "source_observation_attestation_consumed_v115",
                &attestation_id,
                work.workspace_id.as_str(),
                outcome.observed_at.millis(),
                &json!({
                    "attestation_id": attestation_id,
                    "work_id": work.id,
                    "source_binding_id": binding.id,
                    "fact_type": "test.fact",
                    "outcome_id": outcome.id,
                    "consumed_at": outcome.observed_at
                }),
            )
            .unwrap();
    }

    #[test]
    fn spoofed_workspace_cannot_retag_work_or_attach_foreign_evidence() {
        let store = MornStore::open_in_memory().unwrap();
        let mut canonical = fixture_work();
        store.save_work_resource_cas(&mut canonical).unwrap();

        let mut spoofed = canonical.clone();
        spoofed.workspace_id = WorkspaceId::generate();
        assert!(store.save_work_resource_cas(&mut spoofed).is_err());
        assert!(store.save_work_resource(&spoofed).is_err());

        let binding = ExecutionBinding::for_work(&spoofed, "cap:a", "provider:a", "v1");
        assert!(store.save_execution_binding(&spoofed, &binding).is_err());

        let outcome = ObservedOutcome::new(
            spoofed.workspace_id.clone(),
            spoofed.id.clone(),
            "forged outcome",
            OutcomeSourceKind::ExternalSystem,
            "fixture://forged",
            json!({"forged": true}),
        );
        assert!(store.save_observed_outcome(&spoofed, &outcome).is_err());

        let persisted: WorkResource = store
            .load_record("work_resource_v115", canonical.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(persisted.workspace_id, canonical.workspace_id);
    }

    #[test]
    fn execution_receipt_is_executor_evidence_not_accepted_outcome() {
        use morn_harness::ExecutionReceipt;
        use morn_kernel::ids::ActorInstanceId;
        use morn_work::control::WorkPhase;

        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        work.status.phase = WorkPhase::Ready;
        store.save_work_resource(&work).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        store.save_execution_binding(&work, &binding).unwrap();

        let ctx = morn_harness::RuntimeContext::new(
            work.workspace_id.clone(),
            ActorInstanceId::generate_with("actor"),
            work.id.clone(),
        )
        .with_work_binding(work.generation, binding.id.clone())
        .unwrap()
        .with_scope_id("scope://run")
        .unwrap();
        let mut receipt = ExecutionReceipt::from_runtime_context(&ctx, "provider:a", "session-a");
        let event = ExecutionEvent::new(
            work.workspace_id.clone(),
            "session-a",
            morn_harness::ExecutionEventKind::Checkpoint,
            "executor settled",
        );
        receipt.outcome = "completed".to_string();
        receipt.ended_at = Some(morn_kernel::time::Timestamp::now());
        receipt.event_ids.push(event.id.to_string());
        receipt.trace_refs.push(event.id.to_string());

        let mut dangling = receipt.clone();
        dangling.id = morn_kernel::ids::ExecutionReceiptId::generate_with("rcpt");
        assert!(store.save_execution_receipt(&work, &dangling).is_err());

        store.save_execution_event(&work, &event).unwrap();
        let durable: WorkExecutionEvent = store
            .load_record("execution_event_v115", event.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(durable.work_package_id, work.id);
        assert_eq!(durable.work_generation, work.generation);
        assert_eq!(durable.event.session_id, "session-a");
        store.save_execution_receipt(&work, &receipt).unwrap();

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                receipt: Some(&receipt),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Waiting);
        assert!(work.condition_is_true("ExecutionReceiptObserved"));
        assert!(!work.condition_is_true("OutcomeObservation"));
        assert!(!work.condition_is_true("IndependentAcceptance"));

        let mut forged = receipt.clone();
        forged.id = morn_kernel::ids::ExecutionReceiptId::generate_with("rcpt");
        forged.provider_ref = Some("provider:b".to_string());
        assert!(store.save_execution_receipt(&work, &forged).is_err());

        let mut other_session_event = ExecutionEvent::new(
            work.workspace_id.clone(),
            "session-b",
            morn_harness::ExecutionEventKind::Checkpoint,
            "wrong session",
        );
        other_session_event.created_at = receipt.started_at;
        store
            .save_execution_event(&work, &other_session_event)
            .unwrap();
        let mut cross_session = receipt.clone();
        cross_session.id = morn_kernel::ids::ExecutionReceiptId::generate_with("rcpt");
        cross_session.event_ids = vec![other_session_event.id.to_string()];
        cross_session.trace_refs = cross_session.event_ids.clone();
        assert!(store.save_execution_receipt(&work, &cross_session).is_err());
    }

    #[test]
    fn capability_resolution_decision_cannot_be_reused_across_work_generations() {
        use morn_capability::{
            CapabilityKind, EffectClass, ResolvedCapability, WorkcellMember, WorkcellPlan,
        };
        use morn_kernel::ExecutionClass;

        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        work.spec.source_solution_ref = Some("solution://fixture@1.0.0".to_string());
        store.save_work_resource_cas(&mut work).unwrap();
        let decision = CapabilityResolutionDecision::new(
            &work,
            WorkcellPlan {
                members: vec![WorkcellMember {
                    capability: ResolvedCapability {
                        manifest_id: morn_capability::CapabilityManifestId::generate_with(
                            "manifest",
                        ),
                        provider_ref: "morn-native".to_string(),
                        kind: CapabilityKind::Program,
                        estimated_cost_micros: None,
                        maximum_effect: EffectClass::E0LifecycleReversible,
                        compensation_ref: None,
                        idempotency_key_required: false,
                        required_execution_class: ExecutionClass::NoIsolation,
                        required_execution_guarantees: vec![],
                        score: 1,
                        rationale: vec![],
                    },
                    covers: vec!["draft".to_string()],
                }],
                uncovered: vec![],
                total_estimated_cost_micros: 0,
                rationale: vec!["fixture".to_string()],
            },
        )
        .unwrap();
        store.save_capability_resolution(&work, &decision).unwrap();

        let mut next = work.spec.clone();
        next.goal = "changed".to_string();
        work.replace_spec(next);
        store.save_work_resource_cas(&mut work).unwrap();
        assert!(store.save_capability_resolution(&work, &decision).is_err());
    }

    #[test]
    fn persisted_binding_and_manifest_cannot_cross_work_generation_or_profile() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        let other = fixture_work();
        store.save_work_resource(&work).unwrap();
        store.save_work_resource(&other).unwrap();

        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        assert!(store.save_execution_binding(&other, &binding).is_err());
        let mut wrong_profile = binding.clone();
        wrong_profile.profile_ref = "morn.lite@1.0.0".to_string();
        assert!(store.save_execution_binding(&work, &wrong_profile).is_err());
        let mut wrong_site = binding.clone();
        wrong_site.site_ref = Some("plant-b".to_string());
        assert!(store.save_execution_binding(&work, &wrong_site).is_err());

        let manifest = ExecutionManifest::from_binding(
            &work,
            &binding,
            CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
        )
        .unwrap();
        assert!(store.save_execution_manifest(&work, &manifest).is_err());
        store.save_execution_binding(&work, &binding).unwrap();
        assert!(store.save_execution_manifest(&other, &manifest).is_err());
        store.save_execution_manifest(&work, &manifest).unwrap();
    }

    #[test]
    fn durable_external_task_observation_requires_exact_work_binding_and_never_closes_work() {
        use morn_integration::InteropProtocol;
        use morn_integration::{
            ExternalEndpoint, ExternalTaskSnapshot, GovernedExternalTaskObservation,
            McpTaskEvidence, McpTaskState,
        };

        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        let other = fixture_work();
        store.save_work_resource(&work).unwrap();
        store.save_work_resource(&other).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:mcp", "mcp-provider", "2026-07-28");
        store.save_execution_binding(&work, &binding).unwrap();
        let endpoint = ExternalEndpoint {
            protocol: InteropProtocol::Mcp,
            endpoint_ref: "https://mcp.example.com".to_string(),
            protocol_version: Some("2026-07-28".to_string()),
            identity_ref: None,
        };
        let interop = InteropBinding {
            work_ref: work.id.to_string(),
            execution_binding_ref: binding.id.to_string(),
            endpoint: endpoint.clone(),
            capability_ref: binding.capability_manifest_ref.clone(),
        };
        store.save_interop_binding(&work, &interop).unwrap();

        let observation = GovernedExternalTaskObservation::new(
            work.id.clone(),
            work.generation,
            binding.id.clone(),
            endpoint.clone(),
            ExternalTaskSnapshot::Mcp(McpTaskEvidence {
                server_ref: "https://mcp.example.com".to_string(),
                task_id: "task-42".to_string(),
                state: McpTaskState::Completed,
                status_message: Some("executor completed".to_string()),
                input_requests: None,
                result: Some(json!({"artifact":"result"})),
                error: None,
            }),
            vec!["mcp://task-42/status/1".to_string()],
        )
        .unwrap();

        assert!(store
            .save_external_task_observation(&other, &observation)
            .is_err());
        store
            .save_external_task_observation(&work, &observation)
            .unwrap();

        let mut wrong_endpoint = observation.clone();
        wrong_endpoint.id =
            morn_integration::ExternalTaskObservationId::generate_with("external-task-observation");
        wrong_endpoint.endpoint.endpoint_ref = "https://other-mcp.example.com".to_string();
        if let ExternalTaskSnapshot::Mcp(task) = &mut wrong_endpoint.snapshot {
            task.server_ref = "https://other-mcp.example.com".to_string();
        }
        assert!(store
            .save_external_task_observation(&work, &wrong_endpoint)
            .is_err());
        assert!(observation.snapshot.is_executor_terminal());
        assert!(!observation.proves_morn_acceptance());

        let restored: GovernedExternalTaskObservation = store
            .load_record("external_task_observation_v115", observation.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(restored.work_id, work.id);
        assert_eq!(restored.execution_binding_ref, binding.id);
    }

    #[test]
    fn external_task_observation_cannot_predate_binding_or_arrive_from_the_future() {
        use morn_integration::{
            ExternalEndpoint, ExternalTaskSnapshot, GovernedExternalTaskObservation,
            InteropBinding, InteropProtocol, McpTaskEvidence, McpTaskState,
        };

        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:mcp", "mcp-provider", "2026-07-28");
        store.save_execution_binding(&work, &binding).unwrap();
        let endpoint = ExternalEndpoint {
            protocol: InteropProtocol::Mcp,
            endpoint_ref: "https://mcp.example.com".to_string(),
            protocol_version: Some("2026-07-28".to_string()),
            identity_ref: None,
        };
        store
            .save_interop_binding(
                &work,
                &InteropBinding {
                    work_ref: work.id.to_string(),
                    execution_binding_ref: binding.id.to_string(),
                    endpoint: endpoint.clone(),
                    capability_ref: binding.capability_manifest_ref.clone(),
                },
            )
            .unwrap();
        let mut observation = GovernedExternalTaskObservation::new(
            work.id.clone(),
            work.generation,
            binding.id.clone(),
            endpoint,
            ExternalTaskSnapshot::Mcp(McpTaskEvidence {
                server_ref: "https://mcp.example.com".to_string(),
                task_id: "task-time".to_string(),
                state: McpTaskState::Working,
                status_message: None,
                input_requests: None,
                result: None,
                error: None,
            }),
            vec!["mcp://task-time/status".to_string()],
        )
        .unwrap();

        observation.observed_at =
            morn_kernel::time::Timestamp::from_millis(binding.created_at.millis() - 1);
        assert!(store
            .save_external_task_observation(&work, &observation)
            .is_err());

        observation.observed_at = morn_kernel::time::Timestamp::from_millis(
            morn_kernel::time::Timestamp::now().millis() + 60_000,
        );
        assert!(store
            .save_external_task_observation(&work, &observation)
            .is_err());
    }

    #[test]
    fn terminal_external_task_observation_cannot_be_resurrected() {
        use morn_integration::{
            ExternalEndpoint, ExternalTaskSnapshot, GovernedExternalTaskObservation,
            InteropBinding, InteropProtocol, McpTaskEvidence, McpTaskState,
        };

        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:mcp", "mcp-provider", "2026-07-28");
        store.save_execution_binding(&work, &binding).unwrap();
        let endpoint = ExternalEndpoint {
            protocol: InteropProtocol::Mcp,
            endpoint_ref: "https://mcp.example.com".to_string(),
            protocol_version: Some("2026-07-28".to_string()),
            identity_ref: None,
        };
        store
            .save_interop_binding(
                &work,
                &InteropBinding {
                    work_ref: work.id.to_string(),
                    execution_binding_ref: binding.id.to_string(),
                    endpoint: endpoint.clone(),
                    capability_ref: binding.capability_manifest_ref.clone(),
                },
            )
            .unwrap();

        let completed = GovernedExternalTaskObservation::new(
            work.id.clone(),
            work.generation,
            binding.id.clone(),
            endpoint.clone(),
            ExternalTaskSnapshot::Mcp(McpTaskEvidence {
                server_ref: endpoint.endpoint_ref.clone(),
                task_id: "task-terminal".to_string(),
                state: McpTaskState::Completed,
                status_message: None,
                input_requests: None,
                result: Some(json!({"content":[],"isError":false})),
                error: None,
            }),
            vec!["mcp://task-terminal/final".to_string()],
        )
        .unwrap();
        store
            .save_external_task_observation(&work, &completed)
            .unwrap();

        let mut resurrected = GovernedExternalTaskObservation::new(
            work.id.clone(),
            work.generation,
            binding.id.clone(),
            endpoint,
            ExternalTaskSnapshot::Mcp(McpTaskEvidence {
                server_ref: "https://mcp.example.com".to_string(),
                task_id: "task-terminal".to_string(),
                state: McpTaskState::Working,
                status_message: Some("impossible restart".to_string()),
                input_requests: None,
                result: None,
                error: None,
            }),
            vec!["mcp://task-terminal/restarted".to_string()],
        )
        .unwrap();
        resurrected.observed_at =
            morn_kernel::time::Timestamp::from_millis(completed.observed_at.millis() + 1);
        assert!(store
            .save_external_task_observation(&work, &resurrected)
            .is_err());
    }

    #[test]
    fn action_attempt_and_reconciliation_require_their_exact_work_binding() {
        use morn_runtime::{
            ActionAttempt, AttemptState, ReconciliationObservation, ReconciliationRecord,
            ReconciliationRecordId,
        };
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        let other = fixture_work();
        store.save_work_resource(&work).unwrap();
        store.save_work_resource(&other).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        store.save_execution_binding(&work, &binding).unwrap();
        let attempt = ActionAttempt::new(binding.id.clone(), "work-a:create", "create");
        assert!(store.save_action_attempt(&other, &attempt).is_err());
        store.save_action_attempt(&work, &attempt).unwrap();

        let mut record = ReconciliationRecord {
            id: ReconciliationRecordId::generate_with("reconcile"),
            attempt_id: attempt.id.clone(),
            before: AttemptState::OutcomeUnknown,
            after: AttemptState::Reconciling,
            observation: ReconciliationObservation {
                business_key: "some-other-work:create".to_string(),
                committed: None,
                observed: None,
                external_ref: None,
                evidence_refs: vec![],
            },
            created_at: morn_kernel::time::Timestamp::now(),
        };
        assert!(store.save_reconciliation(&work, &record).is_err());
        record.observation.business_key = attempt.business_key.clone();
        assert!(store.save_reconciliation(&other, &record).is_err());
        store.save_reconciliation(&work, &record).unwrap();
    }

    #[test]
    fn outcome_unknown_harness_receipt_forces_reconciliation_before_new_execution() {
        let mut work = fixture_work();
        work.status.phase = WorkPhase::Ready;
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        let mut receipt = morn_harness::ExecutionReceipt::from_runtime_context(
            &morn_harness::RuntimeContext::new(
                work.workspace_id.clone(),
                morn_kernel::ids::ActorInstanceId::generate_with("actor"),
                work.id.clone(),
            )
            .with_work_binding(work.generation, binding.id.clone())
            .unwrap()
            .with_scope_id("scope://unknown-reconciliation-test")
            .unwrap(),
            "provider:a",
            "session-a",
        );
        receipt.outcome = "outcome-unknown".to_string();
        receipt.ended_at = None;
        receipt
            .trace_refs
            .push("provider://ambiguous-turn".to_string());

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                receipt: Some(&receipt),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Reconciling);
        assert!(!work.condition_is_true("ExecutorOutcomeKnown"));
        assert!(work
            .status
            .conditions
            .iter()
            .any(
                |condition| condition.condition_type == "ExecutorOutcomeKnown"
                    && condition.status == ConditionStatus::False
            ));
    }

    #[test]
    fn settled_failed_execution_receipt_blocks_work_instead_of_waiting_for_outcome() {
        let mut work = fixture_work();
        work.status.phase = WorkPhase::Ready;
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        let mut receipt = morn_harness::ExecutionReceipt::from_runtime_context(
            &morn_harness::RuntimeContext::new(
                work.workspace_id.clone(),
                morn_kernel::ids::ActorInstanceId::generate_with("actor"),
                work.id.clone(),
            )
            .with_work_binding(work.generation, binding.id.clone())
            .unwrap()
            .with_scope_id("scope://cleanup-failed")
            .unwrap(),
            "provider:a",
            "session-a",
        );
        receipt.outcome = "cleanup-failed".to_string();
        receipt.ended_at = Some(morn_kernel::time::Timestamp::now());
        receipt.event_ids.push("event://cleanup-failed".to_string());
        receipt
            .trace_refs
            .push("event://cleanup-failed".to_string());

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                receipt: Some(&receipt),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Blocked);
        assert!(work.condition_is_true("ExecutorOutcomeKnown"));
        assert!(work.status.conditions.iter().any(|condition| {
            condition.condition_type == "ExecutorExecutionSucceeded"
                && condition.status == ConditionStatus::False
        }));
        assert!(!work.condition_is_true("OutcomeObservation"));
        assert!(!work.condition_is_true("IndependentAcceptance"));
    }

    #[test]
    fn unknown_executor_outcome_cannot_be_erased_by_creating_a_new_generation() {
        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        store.save_work_resource_cas(&mut work).unwrap();

        let mut unknown = WorkCondition::new("ExecutorOutcomeKnown", ConditionStatus::False);
        unknown.reason = "ambiguous external executor activity".to_string();
        unknown.evidence_refs = vec!["receipt://unknown".to_string()];
        work.set_condition(unknown);
        work.status.phase = WorkPhase::Reconciling;
        store.save_work_resource_cas(&mut work).unwrap();

        let mut next = work.clone();
        let mut spec = next.spec.clone();
        spec.goal = "attempt to bypass unknown outcome".to_string();
        next.replace_spec(spec);
        assert!(store.save_work_resource_cas(&mut next).is_err());

        let mut known = work.clone();
        let mut resolved = WorkCondition::new("ExecutorOutcomeKnown", ConditionStatus::True);
        resolved.reason = "trusted reconciliation resolved executor settlement".to_string();
        resolved.evidence_refs = vec!["exec-reconcile://1".to_string()];
        known.set_condition(resolved);
        known.status.phase = WorkPhase::Blocked;
        store.save_work_resource_cas(&mut known).unwrap();

        let mut permitted = known.clone();
        let mut spec = permitted.spec.clone();
        spec.goal = "explicit next generation after reconciliation".to_string();
        permitted.replace_spec(spec);
        store.save_work_resource_cas(&mut permitted).unwrap();
        assert_eq!(permitted.generation, 2);
    }

    #[test]
    fn same_generation_spec_and_terminal_state_cannot_be_reinterpreted() {
        let store = MornStore::open_in_memory().unwrap();
        let mut canonical = fixture_work();
        store.save_work_resource_cas(&mut canonical).unwrap();

        let mut tampered = canonical.clone();
        tampered.spec.goal = "different instructions".to_string();
        assert!(store.save_work_resource_cas(&mut tampered).is_err());
        assert!(store.save_work_resource(&tampered).is_err());

        canonical.status.phase = WorkPhase::Cancelled;
        store.save_work_resource_cas(&mut canonical).unwrap();
        let mut rollback = canonical.clone();
        rollback.status.phase = WorkPhase::Ready;
        assert!(store.save_work_resource_cas(&mut rollback).is_err());

        canonical.replace_spec({
            let mut next = canonical.spec.clone();
            next.goal = "explicit new requested objective".to_string();
            next
        });
        store.save_work_resource_cas(&mut canonical).unwrap();
        assert_eq!(canonical.generation, 2);
    }

    #[test]
    fn work_projection_cannot_outrun_persisted_outcome_or_acceptance_evidence() {
        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        store.save_work_resource_cas(&mut work).unwrap();

        let mut delivered_without_evidence = work.clone();
        delivered_without_evidence.status.phase = WorkPhase::Delivered;
        delivered_without_evidence.set_condition(WorkCondition::new(
            "OutcomeObservation",
            ConditionStatus::True,
        ));
        assert!(store
            .save_work_resource_cas(&mut delivered_without_evidence)
            .is_err());

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "authoritative delivery result",
            OutcomeSourceKind::ExternalSystem,
            "system://delivery/42",
            json!({"status":"complete"}),
        );
        outcome.pin_work_generation(work.generation).unwrap();
        outcome
            .evidence_refs
            .push("system://delivery/42/receipt".to_string());
        attest_test_outcome(&store, &work, &mut outcome);
        store.save_observed_outcome(&work, &outcome).unwrap();

        work.status.phase = WorkPhase::Delivered;
        work.set_condition(WorkCondition::new(
            "OutcomeObservation",
            ConditionStatus::True,
        ));
        store.save_work_resource_cas(&mut work).unwrap();

        let mut accepted_without_decision = work.clone();
        accepted_without_decision.status.phase = WorkPhase::Accepted;
        accepted_without_decision.set_condition(WorkCondition::new(
            "IndependentAcceptance",
            ConditionStatus::True,
        ));
        accepted_without_decision.set_condition(WorkCondition::new(
            "AcceptedOutcomeSemantics",
            ConditionStatus::True,
        ));
        assert!(store
            .save_work_resource_cas(&mut accepted_without_decision)
            .is_err());

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("reviewer"),
            "independent-reviewer",
            "grounded outcome independently accepted",
        );
        decision.pin_work_generation(work.generation).unwrap();
        decision.outcome_refs.push(outcome.id.clone());
        decision
            .evidence_refs
            .push("review://signed/42".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();

        work.status.phase = WorkPhase::Accepted;
        work.set_condition(WorkCondition::new(
            "IndependentAcceptance",
            ConditionStatus::True,
        ));
        work.set_condition(WorkCondition::new(
            "AcceptedOutcomeSemantics",
            ConditionStatus::True,
        ));
        store.save_work_resource_cas(&mut work).unwrap();
    }

    #[test]
    fn persisted_attempt_cannot_rewind_unknown_effect_or_relabel_business_key() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();
        let binding = ExecutionBinding::for_work(&work, "cap:a", "provider:a", "v1");
        store.save_execution_binding(&work, &binding).unwrap();

        let mut attempt = ActionAttempt::new(binding.id.clone(), "work:create", "create");
        store.save_action_attempt(&work, &attempt).unwrap();
        let stale = attempt.clone();
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        store.save_action_attempt(&work, &attempt).unwrap();
        attempt.mark_outcome_unknown("timeout").unwrap();
        store.save_action_attempt(&work, &attempt).unwrap();
        assert!(store.save_action_attempt(&work, &stale).is_err());

        let mut forged = attempt.clone();
        forged.state = AttemptState::Dispatched;
        assert!(store.save_action_attempt(&work, &forged).is_err());
        forged = attempt.clone();
        forged.business_key = "other:create".to_string();
        assert!(store.save_action_attempt(&work, &forged).is_err());

        attempt.transition(AttemptState::Reconciling).unwrap();
        attempt.external_ref = Some("cmms://order-1".to_string());
        attempt
            .evidence_refs
            .push("cmms://order-1/receipt".to_string());
        attempt.transition(AttemptState::Observed).unwrap();
        store.save_action_attempt(&work, &attempt).unwrap();
        let mut forged = attempt.clone();
        forged.evidence_refs.clear();
        assert!(store.save_action_attempt(&work, &forged).is_err());
        let stored: ActionAttempt = store
            .load_record("action_attempt_v115", attempt.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(stored.state, AttemptState::Observed);
        assert_eq!(stored.business_key, "work:create");
    }

    #[test]
    fn terminal_rejection_requires_the_same_grounded_review_chain_as_acceptance() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();

        let ungrounded = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "unwitnessed result",
            OutcomeSourceKind::ExternalSystem,
            "system://status",
            json!({"ok": false}),
        );
        store.save_observed_outcome(&work, &ungrounded).unwrap();

        let mut reject = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Reject,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "authoritative result does not meet criteria",
        );
        reject.outcome_refs.push(ungrounded.id.clone());
        reject
            .evidence_refs
            .push("review://ticket-reject".to_string());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &reject).is_err());

        let mut grounded = ungrounded.clone();
        grounded.id = morn_kernel::ids::OutcomeRecordId::generate_with("out");
        grounded
            .evidence_refs
            .push("system://status/receipt".to_string());
        store.save_observed_outcome(&work, &grounded).unwrap();

        reject.outcome_refs = vec![grounded.id.clone()];
        reject.evidence_refs.clear();
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &reject).is_err());
        reject
            .evidence_refs
            .push("review://ticket-reject".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &reject).unwrap();
    }

    #[test]
    fn old_generation_outcome_and_acceptance_cannot_advance_new_work_generation() {
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};
        use serde_json::json;

        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        store.save_work_resource_cas(&mut work).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "generation one result",
            OutcomeSourceKind::ExternalSystem,
            "system://result/1",
            json!({"status":"complete"}),
        );
        outcome
            .evidence_refs
            .push("system://result/1/receipt".to_string());
        attest_test_outcome(&store, &work, &mut outcome);
        store.save_observed_outcome(&work, &outcome).unwrap();

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("accept"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("reviewer"),
            "independent-reviewer",
            "generation one accepted",
        );
        decision.outcome_refs.push(outcome.id.clone());
        decision
            .evidence_refs
            .push("review://generation-one".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();

        let mut next = work.spec.clone();
        next.goal = "different generation two objective".to_string();
        work.replace_spec(next);
        store.save_work_resource_cas(&mut work).unwrap();
        assert_eq!(work.generation, 2);

        assert!(store.save_observed_outcome(&work, &outcome).is_err());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).is_err());

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                outcome: Some(&outcome),
                acceptance: Some(&decision),
                ..Default::default()
            },
        );
        assert_ne!(work.status.phase, WorkPhase::Accepted);
        assert_ne!(work.status.phase, WorkPhase::Delivered);
    }

    #[test]
    fn terminal_acceptance_must_match_pinned_acceptance_spec_before_persistence() {
        let store = MornStore::open_in_memory().unwrap();
        let mut work = fixture_work();
        let expected = AcceptanceSpecId::generate_with("acceptance");
        work.spec.acceptance_ref = Some(expected.to_string());
        store.save_work_resource(&work).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "delivery-impact review",
            OutcomeSourceKind::ExternalSystem,
            "cmms://plant-a/status",
            json!({"delivered": true}),
        );
        outcome
            .evidence_refs
            .push("cmms://plant-a/status/receipt".to_string());
        attest_test_outcome(&store, &work, &mut outcome);
        store.save_observed_outcome(&work, &outcome).unwrap();

        let mut wrong = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("other-acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "reviewed against the wrong criteria",
        );
        wrong.outcome_refs.push(outcome.id.clone());
        wrong
            .evidence_refs
            .push("review://ticket-wrong".to_string());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &wrong).is_err());

        let mut correct = AcceptanceDecision::new(
            work.id.clone(),
            expected,
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "reviewed against the pinned criteria",
        );
        correct.outcome_refs.push(outcome.id.clone());
        correct
            .evidence_refs
            .push("review://ticket-correct".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &correct).unwrap();
    }

    #[test]
    fn final_acceptance_requires_deployment_attested_source_provenance() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "source-grounded but not deployment-attested",
            OutcomeSourceKind::ExternalSystem,
            "erp://delivery/42",
            json!({"status":"delivered"}),
        );
        outcome
            .evidence_refs
            .push("erp://delivery/42/receipt".to_string());
        store.save_observed_outcome(&work, &outcome).unwrap();

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("reviewer"),
            "independent-reviewer",
            "reviewed authoritative evidence",
        );
        decision.outcome_refs.push(outcome.id.clone());
        decision
            .evidence_refs
            .push("review://signed/42".to_string());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).is_err());

        let mut attested = outcome.clone();
        attested.id = morn_kernel::ids::OutcomeRecordId::generate_with("out");
        attest_test_outcome(&store, &work, &mut attested);
        store.save_observed_outcome(&work, &attested).unwrap();
        decision.outcome_refs = vec![attested.id.clone()];
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();
    }

    #[test]
    fn acceptance_and_customer_value_cannot_claim_unwitnessed_or_unaccepted_outcomes() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        store.save_work_resource(&work).unwrap();
        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "delivery-impact review",
            OutcomeSourceKind::ExternalSystem,
            "cmms://plant-a/status",
            json!({"delivered": true}),
        );
        store.save_observed_outcome(&work, &outcome).unwrap();

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("accept"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "independent review of source facts",
        );
        decision.outcome_refs.push(outcome.id.clone());
        decision.evidence_refs.push("review://ticket-a".to_string());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).is_err());

        let mut fake_value = ValueAssessment::new(
            work.id.clone(),
            outcome.id.clone(),
            ValueEvidenceClass::CustomerValidated,
        );
        fake_value.acceptance_ref = Some(decision.id.clone());
        fake_value.customer_value_attestation_ref =
            Some("customer-value-attestation:test".to_string());
        fake_value
            .kpis
            .push(("delivery_minutes_saved".to_string(), 12.0));
        fake_value
            .evidence_refs
            .push("fixture://not-customer".to_string());
        assert!(store.save_value_assessment(&work, &fake_value).is_err());

        outcome.id = morn_kernel::ids::OutcomeRecordId::generate_with("out");
        outcome
            .evidence_refs
            .push("cmms://plant-a/status/receipt".to_string());
        attest_test_outcome(&store, &work, &mut outcome);
        store.save_observed_outcome(&work, &outcome).unwrap();
        decision.outcome_refs = vec![outcome.id.clone()];
        decision.evidence_refs.clear();
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).is_err());
        decision.evidence_refs.push("review://ticket-a".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();

        fake_value.outcome_ref = outcome.id.clone();
        fake_value.evidence_refs = vec!["customer://signed/value-a".to_string()];
        let attestation = morn_assurance::CustomerValueAttestation {
            attestation_id: "customer-value-attestation:test".to_string(),
            workspace_id: work.workspace_id.clone(),
            work_package_id: work.id.clone(),
            work_generation: work.generation,
            outcome_id: outcome.id.clone(),
            acceptance_id: decision.id.clone(),
            baseline_ref: None,
            kpis: vec![morn_assurance::CustomerValueMetric {
                name: "delivery_minutes_saved".to_string(),
                value: 12.0,
            }],
            evidence_refs: vec!["customer://signed/value-a".to_string()],
            issuer: "customer-governance".to_string(),
            observed_at: morn_kernel::time::Timestamp::from_millis(1),
            valid_until: None,
        };
        store
            .save_record_immutable(
                "customer_value_attestation_v115",
                &attestation.attestation_id,
                work.workspace_id.as_str(),
                attestation.observed_at.millis(),
                &attestation,
            )
            .unwrap();
        store.save_value_assessment(&work, &fake_value).unwrap();
        assert!(fake_value.is_customer_value_claim());
    }

    #[test]
    fn acceptance_and_value_cannot_borrow_outcomes_from_other_workspaces() {
        let store = MornStore::open_in_memory().unwrap();
        let work = fixture_work();
        let other = fixture_work();
        store.save_work_resource(&work).unwrap();
        store.save_work_resource(&other).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "maintenance review complete",
            OutcomeSourceKind::ExternalSystem,
            "cmms://plant-a/orders/123",
            json!({"reviewed": true}),
        );
        outcome
            .evidence_refs
            .push("cmms://plant-a/orders/123/receipt".to_string());
        assert!(store.save_observed_outcome(&other, &outcome).is_err());
        attest_test_outcome(&store, &work, &mut outcome);
        store.save_observed_outcome(&work, &outcome).unwrap();

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "verified",
        );
        decision.outcome_refs.push(outcome.id.clone());
        decision
            .evidence_refs
            .push("review://independent-witness".to_string());
        assert!(ControlPlaneStore::save_acceptance_decision(&store, &other, &decision).is_err());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();

        let mut foreign_decision = AcceptanceDecision::new(
            other.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "independent-reviewer",
            "incorrect cross-work outcome",
        );
        foreign_decision.outcome_refs.push(outcome.id.clone());
        assert!(
            ControlPlaneStore::save_acceptance_decision(&store, &other, &foreign_decision).is_err()
        );

        let assessment = ValueAssessment::new(
            work.id.clone(),
            outcome.id.clone(),
            ValueEvidenceClass::Fixture,
        );
        assert!(store.save_value_assessment(&other, &assessment).is_err());
        store.save_value_assessment(&work, &assessment).unwrap();
    }
}

#[cfg(test)]
mod terminal_phase_tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};

    #[test]
    fn rejection_without_grounded_matching_review_cannot_terminally_close_work() {
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};
        use serde_json::json;

        let spec = morn_work::control::WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review result",
            "morn.lite@1.0.0",
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        work.status.phase = WorkPhase::Waiting;
        let outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "result",
            OutcomeSourceKind::ExternalSystem,
            "system://result",
            json!({"ok": false}),
        );
        let mut reject = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Reject,
            PrincipalId::generate_with("principal"),
            "reviewer",
            "does not satisfy criteria",
        );
        reject.outcome_refs.push(outcome.id.clone());
        reject.evidence_refs.push("review://ticket".to_string());

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                outcome: Some(&outcome),
                acceptance: Some(&reject),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Waiting);

        let mut grounded = outcome;
        grounded
            .evidence_refs
            .push("system://result/receipt".to_string());
        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                outcome: Some(&grounded),
                acceptance: Some(&reject),
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Rejected);
    }

    #[test]
    fn late_evidence_cannot_silently_reopen_accepted_work() {
        let spec = morn_work::control::WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "accepted work",
            "morn.lite@1.0.0",
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        work.status.phase = WorkPhase::Accepted;
        work.status.observed_generation = work.generation;

        WorkController.reconcile(
            &mut work,
            &DomainProfile::lite_v1(),
            &ControllerInputs {
                capability_resolved: false,
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Accepted);

        WorkProgressController.reconcile(&mut work, &WorkProgressInputs::default());
        assert_eq!(work.status.phase, WorkPhase::Accepted);

        let mut next = work.spec.clone();
        next.goal = "explicitly changed generation".to_string();
        work.replace_spec(next);
        assert_eq!(work.status.phase, WorkPhase::Proposed);
        WorkController.reconcile(
            &mut work,
            &DomainProfile::lite_v1(),
            &ControllerInputs {
                capability_resolved: true,
                ..Default::default()
            },
        );
        assert_eq!(work.status.phase, WorkPhase::Ready);
    }
}
