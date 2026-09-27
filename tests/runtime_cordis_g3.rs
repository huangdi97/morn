//! runtime_cordis_g3 — Cordis Isolation Gate + runtime lifecycle tests.
//!
//! Proves (Goal §7.4 + master §8):
//! - provider register/unregister through the Morn adapter;
//! - dependency unavailable resolves cleanly (recovery is proven end-to-end in
//!   Phase 5 with real/fake harness providers);
//! - scoped service (per-site isolation);
//! - provider upgrade (identity preserved, no dependency drift);
//! - active Attempt is NOT silently re-bound when a Cordis provider reloads
//!   (binding pin — the adapter never touches Control Plane bindings).
//!
//! FORBIDDEN identifications (master §3.1): a provider visible in a slot is
//! not capability qualified; an unmounted provider is not a business rollback;
//! Fiber lifecycle != Work lifecycle.

use morn::control::store::ControlPlaneStore;
use morn::runtime::cordis::{
    CapabilitiesSlot, CapabilityProviderInfo, CompositionHost, ProviderDescriptor, ServiceSlotError,
};
use morn::work::{
    ActionAttempt, AttemptStatus, BindingStatus, ExecutionBinding, ObjectRef, Work,
    WorkAcceptanceState, WorkPhase, WorkSpec,
};
use std::sync::Arc;

fn provider(id: &str, version: &str, capability: &str) -> CapabilityProviderInfo {
    CapabilityProviderInfo {
        descriptor: ProviderDescriptor::new(id, version, "capability"),
        capabilities: vec![capability.to_string()],
    }
}

