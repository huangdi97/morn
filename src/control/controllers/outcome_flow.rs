//! outcome_flow — ReceiptController / ReconciliationController / OutcomeController /
//! AcceptanceController.
//!
//! INVARIANT: a Receipt comes from an identifiable source; reconciliation
//! prefers source-of-truth readback over any self-description; an Agent
//! self-report of success is never an ObservedOutcome. A timeout is never
//! auto-converted to failure — it goes OUTCOME_UNKNOWN → RECONCILING.

use super::ControllerOutcome;
use crate::control::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{
    AcceptanceDecision, AcceptanceType, AttemptStatus, Condition, ConditionType, ObservedOutcome,
    Receipt, ReconciliationFound, ReconciliationRecord, ReconciliationResult,
};

/// ReceiptController — records receipts for dispatched attempts.
pub struct ReceiptController;

impl ReceiptController {
    pub const ID: &'static str = "receipt-controller";

    /// Records a Receipt. When the receipt source is a source-of-truth
    /// readback (not just a transport ack), the attempt advances to OBSERVED
    /// and the ExternalActionVerified gate is produced.
    pub fn record(
        &self,
        store: &ControlPlaneStore,
        receipt: &Receipt,
    ) -> Result<ControllerOutcome, MornError> {
        let inserted = store.insert_receipt(receipt)?;
        let mut outcome = ControllerOutcome::new(Self::ID, &receipt.attempt_ref);
        if !inserted {
            return Ok(outcome.with_note("receipt already recorded (idempotent)"));
        }

        // COMPATIBILITY: a connector ack proves transport, not business commit.
        // Only a source-of-truth readback verifies the external action.
        if receipt.source == "cmms-readback" || receipt.source == "human-confirmation" {
            // SAFETY: the attempt state machine never downgrades — OBSERVED
            // precedes VERIFIED, so a readback receipt on an already-observed
            // or verified attempt is an idempotent re-record, not a step back.
            let current = store.get_attempt(&receipt.attempt_ref)?.map(|a| a.status);
            let would_downgrade = matches!(
                current,
                Some(AttemptStatus::Observed) | Some(AttemptStatus::Verified)
            );
            if !would_downgrade {
                store.update_attempt_status(&receipt.attempt_ref, AttemptStatus::Observed)?;
            }
            outcome = outcome
                .with_condition(Condition::satisfied(ConditionType::ExternalActionVerified))
                .with_note("external action verified by source-of-truth readback");
        } else {
            outcome = outcome.with_note(&format!(
                "transport ack recorded from '{}' (not business commit)",
                receipt.source
            ));
        }
        Ok(outcome)
    }
}

/// ReconciliationController — reconciles unknown outcomes by business key.
pub struct ReconciliationController;

impl ReconciliationController {
    pub const ID: &'static str = "reconciliation-controller";

    /// Reconciles an attempt whose outcome is unknown: queries the
    /// source-of-truth system by business key and records the result.
    ///
    /// `readback` queries the external system; it must not be replaced by an
    /// Agent self-description.
    pub fn reconcile(
        &self,
        store: &ControlPlaneStore,
        attempt_id: &str,
        readback: impl Fn(&str) -> Result<Option<String>, MornError>,
    ) -> Result<Option<ControllerOutcome>, MornError> {
        let Some(attempt) = store.get_attempt(attempt_id)? else {
            return Ok(None);
        };
        if attempt.status != AttemptStatus::OutcomeUnknown
            && attempt.status != AttemptStatus::Reconciling
        {
            return Ok(None);
        }

        store.update_attempt_status(attempt_id, AttemptStatus::Reconciling)?;

        let external_ref = readback(&attempt.idempotency_key)?;
        let (found, result) = match &external_ref {
            Some(_) => (ReconciliationFound::Yes, ReconciliationResult::Committed),
            None => (ReconciliationFound::No, ReconciliationResult::NotCommitted),
        };

        let mut record =
            ReconciliationRecord::new(&format!("recon-{}", attempt_id), attempt_id, result);
        record.found = Some(found);
        record.external_ref_found = external_ref.clone();
        record.decided_by = Some("source-of-truth readback".to_string());
        store.insert_reconciliation(&record)?;

        match result {
            ReconciliationResult::Committed => {
                store.update_attempt_status(attempt_id, AttemptStatus::Verified)?;
                Ok(Some(
                    ControllerOutcome::new(Self::ID, attempt_id)
                        .with_condition(Condition::satisfied(ConditionType::ExternalActionVerified))
                        .with_note(&format!(
                            "reconciled: committed as {external_ref:?}; no duplicate action taken"
                        )),
                ))
            }
            ReconciliationResult::NotCommitted => Ok(Some(
                ControllerOutcome::new(Self::ID, attempt_id)
                    .with_condition(Condition::unsatisfied(
                        ConditionType::ExternalActionVerified,
                        "external action not committed (readback)",
                    ))
                    .with_note(
                        "reconciled: not committed; safe to re-dispatch under a new attempt",
                    ),
            )),
            ReconciliationResult::Unknown => Ok(Some(
                ControllerOutcome::new(Self::ID, attempt_id)
                    .with_condition(Condition::unknown(
                        ConditionType::ExternalActionVerified,
                        "still unknown after readback",
                    ))
                    .with_note("reconciled: STILL_UNKNOWN; manual intervention required"),
            )),
        }
    }
}

