//! HTTP API (axum): the shared backend for Workbench / Studio / Console / Hub.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use morn_assurance::evaluation::EvalStep;
use morn_assurance::simulation::{FaultInjection, FaultKind};
use morn_biolab::dream_factory::LiteratureSource;
use morn_harness::HarnessProvider;
use morn_kernel::error::Error;
use morn_kernel::ids::{ArtifactId, ArtifactVersionId, ScientificClaimId};
use morn_work::durable::{Signal, SignalKind};
use morn_work::workflow::WorkflowStepKind;

use crate::app::AppState;

/// API error envelope.
#[derive(Debug)]
pub struct AppError(Error);

impl From<Error> for AppError {
    fn from(e: Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, msg) = match self.0 {
            Error::NotFound(m) => (StatusCode::NOT_FOUND, m),
            Error::Validation(m) => (StatusCode::BAD_REQUEST, m),
            Error::Forbidden(m) | Error::NotAuthorized(m) => (StatusCode::FORBIDDEN, m),
            Error::Conflict(m) => (StatusCode::CONFLICT, m),
            other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        };
        (code, Json(json!({ "error": msg }))).into_response()
    }
}

type ApiResult = Result<Json<Value>, AppError>;

/// Build the API router shared by all product surfaces.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/workspaces", get(list_workspaces))
        .route("/api/workbench", get(workbench))
        .route("/api/studio", get(studio))
        .route("/api/console", get(console))
        .route("/api/hub", get(hub))
        .route("/api/evolution", get(evolution_center))
        .route("/api/biolab/run", post(run_biolab))
        .route("/api/biolab/result", get(biolab_result))
        .route("/api/compiler/run", post(compiler_run))
        .route("/api/compiler/approve", post(compiler_approve))
        .route("/api/compiler/compile", post(compiler_compile))
        .route("/api/compiler/manifest", get(compiler_manifest))
        .route("/api/durable/start", post(durable_start))
        .route("/api/durable/signal", post(durable_signal))
        .route("/api/durable/runs", get(durable_runs))
        .route("/api/durable/attention", get(durable_attention))
        .route("/api/evaluation/run", post(evaluation_run))
        .route("/api/shadow/compare", post(shadow_compare))
        .route("/api/replay/run", post(replay_run))
        .route("/api/biolab/loop-a", post(biolab_loop_a))
        .route("/api/biolab/loop-c", post(biolab_loop_c))
        .route("/api/biolab/assets", get(biolab_assets))
        .route("/api/hub2", get(hub_v2))
        .route("/api/evolution/analyze", post(evolution_analyze))
        .route("/api/evolution/flywheel", get(evolution_flywheel))
        .route("/api/distill/run", post(distill_run))
        .route("/api/certify/run", post(certify_run))
        .route("/api/certify/list", get(certify_list))
        .route("/api/managed/start", post(managed_start))
        .route("/api/managed/accept", post(managed_accept))
        .route("/api/managed/runs", get(managed_runs))
        .route("/api/replacement/shadow", post(replacement_shadow))
        .route("/api/managed/deliver", post(managed_deliver))
        .route("/api/replacement/r4", post(replacement_r4))
        .route("/api/replacement/records", get(replacement_records))
        .route("/api/hub3", get(hub_v3))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

async fn health() -> ApiResult {
    Ok(Json(json!({ "status": "ok", "service": "morn-app" })))
}

async fn list_workspaces(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let workspaces = guard.store.list_workspaces().map_err(Error::internal)?;
    Ok(Json(json!({ "workspaces": workspaces })))
}

