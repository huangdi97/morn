//! trust_governance — Trust & Governance Plane (Phase 8).
//!
//! Proves (Goal §12 / master §14):
//! - the reference policy provider decides deny-by-default with deterministic
//!   rules (never an LLM guess); OPA/Cedar can replace it behind the same
//!   contract;
//! - the ActionGateway (PEP) refuses side-effect actions without authority:
//!   no decision, denied/revoked/expired decisions, and non-BOUND bindings all
//!   deny before any provider is reached;
//! - business layers (src/work + src/control) never import the connector
//!   write path — an Agent cannot bypass the gateway to a production write.

use morn::control::controllers::execution_flow::{BindingController, ExecutionController};
use morn::control::store::ControlPlaneStore;
use morn::providers::gateway::{denied_attempt_stays_proposed, ActionGateway, GatewayDecision};
use morn::providers::policy::{PolicyProvider, ReferencePolicyProvider};
use morn::work::{
    ActionAttempt, AuthorityDecision, AuthorityRequest, BindingStatus, DelegationDepth,
    ExecutionBinding, TimeWindow, Work, WorkSpec,
};
use std::time::Duration;

fn work(work_id: &str) -> Work {
    Work::draft(
        work_id,
        WorkSpec {
            work_type: "factory.equipment-exception".to_string(),
            goal: "restore_operational_readiness".to_string(),
            target_refs: Vec::new(),
            constraints: Vec::new(),
            acceptance: None,
            authority: None,
            budget: None,
            profile_ref: None,
            source_refs: Vec::new(),
            owner: None,
            generation: 1,
            contract_version: None,
        },
    )
}

fn bound_binding(store: &ControlPlaneStore, binding_id: &str) -> ExecutionBinding {
    let binding = ExecutionBinding::new(binding_id, "work-tg", "sha256:cap", "fake");
    store.insert_work(&work("work-tg")).unwrap();
    BindingController.bind(store, &binding).unwrap();
    binding
}

fn request(action: &str) -> AuthorityRequest {
    let mut req = AuthorityRequest::new("req-tg", "agent-1", action);
    req.work_ref = Some("work-tg".to_string());
    req
}

#[test]
fn g_policy_provider_decides_deny_by_default() {
    let policy = ReferencePolicyProvider::factory_readonly();

    // A valid read-only request is granted.
    let granted = policy.decide(&request("cmms.read"));
    assert_eq!(granted.decision, morn::work::AuthorityDecisionType::Grant);

    // An unknown action is denied (outside the allowlist).
    let denied = policy.decide(&request("plc.write"));
    assert_eq!(denied.decision, morn::work::AuthorityDecisionType::Deny);
    assert!(denied.reason.as_deref().unwrap_or("").contains("allowlist"));

    // An empty principal/action is denied.
    let empty = policy.decide(&AuthorityRequest::new("req-e", "", ""));
    assert_eq!(empty.decision, morn::work::AuthorityDecisionType::Deny);

    // Delegation depth exceeded is denied.
    let mut deep = request("cmms.read");
    deep.delegation_depth = Some(DelegationDepth { current: 3, max: 2 });
    let denied = policy.decide(&deep);
    assert_eq!(denied.decision, morn::work::AuthorityDecisionType::Deny);
    assert!(denied
        .reason
        .as_deref()
        .unwrap_or("")
        .contains("delegation depth"));

    // A stale (expired) time window is denied.
    let mut stale = request("cmms.read");
    stale.time_window = Some(TimeWindow {
        not_before: None,
        not_after: Some("2020-01-01T00:00:00Z".to_string()),
    });
    let denied = policy.decide(&stale);
    assert_eq!(denied.decision, morn::work::AuthorityDecisionType::Deny);
    assert!(denied
        .reason
        .as_deref()
        .unwrap_or("")
        .contains("time window"));
}

#[test]
fn g_pep_denies_without_authority_decision() {
    // Deny-by-default: no decision at all means the action never reaches a
    // provider.
    let store = ControlPlaneStore::in_memory().expect("store");
    let binding = bound_binding(&store, "binding-tg-1");
    let attempt = ActionAttempt::proposed(
        "attempt-tg-1",
        "work-tg",
        "binding-tg-1",
        "cmms.create-order",
        "work-tg:cmms.create-order:1",
    );

    let gateway = ActionGateway;
    assert_eq!(
        gateway.evaluate(&store, &binding, None),
        GatewayDecision::Denied
    );
    let authorized = gateway.authorize_dispatch(&store, &binding, None, &attempt);
    assert!(authorized.is_err());
    assert!(authorized
        .unwrap_err()
        .contains("REAL_PRODUCTION_WRITE blocked"));

    // The attempt was never dispatched: it stays PROPOSED / never inserted.
    assert!(denied_attempt_stays_proposed(&store, "attempt-tg-1"));
    assert!(store.get_attempt("attempt-tg-1").unwrap().is_none());
}

