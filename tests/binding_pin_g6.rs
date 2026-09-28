//! binding_pin_g6 — ExecutionBinding fault-injection tests (G6 Binding Pin Gate).
//!
//! Proves (Goal §9 rules 1–6 / master §10.3 / ADR-MORN-004):
//! - unloading a provider mid-execution never silently migrates the active
//!   Attempt (the composition adapter has no API to touch Control Plane
//!   bindings; suspend is an explicit business action);
//! - a provider restart does not re-execute already-committed side effects
//!   (idempotency by business key; OUTCOME_UNKNOWN → RECONCILING → VERIFIED);
//! - loading a second provider creates a NEW binding (and a new attempt when
//!   needed) — the old attempt is never auto-resumed;
//! - an attempt that already caused a side effect is reconciled first, never
//!   re-run from scratch under a new provider.

use morn::control::controllers::execution_flow::{BindingController, ExecutionController};
use morn::control::controllers::outcome_flow::{ReceiptController, ReconciliationController};
use morn::control::store::ControlPlaneStore;
use morn::core::error::MornError;
use morn::providers::connector::{ConnectorProvider, FakeCmmsConnector};
use morn::providers::harness::{HarnessInput, HarnessProvider};
use morn::providers::harness_fake::FakeHarnessProvider;
use morn::runtime::cordis::{
    CapabilitiesSlot, CapabilityProviderInfo, CompositionHost, ProviderDescriptor,
};
use morn::work::{
    ActionAttempt, AttemptStatus::*, BindingStatus, ExecutionBinding, Receipt, Work, WorkSpec,
};
use std::sync::Arc;

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

fn provider(id: &str, version: &str, capability: &str) -> CapabilityProviderInfo {
    CapabilityProviderInfo {
        descriptor: ProviderDescriptor::new(id, version, "capability"),
        capabilities: vec![capability.to_string()],
    }
}

/// Sets up a Work + BOUND binding + dispatched attempt on a fresh store.
fn setup_dispatched(
    store: &ControlPlaneStore,
    work_id: &str,
    binding_id: &str,
    attempt_id: &str,
    provider_id: &str,
) {
    store.insert_work(&work(work_id)).unwrap();
    BindingController
        .bind(
            store,
            &ExecutionBinding::new(binding_id, work_id, "sha256:cap", provider_id),
        )
        .unwrap();
    let attempt = ActionAttempt::proposed(
        attempt_id,
        work_id,
        binding_id,
        "cmms.create-order",
        &format!("{work_id}:cmms.create-order:1"),
    );
    ExecutionController.dispatch(store, &attempt).unwrap();
}

#[test]
fn g6_unload_during_execution_pin_unchanged() {
    // SAFETY: the adapter unmount is a runtime effect; it must never touch
    // Control Plane bindings. Suspend is an explicit business decision.
    let store = ControlPlaneStore::in_memory().expect("store");
    setup_dispatched(&store, "work-g6a", "binding-g6a", "attempt-g6a", "dsh");

    let host = CompositionHost::new();
    let mounted = host
        .mount(Arc::new(CapabilitiesSlot::default().with_provider(
            provider("dsh", "1.0.0", "equipment.anomaly.investigate"),
        )))
        .expect("mount");

    // Unload the provider MID-EXECUTION (the attempt is DISPATCHED).
    mounted.unmount().expect("unmount");

    // The stored Binding is NOT silently migrated/revoked by the adapter.
    let binding = store.get_binding("binding-g6a").unwrap().expect("binding");
    assert_eq!(
        binding.status,
        BindingStatus::Bound,
        "adapter unmount must not touch the binding"
    );
    assert_eq!(binding.binding_generation, 1);
    let attempt = store.get_attempt("attempt-g6a").unwrap().expect("attempt");
    assert_eq!(attempt.binding_ref, "binding-g6a", "attempt stays pinned");
    assert_eq!(
        attempt.status, Dispatched,
        "attempt state unchanged by unmount"
    );

    // Business layer explicitly suspends (provider failure path, rule 4).
    BindingController
        .suspend(
            &store,
            "binding-g6a",
            "work-g6a",
            "provider unloaded mid-execution",
        )
        .unwrap();
    let binding = store.get_binding("binding-g6a").unwrap().expect("binding");
    assert_eq!(
        binding.status,
        BindingStatus::Suspended,
        "explicit suspend recorded"
    );
    // History stays: the binding row and attempt survive with the suspend.
    assert!(store.get_attempt("attempt-g6a").unwrap().is_some());

    // Re-mount (provider recovers): the OLD binding is not auto-restored —
    // reload only affects future resolution (rule 3).
    host.mount(Arc::new(CapabilitiesSlot::default().with_provider(
        provider("dsh", "1.0.1", "equipment.anomaly.investigate"),
    )))
    .expect("re-mount");
    let binding = store.get_binding("binding-g6a").unwrap().expect("binding");
    assert_eq!(
        binding.status,
        BindingStatus::Suspended,
        "re-mount must not silently restore the old binding"
    );
    assert_eq!(binding.binding_generation, 1);
}