async fn workbench(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let ws = &guard.workspace;
    let world = &guard.biolab.world;
    let work = &guard.biolab.work;
    let durable = &guard.durable;
    let e2e = guard.e2e_result.as_ref();

    let objects: Vec<Value> = world
        .objects()
        .iter()
        .map(|o| json!({ "id": o.id, "type": o.object_type_id, "state": o.state(), "version": o.version() }))
        .collect();
    let work_packages: Vec<Value> = work
        .work_packages()
        .iter()
        .map(|wp| json!({ "id": wp.id, "objective": wp.objective, "status": wp.status, "acceptance_spec_id": wp.acceptance_spec_id }))
        .collect();
    let attention: Vec<Value> = durable
        .open_attention()
        .iter()
        .map(
            |a| json!({ "id": a.id, "kind": a.kind, "subject": a.subject, "priority": a.priority }),
        )
        .collect();
    let outcomes: Vec<Value> = world
        .outcomes()
        .iter()
        .map(
            |o| json!({ "id": o.id, "objective": o.objective, "acceptance_met": o.acceptance_met }),
        )
        .collect();

    Ok(Json(json!({
        "mission": { "id": ws.id, "name": ws.name, "kind": ws.kind, "status": ws.status },
        "world_objects": objects,
        "work_packages": work_packages,
        "artifacts": artifacts_count(&guard),
        "attention": attention,
        "outcomes": outcomes,
        "harness": {
            "native": { "provider": guard.native_harness.provider_name(), "status": "mounted" },
            "dsh": { "provider": guard.dsh_harness.provider_name(), "status": "fixture-mode" }
        },
        "evolution_candidates": guard.evolution.candidates().len(),
        "e2e_result": e2e.map(|r| json!({ "all_ok": r.all_ok(), "steps": r.steps, "claim_id": r.claim_id })),
    })))
}

fn artifacts_count(guard: &crate::app::AppInner) -> Value {
    json!({ "versions": guard.biolab.artifacts.all_version_count() })
}

async fn studio(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let work = &guard.biolab.work;
    let world = &guard.biolab.world;
    let wps: Vec<Value> = work
        .work_packages()
        .iter()
        .map(|wp| {
            json!({
                "id": wp.id,
                "objective": wp.objective,
                "execution_mode": wp.execution_mode.as_ref().map(|m| json!({
                    "nature": m.nature,
                    "executor": m.executor,
                    "rationale": m.rationale
                })),
                "allowed_actions": wp.allowed_actions,
                "prohibited_actions": wp.prohibited_actions,
            })
        })
        .collect();
    let object_types: Vec<Value> = world
        .object_types()
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.name, "allowed_states": t.allowed_states }))
        .collect();
    Ok(Json(json!({
        "work_packages": wps,
        "object_types": object_types,
        "roles": ["analyst", "pipeline", "statistical_reviewer", "pi"],
        "manifest_preview": {
            "morn": { "domain": "biolab", "version": "1.0" },
            "workcontracts": { "dataset_to_claim": { "acceptance": "dataset_to_reviewed_claim" } },
            "roles": { "pi": { "member_type": "human" }, "analyst": { "member_type": "actor" } }
        }
    })))
}

async fn console(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let ws = &guard.workspace;
    let world = &guard.biolab.world;
    let ledger = world.ledger();
    let approvals = &guard.biolab.gateway.approved_roles;
    let outcomes: Vec<Value> = world
        .outcomes()
        .iter()
        .map(|o| json!({ "id": o.id, "objective": o.objective, "acceptance_met": o.acceptance_met, "snapshots": o.state_snapshot_ids }))
        .collect();
    let traces: Vec<Value> = ledger
        .entries()
        .iter()
        .map(|e| json!({ "seq": e.seq, "event_type": e.event_type, "subject": e.subject, "summary": e.summary, "principal": e.principal, "payload_hash": e.payload_hash }))
        .collect();
    let promotion_decisions: Vec<Value> = guard
        .evolution
        .promotions()
        .iter()
        .map(|p| json!({ "id": p.id, "outcome": p.outcome, "previous_version": p.previous_version, "new_version": p.new_version, "rollback_ref": p.rollback_ref }))
        .collect();
    Ok(Json(json!({
        "identity": { "workspace": ws.name, "owner": ws.owner },
        "world_state": world.objects().len(),
        "work": guard.biolab.work.work_packages().len(),
        "harness_health": {
            "native": guard.native_harness.provider_name(),
            "dsh": guard.dsh_harness.provider_name()
        },
        "approvals_satisfied": approvals,
        "attention": guard.durable.open_attention().len(),
        "policy": "biolab-policy@1.0",
        "traces": traces,
        "outcomes": outcomes,
        "evolution_promotions": promotion_decisions,
        "version_rollback": guard.evolution.rollbacks().len(),
    })))
}

