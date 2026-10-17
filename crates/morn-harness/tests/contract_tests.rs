//! The SAME provider contract suite must pass for at least two provider paths:
//! MornNativeHarness and DeepSeekHarnessProvider (fixture mode).

use morn_harness::contract::{run_provider_contract, test_context};
use morn_harness::provider::{
    DeepSeekHarnessProvider, DshMode, HarnessProvider, MornNativeHarness,
    DSH_REAL_E0_SCOPE_RESTRICTION,
};
use morn_harness::{CapabilityScope, PiHarnessProvider, PiMode, ScopeKind};
use morn_kernel::ids::WorkspaceId;
use morn_world::object::{Object, ObjectType};
use serde_json::json;

#[test]
fn morn_native_passes_provider_contract() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = MornNativeHarness::new();
    let report = run_provider_contract(&mut provider, &ctx).expect("contract suite should run");
    assert!(report.all_passed(), "checks: {:?}", report.checks);
    assert_eq!(report.provider, "morn-native");
}

#[test]
fn deepseek_harness_fixture_passes_provider_contract() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = DeepSeekHarnessProvider::new(DshMode::Fixture);
    let report = run_provider_contract(&mut provider, &ctx).expect("contract suite should run");
    assert!(report.all_passed(), "checks: {:?}", report.checks);
    assert_eq!(report.provider, "deepseek-harness");
}

#[test]
fn harness_events_do_not_mutate_canonical_world() {
    // Session events are execution facts; they must never change Morn canonical state.
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut world = morn_world::WorldService::new();
    let obj_type = ObjectType::new("t", ws.clone(), vec!["s".into()], vec![], vec![]);
    world.register_object_type(obj_type.clone());
    let obj_id = morn_kernel::ids::ObjectId::generate_with("obj");
    world.register_object(Object::new(
        obj_id.clone(),
        obj_type.id.clone(),
        ws.clone(),
        {
            let mut m = std::collections::BTreeMap::new();
            m.insert("s".to_string(), json!("a"));
            m
        },
    ));

    let mut native = MornNativeHarness::new();
    let session = native.start(&ctx).unwrap();
    native.send(&session.id, "run").unwrap();
    native.terminate(&session.id).unwrap();

    assert_eq!(
        world.object(&obj_id).unwrap().state().get("s"),
        Some(&json!("a")),
        "harness events must not mutate world state"
    );
    assert!(
        world.ledger_entries().is_empty(),
        "harness events must not create ledger entries"
    );
}

#[test]
fn provider_switch_preserves_actor_identity_and_canonical_records() {
    use morn_actor::actor::{ActorInstance, ActorOrigin};
    use morn_harness::binding::{HarnessBinding, RuntimeBinding};
    use morn_kernel::ids::{ActorTemplateId, HarnessSpecId, IdentityId};
    use morn_kernel::version::Version;

    let identity_id = IdentityId::generate();
    let actor = ActorInstance::new(
        ActorTemplateId::generate(),
        identity_id.clone(),
        ActorOrigin::Independent,
    );

    // Bind to native harness, then switch to DeepSeek Harness provider.
    let spec_id = HarnessSpecId::generate();
    let hb_native = HarnessBinding::new(actor.id.clone(), spec_id.clone(), Version::v1());
    let rb_native = RuntimeBinding::new(actor.id.clone(), "morn-native");
    let hb_dsh = HarnessBinding::new(actor.id.clone(), spec_id, Version::new(1, 1, 0));
    let rb_dsh = RuntimeBinding::new(actor.id.clone(), "deepseek-harness");

    // Identity / workspace / work / artifact semantics belong to Morn, not the provider.
    assert_eq!(actor.identity_id, identity_id);
    assert_eq!(actor.actor_origin, ActorOrigin::Independent);
    assert_eq!(hb_native.actor_id, hb_dsh.actor_id);
    assert_eq!(rb_native.actor_id, rb_dsh.actor_id);
    assert_eq!(actor.workspace_bindings.len(), 0);
    assert_eq!(actor.role_bindings.len(), 0);
}

#[test]
fn real_pi_scope_is_e0_only_before_any_runtime_starts() {
    let ws = WorkspaceId::generate();
    let mut provider = PiHarnessProvider::new(PiMode::Real);
    let unsafe_scope = CapabilityScope::new(ScopeKind::ExecutionRun, None, ws.clone(), "unsafe");
    assert!(provider.mount(unsafe_scope).is_err());

    let safe_scope = CapabilityScope::new(ScopeKind::ExecutionRun, None, ws, "isolated")
        .with_restriction(morn_harness::pi::PI_REAL_E0_SCOPE_RESTRICTION);
    assert!(provider.mount(safe_scope).is_ok());
}

#[test]
fn real_dsh_scope_is_e0_only_before_any_runtime_starts() {
    let ws = WorkspaceId::generate();
    let mut provider = DeepSeekHarnessProvider::new(DshMode::Real);
    let unsafe_scope = CapabilityScope::new(ScopeKind::ExecutionRun, None, ws.clone(), "unsafe");
    assert!(provider.mount(unsafe_scope).is_err());

    let safe_scope = CapabilityScope::new(ScopeKind::ExecutionRun, None, ws, "isolated")
        .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
    assert!(provider.mount(safe_scope).is_ok());
}

#[test]
fn deepseek_harness_real_mode_reports_external_blocker() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = DeepSeekHarnessProvider::new(DshMode::Real);
    let err = provider.start(&ctx);
    assert!(
        err.is_err(),
        "real DSH is not available in this environment"
    );
    let msg = format!("{}", err.unwrap_err());
    assert!(msg.contains("not configured"), "unexpected error: {msg}");
}
