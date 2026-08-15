#![cfg(feature = "domain-biolab")]
//! Goal 3 E2E: Goal2 Solution -> repeated runs -> trace -> Evolution Candidate ->
//! Distillation -> Replay/Evaluation -> Shadow -> Certification ->
//! Certified Work Capability -> Managed Delivery -> DeliveryReceipt ->
//! Human Acceptance -> Baseline Comparison -> Partial Replace Candidate ->
//! outcome feedback.

use morn_app::AppState;
use morn_assurance::certification::{CertificationEvidence, CertificationSpec};
use morn_assurance::evaluation::{EvalStep, EvaluationDecision};
use morn_assurance::managed_work::{AcceptanceSource, DeliveryReceipt, ManagedWorkRun, SloConfig};
use morn_assurance::replacement::WorkVariantMetrics;
use morn_assurance::replay::{simple_hash, ReplayScenario};
use morn_evolution::distillation::{
    qc_rule, ActorBaseline, DistillationInput, DistillationOutput, RegressionCase,
};
use morn_evolution::flywheel::{HumanCorrection, TraceRecord};
use morn_foundry::problem_spec::SolutionRequest;
use morn_kernel::ids::{ManagedWorkRunId, ShadowProfileId};
use serde_json::json;

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g3_e2e_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

