//! gateway — ActionGateway / Policy Enforcement Point (Goal §12.3).
//!
//! SAFETY: ALL side-effect actions must pass through this gateway. An Agent
//! never calls a real production-write path directly. The gateway evaluates
//! the AuthorityDecision against the ExecutionBinding and denies by default:
//! no decision, a revoked or expired decision, or a non-BOUND binding all
//! refuse the action before it can reach a provider.
//!
//! Call chain (master §14.3):
//! Work → Action proposal → Action Gateway (PEP) → AuthorityDecision →
//! ExecutionBinding → Provider → Receipt/Reconciliation.

use crate::control::store::ControlPlaneStore;
use crate::work::{AttemptStatus, BindingStatus, ExecutionBinding};

/// The gateway's decision for a proposed action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayDecision {
    /// The action may proceed to the Provider (a new attempt is dispatched).
    Allowed,
    /// The action is refused before reaching any provider (deny-by-default).
    Denied,
}

/// ActionGateway — the PEP for side-effect actions.
///
/// The evaluation is deterministic and deny-by-default. The decision provider
/// is replaceable (the `AuthorityDecision` is an input, not the gateway's
/// own); enforcement happens HERE.
pub struct ActionGateway;

impl ActionGateway {
    pub const ID: &'static str = "action-gateway";

    /// Evaluates a proposed side-effect action against its authority decision
    /// and execution binding.
    ///
    /// Refuses (deny-by-default) when:
    /// - there is no authority decision;
    /// - the decision is denied, revoked, or expired;
    /// - the decision's request is outside its time window;
    /// - the binding is missing or not BOUND.
    pub fn evaluate(
        &self,
        store: &ControlPlaneStore,
        binding: &ExecutionBinding,
        decision: Option<&crate::work::AuthorityDecision>,
    ) -> GatewayDecision {
        // No decision at all: deny (absence of authority is not authority).
        let Some(decision) = decision else {
            return GatewayDecision::Denied;
        };
        let now = crate::work::now_iso();
        if !decision.authorizes(&now) {
            return GatewayDecision::Denied;
        }
        // The binding must exist and be BOUND for the action to dispatch.
        let Some(stored) = store.get_binding(&binding.binding_id).ok().flatten() else {
            return GatewayDecision::Denied;
        };
        if stored.status != BindingStatus::Bound {
            return GatewayDecision::Denied;
        }
        GatewayDecision::Allowed
    }

    /// Convenience helper: the full pre-dispatch check used by the business
    /// path. Returns `Err` with a typed reason when the gateway denies; the
    /// attempt then stays PROPOSED and never reaches a provider.
    pub fn authorize_dispatch(
        &self,
        store: &ControlPlaneStore,
        binding: &ExecutionBinding,
        decision: Option<&crate::work::AuthorityDecision>,
        attempt: &crate::work::ActionAttempt,
    ) -> Result<(), String> {
        match self.evaluate(store, binding, decision) {
            GatewayDecision::Allowed => {
                let _ = attempt;
                Ok(())
            }
            GatewayDecision::Denied => Err(format!(
                "action gateway denied '{}' (REAL_PRODUCTION_WRITE blocked without authority)",
                attempt.action
            )),
        }
    }
}

/// SAFETY: an attempt that the gateway denied never reaches the provider —
/// this helper proves the invariant for tests that walk the full chain.
pub fn denied_attempt_stays_proposed(store: &ControlPlaneStore, attempt_id: &str) -> bool {
    match store.get_attempt(attempt_id) {
        Ok(Some(attempt)) => attempt.status == AttemptStatus::Proposed,
        _ => true, // never inserted is also "never reached a provider"
    }
}
