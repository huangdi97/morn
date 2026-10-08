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
    // An external lookup must answer for this exact business/idempotency key.
    // A receipt for another action is never evidence that our effect occurred.
    if observation.business_key != attempt.business_key {
        return Err(Error::validation(
            "reconciliation result business key does not match the attempted effect",
        ));
    }
    if observation.committed == Some(true)
        && observation.observed == Some(true)
        && (observation
            .external_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
            || observation.evidence_refs.is_empty())
    {
        return Err(Error::validation(
            "observed external commit requires a non-empty external reference and evidence",
        ));
    }
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
    use std::collections::HashMap;

    use morn_kernel::ids::RuntimeBindingId;

    #[derive(Debug, Default)]
    struct StatefulFakeCmms {
        orders: HashMap<String, String>,
        create_calls: u32,
    }

    impl StatefulFakeCmms {
        fn create_then_timeout(&mut self, business_key: &str) -> Result<()> {
            self.create_calls += 1;
            self.orders
                .entry(business_key.to_string())
                .or_insert_with(|| "MO-88273".to_string());
            Err(Error::external(
                "client timed out after CMMS committed the maintenance order",
            ))
        }

        fn query_by_business_key(&self, business_key: &str) -> Option<&str> {
            self.orders.get(business_key).map(String::as_str)
        }
    }

    struct StatefulCmmsReconciler<'a> {
        cmms: &'a StatefulFakeCmms,
    }

    impl OutcomeReconciler for StatefulCmmsReconciler<'_> {
        fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
            let external_ref = self
                .cmms
                .query_by_business_key(&attempt.business_key)
                .map(str::to_string);
            Ok(ReconciliationObservation {
                business_key: attempt.business_key.clone(),
                committed: Some(external_ref.is_some()),
                observed: Some(external_ref.is_some()),
                evidence_refs: external_ref
                    .as_ref()
                    .map(|reference| vec![format!("cmms://orders/{reference}")])
                    .unwrap_or_default(),
                external_ref,
            })
        }
    }

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
    fn authoritative_query_after_timeout_proves_exactly_one_external_create() {
        let business_key = "work-1042:maintenance";
        let mut cmms = StatefulFakeCmms::default();
        let timeout = cmms.create_then_timeout(business_key);
        assert!(timeout.is_err());
        assert_eq!(cmms.create_calls, 1);
        assert_eq!(cmms.query_by_business_key(business_key), Some("MO-88273"));

        let mut attempt = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            business_key,
            "create-maintenance-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt
            .mark_outcome_unknown("timeout after commit")
            .unwrap();

        let record =
            reconcile_attempt(&mut attempt, &StatefulCmmsReconciler { cmms: &cmms }).unwrap();
        assert_eq!(record.after, AttemptState::Observed);
        assert_eq!(attempt.external_ref.as_deref(), Some("MO-88273"));
        assert_eq!(cmms.create_calls, 1, "reconciliation must not create again");
        assert!(
            attempt.transition(AttemptState::Dispatched).is_err(),
            "an observed ambiguous attempt cannot be silently redispatched"
        );
    }

    struct WrongBusinessKey;
    impl OutcomeReconciler for WrongBusinessKey {
        fn reconcile(&self, _attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
            Ok(ReconciliationObservation {
                business_key: "another-work:order".to_string(),
                committed: Some(true),
                observed: Some(true),
                external_ref: Some("MO-OTHER".to_string()),
                evidence_refs: vec!["cmms://other-receipt".to_string()],
            })
        }
    }

    struct UnsubstantiatedCommit;
    impl OutcomeReconciler for UnsubstantiatedCommit {
        fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
            Ok(ReconciliationObservation {
                business_key: attempt.business_key.clone(),
                committed: Some(true),
                observed: Some(true),
                external_ref: None,
                evidence_refs: vec![],
            })
        }
    }

    #[test]
    fn foreign_or_unsubstantiated_receipt_never_proves_our_effect() {
        let mut attempt = ActionAttempt::new(
            RuntimeBindingId::generate_with("binding"),
            "work-1042:maintenance",
            "create-maintenance-order",
        );
        attempt.transition(AttemptState::Authorized).unwrap();
        attempt.transition(AttemptState::Dispatched).unwrap();
        attempt.mark_outcome_unknown("timeout").unwrap();

        assert!(reconcile_attempt(&mut attempt, &WrongBusinessKey).is_err());
        assert_eq!(attempt.state, AttemptState::Reconciling);
        assert!(attempt.evidence_refs.is_empty());
        assert!(attempt.external_ref.is_none());
        assert!(reconcile_attempt(&mut attempt, &UnsubstantiatedCommit).is_err());
        assert_eq!(attempt.state, AttemptState::Reconciling);
        assert!(attempt.evidence_refs.is_empty());
        assert!(attempt.external_ref.is_none());
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
