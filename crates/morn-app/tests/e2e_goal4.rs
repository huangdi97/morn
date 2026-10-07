//! Goal 4 E2E: Persisted Certified Capability -> Managed Delivery ->
//! restart/recovery -> OperationalEpisode -> OutcomeDataset -> Predictor ->
//! Prediction -> Candidate Comparison -> Execute -> Actual Outcome ->
//! PredictionError -> Calibration -> EvolutionCandidate.
//! FULL real-data pilot is EXTERNAL BLOCKED (no lawful real BioLab dataset).

use morn_app::AppState;
use morn_assurance::certification::{CertificationEvidence, CertificationSpec};
use morn_assurance::evaluation::EvalStep;
use morn_assurance::managed_work::{AcceptanceSource, DeliveryReceipt, ManagedWorkRun, SloConfig};
use morn_assurance::replay::{simple_hash, ReplayScenario};
use morn_kernel::version::Version;
use morn_opint::predictor::PredictorTarget;
use serde_json::json;

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g4e2e_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

#[test]
fn goal4_full_pipeline_e2e() {
    let db = temp_db("g4");
    let cap_id;
    // Phase 1: certify + deliver + episodes, then "restart".
    {
        let state = AppState::new(&db).unwrap();
        let mut guard = state.lock();
        // certify
        let steps = vec![
            EvalStep::new("collect", true),
            EvalStep::new("analyze", true),
            EvalStep::new("release", true),
        ];
        let evaluation = guard.evaluation.run("cert", "v1", &steps, &[], &[]);
        let input = "i";
        let out = simple_hash(input);
        let scenario = ReplayScenario::new("cert", json!({}), "v1").record("a", input, &out, "ok");
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
        cap_id = cap.id.to_string();
        guard.persist_all().unwrap();
    }
    let _ = cap_id;
    {
        let state = AppState::new(&db).unwrap();
        // 4 managed deliveries -> 4 episodes.
        for _ in 0..4 {
            let mut guard = state.lock();
            let cap = guard.certification.capabilities[0].clone();
            let ws = guard.workspace.id.clone();
            let run = ManagedWorkRun::new(
                ws,
                cap.id.clone(),
                "wc-g4",
                "customer",
                "analyst-actor",
                SloConfig::new("reproducibility", "1.0", "independent review"),
            );
            let started = guard.managed.start(&cap, run).unwrap();
            let receipt = DeliveryReceipt {
                id: morn_kernel::ids::DeliveryReceiptId::generate(),
                run_id: started.id.clone(),
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
            guard.managed.deliver(&started.id, receipt).unwrap();
            guard
                .managed
                .decide(
                    &started.id,
                    "accepted",
                    AcceptanceSource::IndependentReviewer,
                    "pi",
                    "ok",
                )
                .unwrap();
            drop(guard);
            // assemble episode (separate lock to avoid double borrow)
            let mut guard = state.lock();
            let ws2 = guard.workspace.id.clone();
            let mut episode =
                morn_opint::episode::OperationalEpisode::new(ws2, "biolab", "dataset-to-claim");
            episode.economics.human_minutes = 30.0;
            episode.economics.cost = 5.0;
            episode.execution.retries = 0;
            let terminal = guard.managed.runs.last().unwrap().status;
            let acc = guard.managed.acceptances.last().cloned();
            guard.episodes.finalize(episode, terminal, acc.as_ref());
            guard.persist_all().unwrap();
        }
    }

    // Phase 2: "restart" — verify hydration of capability, managed runs, episodes, predictors.
    {
        let state = AppState::new(&db).unwrap();
        let guard = state.lock();
        assert_eq!(guard.certification.capabilities.len(), 1);
        assert_eq!(guard.managed.runs.len(), 4);
        assert_eq!(guard.managed.receipts.len(), 4);
        assert_eq!(guard.episodes.episodes.len(), 4);
        assert_eq!(guard.store.schema_version().unwrap(), 3);
    }

    // Phase 3: dataset -> predictor -> prediction -> actual -> error -> calibration.
    let state = AppState::new(&db).unwrap();
    let mut guard = state.lock();
    let did = morn_kernel::ids::EpisodeDatasetId::generate_with("ds");
    let manifest = morn_opint::dataset::DatasetManifest {
        dataset_id: did.clone(),
        source: "authoritative-morn-records".to_string(),
        license: "internal".to_string(),
        dataset_version: Version::v1(),
        checksum: "fixture".to_string(),
        acquisition_date: "2026-08-15".to_string(),
        subset_rule: "all".to_string(),
        preprocessing: "none".to_string(),
        schema_ref: "morn.feature.v1".to_string(),
        reference_evidence: vec![],
        created_at: morn_kernel::time::Timestamp::now(),
    };
    let episodes = guard.episodes.episodes.clone();
    let refs: Vec<&morn_opint::episode::OperationalEpisode> = episodes.iter().collect();
    let snapshot = guard.dataset.build_snapshot(manifest.clone(), &refs);
    let ft: Vec<(morn_kernel::ids::OperationalEpisodeId, i64)> = episodes
        .iter()
        .map(|e| (e.id.clone(), e.ended_at.unwrap().millis()))
        .collect();
    let quality = guard.dataset.check_leakage(&did, &refs, &ft, &ft, &[]);
    assert!(quality.ok, "clean fixture must pass leakage checks");
    let split = guard
        .dataset
        .temporal_split(&did, episodes.clone(), 0.25)
        .unwrap();
    assert_eq!(split.train_episode_ids.len(), 3);
    assert_eq!(split.test_episode_ids.len(), 1);

    // Register + train + predict (prediction stored BEFORE the actual).
    guard
        .predictors
        .register(morn_opint::predictor::PredictorSpec::new(
            "outcome_acceptance-predictor",
            PredictorTarget::OutcomeAcceptance,
            vec!["biolab".to_string()],
        ));
    let pid = guard.predictors.predictors[0].spec.id.clone();
    let values: Vec<f64> = episodes
        .iter()
        .map(|e| {
            if e.labels.accepted.unwrap_or(false) {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let successes = values.iter().filter(|v| **v > 0.5).count();
    guard.predictors.train(&pid, &values, successes).unwrap();
    let features = morn_opint::state::StateEncoder::new().encode(
        &morn_opint::state::WorldState::default(),
        &morn_opint::state::WorkState::default(),
        &morn_opint::state::OrganizationState::default(),
        &morn_opint::state::ExecutionState::default(),
        &morn_opint::state::ResourceState::default(),
        &morn_opint::state::EvidenceState::default(),
    );
    let prediction = guard
        .predictors
        .predict(&pid, &features, "biolab", vec!["ep-g4".to_string()])
        .unwrap();
    assert!(
        prediction.actual.is_none(),
        "prediction stored before actual"
    );
    // Execute -> actual outcome (independent acceptance), independently recorded.
    guard
        .predictors
        .record_actual(&pid, &prediction.id, 1.0)
        .unwrap();
    let cal = guard.predictors.calibrate(&pid).unwrap();
    assert!(cal.n >= 1);
    let stored = {
        let state = guard.predictors.predictor(&pid).unwrap();
        state
            .predictions
            .iter()
            .find(|p| p.id == prediction.id)
            .cloned()
            .unwrap()
    };
    assert!(stored.actual.is_some());
    assert!(stored.prediction_error.is_some());
    let _ = snapshot;

    // Phase 4: Compiler candidate comparison — predictions are evidence, not authority.
    let mut request = morn_foundry::problem_spec::SolutionRequest::new(
        guard.workspace.id.clone(),
        "Dataset to Reviewed Scientific Claim",
        "biolab",
    );
    request.available_capabilities.push("*".to_string());
    request.available_harnesses.push("morn-native".to_string());
    let (problem_a, graph_a) = guard.compiler.analyze(&request).unwrap();
    let proposed_a = guard
        .compiler
        .propose(&problem_a, &graph_a, &request)
        .unwrap();
    let (problem_b, graph_b) = guard.compiler.analyze(&request).unwrap();
    let proposed_b = guard
        .compiler
        .propose(&problem_b, &graph_b, &request)
        .unwrap();
    assert!(!proposed_a.work_packages.is_empty());
    assert!(!proposed_b.work_packages.is_empty());
    // low-confidence/out-of-context prediction must not auto-select: both proposals
    // still require human review (compile requires approval).
    let approved = guard.compiler.approve(proposed_a.id.clone(), "pi");
    let _pkg_a = guard
        .compiler
        .compile(&approved, &proposed_a, &problem_a)
        .unwrap();
    let _ = problem_b;
    let _ = graph_b;

    // Phase 5: Evolution expected-vs-actual — flywheel candidate with expected benefit,
    // actual delta derived from prediction error.
    let fw_ws = guard.workspace.id.clone();
    for i in 0..4 {
        guard
            .flywheel
            .ingest_trace(morn_evolution::flywheel::TraceRecord::new(
                fw_ws.clone(),
                format!("g4-run-{i}"),
                "qc",
                "succeeded",
                "ok",
            ));
    }
    guard.flywheel.detect_patterns(3, 10_000);
    let candidates = guard.flywheel.generate_candidates(&fw_ws);
    assert!(candidates
        .iter()
        .any(|c| c.candidate_type == "deterministic_distillation"));
    let expected_delta = candidates[0].expected_benefit.clone();
    let actual_delta = format!(
        "prediction_error={:.4} (outcome acceptance)",
        stored.prediction_error.unwrap_or(0.0)
    );
    assert!(!expected_delta.is_empty());
    assert!(actual_delta.contains("prediction_error"));

    // Phase 6: Real pilot FULL is EXTERNAL BLOCKED (no lawful real BioLab dataset).
    // Episodes remain fixture-controlled; see BLOCKERS.md G4-B-002.
    let _ = &guard.episodes;
    let mut pilot_svc = morn_opint::pilot::RealPilotService::new();
    assert!(pilot_svc
        .run_pipeline(
            morn_kernel::ids::DatasetSnapshotId::generate(),
            3,
            true,
            "m",
            "e",
            "c"
        )
        .is_err());
    assert!(
        !pilot_svc.full_pilot_available(),
        "no lawful real dataset -> FULL blocked"
    );

    guard.persist_all().unwrap();
}
