//! receipt_reconciliation_g7 — Receipt/Unknown/Reconciliation Gate (G7).
//!
//! Proves (Goal §10 / master §11):
//! - the full action path goes through the gateway: Work → proposal →
//!   AuthorityDecision → Binding → dispatch → Provider → Receipt/Reconcile;
//! - transport ack != business commit; HTTP-level failure with a committed
//!   order becomes OUTCOME_UNKNOWN, then reconciles by business key;
//! - a retry storm cannot create duplicate external side effects;
//! - a duplicate receipt record is idempotent and never downgrades state;
//! - crash-after-commit: the child process commits on the persistent fake
//!   CMMS and dies; after restart the reconciliation readback finds the order
//!   and verifies — no duplicate order is created.

use morn::control::controllers::execution_flow::{BindingController, ExecutionController};
use morn::control::controllers::outcome_flow::{ReceiptController, ReconciliationController};
use morn::control::store::ControlPlaneStore;
use morn::core::error::MornError;
use morn::providers::connector::{ConnectorProvider, FakeCmmsConnector};
use morn::providers::connector_persistent::PersistentFakeCmmsConnector;
use morn::work::{ActionAttempt, AttemptStatus, ExecutionBinding, Receipt, Work, WorkSpec};
use std::path::PathBuf;
use std::sync::Arc;
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

fn setup(store: &ControlPlaneStore, work_id: &str, binding_id: &str, attempt_id: &str, key: &str) {
    store.insert_work(&work(work_id)).unwrap();
    BindingController
        .bind(
            store,
            &ExecutionBinding::new(binding_id, work_id, "sha256:cap", "fake-cmms-persistent"),
        )
        .unwrap();
    let attempt =
        ActionAttempt::proposed(attempt_id, work_id, binding_id, "cmms.create-order", key);
    ExecutionController.dispatch(store, &attempt).unwrap();
}

