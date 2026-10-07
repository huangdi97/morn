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
#[cfg(feature = "domain-biolab")]
use morn_biolab_reference::dream_factory::LiteratureSource;
use morn_harness::HarnessProvider;
use morn_kernel::error::Error;
#[cfg(feature = "domain-biolab")]
use morn_kernel::ids::{ArtifactId, ArtifactVersionId};
use morn_opint::predictor::PredictorTarget;
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
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/v115/status", get(v115_status))
        .route("/api/v115/artifact/openapi/compile", post(v115_compile_openapi))
        .route("/api/workspaces", get(list_workspaces))
        .route("/api/workbench", get(workbench))
        .route("/api/studio", get(studio))
        .route("/api/console", get(console))
        .route("/api/hub", get(hub))
        .route("/api/evolution", get(evolution_center))
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
        .route("/api/rollback/request", post(rollback_request))
        .route("/api/rollback/approve", post(rollback_approve))
        .route("/api/rollback/execute", post(rollback_execute))
        .route("/api/rollback/records", get(rollback_records))
        .route("/api/opint/episode", post(opint_episode))
        .route("/api/opint/episodes", get(opint_episodes))
        .route("/api/opint/dataset", post(opint_dataset))
        .route("/api/opint/predictor/train", post(opint_train))
        .route("/api/opint/predict", post(opint_predict))
        .route("/api/opint/actual", post(opint_actual))
        .route("/api/opint/calibrate", post(opint_calibrate))
        .route("/api/opint/registry", get(opint_registry))
        .route("/api/opint/drift", get(opint_drift))
        .route("/api/demo/bootstrap", post(demo_bootstrap));
    #[cfg(feature = "domain-biolab")]
    let app = app
        .route("/api/biolab/run", post(run_biolab))
        .route("/api/biolab/result", get(biolab_result))
        .route("/api/biolab/loop-a", post(biolab_loop_a))
        .route("/api/biolab/loop-c", post(biolab_loop_c))
        .route("/api/biolab/assets", get(biolab_assets));
    app.with_state(state).layer(CorsLayer::permissive())
}

async fn health() -> ApiResult {
    Ok(Json(json!({ "status": "ok", "service": "morn-app" })))
}

async fn v115_status() -> ApiResult {
    let protocol = morn_kernel::protocol::ProtocolSnapshot::v11_5();
    let profile = morn_profile::DomainProfile::factory_readonly_v1();
    let required_guarantees: Vec<String> = profile
        .requirements
        .iter()
        .filter(|item| item.level == morn_profile::RequirementLevel::Required)
        .map(|item| item.semantic.clone())
        .collect();

    Ok(Json(json!({
        "architecture": {
            "definition": "protocol-driven outcome-oriented work control plane",
            "protocol_version": protocol.protocol_version,
            "semantic_slots": protocol.semantic_slots,
            "control_model": "desired/observed + controllers + reconciliation",
            "composition_runtime": {
                "name": "Cordis",
                "role": "node-local composition runtime",
                "reference_version": "4.0.4",
                "business_truth": false
            }
        },
        "providers": {
            "harness": [
                { "id": "morn-native", "status": "reference" },
                { "id": "deepseek-harness", "status": "fixture-contract; real external" },
                { "id": "pi", "status": "fixture-contract; real transport not configured" }
            ],
            "execution_environment": ["process", "container", "microvm", "full-vm", "remote", "physical"],
            "authority": "provider-neutral; native policy reference, OPA/Cedar/customer IAM compatible by contract"
        },
        "capability_supply_chain": {
            "stages": ["Declared", "Observed", "Qualified", "Admitted", "Suspended", "Retired"],
            "artifact_compilers": ["OpenAPI2Capability"],
            "qualification_is_not_admission": true
        },
        "factory_profile": {
            "id": profile.id,
            "version": profile.version,
            "minimum_isolation": profile.minimum_isolation,
            "required_guarantees": required_guarantees,
            "production_write": false,
            "first_wedge": "outage/insert-order -> capacity -> delivery-impact review"
        },
        "claims": {
            "local_engineering": "reference implementation + fixture/conformance tests",
            "real_dsh": "external-blocked until real DSH distribution/configuration is available",
            "real_factory": "external-blocked until lawful site data/authority exists",
            "production_write": "not entered"
        }
    })))
}

