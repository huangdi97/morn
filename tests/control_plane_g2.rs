//! control_plane_g2 — Durable State Gate (G2).
//!
//! Proves (Goal §6.4):
//! 1. A real process is killed after an Attempt is committed;
//! 2. The process "restarts" (a fresh store opens the same DB);
//! 3. The Work still exists;
//! 4. The Attempt/Binding are not lost;
//! 5. Reconciliation continues;
//! 6. No committed side effect is re-executed (idempotency by business key).
//!
//! The child process writes state, signals readiness via a marker file, and
//! is then killed by this test — a real kill, not a simulated one.

use morn::control::controllers::{BindingController, ReconciliationController, WorkController};
use morn::control::store::ControlPlaneStore;
use morn::work::{ActionAttempt, AttemptStatus, ExecutionBinding, Work, WorkSpec, WorkStatus};
use std::path::PathBuf;
use std::time::Duration;

fn g2_db_path() -> PathBuf {
    std::env::temp_dir().join("morn_g2_test").join("control.db")
}

fn sample_work(work_id: &str) -> Work {
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

/// The child mode: store Work + Binding + Attempt, commit, then wait to be
/// killed by the parent.
fn g2_child_main(db_path: &PathBuf, marker: &PathBuf) {
    let store = ControlPlaneStore::with_path(db_path).expect("child store must open");
    let work = sample_work("work-g2");
    store.insert_work(&work).expect("child must store work");

    let binding = ExecutionBinding::new("binding-g2", "work-g2", "sha256:g2", "fake-provider");
    store
        .insert_binding(&binding)
        .expect("child must store binding");

    let attempt = ActionAttempt::proposed(
        "attempt-g2",
        "work-g2",
        "binding-g2",
        "cmms.create-order",
        "work-g2:cmms.create-order:1",
    );
    let inserted = store
        .insert_attempt(&attempt)
        .expect("child must store attempt");
    assert!(inserted, "attempt must be inserted exactly once");
    store
        .update_attempt_status("attempt-g2", AttemptStatus::Dispatched)
        .expect("child must mark dispatched");

    std::fs::write(marker, b"ready").expect("child must write marker");
    // Wait to be killed — canonical progress is already in SQLite.
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn g2_process_kill_after_attempt_preserves_work_and_prevents_duplicate() {
    let dir = std::env::temp_dir().join("morn_g2_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = g2_db_path();
    let marker = dir.join("child-ready.marker");

    // Spawn ourselves in child mode.
    let exe = std::env::current_exe().expect("test exe must resolve");
    // SAFETY: the child runs ONLY the child-runner test (filter + exact), so
    // it writes state, signals readiness, and waits to be killed — no recursion.
    let mut child = std::process::Command::new(exe)
        .args(["g2_child_runner", "--exact", "--nocapture"])
        .env("MORN_G2_CHILD", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("child must spawn");

    // The child binary runs ALL tests, so also run this test in child mode via
    // the env gate below. Simpler: the child runs the same test binary; we
    // detect readiness by polling for the marker written by the child branch.
    let mut ready = false;
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(500));
        if marker.exists() {
            ready = true;
            break;
        }
        // Bail out early if the child already exited unexpectedly.
        if let Ok(Some(_status)) = child.try_wait() {
            break;
        }
    }
    assert!(ready, "child never became ready; cannot run the kill test");

    // SAFETY: kill only the child this test spawned (real kill -9 equivalent).
    let _ = child.kill();
    let _ = child.wait();

    // "Restart": a fresh store over the same DB.
    let store = ControlPlaneStore::with_path(&db_path).expect("restarted store must open");

    // 3. Work still exists.
    let work = store
        .get_work("work-g2")
        .expect("work query")
        .expect("work must survive the kill");
    assert_eq!(work.spec.goal, "restore_operational_readiness");

    // 4. Attempt/Binding not lost.
    let binding = store
        .get_binding("binding-g2")
        .expect("binding query")
        .expect("binding must survive");
    assert_eq!(binding.provider_id, "fake-provider");
    let attempt = store
        .get_attempt("attempt-g2")
        .expect("attempt query")
        .expect("attempt must survive");
    assert_eq!(attempt.status, AttemptStatus::Dispatched);

    // 5. Reconciliation continues (source-of-truth readback finds MO-xxxxx).
    let controller = ReconciliationController;
    // The attempt is DISPATCHED, not OUTCOME_UNKNOWN; move it into the unknown
    // path first (as the timeout path would: dispatch -> OUTCOME_UNKNOWN).
    store
        .update_attempt_status("attempt-g2", AttemptStatus::OutcomeUnknown)
        .unwrap();
    let outcome = controller
        .reconcile(&store, "attempt-g2", |_idempotency_key| {
            // Source-of-truth readback: the external system committed.
            Ok(Some("MO-88273".to_string()))
        })
        .expect("reconcile must run")
        .expect("reconcile must produce an outcome");
    assert!(outcome
        .notes
        .iter()
        .any(|n| n.contains("no duplicate action taken")));

    let attempt = store
        .get_attempt("attempt-g2")
        .expect("attempt query")
        .expect("attempt present");
    assert_eq!(attempt.status, AttemptStatus::Verified);

    // 6. No duplicate execution: re-dispatching the same idempotency key is
    // prevented at the persistence layer.
    let duplicate = ActionAttempt::proposed(
        "attempt-g2-dup",
        "work-g2",
        "binding-g2",
        "cmms.create-order",
        "work-g2:cmms.create-order:1",
    );
    let inserted = store.insert_attempt(&duplicate).expect("duplicate insert");
    assert!(!inserted, "duplicate idempotency key must be rejected");
}

/// Child-mode entry: when MORN_G2_CHILD is set, run the child main instead of
/// the normal test flow. Invoked by the parent test through the same binary.
#[test]
fn g2_child_runner() {
    if std::env::var("MORN_G2_CHILD").is_err() {
        return;
    }
    let dir = std::env::temp_dir().join("morn_g2_test");
    let db_path = g2_db_path();
    let marker = dir.join("child-ready.marker");
    g2_child_main(&db_path, &marker);
    unreachable!("g2_child_main waits forever until killed");
}

#[test]
fn g2_work_controller_phase_advancement_is_idempotent() {
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&sample_work("work-1")).unwrap();

    let controller = WorkController;
    let first = controller.reconcile(&store, "work-1").unwrap();
    assert!(first.is_some(), "first reconcile advances the phase");
    let phase_after_first = store.get_work("work-1").unwrap().unwrap().status.phase;

    // Re-running with the same state produces no change.
    let second = controller.reconcile(&store, "work-1").unwrap();
    assert!(second.is_none(), "second reconcile is a no-op (idempotent)");
    assert_eq!(
        store.get_work("work-1").unwrap().unwrap().status.phase,
        phase_after_first
    );
}

#[test]
fn g2_replay_rebuilds_consistent_projection() {
    let store = ControlPlaneStore::in_memory().expect("store");
    let mut work = sample_work("work-2");
    store.insert_work(&work).unwrap();

    // Record events: phase change + an unknown.
    let events = vec![
        morn::control::projection::StoredEvent {
            id: "evt-1".to_string(),
            work_id: Some("work-2".to_string()),
            event_type: morn::control::projection::event_types::STATUS_PHASE_CHANGED.to_string(),
            payload_json: r#"{"phase":"EXECUTING"}"#.to_string(),
            supersedes: None,
            retracts: None,
            valid_time: None,
            recorded_at: "2026-09-25T10:00:00Z".to_string(),
        },
        morn::control::projection::StoredEvent {
            id: "evt-2".to_string(),
            work_id: Some("work-2".to_string()),
            event_type: morn::control::projection::event_types::UNKNOWN_RECORDED.to_string(),
            payload_json: r#"{"unknown":"cmms-timeout"}"#.to_string(),
            supersedes: None,
            retracts: None,
            valid_time: None,
            recorded_at: "2026-09-25T10:01:00Z".to_string(),
        },
    ];
    for event in &events {
        assert!(store.append_event(event).unwrap(), "event must append");
    }

    // Deterministic replay: two runs produce the same projection.
    let stored = store.get_events("work-2").unwrap();
    assert_eq!(stored.len(), 2, "events are idempotent by id");
    let base = WorkStatus {
        observed_generation: 0,
        phase: morn::work::WorkPhase::Draft,
        conditions: Vec::new(),
        active_binding_refs: Vec::new(),
        active_attempt_refs: Vec::new(),
        unresolved_unknowns: Vec::new(),
        observed_outcome_refs: Vec::new(),
        acceptance_state: morn::work::WorkAcceptanceState::Pending,
        blocker_refs: Vec::new(),
        updated_at: None,
    };
    let projection_a = morn::control::projection::project_work(base.clone(), &stored);
    let projection_b = morn::control::projection::project_work(base, &stored);
    assert_eq!(projection_a, projection_b, "replay is deterministic");
    assert_eq!(projection_a.phase, morn::work::WorkPhase::Executing);
    assert!(projection_a
        .unresolved_unknowns
        .contains(&"cmms-timeout".to_string()));

    // Event idempotency: appending the same event id again is a no-op.
    assert!(
        !store.append_event(&events[0]).unwrap(),
        "duplicate event id ignored"
    );
    assert_eq!(store.get_events("work-2").unwrap().len(), 2);

    let _ = &mut work;
}

#[test]
fn g2_phase_event_deserializes_from_string_value() {
    // REGRESSION: serde_json::from_str::<WorkPhase>("EXECUTING") fails because
    // a bare identifier is not a JSON document ("expected value"); unit
    // variants deserialize from a JSON string *value* via from_value. The
    // projection must keep using from_value for enum payloads.
    let bare: Result<morn::work::WorkPhase, _> = serde_json::from_str("EXECUTING");
    assert!(bare.is_err(), "from_str on a bare identifier must fail");

    let from_value = serde_json::from_value::<morn::work::WorkPhase>(serde_json::Value::String(
        "EXECUTING".to_string(),
    ));
    assert_eq!(from_value.unwrap(), morn::work::WorkPhase::Executing);

    // End-to-end: a phase event payload drives the projection.
    let stored = morn::control::projection::StoredEvent {
        id: "e".to_string(),
        work_id: Some("w".to_string()),
        event_type: morn::control::projection::event_types::STATUS_PHASE_CHANGED.to_string(),
        payload_json: r#"{"phase":"EXECUTING"}"#.to_string(),
        supersedes: None,
        retracts: None,
        valid_time: None,
        recorded_at: "2026-09-25T10:00:00Z".to_string(),
    };
    let base = morn::work::WorkStatus {
        observed_generation: 0,
        phase: morn::work::WorkPhase::Draft,
        conditions: Vec::new(),
        active_binding_refs: Vec::new(),
        active_attempt_refs: Vec::new(),
        unresolved_unknowns: Vec::new(),
        observed_outcome_refs: Vec::new(),
        acceptance_state: morn::work::WorkAcceptanceState::Pending,
        blocker_refs: Vec::new(),
        updated_at: None,
    };
    let projected = morn::control::projection::project_work(base, &[stored]);
    assert_eq!(projected.phase, morn::work::WorkPhase::Executing);
}

#[test]
fn g2_outbox_delivers_each_event_exactly_once() {
    let store = ControlPlaneStore::in_memory().expect("store");
    assert!(store
        .enqueue_outbox("ob-1", "evt-1", "io.morn.test.v1", "{}")
        .unwrap());
    // Duplicate enqueue is ignored (idempotent by id).
    assert!(!store
        .enqueue_outbox("ob-1", "evt-1", "io.morn.test.v1", "{}")
        .unwrap());

    let claimed = store.claim_next_outbox().unwrap().expect("entry claimed");
    assert_eq!(claimed.id, "ob-1");
    assert_eq!(claimed.status, "DISPATCHED");
    // A claimed entry is not returned again.
    assert!(store.claim_next_outbox().unwrap().is_none());

    store.complete_outbox("ob-1").unwrap();
    assert!(
        store.claim_next_outbox().unwrap().is_none(),
        "done entries are not redelivered"
    );
}

#[test]
fn g2_controller_lease_is_exclusive() {
    let store = ControlPlaneStore::in_memory().expect("store");
    let acquired = store
        .acquire_lease(
            "work-controller",
            "2099-01-01T00:00:00Z",
            Some(r#"{"cursor":1}"#),
        )
        .unwrap();
    assert!(acquired, "first holder acquires the lease");
    let blocked = store
        .acquire_lease("work-controller", "2099-01-01T00:00:00Z", None)
        .unwrap();
    assert!(
        !blocked,
        "second holder is blocked while the lease is unexpired"
    );
    assert_eq!(
        store.get_checkpoint("work-controller").unwrap().as_deref(),
        Some(r#"{"cursor":1}"#)
    );
    store.release_lease("work-controller").unwrap();
    let reacquired = store
        .acquire_lease("work-controller", "2099-01-01T00:00:00Z", None)
        .unwrap();
    assert!(reacquired, "released leases can be re-acquired");
}