async fn hub(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let world = &guard.biolab.world;
    let object_types: Vec<Value> = world
        .object_types()
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.name, "trust": "Verified", "lifecycle": "active" }))
        .collect();
    Ok(Json(json!({
        "domain_packs": [ { "id": "biolab@1.0", "name": "BioLab Domain Pack", "trust": "Verified" } ],
        "actor_templates": [ { "id": "analyst@1.0", "name": "Bioinformatics Analyst", "trust": "CommunityTested" } ],
        "harness_templates": [
            { "id": "morn-native@1.0", "name": "Morn Native Harness", "trust": "Verified" },
            { "id": "deepseek-harness@0.1", "name": "DeepSeek Harness (spike)", "trust": "Unverified" }
        ],
        "work_package_templates": [ { "id": "dataset_to_claim@1.0", "name": "Dataset -> Reviewed Claim", "trust": "Verified" } ],
        "workcell_blueprints": [ { "id": "analysis-workcell@1.0", "name": "Analysis Workcell", "trust": "CommunityTested" } ],
        "evaluation_packs": [ { "id": "biolab-analysis-suite@1.0", "name": "BioLab Analysis Suite", "trust": "Verified" } ],
        "operational_object_types": object_types,
    })))
}

async fn evolution_center(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let candidates: Vec<Value> = guard
        .evolution
        .candidates()
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "candidate_type": c.candidate_type,
                "proposed_change": c.proposed_change,
                "current_version": c.current_version,
                "risk": c.risk,
                "status": c.status
            })
        })
        .collect();
    Ok(Json(json!({
        "candidates": candidates,
        "promotions": guard.evolution.promotions().len(),
        "rollbacks": guard.evolution.rollbacks().len(),
        "promotion_gate": "evaluation passed + policy + required approvals",
    })))
}

async fn run_biolab(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let result = guard.biolab.run_dataset_to_claim_e2e("aging_pilot", 128)?;
    guard.e2e_result = Some(result.clone());
    Ok(Json(json!({ "ok": true, "result": result })))
}

async fn biolab_result(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    match guard.e2e_result.as_ref() {
        Some(r) => Ok(Json(json!({ "ok": true, "result": r }))),
        None => Ok(Json(
            json!({ "ok": false, "detail": "no E2E run yet; POST /api/biolab/run first" }),
        )),
    }
}

// ---- Goal 2 v0.2 endpoints ----