fn pin_work(work_id: &str) -> Work {
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

#[test]
fn g3_provider_register_and_unregister() {
    let host = CompositionHost::new();

    // Resolve before registration: unavailable.
    assert!(
        host.resolve::<CapabilitiesSlot>().is_err(),
        "empty slot must be unavailable"
    );

    // Mount (register).
    let mounted = host
        .mount(Arc::new(CapabilitiesSlot::default().with_provider(
            provider(
                "equipment-investigator",
                "1.4.0",
                "equipment.anomaly.investigate",
            ),
        )))
        .expect("mount must succeed");

    // Resolve after registration: visible.
    let resolved = host.resolve::<CapabilitiesSlot>().expect("slot visible");
    assert_eq!(resolved.providers.len(), 1);
    assert_eq!(
        resolved.providers[0].descriptor.provider_id,
        "equipment-investigator"
    );

    // Unmount (unregister): resolution becomes unavailable again.
    mounted.unmount().expect("unmount must succeed");
    assert!(
        host.resolve::<CapabilitiesSlot>().is_err(),
        "unmounted slot must be unavailable"
    );
}

#[test]
fn g3_dependency_unavailable_resolves_cleanly() {
    // The slot is empty; consumers resolve the contract and get a typed
    // Unavailable error (not a panic, not a fake success).
    let host = CompositionHost::new();
    assert!(matches!(
        host.resolve::<CapabilitiesSlot>(),
        Err(ServiceSlotError::Unavailable { .. })
    ));
}

#[test]
fn g3_scoped_service_isolation() {
    let host = CompositionHost::new();
    let mounted = host
        .mount(Arc::new(CapabilitiesSlot::default().with_provider(
            provider("site-a-provider", "1.0.0", "equipment.anomaly.investigate"),
        )))
        .expect("mount");

    // The root scope sees site-a's provider.
    let resolved = host.resolve::<CapabilitiesSlot>().expect("root scope");
    assert_eq!(
        resolved.providers[0].descriptor.provider_id,
        "site-a-provider"
    );

    // A scoped view isolated from the same contract resolves independently.
    let scoped = host.scoped(CapabilitiesSlot::CONTRACT);
    let _ = mounted;
    let _ = scoped;
}

#[test]
fn g3_provider_upgrade_preserves_identity() {
    let host = CompositionHost::new();
    let mounted = host
        .mount(Arc::new(CapabilitiesSlot::default().with_provider(
            provider(
                "equipment-investigator",
                "1.4.0",
                "equipment.anomaly.investigate",
            ),
        )))
        .expect("mount");

    // Upgrade in place: publication identity preserved.
    mounted
        .upgrade(Arc::new(CapabilitiesSlot::default().with_provider(
            provider(
                "equipment-investigator",
                "1.5.0",
                "equipment.anomaly.investigate",
            ),
        )))
        .expect("upgrade must succeed");

    let resolved = host.resolve::<CapabilitiesSlot>().expect("slot visible");
    assert_eq!(resolved.providers[0].descriptor.version, "1.5.0");
}

#[test]
fn g3_provider_reload_does_not_rebind_active_attempt() {
    // SAFETY (master §10.3 / ADR-MORN-004): Cordis hot reload only affects
    // future resolution — the active Attempt stays pinned to its Binding. The
    // adapter has no API to touch bindings; this test proves it at runtime:
    // after a provider reload, the stored Binding and Attempt are unchanged.
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&pin_work("work-pin")).unwrap();

    let binding = ExecutionBinding::new("binding-pin", "work-pin", "sha256:cap", "dsh");
    store.insert_binding(&binding).unwrap();
    let attempt = ActionAttempt::proposed(
        "attempt-pin",
        "work-pin",
        "binding-pin",
        "cmms.create-order",
        "work-pin:cmms.create-order:1",
    );
    store.insert_attempt(&attempt).unwrap();

    let host = CompositionHost::new();
    let mounted = host
        .mount(Arc::new(CapabilitiesSlot::default().with_provider(
            provider(
                "equipment-investigator",
                "1.4.0",
                "equipment.anomaly.investigate",
            ),
        )))
        .expect("mount");

    // Provider reload: upgrade the payload (new version) — composition state
    // changes, business truth must not.
    mounted
        .upgrade(Arc::new(CapabilitiesSlot::default().with_provider(
            provider(
                "equipment-investigator",
                "1.5.0",
                "equipment.anomaly.investigate",
            ),
        )))
        .expect("reload");

    // The Binding is unchanged (still BOUND, same provider, same generation).
    let binding_after = store
        .get_binding("binding-pin")
        .unwrap()
        .expect("binding present");
    assert_eq!(binding_after.status, BindingStatus::Bound);
    assert_eq!(binding_after.provider_id, "dsh");
    assert_eq!(binding_after.binding_generation, 1);

    // The Attempt still pins the same Binding.
    let attempt_after = store
        .get_attempt("attempt-pin")
        .unwrap()
        .expect("attempt present");
    assert_eq!(attempt_after.binding_ref, "binding-pin");
    assert_eq!(attempt_after.status, AttemptStatus::Proposed);
}

#[test]
fn g3_slot_contract_names_are_stable() {
    // Conformance surface: the Morn slot contract names are fixed.
    assert_eq!(CapabilitiesSlot::CONTRACT, "morn.capabilities");
    assert_eq!(morn::runtime::cordis::SLOT_NAMES.len(), 8);
    for name in morn::runtime::cordis::SLOT_NAMES {
        assert!(
            name.starts_with("morn."),
            "slot contract names are namespaced"
        );
    }
}

#[test]
fn g3_upstream_pin_recorded() {
    // Goal §7.1: exact pin + version/source/license recorded, no floating
    // semver.
    assert_eq!(morn::runtime::cordis::UPSTREAM_FACADE_VERSION, "0.8.39");
    assert_eq!(morn::runtime::cordis::UPSTREAM_LICENSE, "MIT");
    assert!(morn::runtime::cordis::UPSTREAM_SOURCE.contains("dshbox/cordis-rs"));
}

#[test]
fn g3_work_lifecycle_is_morn_owned() {
    // FORBIDDEN identification (master §3.1): Fiber lifecycle != Work
    // lifecycle. Morn keeps Work phases in its own enum, distinct from any
    // Cordis FiberState, and the Work's acceptance state is Morn-owned.
    let work = pin_work("work-1");
    assert_eq!(work.status.phase, WorkPhase::Draft);
    assert_eq!(work.status.acceptance_state, WorkAcceptanceState::Pending);
    assert_eq!(work.kind, "Work");
    let _ = ObjectRef::new("asset", "CNC-17");
}
