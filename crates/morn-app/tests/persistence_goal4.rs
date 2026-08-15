//! Goal 4 M2: Goal 3 services are repository-backed — restart hydration,
//! workspace isolation, idempotent receipts, API parity.

use morn_app::AppState;
use morn_assurance::certification::{CertificationEvidence, CertificationSpec};
use morn_assurance::evaluation::EvalStep;
use morn_assurance::managed_work::{AcceptanceSource, DeliveryReceipt, ManagedWorkRun, SloConfig};
use morn_assurance::replay::{simple_hash, ReplayScenario};
use morn_kernel::version::Version;
use serde_json::json;

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g4_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

fn certify(state: &AppState) -> String {
    let mut guard = state.lock();
    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("release", true),
    ];
    let evaluation = guard.evaluation.run("cert", "sol-1.0", &steps, &[], &[]);
    let input = "cert-input";
    let out = simple_hash(input);
    let scenario =
        ReplayScenario::new("cert", json!({}), "sol-1.0").record("analyze", input, &out, "ok");
    let replay = guard.replay.run(&scenario, None);
    let spec = CertificationSpec::new("dataset-to-claim", Version::v1());
    let evidence = CertificationEvidence {
        evaluation_results: vec![evaluation],
        replay_reports: vec![replay],
        ..Default::default()
    };
    let run = guard
        .certification
        .start_run(spec.id.clone(), evidence)
        .unwrap();
    let decision = guard.certification.evaluate(&run, &spec).unwrap();
    let cap = guard
        .certification
        .certify(&run.id, &decision, &spec, "pi")
        .unwrap();
    guard.persist_all().unwrap();
    cap.id.to_string()
}

fn managed_delivery(state: &AppState) -> String {
    let mut guard = state.lock();
    let cap_id = guard.certification.capabilities[0].id.clone();
    let ws = guard.workspace.id.clone();
    let run = ManagedWorkRun::new(
        ws,
        cap_id,
        "wc-g4",
        "customer",
        "analyst-actor",
        SloConfig::new("reproducibility", "1.0", "independent review"),
    );
    let cap = guard.certification.capabilities[0].clone();
    let started = guard.managed.start(&cap, run).unwrap();
    let run_id = started.id.clone();
    let receipt = DeliveryReceipt {
        id: morn_kernel::ids::DeliveryReceiptId::generate(),
        run_id: run_id.clone(),
        work_contract_ref: "wc-g4".to_string(),
        capability_ref: cap.id.to_string(),
        capability_version: cap.version.to_string(),
        execution_receipt_refs: vec!["r".to_string()],
        artifacts: vec!["a".to_string()],
        decisions: vec![],
        state_diffs: vec![],
        verification: "passed".to_string(),
        outcome: "claim released".to_string(),
        slo_result: "met".to_string(),
        evidence: vec!["execution_receipt".to_string()],
        failures_retries: vec![],
        human_interventions: 1,
        timestamps: vec![],
    };
    guard.managed.deliver(&run_id, receipt).unwrap();
    guard
        .managed
        .decide(
            &run_id,
            "accepted",
            AcceptanceSource::IndependentReviewer,
            "pi",
            "ok",
        )
        .unwrap();
    guard.persist_all().unwrap();
    run_id.to_string()
}

#[test]
fn goal3_services_survive_restart() {
    let db = temp_db("restart");
    let cap_id;
    {
        let state = AppState::new(&db).unwrap();
        cap_id = certify(&state);
        managed_delivery(&state);
        // evolution flywheel seed
        let mut guard = state.lock();
        let ws = guard.workspace.id.clone();
        for i in 0..4 {
            guard
                .flywheel
                .ingest_trace(morn_evolution::flywheel::TraceRecord::new(
                    ws.clone(),
                    format!("run-{i}"),
                    "qc",
                    "succeeded",
                    "ok",
                ));
        }
        guard.flywheel.detect_patterns(3, 10_000);
        guard.flywheel.generate_candidates(&ws);
        guard.persist_all().unwrap();
    }

    // "Process restart": open the same DB.
    let state = AppState::new(&db).unwrap();
    let guard = state.lock();

    assert_eq!(guard.store.schema_version().unwrap(), 2);
    assert_eq!(guard.certification.capabilities.len(), 1);
    assert_eq!(guard.certification.capabilities[0].id.to_string(), cap_id);
    assert_eq!(guard.managed.runs.len(), 1);
    assert_eq!(
        guard.managed.runs[0].status,
        morn_assurance::managed_work::DeliveryStatus::Closed
    );
    assert_eq!(guard.managed.receipts.len(), 1);
    assert_eq!(guard.managed.acceptances.len(), 1);
    assert_eq!(guard.flywheel.candidates.len(), 1);
    assert_eq!(guard.flywheel.patterns.len(), 1);
    assert_eq!(guard.flywheel.traces.len(), 4);
}

#[test]
fn delivery_receipt_is_immutable_across_restart() {
    let db = temp_db("immutable");
    {
        let state = AppState::new(&db).unwrap();
        certify(&state);
        managed_delivery(&state);
    }
    // Re-saving the same receipt id must be rejected (immutable history).
    let state = AppState::new(&db).unwrap();
    let guard = state.lock();
    let receipts = &guard.managed.receipts;
    assert_eq!(receipts.len(), 1);
    // Attempting to save a duplicate immutable receipt errors.
    let duplicate = receipts[0].clone();
    assert!(guard.store.save_delivery_receipt(&duplicate).is_err());
}