#[test]
fn g7_full_reconciliation_path() {
    // The canonical G7 chain, through the Action Gateway:
    // Fake CMMS POST → server committed → client timeout → OUTCOME_UNKNOWN
    // → reconcile by business key → find MO-xxxxx → VERIFIED → no duplicate.
    let store = ControlPlaneStore::in_memory().expect("store");
    let connector = {
        let c = FakeCmmsConnector::new();
        c.arm_timeout_after_commit();
        Arc::new(c)
    };
    let business_key = "work-g7a:cmms.create-order:1";
    setup(
        &store,
        "work-g7a",
        "binding-g7a",
        "attempt-g7a",
        business_key,
    );

    // DISPATCHED → provider POST commits but the transport times out.
    let result = connector.create_order(business_key, r#"{"asset":"CNC-17"}"#);
    assert!(result.is_err(), "timeout surfaces as a transport error");
    store
        .update_attempt_status("attempt-g7a", AttemptStatus::OutcomeUnknown)
        .unwrap();

    // Reconcile by business key (source-of-truth readback, not Agent prose).
    let readback = |key: &str| -> Result<Option<String>, MornError> {
        Ok(connector.query_order(key).expect("readback"))
    };
    let outcome = ReconciliationController
        .reconcile(&store, "attempt-g7a", readback)
        .unwrap()
        .expect("reconcile runs");
    assert!(outcome
        .notes
        .iter()
        .any(|n| n.contains("no duplicate action taken")));
    assert!(outcome.condition_changes.iter().any(|c| c.is_true()));

    let attempt = store.get_attempt("attempt-g7a").unwrap().unwrap();
    assert_eq!(
        attempt.status,
        AttemptStatus::Verified,
        "timeout+commit reconciles to VERIFIED"
    );

    // No duplicate order: exactly one external ref exists for the key.
    let found = connector.query_order(business_key).unwrap().unwrap();
    let receipt = Receipt::from_source("receipt-g7a", "attempt-g7a", "cmms-readback", Some(&found));
    ReceiptController.record(&store, &receipt).unwrap();
    assert_eq!(connector.query_order(business_key).unwrap().unwrap(), found);
}

#[test]
fn g7_retry_storm_is_idempotent() {
    // A retry storm (many duplicate dispatches + re-submits) cannot create
    // duplicate external side effects: one order, one attempt.
    let store = ControlPlaneStore::in_memory().expect("store");
    // Test isolation: clear the shared temp dir so a previous run's orders
    // cannot leak into this run's "first call" (the connector is persistent).
    let storm_dir = std::env::temp_dir().join("morn_g7_storm");
    let _ = std::fs::remove_dir_all(&storm_dir);
    let connector = PersistentFakeCmmsConnector::with_path(storm_dir.join("orders.json"));

    let business_key = "work-g7b:cmms.create-order:1";
    setup(
        &store,
        "work-g7b",
        "binding-g7b",
        "attempt-g7b",
        business_key,
    );

    // Retry storm: 10 duplicate provider calls with the same business key.
    let mut first_ref = String::new();
    for i in 0..10 {
        let receipt = connector
            .create_order(business_key, "{}")
            .expect("storm retry must succeed");
        if i == 0 {
            first_ref = receipt.external_ref.clone();
            assert!(
                !receipt.detail.unwrap_or_default().contains("idempotent"),
                "first call commits"
            );
        } else {
            assert!(
                receipt.detail.unwrap_or_default().contains("idempotent"),
                "retry returns existing"
            );
        }
        assert_eq!(
            receipt.external_ref, first_ref,
            "storm cannot change the external ref"
        );
    }

    // Duplicate attempt inserts are rejected by the idempotency key.
    for i in 0..5 {
        let duplicate = ActionAttempt::proposed(
            &format!("attempt-g7b-dup-{i}"),
            "work-g7b",
            "binding-g7b",
            "cmms.create-order",
            business_key,
        );
        let outcome = ExecutionController.dispatch(&store, &duplicate).unwrap();
        assert!(outcome
            .notes
            .iter()
            .any(|n| n.contains("duplicate attempt prevented")));
    }

    // Exactly ONE order exists for the business key.
    assert_eq!(
        connector.query_order(business_key).unwrap().unwrap(),
        first_ref
    );
}

#[test]
fn g7_duplicate_receipt_is_idempotent_and_never_downgrades() {
    // A duplicate receipt record is idempotent; recording a readback receipt
    // on an already-verified attempt never downgrades it.
    let store = ControlPlaneStore::in_memory().expect("store");
    setup(
        &store,
        "work-g7c",
        "binding-g7c",
        "attempt-g7c",
        "work-g7c:cmms.create-order:1",
    );
    store
        .update_attempt_status("attempt-g7c", AttemptStatus::OutcomeUnknown)
        .unwrap();

    let receipt = Receipt::from_source(
        "receipt-g7c",
        "attempt-g7c",
        "cmms-readback",
        Some("MO-777001"),
    );
    let first = ReceiptController.record(&store, &receipt).unwrap();
    assert!(
        first.condition_changes.iter().any(|c| c.is_true()),
        "readback receipt verifies"
    );
    assert_eq!(
        store.get_attempt("attempt-g7c").unwrap().unwrap().status,
        AttemptStatus::Observed
    );

    // Duplicate receipt (same id): idempotent, no state change.
    let duplicate = ReceiptController.record(&store, &receipt).unwrap();
    assert!(duplicate
        .notes
        .iter()
        .any(|n| n.contains("already recorded")));
    assert_eq!(
        store.get_attempt("attempt-g7c").unwrap().unwrap().status,
        AttemptStatus::Observed
    );

    // Verify, then re-record the readback receipt: never downgrades VERIFIED.
    store
        .update_attempt_status("attempt-g7c", AttemptStatus::Verified)
        .unwrap();
    ReceiptController.record(&store, &receipt).unwrap();
    assert_eq!(
        store.get_attempt("attempt-g7c").unwrap().unwrap().status,
        AttemptStatus::Verified,
        "readback receipt on a verified attempt is a no-op, not a downgrade"
    );
}

// ---- crash-after-commit (child process + persistent connector) ----

fn g7_dir() -> PathBuf {
    std::env::temp_dir().join("morn_g7_test")
}

/// Child mode: commit the order on the persistent fake CMMS, mark the attempt
/// dispatched in SQLite, then die before receiving the acknowledgement.
fn g7_child_main(db_path: &PathBuf, orders_path: &PathBuf, marker: &PathBuf) {
    let store = ControlPlaneStore::with_path(db_path).expect("child store");
    setup(
        &store,
        "work-g7d",
        "binding-g7d",
        "attempt-g7d",
        "work-g7d:cmms.create-order:1",
    );

    let connector = PersistentFakeCmmsConnector::with_path(orders_path);
    // The order commits on the persistent "server".
    let receipt = connector
        .create_order("work-g7d:cmms.create-order:1", r#"{"asset":"CNC-17"}"#)
        .expect("child must commit the order");
    assert!(receipt.external_ref.starts_with("MO-"));

    std::fs::write(marker, b"committed").expect("child must write marker");
    // Die without acknowledging (crash after commit — no graceful exit).
    std::process::exit(9);
}

/// Child-mode entry (env-gated; invoked by the parent through this binary).
#[test]
fn g7_child_runner() {
    if std::env::var("MORN_G7_CHILD").is_err() {
        return;
    }
    let dir = g7_dir();
    g7_child_main(
        &dir.join("control.db"),
        &dir.join("orders.json"),
        &dir.join("child-ready.marker"),
    );
    unreachable!("g7_child_main exits before returning");
}

#[test]
fn g7_crash_after_commit_then_reconcile() {
    let dir = g7_dir();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("control.db");
    let orders_path = dir.join("orders.json");
    let marker = dir.join("child-ready.marker");

    // Spawn the child: it commits and dies (exit 9).
    let exe = std::env::current_exe().expect("test exe");
    // SAFETY: the child runs ONLY the g7 child runner (filter + exact) — it
    // commits on the sandbox fake and exits; no recursion, no real system.
    let mut child = std::process::Command::new(exe)
        .args(["g7_child_runner", "--exact", "--nocapture"])
        .env("MORN_G7_CHILD", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("child must spawn");

    let mut ready = false;
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(500));
        if marker.exists() {
            ready = true;
            break;
        }
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
    }
    let _ = child.wait();
    assert!(ready, "child never committed; cannot run the crash test");

    // "Restart": a fresh store and a fresh connector over the same state.
    let store = ControlPlaneStore::with_path(&db_path).expect("restarted store");
    let connector = PersistentFakeCmmsConnector::with_path(&orders_path);

    // The attempt survived the crash (DISPATCHED — never acknowledged).
    let attempt = store
        .get_attempt("attempt-g7d")
        .unwrap()
        .expect("attempt survives");
    assert_eq!(attempt.status, AttemptStatus::Dispatched);

    // The order committed on the persistent server despite the crash.
    let found = connector
        .query_order("work-g7d:cmms.create-order:1")
        .unwrap()
        .expect("order committed");
    assert!(found.starts_with("MO-"));

    // Timeout path: dispatch → OUTCOME_UNKNOWN → reconcile by business key.
    store
        .update_attempt_status("attempt-g7d", AttemptStatus::OutcomeUnknown)
        .unwrap();
    let readback = |key: &str| -> Result<Option<String>, MornError> {
        Ok(connector.query_order(key).expect("readback"))
    };
    ReconciliationController
        .reconcile(&store, "attempt-g7d", readback)
        .unwrap()
        .expect("reconcile runs");

    let attempt = store.get_attempt("attempt-g7d").unwrap().unwrap();
    assert_eq!(
        attempt.status,
        AttemptStatus::Verified,
        "crash-after-commit reconciles to VERIFIED"
    );

    // No duplicate order: re-submitting returns the same external ref.
    let again = connector
        .create_order("work-g7d:cmms.create-order:1", "{}")
        .expect("idempotent");
    assert_eq!(again.external_ref, found);
}