async fn compiler_run(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let goal = body
        .get("goal")
        .and_then(Value::as_str)
        .unwrap_or("Dataset to Reviewed Scientific Claim");
    let domain = body
        .get("domain")
        .and_then(Value::as_str)
        .unwrap_or("biolab");
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();
    let mut request = morn_foundry::problem_spec::SolutionRequest::new(ws, goal, domain);
    if let Some(caps) = body.get("capabilities").and_then(Value::as_array) {
        request.available_capabilities = caps
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(harnesses) = body.get("harnesses").and_then(Value::as_array) {
        request.available_harnesses = harnesses
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(constraints) = body.get("constraints").and_then(Value::as_array) {
        request.constraints = constraints
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    let (problem, graph) = guard.compiler.analyze(&request)?;
    let proposed = guard.compiler.propose(&problem, &graph, &request)?;
    let report = guard.compiler.validate(&proposed, &graph, &request)?;
    guard.last_problem = Some(problem.clone());
    guard.last_proposed = Some(proposed.clone());
    Ok(Json(json!({
        "problem": problem,
        "work_graph": graph,
        "proposed": proposed,
        "validation": report,
        "decision_sources": guard.compiler.decision_sources,
    })))
}

async fn compiler_approve(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let approver = body
        .get("approver")
        .and_then(Value::as_str)
        .unwrap_or("human");
    let mut guard = state.lock();
    let proposed = guard
        .last_proposed
        .as_ref()
        .ok_or_else(|| AppError(morn_kernel::error::Error::validation("run compiler first")))?;
    let approved = guard.compiler.approve(proposed.id.clone(), approver);
    guard.last_approved = Some(approved.clone());
    Ok(Json(json!({ "approved": approved })))
}

async fn compiler_compile(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let approved = guard
        .last_approved
        .as_ref()
        .ok_or_else(|| AppError(morn_kernel::error::Error::validation("approve first")))?;
    let proposed = guard
        .last_proposed
        .as_ref()
        .ok_or_else(|| AppError(morn_kernel::error::Error::validation("run compiler first")))?;
    let problem = guard
        .last_problem
        .as_ref()
        .ok_or_else(|| AppError(morn_kernel::error::Error::validation("run compiler first")))?;
    let pkg = guard.compiler.compile(approved, proposed, problem)?;
    guard.last_package = Some(pkg.clone());
    Ok(Json(json!({ "package": pkg, "manifest": pkg.manifest })))
}

async fn compiler_manifest(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    match guard.last_package.as_ref() {
        Some(pkg) => {
            let exported = guard.manifest.export(pkg).map_err(AppError)?;
            Ok(Json(
                json!({ "manifest": pkg.manifest, "exported": exported }),
            ))
        }
        None => Ok(Json(
            json!({ "manifest": null, "detail": "no compiled package yet" }),
        )),
    }
}

async fn durable_start(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();
    let definition = morn_work::workflow::WorkflowDefinition::new(ws, "dataset-to-claim-durable")
        .add_step(morn_work::workflow::WorkflowStep::new(
            "analyze",
            WorkflowStepKind::Auto,
        ))
        .add_step(morn_work::workflow::WorkflowStep::new(
            "review",
            WorkflowStepKind::SignalWait,
        ))
        .add_step(morn_work::workflow::WorkflowStep::new(
            "release",
            WorkflowStepKind::Auto,
        ));
    let def_id = definition.id.clone();
    guard.durable_v2.register_workflow(definition);
    let run = guard.durable_v2.start_run(&def_id, None)?;
    Ok(Json(json!({ "run": run })))
}

async fn durable_signal(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let run_id = body.get("run_id").and_then(Value::as_str).unwrap_or("");
    let kind = body
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("HumanApproval");
    let identity = body
        .get("identity")
        .and_then(Value::as_str)
        .unwrap_or("pi-1");
    let authority = body
        .get("authority")
        .and_then(Value::as_str)
        .unwrap_or("pi");
    let mut guard = state.lock();
    let kind = match kind {
        "ExternalEvent" => SignalKind::ExternalEvent,
        "ManualResume" => SignalKind::ManualResume,
        "Cancel" => SignalKind::Cancel,
        "DataArrived" => SignalKind::DataArrived,
        "ReviewerResponse" => SignalKind::ReviewerResponse,
        _ => SignalKind::HumanApproval,
    };
    let signal = Signal::new(
        morn_kernel::ids::WorkflowRunId::new(run_id),
        kind,
        "signal",
        identity,
        authority,
    );
    guard.durable_v2.deliver_signal(signal)?;
    Ok(Json(json!({ "ok": true })))
}

async fn durable_runs(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({ "runs": guard.durable_v2.runs() })))
}

async fn durable_attention(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(
        json!({ "attention": guard.durable_v2.open_attention() }),
    ))
}

async fn evaluation_run(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let scenario = body
        .get("scenario")
        .and_then(Value::as_str)
        .unwrap_or("bio-analysis");
    let mut guard = state.lock();
    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("review", true),
        EvalStep::new("approve", false),
        EvalStep::new("release", true),
    ];
    let faults: Vec<FaultInjection> = body
        .get("faults")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let kind = f.get("kind").and_then(Value::as_str)?;
                    let target = f
                        .get("target_step")
                        .and_then(Value::as_str)
                        .unwrap_or("analyze");
                    let kind = match kind {
                        "ToolTimeout" => FaultKind::ToolTimeout,
                        "ToolError" => FaultKind::ToolError,
                        "HarnessCrash" => FaultKind::HarnessCrash,
                        "ModelUnavailable" => FaultKind::ModelUnavailable,
                        "PermissionDenied" => FaultKind::PermissionDenied,
                        "ApprovalMissing" => FaultKind::ApprovalMissing,
                        "UntrustedContext" => FaultKind::UntrustedContext,
                        "RepresentationViolation" => FaultKind::RepresentationViolation,
                        "EvidenceConflict" => FaultKind::EvidenceConflict,
                        "MalformedData" => FaultKind::MalformedData,
                        "BudgetExhausted" => FaultKind::BudgetExhausted,
                        "DeadlineExpired" => FaultKind::DeadlineExpired,
                        _ => FaultKind::ToolError,
                    };
                    Some(FaultInjection::new(kind, target, "injected"))
                })
                .collect()
        })
        .unwrap_or_default();
    let previous: Vec<String> = body
        .get("previous")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let result = guard
        .evaluation
        .run(scenario, "sol-1.1", &steps, &faults, &previous);
    Ok(Json(json!({ "result": result })))
}

