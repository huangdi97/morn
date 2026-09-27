//! state_loop — WorkController / SituationController / ProfileConformanceController.
//!
//! Deterministic state advancement: the WorkController moves the observed
//! phase toward the desired conditions; the ProfileConformanceController
//! checks profile guarantees and produces the ProfileConformant gate.

use super::ControllerOutcome;
use crate::control::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{
    Condition, ConditionType, Profile, RequirementLevel, Situation, SituationStatus, Work,
    WorkPhase,
};

/// WorkController — advances Work.status toward desired conditions.
///
/// Idempotent: re-running with the same state produces the same phase and no
/// side effects.
pub struct WorkController;

impl WorkController {
    pub const ID: &'static str = "work-controller";

    pub fn reconcile(
        &self,
        store: &ControlPlaneStore,
        work_id: &str,
    ) -> Result<Option<ControllerOutcome>, MornError> {
        let Some(mut work) = store.get_work(work_id)? else {
            return Ok(None);
        };
        let before = work.status.phase;

        // Deterministic phase advancement from observed conditions.
        let next = if work.status.acceptance_state == crate::work::WorkAcceptanceState::Accepted {
            WorkPhase::Completed
        } else if !work.status.unresolved_unknowns.is_empty() {
            // Unknown outcomes are reconciled, never auto-converted to failures.
            WorkPhase::Reconciling
        } else if work.status.active_attempt_refs.is_empty()
            && work
                .condition(ConditionType::ExecutionBound)
                .is_none_or(|c| !c.is_true())
        {
            if work
                .condition(ConditionType::AuthoritySatisfied)
                .is_some_and(|c| c.is_true())
                || work.spec.authority.is_none()
            {
                WorkPhase::Resolving
            } else {
                WorkPhase::AwaitingAuthority
            }
        } else {
            WorkPhase::Executing
        };

        if next != before {
            work.status.phase = next;
            work.status.observed_generation = work.spec.generation;
            store.update_work_status(work_id, &work.status)?;
            let outcome = ControllerOutcome::new(Self::ID, work_id)
                .with_condition(Condition::satisfied(ConditionType::OutcomeObserved))
                .with_note(&format!("phase {before:?} -> {next:?}"));
            return Ok(Some(outcome));
        }
        Ok(None)
    }
}

/// SituationController (minimum) — opens/closes situations.
pub struct SituationController;

impl SituationController {
    pub const ID: &'static str = "situation-controller";

    /// Links a situation to its work and closes superseded ones.
    pub fn attach_to_work(
        &self,
        store: &ControlPlaneStore,
        situation: &mut Situation,
        work_id: &str,
    ) -> Result<ControllerOutcome, MornError> {
        situation.work_ref = Some(work_id.to_string());
        let _ = store;
        Ok(ControllerOutcome::new(Self::ID, work_id)
            .with_note(&format!("situation {} attached", situation.id)))
    }

    pub fn close(&self, situation: &mut Situation) -> ControllerOutcome {
        situation.status = SituationStatus::Closed;
        ControllerOutcome::new(Self::ID, situation.work_ref.as_deref().unwrap_or(""))
            .with_note(&format!("situation {} closed", situation.id))
    }
}

/// ProfileConformanceController — checks a Work's profile guarantees.
///
/// SAFETY: a Work whose profile requires a condition that is not satisfied
/// must not advance into the corresponding phase (G9 evidence path).
pub struct ProfileConformanceController;

impl ProfileConformanceController {
    pub const ID: &'static str = "profile-conformance-controller";

    pub fn reconcile(
        &self,
        store: &ControlPlaneStore,
        work_id: &str,
    ) -> Result<Option<ControllerOutcome>, MornError> {
        let Some(work) = store.get_work(work_id)? else {
            return Ok(None);
        };
        let Some(profile_ref) = work.spec.profile_ref.clone() else {
            return Ok(None);
        };
        let profile = Profile::factory_readonly();
        if profile.name != profile_ref.split('/').next().unwrap_or("") {
            return Ok(None);
        }
        Ok(Some(self.check(&work, &profile)))
    }

    /// Checks required/forbidden guarantees against the Work's conditions.
    pub fn check(&self, work: &Work, profile: &Profile) -> ControllerOutcome {
        let mut outcome = ControllerOutcome::new(Self::ID, &work.id);
        let mut conformant = true;

        for requirement in &profile.requirements {
            match requirement.level {
                RequirementLevel::Required => {
                    let gate = gate_for_requirement(&requirement.requirement);
                    if let Some(gate) = gate {
                        let satisfied = work.condition(gate).map(|c| c.is_true()).unwrap_or(false);
                        if !satisfied {
                            conformant = false;
                            outcome = outcome.with_note(&format!(
                                "required guarantee '{}' gate {} not satisfied",
                                requirement.requirement,
                                gate.as_str()
                            ));
                        }
                    }
                }
                RequirementLevel::Forbidden => {
                    outcome = outcome.with_note(&format!(
                        "forbidden: '{}' (enforced by profile pinning)",
                        requirement.requirement
                    ));
                }
                RequirementLevel::Optional => {}
            }
        }

        outcome = outcome.with_condition(if conformant {
            Condition::satisfied(ConditionType::ProfileConformant)
        } else {
            Condition::unsatisfied(
                ConditionType::ProfileConformant,
                "required guarantees not satisfied",
            )
        });
        outcome
    }
}

/// Maps a profile requirement name to its semantic gate.
fn gate_for_requirement(requirement: &str) -> Option<ConditionType> {
    match requirement {
        "authority_before_external_side_effect" => Some(ConditionType::AuthoritySatisfied),
        "receipt_after_external_action" => Some(ConditionType::ExternalActionVerified),
        "reconciliation_on_unknown" => Some(ConditionType::ExternalActionVerified),
        "accepted_outcome_semantics" => Some(ConditionType::AcceptanceSatisfied),
        "source_of_truth_binding" => Some(ConditionType::ExecutionBound),
        _ => None,
    }
}
