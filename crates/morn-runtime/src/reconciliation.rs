//! Reconciliation of ambiguous external side effects.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

use crate::attempt::{ActionAttempt, ActionAttemptId, AttemptState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ReconciliationRecordTag;
pub type ReconciliationRecordId = Id<ReconciliationRecordTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationObservation {
    pub business_key: String,
    pub committed: Option<bool>,
    pub observed: Option<bool>,
    pub external_ref: Option<String>,
    pub evidence_refs: Vec<String>,
}

pub trait OutcomeReconciler: Send + Sync {
    fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationRecord {
    pub id: ReconciliationRecordId,
    pub attempt_id: ActionAttemptId,
    pub before: AttemptState,
    pub after: AttemptState,
    pub observation: ReconciliationObservation,
    pub created_at: Timestamp,
}

/// Reconcile exactly one ambiguous attempt. Absence is only treated as failure
/// when the provider gives an authoritative committed=false result.
pub fn reconcile_attempt(
    attempt: &mut ActionAttempt,
    provider: &dyn OutcomeReconciler,
) -> Result<ReconciliationRecord> {
    if attempt.state != AttemptState::OutcomeUnknown && attempt.state != AttemptState::Reconciling {
        return Err(Error::invalid_state(
            "only OutcomeUnknown/Reconciling attempts can be reconciled",
        ));
    }
    let before = attempt.state;
    if attempt.state == AttemptState::OutcomeUnknown {
        attempt.transition(AttemptState::Reconciling)?;
    }

    let observation = provider.reconcile(attempt)?;
    attempt
        .evidence_refs
        .extend(observation.evidence_refs.clone());

    match (observation.committed, observation.observed) {
        (Some(true), Some(true)) => {
            attempt.external_ref = observation.external_ref.clone();
            attempt.transition(AttemptState::Observed)?;
        }
        (Some(false), _) => {
            attempt.transition(AttemptState::Failed)?;
        }
        _ => {
            attempt.transition(AttemptState::OutcomeUnknown)?;
        }
    }

    Ok(ReconciliationRecord {
        id: ReconciliationRecordId::generate_with("reconcile"),
        attempt_id: attempt.id.clone(),
        before,
        after: attempt.state,
        observation,
        created_at: Timestamp::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::RuntimeBindingId;

    struct CommittedAfterTimeout;

    impl OutcomeReconciler for CommittedAfterTimeout {
        fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
            Ok(ReconciliationObservation {
                business_key: attempt.business_key.clone(),
                committed: Some(true),
                observed: Some(true),
                external_ref: Some("MO-88273".into()),
                evidence_refs: vec!["cmms:MO-88273".into()],
            })
        }
    }

    #[test]
    fn timeout_after_commit_resolves_without_second_dispatch() {
        let mut attempt = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            "work-1042:maintenance",
            "create-maintenance-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt.mark_outcome_unknown("timeout").unwrap();

        let record = reconcile_attempt(&mut attempt, &CommittedAfterTimeout).unwrap();
        assert_eq!(attempt.state, AttemptState::Observed);
        assert_eq!(attempt.external_ref.as_deref(), Some("MO-88273"));
        assert_eq!(record.after, AttemptState::Observed);

        attempt.transition(AttemptState::Verified).unwrap();
    }
}
