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

use morn_harness::HarnessProvider;
use morn_kernel::error::Error;

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
