//! M20 Pure Core E2E (zero-domain): start -> workspace -> generic work ->
//! capability/workcell -> execute -> approval -> artifact -> outcome ->
//! restart -> node failover -> governed connector action -> audit/history.

use morn_app::AppState;
use morn_capability::effect::EffectContract;
use morn_integration::ConnectorProvider;
use morn_kernel::policy::{Policy, PolicyRule};
use morn_kernel::version::Version;
use morn_node::{DistributedRuntime, MornNode, NodeType, WorkflowRunId};
use morn_package::{PackLifecycle, PackManifest, PackStatus};
use morn_runtime::gateway::ActionGateway;

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g5_pure_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

#[test]
fn pure_core_e2e_zero_domain() {
    let db = temp_db("pure");
    let ws_id;
    // Phase 1: start + workspace + generic work + capability/workcell + execute
    // + approval + artifact + outcome.
    {
        let state = AppState::new(&db).unwrap();
        let mut guard = state.lock();
        ws_id = guard.workspace.id.clone();

        // Generic object.
        let obj_type = morn_world::object::ObjectType::new(
            "generic.Task",
            ws_id.clone(),
            vec!["status".to_string()],
            vec!["pending".to_string(), "done".to_string()],
            vec![],
        );
        guard.world.register_object_type(obj_type.clone());
        let obj = morn_world::object::Object::new(
            morn_kernel::ids::ObjectId::generate_with("task"),
            obj_type.id.clone(),
            ws_id.clone(),
            {
                let mut s = std::collections::BTreeMap::new();
                s.insert("status".to_string(), serde_json::json!("pending"));
                s
            },
        );
        let obj_id = obj.id.clone();
        guard.world.register_object(obj);

        // Generic work package + acceptance.
        let acc = morn_work::acceptance::AcceptanceSpec::new("generic-acceptance")
            .with_required_artifacts(vec!["result".to_string()]);
        let acc_id = acc.id.clone();
        guard.work.add_acceptance_spec(acc);
        let wp = morn_work::work_package::WorkPackage::new(
            ws_id.clone(),
            "Generic deliverable",
            morn_kernel::ids::PrincipalId::generate_with("owner"),
        )
        .with_acceptance_spec(acc_id)
        .with_execution_mode(morn_work::execution_mode::ExecutionMode::deterministic(
            morn_work::execution_mode::ExecutorType::Program,
        ));
        let wp_id = wp.id.clone();
        guard.work.add_work_package(wp);

        // Capability + governed action (E1 transactional).
        let policy = Policy::new(
            ws_id.clone(),
            "core",
            vec![PolicyRule::allow("execute_task")],
        );
        let mut gateway = ActionGateway::new(policy, EffectContract::e1());
        let proposal = morn_world::action::ActionProposal::new(
            morn_kernel::ids::ActionTypeId::generate(),
            obj_id.clone(),
            ws_id.clone(),
            std::collections::BTreeMap::new(),
            "program",
        );
        let authorized = gateway.authorize(&proposal, "execute_task").unwrap();
        let mut new_state = std::collections::BTreeMap::new();
        new_state.insert("status".to_string(), serde_json::json!("done"));
        let outcome = gateway
            .execute(
                &authorized,
                obj_id.clone(),
                new_state,
                Some("program executed".to_string()),
                "program",
                &mut guard.world,
            )
            .unwrap();
        assert!(outcome.verified);

        // Artifact (approved for downstream).
        let (artifact, version) = guard
            .artifacts
            .create(
                "generic.result",
                "morn@1",
                ws_id.clone(),
                morn_kernel::ids::PrincipalId::generate(),
                "ref",
                serde_json::json!({"ok": true}),
                "c",
            )
            .unwrap();
        guard.artifacts.submit(&version.id).unwrap();
        guard.artifacts.mark_in_review(&version.id).unwrap();
        guard
            .artifacts
            .review(morn_artifact::review::Review::new(
                version.id.clone(),
                morn_kernel::ids::PrincipalId::generate(),
                morn_artifact::review::ReviewDecision::Approve,
                "ok",
            ))
            .unwrap();
        guard
            .artifacts
            .approve(morn_artifact::approval::ArtifactApproval::approve(
                version.id.clone(),
                morn_kernel::ids::PrincipalId::generate(),
                None,
            ))
            .unwrap();
        assert!(guard.artifacts.require_approved(&version.id).is_ok());
        let _ = artifact;

        // Accept work + outcome.
        guard
            .work
            .attempt_accept(
                &wp_id,
                &morn_work::service::AcceptanceEvidence {
                    produced_artifacts: vec!["result".to_string()],
                    verification_passed: true,
                    required_approvals: vec![],
                    forbidden_condition_hit: None,
                },
            )
            .unwrap();
        guard
            .world
            .record_outcome(morn_world::outcome::OutcomeRecord::new(
                ws_id.clone(),
                "generic outcome",
                true,
            ));
        guard.persist_all().unwrap();
    }

    // Phase 2: restart -> history readable.
    let state = AppState::new(&db).unwrap();
    let guard = state.lock();
    assert_eq!(guard.world.objects().len(), 1);
    assert_eq!(guard.work.work_packages().len(), 1);
    assert_eq!(guard.artifacts.all_version_count(), 1);
    assert_eq!(guard.world.outcomes().len(), 1);
    let _ = ws_id;

    // Phase 3: node failover (pure core distributed proof) + connector action.
    let mut rt = DistributedRuntime::new();
    let a = MornNode::new("a", NodeType::Desktop);
    let b = MornNode::new("b", NodeType::Worker);
    let a_id = a.id.clone();
    let b_id = b.id.clone();
    rt.register_node(a);
    rt.register_node(b);
    let run = WorkflowRunId::generate();
    assert!(rt.claim(run.clone(), a_id, 60));
    rt.claims.get_mut(&run).unwrap().lease_until = morn_kernel::time::Timestamp::from_millis(1);
    let (_, applied, already) = rt.failover(&run, b_id, &["e1".into(), "e1".into()], &[]);
    assert_eq!(applied, 1);
    assert_eq!(already, 0);

    // Connector governed action (idempotent).
    let mut conn = morn_integration::GenericFixtureConnector::new();
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
    let _ = conn.execute_governed(&req, &tok(&req)).unwrap();
    let r2 = conn.execute_governed(&req, &tok(&req)).unwrap();
    assert_eq!(conn.applied_external_ids.len(), 1);
    assert_eq!(r2.external_id.as_deref(), Some("dup"));

    // Pack lifecycle (pure core, no domain): a hello-domain pack can be
    // initialized and lifecycle'd without any concrete domain.
    let mut lc = PackLifecycle::new();
    let pid = lc
        .init(PackManifest::new(
            "hello-domain",
            "domain-pack",
            Version::v1(),
        ))
        .unwrap();
    lc.validate(&pid).unwrap();
    lc.build(&pid).unwrap();
    lc.install(&pid).unwrap();
    lc.enable(&pid).unwrap();
    lc.uninstall(&pid).unwrap();
    assert_eq!(lc.inspect(&pid).unwrap().status, PackStatus::Uninstalled);
    assert!(!lc.history.is_empty());
}
