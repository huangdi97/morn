//! execution_flow — BindingController / AuthorityController / ExecutionController.
//!
//! The attempt path: Work → Action proposal → AuthorityDecision →
//! ExecutionBinding → Provider dispatch. An Attempt requires a Binding before
//! it starts and pins it; an Agent never holds production credentials.

use super::ControllerOutcome;
use crate::control::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{
    ActionAttempt, AttemptStatus, AuthorityDecision, AuthorityRequest, BindingStatus, Condition,
    ConditionType, ExecutionBinding,
};

/// BindingController — creates Bindings for a Work.
pub struct BindingController;

impl BindingController {
    pub const ID: &'static str = "binding-controller";

    /// Creates a Binding for a Work (idempotent by binding_id) and records the
    /// ExecutionBound condition.
    pub fn bind(
        &self,
        store: &ControlPlaneStore,
        binding: &ExecutionBinding,
    ) -> Result<ControllerOutcome, MornError> {
        let inserted = store.insert_binding(binding)?;
        let note = if inserted {
            "binding created"
        } else {
            "binding already exists (idempotent)"
        };
        Ok(ControllerOutcome::new(Self::ID, &binding.work_ref)
            .with_condition(Condition::satisfied(ConditionType::ExecutionBound))
            .with_note(note))
    }

    /// Suspends a Binding on provider failure (active attempts move to
    /// SUSPENDED/UNKNOWN; the binding history stays).
    pub fn suspend(
        &self,
        store: &ControlPlaneStore,
        binding_id: &str,
        work_ref: &str,
        reason: &str,
    ) -> Result<ControllerOutcome, MornError> {
        store.update_binding_status(binding_id, BindingStatus::Suspended)?;
        Ok(ControllerOutcome::new(Self::ID, work_ref)
            .with_note(&format!("binding {binding_id} suspended: {reason}")))
    }
}

/// AuthorityController — records authority requests/decisions.
pub struct AuthorityController;

impl AuthorityController {
    pub const ID: &'static str = "authority-controller";

    /// Decides an authority request: grants produce AuthoritySatisfied; denials
    /// do not. Decision and enforcement stay separated (PEP is the gateway).
    /// The store parameter is reserved for persisting decisions in Phase 8.
    pub fn decide(
        &self,
        _store: &ControlPlaneStore,
        work_ref: &str,
        request: &AuthorityRequest,
        decision: &AuthorityDecision,
    ) -> Result<ControllerOutcome, MornError> {
        let now = crate::work::now_iso();
        if !request.is_within_time_window(&now) {
            return Ok(ControllerOutcome::new(Self::ID, work_ref)
                .with_condition(Condition::unsatisfied(
                    ConditionType::AuthoritySatisfied,
                    "request outside time window",
                ))
                .with_note("stale authority request"));
        }
        if decision.authorizes(&now) {
            Ok(ControllerOutcome::new(Self::ID, work_ref)
                .with_condition(Condition::satisfied(ConditionType::AuthoritySatisfied))
                .with_note(&format!("authority {} granted", decision.id)))
        } else {
            Ok(ControllerOutcome::new(Self::ID, work_ref)
                .with_condition(Condition::unsatisfied(
                    ConditionType::AuthoritySatisfied,
                    decision.reason.as_deref().unwrap_or("denied"),
                ))
                .with_note(&format!("authority {} not granted", decision.id)))
        }
    }
}

/// ExecutionController — dispatches Attempts under a Binding.
pub struct ExecutionController;

impl ExecutionController {
    pub const ID: &'static str = "execution-controller";

    /// Dispatches an Attempt: PROPOSED → AUTHORIZED → DISPATCHED.
    ///
    /// Idempotency: an attempt idempotency_key is required; a duplicate insert
    /// returns false and is not dispatched again. After dispatch, the attempt
    /// pin is recorded on the Work.
    pub fn dispatch(
        &self,
        store: &ControlPlaneStore,
        attempt: &ActionAttempt,
    ) -> Result<ControllerOutcome, MornError> {
        let Some(binding) = store.get_binding(&attempt.binding_ref)? else {
            // SAFETY: no Attempt starts without a Binding.
            return Ok(ControllerOutcome::new(Self::ID, &attempt.work_ref)
                .with_condition(Condition::unsatisfied(
                    ConditionType::ExecutionBound,
                    "attempt has no binding",
                ))
                .with_note("dispatch refused: binding missing"));
        };
        if binding.status != BindingStatus::Bound {
            return Ok(ControllerOutcome::new(Self::ID, &attempt.work_ref)
                .with_condition(Condition::unsatisfied(
                    ConditionType::ExecutionBound,
                    "binding not BOUND",
                ))
                .with_note("dispatch refused: binding suspended/revoked"));
        }

        let inserted = store.insert_attempt(attempt)?;
        if !inserted {
            return Ok(ControllerOutcome::new(Self::ID, &attempt.work_ref)
                .with_note("duplicate attempt prevented by idempotency key"));
        }
        store.update_attempt_status(&attempt.id, AttemptStatus::Dispatched)?;

        // Record the attempt pin on the Work status.
        if let Some(mut work) = store.get_work(&attempt.work_ref)? {
            if !work
                .status
                .active_attempt_refs
                .iter()
                .any(|a| a == &attempt.id)
            {
                work.status.active_attempt_refs.push(attempt.id.clone());
                store.update_work_status(&attempt.work_ref, &work.status)?;
            }
        }

        Ok(ControllerOutcome::new(Self::ID, &attempt.work_ref)
            .with_condition(Condition::satisfied(ConditionType::ActionCommitted))
            .with_note(&format!("attempt {} dispatched", attempt.id)))
    }
}
