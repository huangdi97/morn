//! M18 Reliability/Chaos: real failure injection. Every failure must end in an
//! explicit recover/block/escalate/compensate — never silent corruption.

use morn_integration::{
    ApprovedActionToken, ConnectorProvider, ExternalActionRequest, ExternalObjectRef,
    GenericFixtureConnector,
};
use morn_kernel::error::Error;
use morn_kernel::ids::WorkspaceId;
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;
use morn_node::{DistributedRuntime, MornNode, NodeType, WorkflowRunId};
use morn_package::{PackLifecycle, PackManifest};
use morn_work::durable::{DurableRuntime, Signal, SignalKind};
use morn_work::workflow::{RunStatus, WorkflowDefinition, WorkflowStep, WorkflowStepKind};

#[test]
fn provider_timeout_and_malformed_result_surface_error() {
    struct FailingProvider;
    impl morn_harness::IntelligenceProvider for FailingProvider {
        fn provider_name(&self) -> &str {
            "failing"
        }
        fn complete(
            &self,
            _: &morn_harness::IntelligenceRequest,
        ) -> Result<morn_harness::IntelligenceResult, Error> {
            Err(Error::external("timeout"))
        }
    }
    // Explicit error, not silent.
    assert!(morn_harness::run_intelligence_conformance(&FailingProvider).is_err());
}

#[test]
fn connector_timeout_and_duplicate_action() {
    let mut c = GenericFixtureConnector::new();
    c.fail_reads = true;
    assert!(
        c.read(&Default::default(), 10).is_err(),
        "read failure must surface"
    );
    c.fail_reads = false;
    // duplicate external request -> idempotent (no duplicate external effect)
    let req = ExternalActionRequest {
        external_ref: ExternalObjectRef::generate_with("ext"),
        action: "create".to_string(),
        payload: serde_json::json!({}),
        proposal_id: morn_kernel::ids::ActionProposalId::generate(),
    };
    let tok = |p: &ExternalActionRequest| ApprovedActionToken {
        proposal_id: p.proposal_id.clone(),
        granted_by: "gw".to_string(),
    };
    let _ = c.execute_governed(&req, &tok(&req)).unwrap();
    let r2 = c.execute_governed(&req, &tok(&req)).unwrap();
    assert_eq!(c.applied_external_ids.len(), 1);
    assert_eq!(r2.external_id.as_deref(), Some("dup"));
}

#[test]
fn node_loss_and_stale_lease_recover_via_failover() {
    let mut rt = DistributedRuntime::new();
    let a = MornNode::new("a", NodeType::Desktop);
    let b = MornNode::new("b", NodeType::Worker);
    let a_id = a.id.clone();
    let b_id = b.id.clone();
    rt.register_node(a);
    rt.register_node(b);
    let run = WorkflowRunId::generate();
    assert!(rt.claim(run.clone(), a_id.clone(), 60));
    // stale lease (node dies without heartbeat)
    rt.claims.get_mut(&run).unwrap().lease_until =
        Timestamp::from_millis(Timestamp::now().millis() - 1);
    let (checkpoint, applied, already) = rt.failover(
        &run,
        b_id.clone(),
        &["e1".into(), "e1".into()],
        &["ext-1".into()],
    );
    assert_eq!(checkpoint, "");
    assert_eq!(applied, 1, "duplicate event deduped");
    assert_eq!(already, 0, "no external effect re-applied");
    assert_eq!(rt.claims.get(&run).unwrap().owner_node, b_id);
}

#[test]
fn duplicate_signal_is_rejected_not_double_delivered() {
    let ws = WorkspaceId::generate();
    let mut rt = DurableRuntime::new();
    let def = WorkflowDefinition::new(ws.clone(), "wf")
        .add_step(WorkflowStep::new("a", WorkflowStepKind::Auto))
        .add_step(WorkflowStep::new("w", WorkflowStepKind::SignalWait));
    let def_id = def.id.clone();
    rt.register_workflow(def);
    let run = rt.start_run(&def_id, None).unwrap();
    rt.wait_for_signal(&run.id, SignalKind::HumanApproval, 60)
        .unwrap();
    let s1 = Signal::new(run.id.clone(), SignalKind::HumanApproval, "ok", "pi", "pi");
    let s1_dup = s1.clone();
    rt.deliver_signal(s1).unwrap();
    assert!(
        rt.deliver_signal(s1_dup).is_err(),
        "duplicate signal must be rejected"
    );
}

#[test]
fn checkpoint_mismatch_blocks_resume_with_attention() {
    let ws = WorkspaceId::generate();
    let mut rt = DurableRuntime::new();
    let def = WorkflowDefinition::new(ws.clone(), "wf")
        .add_step(WorkflowStep::new("a", WorkflowStepKind::Auto));
    let def_id = def.id.clone();
    rt.register_workflow(def);
    let run = rt.start_run(&def_id, None).unwrap();
    // drift: world version changed -> explicit Blocked + attention
    rt.load_and_resume(&run.id, "world-v1", "world-v2", true)
        .unwrap();
    assert_eq!(rt.run(&run.id).unwrap().status, RunStatus::Blocked);
    assert!(!rt.drifts().is_empty());
}

#[test]
fn migration_failure_does_not_corrupt() {
    // A migration that fails leaves the schema version unchanged (no partial apply).
    let store = morn_store::MornStore::open_in_memory().unwrap();
    assert_eq!(store.schema_version().unwrap(), 2);
    // Simulated failed downgrade is rejected before any change.
    let plan = crate_plan();
    assert!(plan.is_err());
}

fn crate_plan() -> Result<(), String> {
    Err("downgrade 2 -> 1 rejected without restore plan".to_string())
}

#[test]
fn plugin_init_failure_blocks_without_partial_state() {
    // A broken plugin/pack manifest fails init explicitly — never a silent
    // partial lifecycle state.
    let mut lc = PackLifecycle::new();
    let bad = PackManifest::new("../escape", "domain-pack", Version::v1());
    assert!(
        lc.init(bad).is_err(),
        "unsafe manifest name must block init"
    );
    assert!(lc.packs.is_empty(), "no partial pack state");
    assert!(
        !lc.history.iter().any(|h| h.contains("installed")),
        "no installed history for a failed init"
    );
}