async fn v115_compile_openapi(Json(body): Json<Value>) -> ApiResult {
    use morn_foundry::{
        ArtifactCompiler, ArtifactKind, ArtifactSource, OpenApiJsonCompiler,
    };

    let name = body
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("openapi-capability");
    let source_ref = body
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("inline://openapi");
    let source_digest = body
        .get("source_digest")
        .and_then(Value::as_str)
        .map(str::to_string);
    let content = body
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("content must contain OpenAPI JSON")))?;

    let source = ArtifactSource {
        kind: ArtifactKind::OpenApi,
        name: name.to_string(),
        source_ref: source_ref.to_string(),
        source_digest,
        content: content.to_string(),
    };
    let candidate = OpenApiJsonCompiler.compile(&source)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["evaluate", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn list_workspaces(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let workspaces = guard.store.list_workspaces().map_err(Error::internal)?;
    Ok(Json(json!({ "workspaces": workspaces })))
}

async fn workbench(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let ws = &guard.workspace;
    let world = &guard.world;
    let work = &guard.work;
    let durable = &guard.durable;
    #[cfg(feature = "domain-biolab")]
    let e2e = guard.e2e_result.as_ref();
    #[cfg(feature = "domain-biolab")]
    let e2e_value: Option<Value> =
        e2e.map(|r| json!({ "all_ok": r.all_ok(), "steps": r.steps, "claim_id": r.claim_id }));
    #[cfg(not(feature = "domain-biolab"))]
    let e2e_value: Option<Value> = None;
    // Enabled domain packs advertised to the UI as an extension point.
    #[cfg(feature = "domain-biolab")]
    let domain_packs: Vec<&str> = vec!["biolab-reference"];
    #[cfg(not(feature = "domain-biolab"))]
    let domain_packs: Vec<&str> = vec![];

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
        "domain_packs": domain_packs,
        "e2e_result": e2e_value,
    })))
}

fn artifacts_count(guard: &crate::app::AppInner) -> Value {
    json!({ "versions": guard.artifacts.all_version_count() })
}

async fn studio(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let work = &guard.work;
    let world = &guard.world;
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
        "roles": ["analyst", "reviewer", "approver"],
        "manifest_preview": {
            "morn": { "domain": "generic", "version": "1.0" },
            "workcontracts": { "generic_work": { "acceptance": "generic_acceptance" } },
            "roles": { "approver": { "member_type": "human" }, "analyst": { "member_type": "actor" } }
        }
    })))
}

async fn console(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let ws = &guard.workspace;
    let world = &guard.world;
    let ledger = world.ledger();
    let approvals: Vec<String> = guard
        .managed
        .acceptances
        .iter()
        .map(|a| a.decided_by.clone())
        .collect();
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
        "work": guard.work.work_packages().len(),
        "harness_health": {
            "native": guard.native_harness.provider_name(),
            "dsh": guard.dsh_harness.provider_name()
        },
        "approvals_satisfied": approvals,
        "attention": guard.durable.open_attention().len(),
        "policy": "morn-core-policy@1.0",
        "traces": traces,
        "outcomes": outcomes,
        "evolution_promotions": promotion_decisions,
        "version_rollback": guard.evolution.rollbacks().len(),
    })))
}