#[test]
fn goal3_full_pipeline_e2e() {
    let db = temp_db("g3");
    let state = AppState::new(&db).expect("app state");
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();

    // 1. Goal2 Solution: compile a solution (no gaps, approved).
    let mut request =
        SolutionRequest::new(ws.clone(), "Dataset to Reviewed Scientific Claim", "biolab");
    request.available_capabilities.push("*".to_string());
    request.available_harnesses.push("morn-native".to_string());
    let (problem, graph) = guard.compiler.analyze(&request).expect("analyze");
    let proposed = guard
        .compiler
        .propose(&problem, &graph, &request)
        .expect("propose");
    let validation = guard
        .compiler
        .validate(&proposed, &graph, &request)
        .expect("validate");
    assert!(validation.passed);
    let approved = guard.compiler.approve(proposed.id.clone(), "pi");
    let _pkg = guard
        .compiler
        .compile(&approved, &proposed, &problem)
        .expect("compile");

    // 2. Repeated fixture runs -> trace/outcome collection.
    for i in 0..4 {
        guard
            .biolab
            .run_dataset_to_claim_e2e("aging_pilot", 128)
            .expect("biolab run");
        guard.flywheel.ingest_trace(TraceRecord::new(
            ws.clone(),
            format!("biolab-run-{i}"),
            "qc",
            "succeeded",
            "ok",
        ));
        guard.flywheel.ingest_trace(TraceRecord::new(
            ws.clone(),
            format!("biolab-run-{i}"),
            "analysis",
            "succeeded",
            "ok",
        ));
    }
    guard.flywheel.ingest_human_correction(HumanCorrection::new(
        ws.clone(),
        "biolab-run-2",
        "review",
        "fix evidence link",
    ));

    // 3. Evolution candidate from patterns (deterministic distillation).
    guard.flywheel.detect_patterns(3, 10_000);
    let candidates = guard.flywheel.generate_candidates(&ws);
    assert!(
        candidates
            .iter()
            .any(|c| c.candidate_type == "deterministic_distillation"),
        "repeated success must yield a distillation candidate"
    );

    // 4. Distillation: actor step -> deterministic program with fallback.
    let baseline = ActorBaseline {
        quality: 1.0,
        latency_ms: 1200,
        cost: 4.0,
        human_minutes: 8.0,
    };
    let d_cand = guard
        .distillation
        .distill("qc", "qc-rule", qc_rule, baseline)
        .expect("distill");
    let cases = vec![
        RegressionCase {
            input: DistillationInput {
                rows: 128,
                special: false,
            },
            actor_output: DistillationOutput {
                accepted: true,
                reason: "ok".into(),
            },
            should_fallback: false,
        },
        RegressionCase {
            input: DistillationInput {
                rows: 96,
                special: false,
            },
            actor_output: DistillationOutput {
                accepted: true,
                reason: "ok".into(),
            },
            should_fallback: false,
        },
        RegressionCase {
            input: DistillationInput {
                rows: 33,
                special: false,
            },
            actor_output: DistillationOutput {
                accepted: false,
                reason: "odd".into(),
            },
            should_fallback: false,
        },
        RegressionCase {
            input: DistillationInput {
                rows: 0,
                special: true,
            },
            actor_output: DistillationOutput {
                accepted: false,
                reason: "empty".into(),
            },
            should_fallback: true,
        },
    ];
    let regression = guard
        .distillation
        .run_regression(&d_cand.id, &cases)
        .expect("regression");
    assert!(regression.passed);
    assert_eq!(regression.long_tail_fallback_count, 1);
    let (_, used_program) = guard
        .distillation
        .execute(
            &d_cand.id,
            DistillationInput {
                rows: 64,
                special: false,
            },
        )
        .expect("execute");
    assert!(used_program);

    // 5. Replay + Evaluation (all pass).
    let input = "g3-input";
    let out = simple_hash(input);
    let replay_scenario =
        ReplayScenario::new("g3-replay", json!({}), "sol-1.0").record("analyze", input, &out, "ok");
    let replay = guard.replay.run(&replay_scenario, None);
    assert!(replay.reproduced);

    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("review", true),
        EvalStep::new("release", true),
    ];
    let evaluation = guard.evaluation.run("g3-eval", "sol-1.0", &steps, &[], &[]);
    assert_eq!(evaluation.decision, EvaluationDecision::Pass);

    // 6. Shadow: baseline vs candidate both pass -> Ready.
    let base = guard.evaluation.run("base", "sol-1.0", &steps, &[], &[]);
    let cand = guard.evaluation.run("cand", "sol-1.1", &steps, &[], &[]);
    let shadow = guard
        .shadow
        .compare(ShadowProfileId::generate(), base, cand);
    assert_eq!(
        shadow.comparison.readiness,
        morn_assurance::shadow::Readiness::Ready
    );

    // 7. Certification from evaluation + replay + shadow evidence.
    let spec = CertificationSpec::new(
        "dataset-to-reviewed-claim",
        morn_kernel::version::Version::v1(),
    );
    let evidence = CertificationEvidence {
        evaluation_results: vec![evaluation],
        replay_reports: vec![replay],
        shadow_runs: vec![shadow],
        known_failure_cases: vec![],
        historical_case_refs: vec![],
    };
    let cert_run = guard
        .certification
        .start_run(spec.id.clone(), evidence)
        .expect("cert run");
    let cert_decision = guard
        .certification
        .evaluate(&cert_run, &spec)
        .expect("cert evaluate");
    assert_eq!(
        cert_decision.status,
        morn_assurance::certification::CertificationStatus::Certified
    );
    let capability = guard
        .certification
        .certify(&cert_run.id, &cert_decision, &spec, "pi")
        .expect("certify");
    assert_eq!(
        capability.status,
        morn_assurance::certification::CertificationStatus::Certified
    );

    // 8. Managed Delivery with the certified capability.
    let mw_run = ManagedWorkRun::new(
        ws.clone(),
        capability.id.clone(),
        "contract-dataset-to-claim",
        "customer",
        "analyst-actor",
        SloConfig::new("reproducibility", "1.0", "independent review"),
    );
    let started = guard
        .managed
        .start(&capability, mw_run)
        .expect("managed start");
    assert_eq!(
        started.status,
        morn_assurance::managed_work::DeliveryStatus::Running
    );
    let receipt = DeliveryReceipt {
        id: morn_kernel::ids::DeliveryReceiptId::generate(),
        run_id: started.id.clone(),
        work_contract_ref: "contract-dataset-to-claim".to_string(),
        capability_ref: capability.id.to_string(),
        capability_version: capability.version.to_string(),
        execution_receipt_refs: vec!["rcpt-biolab".to_string()],
        artifacts: vec!["analysis-artifact".to_string()],
        decisions: vec!["pi-approval".to_string()],
        state_diffs: vec!["claim-status-released".to_string()],
        verification: "passed".to_string(),
        outcome: "reviewed scientific claim released".to_string(),
        slo_result: "target met".to_string(),
        evidence: vec![
            "execution_receipt".to_string(),
            "artifact_version".to_string(),
        ],
        failures_retries: vec![],
        human_interventions: 1,
        timestamps: vec!["t0".to_string()],
    };
    guard
        .managed
        .deliver(&started.id, receipt)
        .expect("deliver");
    // Human acceptance independent of executor.
    let acceptance = guard
        .managed
        .decide(
            &started.id,
            "accepted",
            AcceptanceSource::IndependentReviewer,
            "pi",
            "independent acceptance",
        )
        .expect("accept");
    assert_eq!(acceptance.decision, "accepted");
    assert!(guard
        .managed
        .runs
        .iter()
        .all(|r| r.executor != acceptance.decided_by));

    // 9. Replacement baseline comparison -> R4 Partial Replace candidate.
    let mut baseline_m = WorkVariantMetrics::new("manual");
    baseline_m.quality = 0.9;
    baseline_m.acceptance_rate = 0.85;
    baseline_m.human_minutes = 120.0;
    baseline_m.cost_estimate = 200.0;
    baseline_m.evidence_coverage = 0.6;
    let mut candidate_m = WorkVariantMetrics::new("morn-native");
    candidate_m.quality = 0.95;
    candidate_m.acceptance_rate = 0.95;
    candidate_m.human_minutes = 15.0;
    candidate_m.cost_estimate = 40.0;
    candidate_m.evidence_coverage = 1.0;
    let comparison =
        guard
            .replacement
            .shadow_compare("biolab-input-set-1", baseline_m, candidate_m);
    assert!(comparison.candidate_meets_critical);
    assert!(comparison.isolated_side_effects);
    let r4 = guard
        .replacement
        .decide_r4("dataset-to-reviewed-claim", &comparison, true, true, true)
        .expect("r4");
    assert_eq!(r4.status, "approved");
    assert!(r4.rollback_path.contains("existing/manual"));
    assert!(guard.replacement.records.len() == 1);

    // 10. Outcome feedback: the delivery outcome feeds a new trace -> new candidate.
    guard.flywheel.ingest_trace(TraceRecord::new(
        ws,
        "delivery-feedback",
        "qc",
        "succeeded",
        "ok",
    ));
    guard.flywheel.detect_patterns(3, 10_000);
    let fb_ws = guard.workspace.id.clone();
    let feedback_candidates = guard.flywheel.generate_candidates(&fb_ws);
    assert!(
        !feedback_candidates.is_empty(),
        "feedback must keep the flywheel turning"
    );

    // Final: certified capability exists, managed run closed, R4 recorded.
    assert_eq!(guard.certification.capabilities.len(), 1);
    assert_eq!(
        guard.managed.runs[0].status,
        morn_assurance::managed_work::DeliveryStatus::Closed
    );
    assert!(!guard.managed.receipts.is_empty());
    let _ = ManagedWorkRunId::new("x");
}