async fn shadow_compare(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let mut guard = state.lock();
    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("review", true),
        EvalStep::new("release", true),
    ];
    let baseline = guard
        .evaluation
        .run("baseline", "sol-1.0", &steps, &[], &[]);
    let faults: Vec<FaultInjection> = body
        .get("faults")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let kind = f.get("kind").and_then(Value::as_str)?;
                    let target = f
                        .get("target_step")
                        .and_then(Value::as_str)
                        .unwrap_or("analyze");
                    let kind = match kind {
                        "PermissionDenied" => FaultKind::PermissionDenied,
                        "ApprovalMissing" => FaultKind::ApprovalMissing,
                        "ToolError" => FaultKind::ToolError,
                        _ => FaultKind::ToolError,
                    };
                    Some(FaultInjection::new(kind, target, "injected"))
                })
                .collect()
        })
        .unwrap_or_default();
    let candidate = guard
        .evaluation
        .run("candidate", "sol-1.1", &steps, &faults, &[]);
    let profile_id = morn_kernel::ids::ShadowProfileId::generate();
    let run = guard.shadow.compare(profile_id, baseline, candidate);
    Ok(Json(json!({ "shadow_run": run })))
}

async fn replay_run(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let mutate = body
        .get("mutate_step")
        .and_then(Value::as_str)
        .map(String::from);
    let mut guard = state.lock();
    let input = "input-recorded";
    let out = morn_assurance::replay::simple_hash(input);
    let scenario = morn_assurance::replay::ReplayScenario::new(
        "bio-replay",
        serde_json::json!({"version": 1}),
        "sol-1.0",
    )
    .record("analyze", input, &out, "ok");
    let report = guard.replay.run(&scenario, mutate.as_deref());
    Ok(Json(json!({ "replay_report": report })))
}

async fn biolab_loop_a(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let question = body
        .get("question")
        .and_then(Value::as_str)
        .unwrap_or("Is mechanism X reproducible?");
    let mut guard = state.lock();
    let sources: Vec<LiteratureSource> = body
        .get("sources")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(LiteratureSource {
                        name: s.get("name").and_then(Value::as_str)?.to_string(),
                        source_ref: s.get("source_ref").and_then(Value::as_str)?.to_string(),
                        evidence_type: s
                            .get("evidence_type")
                            .and_then(Value::as_str)
                            .unwrap_or("single_cell")
                            .to_string(),
                        conclusion: s
                            .get("conclusion")
                            .and_then(Value::as_str)
                            .unwrap_or("present")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let result = guard.biolab.run_loop_a(question, &sources)?;
    guard.loop_a_result = Some(result.clone());
    Ok(Json(json!({ "loop_a": result })))
}

async fn biolab_loop_c(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let e2e = guard.e2e_result.as_ref().ok_or_else(|| {
        AppError(morn_kernel::error::Error::validation(
            "run BioLab E2E first",
        ))
    })?;
    let claim = ScientificClaimId::new(e2e.claim_id.clone());
    let artifact = ArtifactId::new(e2e.artifact_id.clone());
    let version = ArtifactVersionId::new(e2e.artifact_version_id.clone());
    let result = guard.biolab.run_loop_c(&claim, &artifact, &version)?;
    guard.loop_c_result = Some(result.clone());
    Ok(Json(json!({ "loop_c": result })))
}

async fn biolab_assets(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(guard.biolab.export_dream_factory_assets()))
}

async fn hub_v2(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let assets = guard.biolab.export_dream_factory_assets();
    Ok(Json(json!({
        "solution_templates": [{ "id": "biolab-solution@1.0", "name": "BioLab Solution", "trust": "Verified", "evaluation": "biolab-analysis-suite@1.0" }],
        "domain_packs": assets.get("domain_pack"),
        "evaluation_packs": [{ "id": "biolab-analysis-suite@1.0", "name": "BioLab Analysis Suite", "scenarios": ["tool_failure", "approval_missing", "evidence_conflict"] }],
        "simulation_scenarios": assets.get("simulation_scenarios"),
        "work_capability_candidates": [
            { "id": "dataset-to-reviewed-claim@1.0", "name": "Dataset -> Reviewed Claim", "status": "candidate", "trust": "CommunityTested" }
        ],
        "role_blueprints": assets.get("role_blueprints"),
        "workflow_templates": [{ "id": "dataset-to-claim-durable@1.0", "name": "Durable Dataset -> Claim", "signals": ["review", "approval"] }],
    })))
}

// ---- Goal 3 endpoints ----

async fn evolution_analyze(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();
    // Seed fixture traces from BioLab workcell steps.
    for i in 0..5 {
        guard
            .flywheel
            .ingest_trace(morn_evolution::flywheel::TraceRecord::new(
                ws.clone(),
                format!("biolab-run-{i}"),
                "qc",
                "succeeded",
                "ok",
            ));
    }
    guard
        .flywheel
        .ingest_trace(morn_evolution::flywheel::TraceRecord::new(
            ws.clone(),
            "biolab-run-3",
            "review",
            "failed",
            "approval missing",
        ));
    guard
        .flywheel
        .ingest_human_correction(morn_evolution::flywheel::HumanCorrection::new(
            ws.clone(),
            "biolab-run-4",
            "review",
            "fix evidence link",
        ));
    let patterns = guard.flywheel.detect_patterns(3, 10_000);
    let candidates = guard.flywheel.generate_candidates(&ws);
    Ok(Json(
        json!({ "patterns": patterns, "candidates": candidates }),
    ))
}

async fn evolution_flywheel(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({
        "patterns": guard.flywheel.patterns,
        "candidates": guard.flywheel.candidates,
        "traces": guard.flywheel.traces.len(),
    })))
}