#[test]
fn g6_provider_restart_no_duplicate_side_effect() {
    // Rule: a provider restart must not re-execute committed side effects.
    // The connector is idempotent by business key, and the attempt state
    // machine walks OUTCOME_UNKNOWN → RECONCILING → VERIFIED.
    let store = ControlPlaneStore::in_memory().expect("store");
    setup_dispatched(&store, "work-g6b", "binding-g6b", "attempt-g6b", "fake");

    let connector = Arc::new(FakeCmmsConnector::new());
    let business_key = "work-g6b:cmms.create-order:1";

    // First execution: the side effect commits.
    let receipt = connector
        .create_order(business_key, r#"{"asset":"CNC-17"}"#)
        .expect("order commits");
    store
        .update_attempt_status("attempt-g6b", Committed)
        .unwrap();

    // Provider "restarts" (new session — same provider).
    let harness = FakeHarnessProvider::new("fake");
    let binding = store.get_binding("binding-g6b").unwrap().unwrap();
    let session = harness.start_session(&binding).expect("restart session");
    let _ = harness.submit(
        &session,
        HarnessInput {
            prompt: "retry after restart".to_string(),
            context_json: None,
        },
    );

    // The retried dispatch hits the idempotency key: no duplicate attempt.
    let duplicate = ActionAttempt::proposed(
        "attempt-g6b-dup",
        "work-g6b",
        "binding-g6b",
        "cmms.create-order",
        business_key,
    );
    let outcome = ExecutionController.dispatch(&store, &duplicate).unwrap();
    assert!(
        outcome
            .notes
            .iter()
            .any(|n| n.contains("duplicate attempt prevented")),
        "restart must not create a duplicate attempt: {:?}",
        outcome.notes
    );
    assert!(store.get_attempt("attempt-g6b-dup").unwrap().is_none());

    // Re-submitting the same business key to the connector returns the
    // existing order — no duplicate external side effect.
    let again = connector
        .create_order(business_key, r#"{"asset":"CNC-17"}"#)
        .expect("idempotent");
    assert_eq!(again.external_ref, receipt.external_ref);

    // Timeout before commit on a second key → OUTCOME_UNKNOWN → reconcile.
    let connector2 = FakeCmmsConnector::new();
    connector2.arm_timeout_after_commit();
    let key2 = "work-g6b:cmms.create-order:2";
    let attempt2 = ActionAttempt::proposed(
        "attempt-g6b-2",
        "work-g6b",
        "binding-g6b",
        "cmms.create-order",
        key2,
    );
    ExecutionController.dispatch(&store, &attempt2).unwrap();
    let _ = connector2.create_order(key2, "{}"); // commits, transport timeout
    store
        .update_attempt_status("attempt-g6b-2", OutcomeUnknown)
        .unwrap();

    let readback = |key: &str| -> Result<Option<String>, MornError> {
        Ok(connector2.query_order(key).expect("readback"))
    };
    let outcome = ReconciliationController
        .reconcile(&store, "attempt-g6b-2", readback)
        .unwrap()
        .expect("reconcile runs");
    let attempt = store.get_attempt("attempt-g6b-2").unwrap().unwrap();
    assert_eq!(attempt.status, Verified, "reconciled attempt advances");
    assert!(outcome
        .notes
        .iter()
        .any(|n| n.contains("reconciled") || n.contains("committed")));
    assert!(store.get_attempt("attempt-g6b").unwrap().unwrap().status == Committed);
}

#[test]
fn g6_load_second_provider_creates_new_binding() {
    // Rule 5: switching providers creates a NEW binding (and a new attempt
    // when needed); the old attempt is never auto-resumed onto the new
    // provider.
    let store = ControlPlaneStore::in_memory().expect("store");
    setup_dispatched(&store, "work-g6c", "binding-g6c", "attempt-g6c", "dsh");

    // Provider failure: explicit suspend.
    BindingController
        .suspend(&store, "binding-g6c", "work-g6c", "dsh crashed")
        .unwrap();

    // Load Pi (second provider) and bind it as a NEW binding.
    let new_binding = ExecutionBinding::new("binding-g6c-2", "work-g6c", "sha256:cap", "pi");
    let outcome = BindingController.bind(&store, &new_binding).unwrap();
    assert!(outcome.notes.iter().any(|n| n.contains("binding created")));

    // The new binding is distinct: different provider, own generation.
    let binding2 = store
        .get_binding("binding-g6c-2")
        .unwrap()
        .expect("new binding");
    assert_eq!(binding2.provider_id, "pi");
    assert_eq!(
        binding2.binding_generation, 1,
        "new binding starts its own generation"
    );
    assert_eq!(binding2.status, BindingStatus::Bound);

    // The OLD attempt is untouched: still pinned to the old (suspended)
    // binding, never auto-resumed onto Pi.
    let attempt = store
        .get_attempt("attempt-g6c")
        .unwrap()
        .expect("old attempt");
    assert_eq!(attempt.binding_ref, "binding-g6c");
    assert_ne!(attempt.binding_ref, "binding-g6c-2");

    // A new attempt on the new binding is required for Pi work (and it can
    // dispatch because the NEW binding is BOUND).
    let attempt2 = ActionAttempt::proposed(
        "attempt-g6c-2",
        "work-g6c",
        "binding-g6c-2",
        "cmms.create-order",
        "work-g6c:cmms.create-order:pi:1",
    );
    ExecutionController.dispatch(&store, &attempt2).unwrap();
    assert_eq!(
        store.get_attempt("attempt-g6c-2").unwrap().unwrap().status,
        Dispatched
    );

    // The old suspended binding refuses NEW dispatches.
    let attempt3 = ActionAttempt::proposed(
        "attempt-g6c-3",
        "work-g6c",
        "binding-g6c",
        "cmms.create-order",
        "work-g6c:cmms.create-order:3",
    );
    let outcome = ExecutionController.dispatch(&store, &attempt3).unwrap();
    assert!(
        outcome.notes.iter().any(|n| n.contains("refused")),
        "suspended binding refuses dispatch"
    );
}

#[test]
fn g6_side_effect_attempt_reconciled_not_rerun() {
    // Rule 6: an attempt that already caused a side effect is reconciled
    // first — it is never re-run from scratch under a new provider.
    let store = ControlPlaneStore::in_memory().expect("store");
    setup_dispatched(&store, "work-g6d", "binding-g6d", "attempt-g6d", "dsh");

    let connector = Arc::new(FakeCmmsConnector::new());
    let business_key = "work-g6d:cmms.create-order:1";

    // The side effect committed, but the outcome is unknown (timeout).
    let connector_receipt = connector
        .create_order(business_key, r#"{\"asset\":\"CNC-17\"}"#)
        .expect("committed");
    store
        .update_attempt_status("attempt-g6d", OutcomeUnknown)
        .unwrap();

    // Attempting to re-run from scratch under a NEW provider is refused: the
    // original attempt still owns the idempotency key.
    BindingController
        .suspend(&store, "binding-g6d", "work-g6d", "switching provider")
        .unwrap();
    let new_binding = ExecutionBinding::new("binding-g6d-2", "work-g6d", "sha256:cap", "pi");
    BindingController.bind(&store, &new_binding).unwrap();
    let rerun = ActionAttempt::proposed(
        "attempt-g6d-rerun",
        "work-g6d",
        "binding-g6d-2",
        "cmms.create-order",
        business_key,
    );
    let outcome = ExecutionController.dispatch(&store, &rerun).unwrap();
    assert!(
        outcome
            .notes
            .iter()
            .any(|n| n.contains("duplicate attempt prevented")),
        "side-effect attempt must not be re-run under a new binding: {:?}",
        outcome.notes
    );
    assert!(store.get_attempt("attempt-g6d-rerun").unwrap().is_none());

    // Reconcile FIRST (source-of-truth readback by business key).
    let readback = |key: &str| -> Result<Option<String>, MornError> {
        Ok(connector.query_order(key).expect("readback"))
    };
    ReconciliationController
        .reconcile(&store, "attempt-g6d", readback)
        .unwrap()
        .expect("reconcile");

    let attempt = store.get_attempt("attempt-g6d").unwrap().unwrap();
    assert_eq!(attempt.status, Verified, "reconciled, not re-run");

    // A receipt from the readback verifies the external action.
    let receipt = Receipt::from_source(
        "receipt-g6d",
        "attempt-g6d",
        "cmms-readback",
        Some(connector_receipt.external_ref.as_str()),
    );
    ReceiptController.record(&store, &receipt).unwrap();
    let attempt = store.get_attempt("attempt-g6d").unwrap().unwrap();
    assert_eq!(
        attempt.status, Verified,
        "readback receipt never downgrades a verified attempt"
    );

    // The connector saw exactly ONE order for the business key — no duplicate.
    assert_eq!(
        connector.query_order(business_key).unwrap().unwrap(),
        connector_receipt.external_ref,
        "exactly ONE order for the business key — no duplicate"
    );
}
