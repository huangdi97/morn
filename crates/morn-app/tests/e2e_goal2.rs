//! Goal 2 full E2E: Goal -> Compiler -> ProposedSolution -> Human Review ->
//! SolutionPackage -> Replay -> Evaluation PASS -> Shadow -> BioLab execution ->
//! Artifact/Decision/Outcome -> Final EvaluationReport.

use morn_app::AppState;
use morn_assurance::evaluation::{EvalStep, EvaluationDecision};
use morn_assurance::replay::{simple_hash, ReplayScenario};
use morn_assurance::shadow::Readiness;
use morn_biolab::dream_factory::LiteratureSource;
use morn_foundry::problem_spec::SolutionRequest;
use morn_kernel::ids::{ArtifactId, ArtifactVersionId, ScientificClaimId, ShadowProfileId};
use serde_json::json;

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g2_e2e_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

#[test]
fn goal2_full_pipeline_e2e() {
    let db = temp_db("pipeline");
    let state = AppState::new(&db).expect("app state");
    let mut guard = state.lock();

    // 1. Goal -> Compiler -> ProposedSolution -> Human Review -> SolutionPackage
    let ws = guard.workspace.id.clone();
    let mut request = SolutionRequest::new(ws, "Dataset to Reviewed Scientific Claim", "biolab");
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
    assert!(validation.passed, "validation must pass for the E2E path");
    let approved = guard.compiler.approve(proposed.id.clone(), "pi");
    let pkg = guard
        .compiler
        .compile(&approved, &proposed, &problem)
        .expect("compile");
    assert!(!pkg
        .manifest
        .get("work_packages")
        .and_then(|v| v.as_array())
        .expect("manifest work_packages")
        .is_empty());

    // 2. Historical Replay (no drift) -> reproduced
    let input = "input-recorded";
    let out = simple_hash(input);
    let scenario = ReplayScenario::new("e2e-replay", json!({"version": 1}), "sol-1.0")
        .record("analyze", input, &out, "ok");
    let replay = guard.replay.run(&scenario, None);
    assert!(replay.reproduced, "replay must reproduce without drift");

    // 3. Evaluation -> PASS
    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("review", true),
        EvalStep::new("approve", false),
        EvalStep::new("release", true),
    ];
    let evaluation = guard.evaluation.run("e2e", "sol-1.0", &steps, &[], &[]);
    assert_eq!(evaluation.decision, EvaluationDecision::Pass);

    // 4. Local Shadow -> Ready (baseline == candidate, both pass)
    let baseline = guard.evaluation.run("base", "sol-1.0", &steps, &[], &[]);
    let candidate = guard.evaluation.run("cand", "sol-1.1", &steps, &[], &[]);
    let shadow = guard
        .shadow
        .compare(ShadowProfileId::generate(), baseline, candidate);
    assert_eq!(shadow.comparison.readiness, Readiness::Ready);
    assert!(shadow.isolated_side_effects, "shadow must be isolated");

    // 5. BioLab Workcell execution -> Artifact/Decision/Outcome
    let e2e = guard
        .biolab
        .run_dataset_to_claim_e2e("aging_pilot", 128)
        .expect("biolab e2e");
    assert!(e2e.all_ok(), "BioLab Loop B must complete: {:?}", e2e.steps);

    // Loop A and Loop C complete as well.
    let loop_a = guard
        .biolab
        .run_loop_a(
            "Is mechanism X reproducible?",
            &[LiteratureSource {
                name: "S1".to_string(),
                source_ref: "doi:1".to_string(),
                evidence_type: "single_cell".to_string(),
                conclusion: "present".to_string(),
            }],
        )
        .expect("loop a");
    assert!(
        loop_a.all_ok && loop_a.pi_approved,
        "Loop A must reach approved design"
    );
    let loop_c = guard
        .biolab
        .run_loop_c(
            &ScientificClaimId::new(e2e.claim_id.clone()),
            &ArtifactId::new(e2e.artifact_id.clone()),
            &ArtifactVersionId::new(e2e.artifact_version_id.clone()),
        )
        .expect("loop c");
    assert!(loop_c.all_ok, "Loop C must reach release candidate");

    // Outcome recorded and linked.
    let outcome = guard
        .biolab
        .world
        .outcome(&morn_kernel::ids::OutcomeRecordId::new(
            e2e.outcome_id.clone(),
        ))
        .expect("outcome record");
    assert!(outcome.acceptance_met);
    assert!(!outcome.state_snapshot_ids.is_empty());
    assert!(!outcome.related_artifacts.is_empty());

    // 6. Final EvaluationReport -> PASS
    let final_report = guard.evaluation.run("final", "sol-1.1", &steps, &[], &[]);
    assert_eq!(final_report.decision, EvaluationDecision::Pass);
    assert!(final_report
        .evidence_refs
        .contains(&"execution_receipt".to_string()));
}