async fn distill_run(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_evolution::distillation::{
        qc_rule, ActorBaseline, DistillationInput, DistillationOutput, RegressionCase,
    };
    let baseline = ActorBaseline {
        quality: 1.0,
        latency_ms: 1200,
        cost: 4.0,
        human_minutes: 8.0,
    };
    let candidate = guard
        .distillation
        .distill("qc", "qc-rule", qc_rule, baseline)?;
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
    let report = guard.distillation.run_regression(&candidate.id, &cases)?;
    Ok(Json(
        json!({ "candidate": candidate, "regression": report }),
    ))
}

async fn certify_run(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_assurance::certification::{CertificationEvidence, CertificationSpec};
    use morn_assurance::evaluation::EvalStep;
    use morn_assurance::replay::{simple_hash, ReplayScenario};
    use morn_kernel::version::Version;

    let steps = vec![
        EvalStep::new("collect", true),
        EvalStep::new("analyze", true),
        EvalStep::new("review", true),
        EvalStep::new("release", true),
    ];
    let evaluation = guard
        .evaluation
        .run("cert-eval", "sol-1.0", &steps, &[], &[]);
    let input = "cert-input";
    let out = simple_hash(input);
    let scenario = ReplayScenario::new("cert-replay", serde_json::json!({}), "sol-1.0")
        .record("analyze", input, &out, "ok");
    let replay = guard.replay.run(&scenario, None);
    let evidence = CertificationEvidence {
        evaluation_results: vec![evaluation],
        replay_reports: vec![replay],
        shadow_runs: guard.shadow.runs.clone(),
        known_failure_cases: vec![],
        historical_case_refs: vec![],
    };
    let spec = CertificationSpec::new("dataset-to-reviewed-claim", Version::v1());
    let run = guard.certification.start_run(spec.id.clone(), evidence)?;
    let decision = guard.certification.evaluate(&run, &spec)?;
    let capability = guard
        .certification
        .certify(&run.id, &decision, &spec, "pi")?;
    Ok(Json(
        json!({ "capability": capability, "decision": decision }),
    ))
}

async fn certify_list(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({
        "capabilities": guard.certification.capabilities,
        "decisions": guard.certification.decisions,
        "releases": guard.certification.releases,
    })))
}

