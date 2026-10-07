//! Real-world action attempt lifecycle.
//!
//! A proposal, dispatch, external commit, observed outcome and verification are
//! different facts. In particular, a timeout after dispatch is represented as
//! OutcomeUnknown and must be reconciled before another side effect is tried.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{Id, RuntimeBindingId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActionAttemptTag;
pub type ActionAttemptId = Id<ActionAttemptTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AttemptState {
    Proposed,
    Authorized,
    Dispatched,
    Acknowledged,
    Committed,
    OutcomeUnknown,
    Reconciling,
    Observed,
    Verified,
    Failed,
    Cancelled,
}

impl AttemptState {
    pub fn can_transition_to(self, next: AttemptState) -> bool {
        use AttemptState::*;
        matches!(
            (self, next),
            (Proposed, Authorized)
                | (Proposed, Cancelled)
                | (Authorized, Dispatched)
                | (Authorized, Cancelled)
                | (Dispatched, Acknowledged)
                | (Dispatched, OutcomeUnknown)
                | (Dispatched, Failed)
                | (Acknowledged, Committed)
                | (Acknowledged, OutcomeUnknown)
                | (Acknowledged, Failed)
                | (Committed, Observed)
                | (Committed, OutcomeUnknown)
                | (Committed, Failed)
                | (OutcomeUnknown, Reconciling)
                | (Reconciling, OutcomeUnknown)
                | (Reconciling, Observed)
                | (Reconciling, Failed)
                | (Observed, Verified)
                | (Observed, Failed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CancellationDisposition {
    CancelledBeforeDispatch,
    RequiresReconciliation,
    RequiresCompensation,
    AlreadyTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionAttempt {
    pub id: ActionAttemptId,
    pub binding_id: RuntimeBindingId,
    pub business_key: String,
    pub action: String,
    #[serde(default)]
    pub resource_ref: Option<String>,
    #[serde(default)]
    pub site_ref: Option<String>,
    #[serde(default)]
    pub authority_decision_ref: Option<String>,
    #[serde(default)]
    pub external_action_permit_ref: Option<String>,
    pub state: AttemptState,
    pub external_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ActionAttempt {
    pub fn new(
        binding_id: RuntimeBindingId,
        business_key: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: ActionAttemptId::generate_with("attempt"),
            binding_id,
            business_key: business_key.into(),
            action: action.into(),
            resource_ref: None,
            site_ref: None,
            authority_decision_ref: None,
            external_action_permit_ref: None,
            state: AttemptState::Proposed,
            external_ref: None,
            evidence_refs: Vec::new(),
            last_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, next: AttemptState) -> Result<()> {
        if !self.state.can_transition_to(next) {
            return Err(Error::invalid_state(format!(
                "attempt {} cannot transition from {:?} to {:?}",
                self.id, self.state, next
            )));
        }
        self.state = next;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn mark_outcome_unknown(&mut self, error: impl Into<String>) -> Result<()> {
        self.transition(AttemptState::OutcomeUnknown)?;
        self.last_error = Some(error.into());
        Ok(())
    }

    /// Request cancellation without pretending an already-dispatched real-world
    /// side effect can be rolled back by changing local state.
    pub fn request_cancel(&mut self) -> Result<CancellationDisposition> {
        use AttemptState::*;
        match self.state {
            Proposed | Authorized => {
                self.transition(Cancelled)?;
                Ok(CancellationDisposition::CancelledBeforeDispatch)
            }
            Dispatched | Acknowledged | OutcomeUnknown | Reconciling => {
                Ok(CancellationDisposition::RequiresReconciliation)
            }
            Committed | Observed => Ok(CancellationDisposition::RequiresCompensation),
            Verified | Failed | Cancelled => Ok(CancellationDisposition::AlreadyTerminal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_after_dispatch_never_erases_external_effect_uncertainty() {
        let mut attempt = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            "work-1:create-order",
            "create-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        assert_eq!(
            attempt.request_cancel().unwrap(),
            CancellationDisposition::CancelledBeforeDispatch
        );
        assert_eq!(attempt.state, AttemptState::Cancelled);

        let mut dispatched = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            "work-2:create-order",
            "create-order",
        );
        dispatched.transition(AttemptState::Authorized).unwrap();
        dispatched.transition(AttemptState::Dispatched).unwrap();
        assert_eq!(
            dispatched.request_cancel().unwrap(),
            CancellationDisposition::RequiresReconciliation
        );
        assert_eq!(dispatched.state, AttemptState::Dispatched);

        dispatched.transition(AttemptState::Acknowledged).unwrap();
        dispatched.transition(AttemptState::Committed).unwrap();
        assert_eq!(
            dispatched.request_cancel().unwrap(),
            CancellationDisposition::RequiresCompensation
        );
        assert_eq!(dispatched.state, AttemptState::Committed);
    }

    #[test]
    fn unknown_outcome_cannot_be_blindly_redispatched() {
        let mut attempt = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            "work-1:create-order",
            "create-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt.mark_outcome_unknown("network timeout").unwrap();

        assert!(attempt.transition(AttemptState::Dispatched).is_err());
        attempt.transition(AttemptState::Reconciling).unwrap();
    }
}
