//! M19 Conformance Framework: Provider/Runtime/Connector/Plugin/DomainPack/
//! Architecture conformance kits.

use morn_integration::ConnectorProvider;
use morn_kernel::version::Version;
use morn_package::{PackLifecycle, PackManifest, PackStatus};
use morn_runtime::RuntimeProvider;
use morn_work::durable::{Signal, SignalKind};

#[test]
fn provider_conformance_two_fixtures() {
    for p in [
        &morn_harness::RuleIntelligence as &dyn morn_harness::IntelligenceProvider,
        &morn_harness::SolverIntelligence,
    ] {
        morn_harness::run_intelligence_conformance(p).unwrap();
    }
    // Harness smoke contract on two providers.
    let ws = morn_kernel::ids::WorkspaceId::generate();
    let mut ctx = morn_harness::contract::test_context(&ws);
    ctx.provenance_refs.push("wp-1".to_string());
    let mut native = morn_harness::MornNativeHarness::new();
    assert!(morn_harness::run_harness_smoke(&mut native, &ctx)
        .unwrap()
        .all_ok());
    let mut dsh = morn_harness::provider::DeepSeekHarnessProvider::new(
        morn_harness::provider::DshMode::Fixture,
    );
    assert!(morn_harness::run_harness_smoke(&mut dsh, &ctx)
        .unwrap()
        .all_ok());
}

#[test]
fn runtime_provider_conformance_fixture() {
    // Public RuntimeProvider protocol (morn-runtime): start -> checkpoint ->
    // restore -> signal -> events -> health, plus explicit error on malformed
    // restore and duplicate start.
    let mut rt = morn_runtime::FixtureRuntime::new();
    morn_runtime::run_runtime_conformance(&mut rt).unwrap();
    assert_eq!(rt.provider_name(), "fixture-runtime");
    assert!(rt.health());
    rt.start_run("r2").unwrap();
    assert!(
        rt.start_run("r2").is_err(),
        "duplicate start must be rejected"
    );
    assert!(
        rt.restore("garbage").is_err(),
        "malformed restore must be rejected"
    );
}

#[test]
fn runtime_conformance_checkpoint_restore_signal() {
    let ws = morn_kernel::ids::WorkspaceId::generate();
    let mut rt = morn_work::DurableRuntime::new();
    let def = morn_work::WorkflowDefinition::new(ws.clone(), "wf")
        .add_step(morn_work::WorkflowStep::new(
            "a",
            morn_work::WorkflowStepKind::Auto,
        ))
        .add_step(morn_work::WorkflowStep::new(
            "w",
            morn_work::WorkflowStepKind::SignalWait,
        ));
    let def_id = def.id.clone();
    rt.register_workflow(def);
    let run = rt.start_run(&def_id, None).unwrap();
    rt.wait_for_signal(&run.id, SignalKind::HumanApproval, 60)
        .unwrap();
    let cp = rt.checkpoint(&run.id).unwrap();
    assert!(cp.payload_json.contains("WaitingSignal"));
    // restore from checkpoint payload into a fresh runtime
    let restored: morn_work::WorkflowRun = serde_json::from_str(&cp.payload_json).unwrap();
    let mut rt2 = morn_work::DurableRuntime::new();
    rt2.restore_run(restored);
    rt2.load_and_resume(&run.id, "v1", "v1", true).unwrap();
    rt2.deliver_signal(Signal::new(
        run.id.clone(),
        SignalKind::HumanApproval,
        "ok",
        "pi",
        "pi",
    ))
    .unwrap();
    assert_eq!(
        rt2.run(&run.id).unwrap().status,
        morn_work::RunStatus::Running
    );
}

#[test]
fn connector_conformance_governed_write_idempotent() {
    let mut c = morn_integration::GenericFixtureConnector::new();
    let req = morn_integration::ExternalActionRequest {
        external_ref: morn_integration::ExternalObjectRef::generate_with("ext"),
        action: "create".to_string(),
        payload: serde_json::json!({}),
        proposal_id: morn_kernel::ids::ActionProposalId::generate(),
    };
    let tok = |p: &morn_integration::ExternalActionRequest| morn_integration::ApprovedActionToken {
        proposal_id: p.proposal_id.clone(),
        granted_by: "gw".to_string(),
    };
    let r1 = c.execute_governed(&req, &tok(&req)).unwrap();
    assert!(r1.ok);
    let r2 = c.execute_governed(&req, &tok(&req)).unwrap();
    assert_eq!(c.applied_external_ids.len(), 1);
    assert_eq!(r2.external_id.as_deref(), Some("dup"));
    c.teardown();
}

#[test]
fn plugin_and_domain_pack_conformance() {
    // Plugin conformance: manifest compat + lifecycle, no bypass.
    let mut lc = PackLifecycle::new();
    let id = lc
        .init(PackManifest::new("p", "domain-pack", Version::v1()))
        .unwrap();
    lc.validate(&id).unwrap();
    lc.build(&id).unwrap();
    lc.install(&id).unwrap();
    lc.enable(&id).unwrap();
    lc.uninstall(&id).unwrap();
    assert_eq!(lc.inspect(&id).unwrap().status, PackStatus::Uninstalled);
    assert!(!lc.history.is_empty(), "history preserved after uninstall");

    // DomainPack conformance: install + enable + declare + disable.
    let mut reg = morn_domain_sdk::DomainRegistry::new();
    reg.install(
        morn_domain_sdk::DomainDefinition::new("hello-domain", "1.0.0", "1.0.0").declare(
            "object_type",
            "Thing",
            serde_json::json!({}),
        ),
    )
    .unwrap();
    assert!(reg.enable("hello-domain"));
    assert_eq!(reg.declarations_for("hello-domain", "object_type").len(), 1);
    reg.disable("hello-domain");
    assert!(reg
        .declarations_for("hello-domain", "object_type")
        .is_empty());
}

#[test]
fn architecture_conformance_core_has_no_domain_pack() {
    // Core crates must not declare a domain pack dependency (non-optional).
    for name in [
        "morn-kernel",
        "morn-world",
        "morn-work",
        "morn-artifact",
        "morn-capability",
        "morn-harness",
        "morn-runtime",
        "morn-actor",
        "morn-organization",
        "morn-evolution",
        "morn-foundry",
        "morn-assurance",
        "morn-store",
        "morn-app",
        "morn-opint",
        "morn-integration",
        "morn-process",
        "morn-node",
        "morn-domain-sdk",
        "morn-package",
    ] {
        let toml = std::fs::read_to_string(format!(
            "{}/../../crates/{name}/Cargo.toml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        assert!(
            !toml.contains("domain-packs/biolab-reference"),
            "core crate {name} depends on a domain pack"
        );
    }
}