async fn managed_start(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_assurance::managed_work::{ManagedWorkRun, SloConfig};
    let cap = guard
        .certification
        .capabilities
        .first()
        .cloned()
        .ok_or_else(|| {
            AppError(morn_kernel::error::Error::validation(
                "run /api/certify/run first",
            ))
        })?;
    let ws = guard.workspace.id.clone();
    let run = ManagedWorkRun::new(
        ws,
        cap.id.clone(),
        "contract-dataset-to-claim",
        "customer",
        "analyst-actor",
        SloConfig::new("reproducibility", "1.0", "independent review"),
    );
    let started = guard.managed.start(&cap, run)?;
    Ok(Json(json!({ "run": started })))
}

async fn managed_deliver(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let run_id = body.get("run_id").and_then(Value::as_str).unwrap_or("");
    let mut guard = state.lock();
    use morn_assurance::managed_work::DeliveryReceipt;
    let cap = guard
        .certification
        .capabilities
        .first()
        .cloned()
        .ok_or_else(|| {
            AppError(morn_kernel::error::Error::validation(
                "no certified capability",
            ))
        })?;
    let receipt = DeliveryReceipt {
        id: morn_kernel::ids::DeliveryReceiptId::generate(),
        run_id: morn_kernel::ids::ManagedWorkRunId::new(run_id),
        work_contract_ref: "contract-dataset-to-claim".to_string(),
        capability_ref: cap.id.to_string(),
        capability_version: cap.version.to_string(),
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
    let delivered = guard
        .managed
        .deliver(&morn_kernel::ids::ManagedWorkRunId::new(run_id), receipt)?;
    Ok(Json(json!({ "receipt": delivered })))
}

async fn managed_accept(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let run_id = body.get("run_id").and_then(Value::as_str).unwrap_or("");
    let decided_by = body
        .get("decided_by")
        .and_then(Value::as_str)
        .unwrap_or("pi");
    let mut guard = state.lock();
    let acc = guard.managed.decide(
        &morn_kernel::ids::ManagedWorkRunId::new(run_id),
        "accepted",
        morn_assurance::managed_work::AcceptanceSource::IndependentReviewer,
        decided_by,
        "independent acceptance",
    )?;
    Ok(Json(json!({ "acceptance": acc })))
}

async fn managed_runs(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(
        json!({ "runs": guard.managed.runs, "receipts": guard.managed.receipts.len() }),
    ))
}

async fn replacement_shadow(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_assurance::replacement::WorkVariantMetrics;
    let mut baseline = WorkVariantMetrics::new("manual");
    baseline.quality = 0.9;
    baseline.acceptance_rate = 0.85;
    baseline.human_minutes = 120.0;
    baseline.cycle_time_hours = 48.0;
    baseline.cost_estimate = 200.0;
    baseline.evidence_coverage = 0.6;
    let mut candidate = WorkVariantMetrics::new("morn-native");
    candidate.quality = 0.95;
    candidate.acceptance_rate = 0.95;
    candidate.human_minutes = 15.0;
    candidate.cycle_time_hours = 4.0;
    candidate.cost_estimate = 40.0;
    candidate.evidence_coverage = 1.0;
    let comparison = guard
        .replacement
        .shadow_compare("biolab-input-set-1", baseline, candidate);
    Ok(Json(json!({ "comparison": comparison })))
}

async fn replacement_r4(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let human = body
        .get("human_approved")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut guard = state.lock();
    let comparison = guard
        .replacement
        .comparisons
        .last()
        .cloned()
        .ok_or_else(|| {
            AppError(morn_kernel::error::Error::validation(
                "run /api/replacement/shadow first",
            ))
        })?;
    let r4 =
        guard
            .replacement
            .decide_r4("dataset-to-reviewed-claim", &comparison, true, true, human)?;
    Ok(Json(json!({ "r4": r4 })))
}

async fn replacement_records(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(
        json!({ "records": guard.replacement.records, "candidates": guard.replacement.candidates }),
    ))
}

async fn hub_v3(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({
        "certified_capabilities": guard.certification.capabilities,
        "capability_releases": guard.certification.releases,
        "certification_evidence": guard.certification.decisions.len(),
        "replacement_records": guard.replacement.records,
    })))
}