async fn hub(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let world = &guard.world;
    // Enabled domain packs advertised to the UI as an extension point (same
    // signal as /api/workbench, D-028): zero-domain builds advertise none.
    #[cfg(feature = "domain-biolab")]
    let domain_packs: Vec<&str> = vec!["biolab-reference"];
    #[cfg(not(feature = "domain-biolab"))]
    let domain_packs: Vec<&str> = vec![];
    let object_types: Vec<Value> = world
        .object_types()
        .iter()
        .map(|t| json!({ "id": t.id, "name": t.name, "trust": "Verified", "lifecycle": "active" }))
        .collect();
    Ok(Json(json!({
        "domain_packs": domain_packs,
        "actor_templates": [ { "id": "generic-actor@1.0", "name": "Generic Actor", "trust": "Verified" } ],
        "harness_templates": [
            { "id": "morn-native@1.0", "name": "Morn Native Harness", "trust": "Verified" },
            { "id": "deepseek-harness@0.1", "name": "DeepSeek Harness (spike)", "trust": "Unverified" }
        ],
        "work_package_templates": [ { "id": "generic-work@1.0", "name": "Generic Work Package", "trust": "Verified" } ],
        "workcell_blueprints": [ { "id": "generic-workcell@1.0", "name": "Generic Workcell", "trust": "Verified" } ],
        "evaluation_packs": [ { "id": "morn-eval-suite@1.0", "name": "Morn Evaluation Suite", "trust": "Verified" } ],
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

#[cfg(feature = "domain-biolab")]
async fn run_biolab(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let result = guard.biolab.run_dataset_to_claim_e2e("aging_pilot", 128)?;
    guard.e2e_result = Some(result.clone());
    Ok(Json(json!({ "ok": true, "result": result })))
}

#[cfg(feature = "domain-biolab")]
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
        .unwrap_or("Deliver a reviewed report");
    let domain = body
        .get("domain")
        .and_then(Value::as_str)
        .unwrap_or("generic");
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

#[cfg(feature = "domain-biolab")]
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

#[cfg(feature = "domain-biolab")]
async fn biolab_loop_c(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let e2e = guard.e2e_result.as_ref().ok_or_else(|| {
        AppError(morn_kernel::error::Error::validation(
            "run BioLab E2E first",
        ))
    })?;
    let claim = morn_biolab_reference::ids::ScientificClaimId::new(e2e.claim_id.clone());
    let artifact = ArtifactId::new(e2e.artifact_id.clone());
    let version = ArtifactVersionId::new(e2e.artifact_version_id.clone());
    let result = guard.biolab.run_loop_c(&claim, &artifact, &version)?;
    guard.loop_c_result = Some(result.clone());
    Ok(Json(json!({ "loop_c": result })))
}

#[cfg(feature = "domain-biolab")]
async fn biolab_assets(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(guard.biolab.export_dream_factory_assets()))
}

async fn hub_v2(State(state): State<AppState>) -> ApiResult {
    let _guard = state.lock();
    Ok(Json(json!({
        "solution_templates": [{ "id": "morn-solution@1.0", "name": "Morn Solution", "trust": "Verified", "evaluation": "morn-eval-suite@1.0" }],
        "domain_packs": [],
        "evaluation_packs": [{ "id": "morn-eval-suite@1.0", "name": "Morn Evaluation Suite", "scenarios": ["tool_failure", "approval_missing", "evidence_conflict"] }],
        "simulation_scenarios": ["tool_failure", "approval_missing", "evidence_conflict"],
        "work_capability_candidates": [
            { "id": "generic-work@1.0", "name": "Generic Work", "status": "candidate", "trust": "CommunityTested" }
        ],
        "role_blueprints": ["analyst", "reviewer", "approver"],
        "workflow_templates": [{ "id": "generic-durable-workflow@1.0", "name": "Generic Durable Workflow", "signals": ["review", "approval"] }],
    })))
}

/// Generic (zero-domain) demo bootstrap: seeds a generic object, work package
/// and artifact through the canonical services so the surfaces render real
/// records without any domain pack.
async fn demo_bootstrap(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();
    let obj_type = morn_world::object::ObjectType::new(
        "generic.Object",
        ws.clone(),
        vec!["status".to_string()],
        vec!["created".to_string(), "done".to_string()],
        vec![],
    );
    guard.world.register_object_type(obj_type.clone());
    let obj = morn_world::object::Object::new(
        morn_kernel::ids::ObjectId::generate_with("obj"),
        obj_type.id.clone(),
        ws.clone(),
        {
            let mut s = std::collections::BTreeMap::new();
            s.insert("status".to_string(), serde_json::json!("created"));
            s
        },
    );
    guard.world.register_object(obj);

    let owner = morn_kernel::ids::PrincipalId::generate_with("owner");
    let wp = morn_work::work_package::WorkPackage::new(ws.clone(), "Generic work package", owner);
    let _wp_id = wp.id.clone();
    guard.work.add_work_package(wp);

    let (artifact, version) = guard.artifacts.create(
        "generic.artifact",
        "morn/generic@1",
        ws.clone(),
        morn_kernel::ids::PrincipalId::generate_with("author"),
        "ref://generic",
        serde_json::json!({ "kind": "generic" }),
        "generic-checksum",
    )?;
    let _ = version;
    guard.persist_all()?;
    Ok(Json(json!({
        "objects": guard.world.objects().len(),
        "work_packages": guard.work.work_packages().len(),
        "artifact": artifact.id,
    })))
} // ---- Goal 3 endpoints ----

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
    guard.persist_all()?;
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
    guard.persist_all()?;
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
    guard.persist_all()?;
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

async fn rollback_request(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let target = body
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or("capability_release");
    let current = body
        .get("current")
        .and_then(Value::as_str)
        .unwrap_or("1.1.0");
    let previous = body
        .get("previous")
        .and_then(Value::as_str)
        .unwrap_or("1.0.0");
    let mut guard = state.lock();
    let ws = guard.workspace.id.clone();
    let req = guard
        .rollback
        .request(morn_assurance::rollback::RollbackRequest::new(
            ws,
            target,
            "dataset-to-claim",
            current
                .parse()
                .unwrap_or(morn_kernel::version::Version::new(1, 1, 0)),
            previous
                .parse()
                .unwrap_or(morn_kernel::version::Version::v1()),
            "operator",
            "rollback after regression",
        ))?;
    guard.persist_all()?;
    Ok(Json(json!({ "request": req })))
}

async fn rollback_approve(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let req_id = body.get("request_id").and_then(Value::as_str).unwrap_or("");
    let approver = body.get("approver").and_then(Value::as_str).unwrap_or("pi");
    let mut guard = state.lock();
    guard
        .rollback
        .approve(&morn_kernel::ids::RollbackRequestId::new(req_id), approver)?;
    guard.persist_all()?;
    Ok(Json(json!({ "ok": true })))
}

async fn rollback_execute(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let req_id = body.get("request_id").and_then(Value::as_str).unwrap_or("");
    let mut guard = state.lock();
    let releases = vec![
        morn_kernel::version::Version::v1(),
        morn_kernel::version::Version::new(1, 1, 0),
    ];
    let receipt = guard
        .rollback
        .execute(&morn_kernel::ids::RollbackRequestId::new(req_id), &releases)?;
    guard.persist_all()?;
    Ok(Json(json!({ "receipt": receipt })))
}

async fn rollback_records(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(
        json!({ "requests": guard.rollback.requests, "receipts": guard.rollback.receipts }),
    ))
}

// ---- Goal 4 opint endpoints ----

async fn opint_episode(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_assurance::managed_work::DeliveryStatus;
    use morn_opint::episode::OperationalEpisode;
    let ws = guard.workspace.id.clone();
    let mut episode = OperationalEpisode::new(ws, "biolab", "dataset-to-claim");
    if let Some(receipt) = guard.managed.receipts.last() {
        episode.evidence.artifacts = receipt.artifacts.clone();
        episode.outcome.delivery_receipt_ref = Some(receipt.id.to_string());
        episode.outcome.outcome_record_ref = Some(receipt.outcome.clone());
    }
    if let Some(acc) = guard.managed.acceptances.last() {
        episode.outcome.acceptance_decision_ref = Some(acc.id.to_string());
        episode.evidence.decisions.push(acc.decision.clone());
    }
    if let Some(run) = guard.managed.runs.last() {
        episode.work.package_ref = Some(run.work_contract_ref.clone());
        episode
            .execution
            .harness_refs
            .push("morn-native".to_string());
        episode.execution.retries = run.retries;
        episode.execution.approvals.push("pi".to_string());
        episode.economics.human_minutes = run.human_fallbacks as f64 * 30.0;
        episode.economics.latency_ms = 1000;
        episode.economics.cost = run.retries as f64 * 5.0;
        episode.economics.model_usage = 1;
    }
    let terminal = guard
        .managed
        .runs
        .last()
        .map(|r| r.status)
        .unwrap_or(DeliveryStatus::Requested);
    let acc = guard.managed.acceptances.last().cloned();
    let final_ep = guard.episodes.finalize(episode, terminal, acc.as_ref());
    guard.persist_all()?;
    Ok(Json(json!({ "episode": final_ep })))
}

async fn opint_episodes(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({ "episodes": guard.episodes.episodes })))
}

async fn opint_dataset(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    use morn_kernel::version::Version;
    use morn_opint::dataset::DatasetManifest;
    let did = morn_kernel::ids::EpisodeDatasetId::generate_with("ds");
    let manifest = DatasetManifest {
        dataset_id: did.clone(),
        source: "authoritative-morn-records".to_string(),
        license: "internal".to_string(),
        dataset_version: Version::v1(),
        checksum: "fixture".to_string(),
        acquisition_date: "2026-08-15".to_string(),
        subset_rule: "all episodes".to_string(),
        preprocessing: "none".to_string(),
        schema_ref: "morn.feature.v1".to_string(),
        reference_evidence: vec![],
        created_at: morn_kernel::time::Timestamp::now(),
    };
    let episodes = guard.episodes.episodes.clone();
    let refs: Vec<&morn_opint::episode::OperationalEpisode> = episodes.iter().collect();
    let snapshot = guard.dataset.build_snapshot(manifest.clone(), &refs);
    // Leakage checks (clean fixture): no duplicates/future/workspace violations.
    let ft: Vec<(morn_kernel::ids::OperationalEpisodeId, i64)> = episodes
        .iter()
        .map(|e| (e.id.clone(), e.ended_at.map(|t| t.millis()).unwrap_or(0)))
        .collect();
    let ot: Vec<(morn_kernel::ids::OperationalEpisodeId, i64)> = episodes
        .iter()
        .map(|e| (e.id.clone(), e.ended_at.map(|t| t.millis()).unwrap_or(0)))
        .collect();
    let quality = guard.dataset.check_leakage(&did, &refs, &ft, &ot, &[]);
    let split = if episodes.len() >= 2 {
        Some(guard.dataset.temporal_split(&did, episodes, 0.25)?)
    } else {
        None
    };
    guard.persist_all()?;
    Ok(Json(
        json!({ "snapshot": snapshot, "quality": quality, "split": split }),
    ))
}

async fn opint_train(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let target = body
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or("outcome_acceptance");
    let mut guard = state.lock();
    let target = match target {
        "duration" => PredictorTarget::Duration,
        "failure_risk" => PredictorTarget::FailureRisk,
        "cost" => PredictorTarget::Cost,
        "human_intervention" => PredictorTarget::HumanIntervention,
        "transition_risk" => PredictorTarget::TransitionRisk,
        _ => PredictorTarget::OutcomeAcceptance,
    };
    ensure_predictor(&mut guard, target);
    let id = predictor_id(&guard, target);
    let values: Vec<f64> = guard
        .episodes
        .episodes
        .iter()
        .map(|e| match target {
            PredictorTarget::Duration => e
                .ended_at
                .map(|t| (t.millis() - e.started_at.millis()) as f64)
                .unwrap_or(0.0),
            PredictorTarget::FailureRisk => {
                if e.labels.success {
                    0.0
                } else {
                    1.0
                }
            }
            PredictorTarget::Cost => e.economics.cost,
            PredictorTarget::HumanIntervention => e.economics.human_minutes,
            PredictorTarget::OutcomeAcceptance => {
                if e.labels.accepted.unwrap_or(false) {
                    1.0
                } else {
                    0.0
                }
            }
            PredictorTarget::TransitionRisk => {
                if e.labels.escalation {
                    1.0
                } else {
                    0.0
                }
            }
        })
        .collect();
    let successes = values.iter().filter(|v| **v > 0.5).count();
    let result = guard.predictors.train(&id, &values, successes);
    guard.persist_all()?;
    match result {
        Ok(()) => Ok(Json(json!({ "trained": true, "target": target.as_str() }))),
        Err(e) => Ok(Json(
            json!({ "trained": false, "insufficient_data": true, "detail": e.to_string() }),
        )),
    }
}

async fn opint_predict(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let target = body
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or("outcome_acceptance");
    let context = body
        .get("context")
        .and_then(Value::as_str)
        .unwrap_or("generic");
    let mut guard = state.lock();
    let target = match target {
        "duration" => PredictorTarget::Duration,
        "failure_risk" => PredictorTarget::FailureRisk,
        "cost" => PredictorTarget::Cost,
        "human_intervention" => PredictorTarget::HumanIntervention,
        "transition_risk" => PredictorTarget::TransitionRisk,
        _ => PredictorTarget::OutcomeAcceptance,
    };
    ensure_predictor(&mut guard, target);
    let id = predictor_id(&guard, target);
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
        .predict(&id, &features, context, vec!["ep-1".to_string()]);
    match prediction {
        Ok(pred) => {
            guard.persist_all()?;
            Ok(Json(json!({ "prediction": pred })))
        }
        Err(e) => Err(AppError(e)),
    }
}

async fn opint_actual(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let predictor = body
        .get("predictor_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let prediction = body
        .get("prediction_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let actual = body.get("actual").and_then(Value::as_f64).unwrap_or(0.0);
    let mut guard = state.lock();
    guard.predictors.record_actual(
        &morn_kernel::ids::PredictorSpecId::new(predictor),
        &morn_kernel::ids::PredictionId::new(prediction),
        actual,
    )?;
    guard.persist_all()?;
    Ok(Json(json!({ "ok": true })))
}

async fn opint_calibrate(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    let predictor = body
        .get("predictor_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut guard = state.lock();
    let cal = guard
        .predictors
        .calibrate(&morn_kernel::ids::PredictorSpecId::new(predictor))?;
    guard.persist_all()?;
    Ok(Json(json!({ "calibration": cal })))
}

async fn opint_registry(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let predictors: Vec<serde_json::Value> = guard
        .predictors
        .predictors
        .iter()
        .map(|p| {
            json!({
                "id": p.spec.id,
                "name": p.spec.name,
                "target": p.spec.target,
                "status": p.spec.status,
                "n": p.params.n,
                "insufficient_data": p.params.insufficient_data,
                "predictions": p.predictions.len(),
            })
        })
        .collect();
    Ok(Json(json!({
        "predictors": predictors,
        "episodes": guard.episodes.episodes.len(),
        "snapshots": guard.dataset.snapshots.len(),
        "quality_reports": guard.dataset.quality_reports,
        "splits": guard.dataset.splits.len(),
        "rollback_receipts": guard.rollback.receipts.len(),
    })))
}

async fn opint_drift(State(state): State<AppState>) -> ApiResult {
    let mut guard = state.lock();
    let mut drifts = Vec::new();
    let ids: Vec<morn_kernel::ids::PredictorSpecId> = guard
        .predictors
        .predictors
        .iter()
        .map(|p| p.spec.id.clone())
        .collect();
    for id in ids {
        if let Ok(d) = guard.predictors.drift_check(&id) {
            drifts.push(d);
        }
    }
    guard.persist_all()?;
    Ok(Json(json!({ "drift_reports": drifts })))
}

fn ensure_predictor(guard: &mut crate::app::AppInner, target: PredictorTarget) {
    use morn_opint::predictor::PredictorSpec;
    let exists = guard
        .predictors
        .predictors
        .iter()
        .any(|p| p.spec.target == target);
    if !exists {
        guard.predictors.register(PredictorSpec::new(
            format!("{}-predictor", target.as_str()),
            target,
            vec!["biolab".to_string()],
        ));
    }
}

fn predictor_id(
    guard: &crate::app::AppInner,
    target: PredictorTarget,
) -> morn_kernel::ids::PredictorSpecId {
    guard
        .predictors
        .predictors
        .iter()
        .find(|p| p.spec.target == target)
        .map(|p| p.spec.id.clone())
        .unwrap_or_else(morn_kernel::ids::PredictorSpecId::generate)
}