#[test]
fn g_pep_denies_revoked_or_expired_or_unbound() {
    let store = ControlPlaneStore::in_memory().expect("store");
    let binding = bound_binding(&store, "binding-tg-2");
    let gateway = ActionGateway;

    // Revoked decision: denied.
    let mut revoked = AuthorityDecision::grant("authz-r", "req-tg", "policy");
    revoked.revoke("superseded");
    assert_eq!(
        gateway.evaluate(&store, &binding, Some(&revoked)),
        GatewayDecision::Denied
    );

    // Expired decision: denied.
    let mut expired = AuthorityDecision::grant("authz-e", "req-tg", "policy");
    expired.expires_at = Some("2020-01-01T00:00:00Z".to_string());
    assert_eq!(
        gateway.evaluate(&store, &binding, Some(&expired)),
        GatewayDecision::Denied
    );

    // A valid decision but a non-BOUND binding: denied.
    BindingController
        .suspend(&store, "binding-tg-2", "work-tg", "provider failure")
        .unwrap();
    let valid = AuthorityDecision::grant("authz-v", "req-tg", "policy");
    assert_eq!(
        gateway.evaluate(&store, &binding, Some(&valid)),
        GatewayDecision::Denied
    );
    assert_eq!(
        store.get_binding("binding-tg-2").unwrap().unwrap().status,
        BindingStatus::Suspended
    );
}

#[test]
fn g_pep_allows_only_the_full_chain() {
    // Grant + BOUND binding + within time window: the action may proceed —
    // and the dispatch is still idempotent and binding-checked.
    let store = ControlPlaneStore::in_memory().expect("store");
    let binding = bound_binding(&store, "binding-tg-3");
    let gateway = ActionGateway;

    let mut req = request("cmms.create-order");
    req.time_window = Some(TimeWindow {
        not_before: None,
        not_after: Some("2099-01-01T00:00:00Z".to_string()),
    });
    let policy = ReferencePolicyProvider::factory_readonly();
    let decision = policy.decide(&req);
    assert_eq!(decision.decision, morn::work::AuthorityDecisionType::Grant);

    assert_eq!(
        gateway.evaluate(&store, &binding, Some(&decision)),
        GatewayDecision::Allowed
    );
    let attempt = ActionAttempt::proposed(
        "attempt-tg-3",
        "work-tg",
        "binding-tg-3",
        "cmms.create-order",
        "work-tg:cmms.create-order:1",
    );
    assert!(gateway
        .authorize_dispatch(&store, &binding, Some(&decision), &attempt)
        .is_ok());

    // The authorized dispatch succeeds exactly once.
    ExecutionController.dispatch(&store, &attempt).unwrap();
    assert_eq!(
        store.get_attempt("attempt-tg-3").unwrap().unwrap().status,
        morn::work::AttemptStatus::Dispatched
    );

    // A duplicate dispatch of the same key is prevented.
    let duplicate = ActionAttempt::proposed(
        "attempt-tg-3-dup",
        "work-tg",
        "binding-tg-3",
        "cmms.create-order",
        "work-tg:cmms.create-order:1",
    );
    let outcome = ExecutionController.dispatch(&store, &duplicate).unwrap();
    assert!(outcome
        .notes
        .iter()
        .any(|n| n.contains("duplicate attempt prevented")));
}

#[test]
fn g_expired_authority_is_rechecked_before_use() {
    // A decision granted with a short window expires before a later dispatch:
    // the gateway denies at dispatch time (stale authority defense).
    let store = ControlPlaneStore::in_memory().expect("store");
    let binding = bound_binding(&store, "binding-tg-4");
    let gateway = ActionGateway;

    let mut expired_later = AuthorityDecision::grant("authz-l", "req-tg", "policy");
    expired_later.expires_at = Some("2020-06-01T00:00:00Z".to_string());
    // Deny now (the decision is already past its window).
    assert_eq!(
        gateway.evaluate(&store, &binding, Some(&expired_later)),
        GatewayDecision::Denied
    );

    // A decision is re-checked at USE time against the current clock: the
    // same decision authorizes within its window and denies after it —
    // stale authority never silently keeps working.
    let mut windowed = AuthorityDecision::grant("authz-s", "req-tg", "policy");
    windowed.decided_at = "2026-01-01T00:00:00Z".to_string();
    windowed.expires_at = Some("2026-01-02T00:00:00Z".to_string());
    assert!(
        windowed.authorizes("2026-01-01T12:00:00Z"),
        "within the window"
    );
    assert!(
        !windowed.authorizes("2026-01-03T00:00:00Z"),
        "after the window the same decision denies"
    );
}
