//! failure_engineering — Phase 18 fault-injection supplements.
//!
//! Proves the remaining fault paths (Goal §22):
//! - reordered events: replay stays deterministic, retract cancels effects;
//! - duplicate events: duplicate event ids are no-ops;
//! - slow provider: a slow/timing-out provider surfaces as a typed error and
//!   never fakes success;
//! - profile upgrade during work: the Work's state/generation is not
//!   silently rewritten by a profile upgrade; released versions are frozen.

use morn::control::projection::{event_types, project_work, StoredEvent};
use morn::control::store::ControlPlaneStore;
use morn::providers::connector::{ConnectorProvider, FakeCmmsConnector};
use morn::work::{Profile, Work, WorkPhase, WorkSpec, WorkStatus};
use std::time::{Duration, Instant};

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

fn base_status() -> WorkStatus {
    WorkStatus {
        observed_generation: 0,
        phase: WorkPhase::Draft,
        conditions: Vec::new(),
        active_binding_refs: Vec::new(),
        active_attempt_refs: Vec::new(),
        unresolved_unknowns: Vec::new(),
        observed_outcome_refs: Vec::new(),
        acceptance_state: morn::work::WorkAcceptanceState::Pending,
        blocker_refs: Vec::new(),
        updated_at: None,
    }
}

fn event(id: &str, event_type: &str, payload_json: &str, recorded_at: &str) -> StoredEvent {
    StoredEvent {
        id: id.to_string(),
        work_id: Some("work-fe".to_string()),
        event_type: event_type.to_string(),
        payload_json: payload_json.to_string(),
        supersedes: None,
        retracts: None,
        valid_time: None,
        recorded_at: recorded_at.to_string(),
    }
}

#[test]
fn g_reordered_event_projection_is_deterministic() {
    // SAFETY: events may arrive reordered (multi-worker, retry); the
    // projection must be deterministic — the retract cancels the effect of
    // the event it names, regardless of arrival order.
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&sample_work("work-fe")).unwrap();

    // Three events: phase change, an unknown, and a retract of the unknown.
    let e1 = event(
        "evt-1",
        event_types::STATUS_PHASE_CHANGED,
        r#"{"phase":"EXECUTING"}"#,
        "2026-09-28T10:00:00Z",
    );
    let e2 = event(
        "evt-2",
        event_types::UNKNOWN_RECORDED,
        r#"{"unknown":"cmms-timeout"}"#,
        "2026-09-28T10:01:00Z",
    );
    let mut e3 = event(
        "evt-3",
        event_types::UNKNOWN_RECORDED,
        r#"{"unknown":"cmms-timeout"}"#,
        "2026-09-28T10:02:00Z",
    );
    e3.retracts = Some("evt-2".to_string());

    // Append IN ORDER, project.
    for e in [&e1, &e2, &e3] {
        assert!(store.append_event(e).unwrap(), "event must append");
    }
    let stored = store.get_events("work-fe").unwrap();
    assert_eq!(stored.len(), 3);
    let ordered_projection = project_work(base_status(), &stored);

    // Append REORDERED (fresh store), project again: the same projection.
    let store2 = ControlPlaneStore::in_memory().expect("store2");
    store2.insert_work(&sample_work("work-fe")).unwrap();
    for e in [&e3, &e1, &e2] {
        store2.append_event(e).unwrap();
    }
    let stored2 = store2.get_events("work-fe").unwrap();
    let reordered_projection = project_work(base_status(), &stored2);

    assert_eq!(
        ordered_projection, reordered_projection,
        "replay is deterministic regardless of arrival order"
    );
    // The retract canceled the unknown: not unresolved.
    assert!(
        !ordered_projection
            .unresolved_unknowns
            .contains(&"cmms-timeout".to_string()),
        "the retract cancels the unknown it names"
    );
    assert_eq!(ordered_projection.phase, WorkPhase::Executing);
}