/// OutcomeController — records observed outcomes.
pub struct OutcomeController;

impl OutcomeController {
    pub const ID: &'static str = "outcome-controller";

    /// Records an ObservedOutcome from an identifiable source and produces the
    /// OutcomeObserved gate. The Work's unresolved unknowns are cleared only by
    /// a real observation, never by an agent self-report.
    pub fn record(
        &self,
        store: &ControlPlaneStore,
        outcome: &ObservedOutcome,
    ) -> Result<ControllerOutcome, MornError> {
        let inserted = store.insert_outcome(outcome)?;
        if !inserted {
            return Ok(ControllerOutcome::new(Self::ID, &outcome.work_ref)
                .with_note("outcome already recorded (idempotent)"));
        }

        if let Some(mut work) = store.get_work(&outcome.work_ref)? {
            work.status.observed_outcome_refs.push(outcome.id.clone());
            work.status.unresolved_unknowns.clear();
            store.update_work_status(&outcome.work_ref, &work.status)?;
        }

        Ok(ControllerOutcome::new(Self::ID, &outcome.work_ref)
            .with_condition(Condition::satisfied(ConditionType::OutcomeObserved))
            .with_note(&format!(
                "outcome {} recorded from {:?}",
                outcome.id, outcome.source
            )))
    }
}

/// AcceptanceController — records acceptance decisions.
pub struct AcceptanceController;

impl AcceptanceController {
    pub const ID: &'static str = "acceptance-controller";

    /// Records an AcceptanceDecision and updates the Work's acceptance state.
    /// Accepted outcomes are the unit of value.
    pub fn record(
        &self,
        store: &ControlPlaneStore,
        acceptance: &AcceptanceDecision,
    ) -> Result<ControllerOutcome, MornError> {
        let inserted = store.insert_acceptance(acceptance)?;
        if !inserted {
            return Ok(ControllerOutcome::new(Self::ID, &acceptance.work_ref)
                .with_note("acceptance already recorded (idempotent)"));
        }

        if let Some(mut work) = store.get_work(&acceptance.work_ref)? {
            work.status.acceptance_state = match acceptance.decision {
                AcceptanceType::Accept => crate::work::WorkAcceptanceState::Accepted,
                AcceptanceType::Reject => crate::work::WorkAcceptanceState::Rejected,
                AcceptanceType::Conditional => crate::work::WorkAcceptanceState::Conditional,
            };
            work.set_condition(Condition::satisfied(ConditionType::AcceptanceSatisfied));
            store.update_work_status(&acceptance.work_ref, &work.status)?;
        }

        Ok(ControllerOutcome::new(Self::ID, &acceptance.work_ref)
            .with_condition(Condition::satisfied(ConditionType::AcceptanceSatisfied))
            .with_note(&format!(
                "acceptance {} recorded ({:?})",
                acceptance.id, acceptance.decision
            )))
    }
}