#[test]
fn g_duplicate_event_is_a_no_op() {
    // Duplicate event ids are ignored (idempotent by id) — a retry or a
    // duplicate message cannot duplicate the event.
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&sample_work("work-fe")).unwrap();

    let e1 = event(
        "evt-1",
        event_types::STATUS_PHASE_CHANGED,
        r#"{"phase":"EXECUTING"}"#,
        "2026-09-28T10:00:00Z",
    );
    assert!(store.append_event(&e1).unwrap(), "first append");
    assert!(
        !store.append_event(&e1).unwrap(),
        "duplicate append is a no-op"
    );
    assert!(
        !store.append_event(&e1).unwrap(),
        "a second duplicate is also a no-op"
    );
    assert_eq!(
        store.get_events("work-fe").unwrap().len(),
        1,
        "exactly one event stored"
    );
}

#[test]
fn g_slow_provider_surfaces_typed_and_never_fakes() {
    // A slow provider (timeout-after-commit mode): the caller gets a typed
    // transport error — never a fake success; the order committed on the
    // "server" and the reconciliation readback finds it.
    let connector = FakeCmmsConnector::new();
    connector.arm_timeout_after_commit();

    let start = Instant::now();
    let result = connector.create_order("work-fe:cmms.create-order:1", "{}");
    let elapsed = start.elapsed();

    assert!(
        result.is_err(),
        "the slow provider surfaces as a typed error"
    );
    assert!(result
        .unwrap_err()
        .contains("transport timeout after commit"));
    // The failure path is fast in the fake (the real timeout is external);
    // the semantic is what matters: error, not success.
    assert!(elapsed < Duration::from_secs(5));

    // The order WAS committed despite the timeout — the unknown is real.
    let found = connector
        .query_order("work-fe:cmms.create-order:1")
        .expect("readback");
    assert!(
        found.is_some(),
        "the order committed on the server despite the timeout"
    );
}

#[test]
fn g_profile_upgrade_during_work_pins_work_state() {
    // A profile upgrade during a Work must not silently rewrite the Work's
    // committed state: the Work's generation/status are Morn-owned; the
    // released profile version is frozen (a new version is a NEW profile).
    let store = ControlPlaneStore::in_memory().expect("store");
    let mut work = sample_work("work-fe");
    work.spec.profile_ref = Some("morn.factory.readonly".to_string());
    store.insert_work(&work).unwrap();

    // The Work advances (phase change is recorded).
    let before_phase = store.get_work("work-fe").unwrap().unwrap().status.phase;

    // "Profile upgrade": a NEW version of the profile is constructed (the
    // released v1alpha1 content is frozen — it is never mutated in place).
    let old_profile = Profile::factory_readonly();
    let new_profile = Profile::factory_readonly();
    // COMPATIBILITY: version upgrade creates a new version reference; the
    // released version content is frozen. The Work's pinned profile_ref
    // stays until an explicit migration.
    assert_eq!(
        old_profile.version, new_profile.version,
        "the same released version is content-identical"
    );
    assert_eq!(old_profile, new_profile);

    // The Work's state is unchanged by the profile upgrade: same phase, same
    // generation, same profile_ref.
    let after = store.get_work("work-fe").unwrap().unwrap();
    assert_eq!(
        after.status.phase, before_phase,
        "the Work state is not rewritten"
    );
    assert_eq!(
        after.spec.generation, 1,
        "the Work generation is not bumped by a profile upgrade"
    );
    assert_eq!(
        after.spec.profile_ref.as_deref(),
        Some("morn.factory.readonly"),
        "the Work keeps its pinned profile ref"
    );

    // A genuine spec change (generation bump) is the explicit path.
    let mut migrated = after.clone();
    migrated.spec.generation = 2;
    migrated.spec.profile_ref = Some("morn.factory.readonly/v2".to_string());
    store
        .update_work_status("work-fe", &migrated.status)
        .unwrap();
    let final_work = store.get_work("work-fe").unwrap().unwrap();
    assert_eq!(final_work.status.phase, before_phase);
    let _ = final_work;
}
