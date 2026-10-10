//! HTTP API (axum): the shared backend for Workbench / Studio / Console / Hub.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::cors::{AllowOrigin, CorsLayer};

use morn_assurance::evaluation::EvalStep;
use morn_assurance::simulation::{FaultInjection, FaultKind};
use morn_assurance::VerifiedReleaseRequest;
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

fn approved_local_origin(value: &HeaderValue) -> bool {
    matches!(
        value.to_str().ok(),
        Some(
            "http://127.0.0.1:5173"
                | "http://localhost:5173"
                | "tauri://localhost"
                | "http://tauri.localhost"
        )
    )
}

/// The reference server is localhost-only, but browsers can still issue
/// cross-origin POSTs to localhost. Reject browser-originated mutations unless
/// they come from an explicit Morn/Tauri development origin. Requests without
/// Origin (CLI/native provider traffic) remain allowed.
async fn local_origin_guard(req: Request, next: Next) -> Response {
    let mutating = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if mutating {
        if let Some(origin) = req.headers().get(header::ORIGIN) {
            if !approved_local_origin(origin) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "cross-origin mutation rejected by local control-plane origin guard"
                    })),
                )
                    .into_response();
            }
        }
    }
    next.run(req).await
}

/// Build the API router shared by all product surfaces.
pub fn router(state: AppState) -> Router {
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/v115/status", get(v115_status))
        .route("/api/v115/ui/extensions", get(v115_ui_extensions))
        .route("/api/v115/control-plane", get(v115_control_plane))
        .route("/api/v115/work/reconcile", post(v115_work_reconcile))
        .route(
            "/api/v115/work/reconcile-business-evidence",
            post(v115_work_reconcile_business_evidence),
        )
        .route(
            "/api/v115/work/reconcile-executor-outcome",
            post(v115_work_reconcile_executor_outcome),
        )
        .route("/api/v115/work/resolve", post(v115_work_resolve))
        .route(
            "/api/v115/source-of-truth/catalog",
            get(v115_source_of_truth_catalog),
        )
        .route(
            "/api/v115/source-of-truth/observations",
            get(v115_source_observations),
        )
        .route(
            "/api/v115/work/bind-source-of-truth",
            post(v115_work_bind_source_of_truth),
        )
        .route(
            "/api/v115/work/observe-outcome",
            post(v115_work_observe_outcome),
        )
        .route("/api/v115/work/bind-e0", post(v115_work_bind_e0))
        .route(
            "/api/v115/work/bind-attested-e0",
            post(v115_work_bind_attested_e0),
        )
        .route("/api/v115/work/execute-e0", post(v115_work_execute_e0))
        .route(
            "/api/v115/acceptance/reviewers",
            get(v115_acceptance_reviewers),
        )
        .route(
            "/api/v115/work/review-outcome",
            post(v115_work_review_outcome),
        )
        .route("/api/v115/work/assess-value", post(v115_work_assess_value))
        .route("/api/v115/solutions", get(v115_solutions))
        .route("/api/v115/capabilities", get(v115_capabilities))
        .route("/api/v115/discovery", get(v115_discovery))
        .route(
            "/api/v115/capability/observe",
            post(v115_capability_observe),
        )
        .route(
            "/api/v115/capability/qualify",
            post(v115_capability_qualify),
        )
        .route(
            "/api/v115/capability/release",
            post(v115_capability_release),
        )
        .route("/api/v115/capability/admit", post(v115_capability_admit))
        .route(
            "/api/v115/capability/revoke-release",
            post(v115_capability_revoke_release),
        )
        .route("/api/v115/creator/draft", post(v115_creator_draft))
        .route(
            "/api/v115/artifact/openapi/compile",
            post(v115_compile_openapi),
        )
        .route(
            "/api/v115/artifact/procedure/compile",
            post(v115_compile_procedure),
        )
        .route(
            "/api/v115/artifact/repository/compile",
            post(v115_compile_repository),
        )
        .route("/api/v115/artifact/paper/compile", post(v115_compile_paper))
        .route("/api/v115/artifact/model/compile", post(v115_compile_model))
        .route(
            "/api/v115/artifact/workflow/compile",
            post(v115_compile_workflow),
        )
        .route(
            "/api/v115/solution/instantiate",
            post(v115_instantiate_solution),
        )
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
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            HeaderValue::from_static("http://127.0.0.1:5173"),
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("tauri://localhost"),
            HeaderValue::from_static("http://tauri.localhost"),
        ]))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);
    app.with_state(state)
        .layer(middleware::from_fn(local_origin_guard))
        .layer(cors)
}

async fn health() -> ApiResult {
    Ok(Json(json!({ "status": "ok", "service": "morn-app" })))
}

async fn v115_ui_extensions() -> ApiResult {
    use morn_domain_sdk::UiExtensionSpec;
    #[cfg(feature = "domain-biolab")]
    use morn_domain_sdk::{UiActionSpec, UiRenderer, UiSurface};

    #[cfg(feature = "domain-biolab")]
    let mut extensions: Vec<UiExtensionSpec> = vec![UiExtensionSpec {
        id: "biolab-reference-summary".to_string(),
        domain: "biolab-reference".to_string(),
        surface: UiSurface::Workbench,
        slot: "domain-summary".to_string(),
        title: "BioLab Reference".to_string(),
        renderer: UiRenderer::Status,
        data_endpoint: None,
        actions: vec![UiActionSpec {
            id: "run-biolab-e2e".to_string(),
            label: "Run BioLab E2E".to_string(),
            method: "POST".to_string(),
            endpoint: "/api/biolab/run".to_string(),
            authority_semantic: Some("ReferenceFixtureAction".to_string()),
        }],
        required_profile: None,
        priority: 100,
    }];
    #[cfg(not(feature = "domain-biolab"))]
    let mut extensions: Vec<UiExtensionSpec> = Vec::new();

    for extension in &extensions {
        extension
            .validate()
            .map_err(|error| AppError(Error::validation(error)))?;
    }
    extensions.sort_by_key(|extension| extension.priority);

    Ok(Json(json!({
        "extensions": extensions,
        "execution_model": "declarative-slot",
        "arbitrary_remote_js": false,
        "business_truth": false
    })))
}

async fn v115_status(State(state): State<AppState>) -> ApiResult {
    let protocol = morn_kernel::protocol::ProtocolSnapshot::v11_5();
    let profile = morn_profile::DomainProfile::factory_readonly_v1();
    let profile_registry = morn_profile::reference_profile_registry();
    let profiles: Vec<Value> = profile_registry
        .list()
        .into_iter()
        .map(|item| json!({ "id": item.id, "version": item.version }))
        .collect();
    let mut provider_catalog = morn_runtime::reference_provider_catalog();
    let (
        dsh_harness,
        pi_harness,
        execution_environment_attestations,
        build_identity_ref,
        evidence_ledger,
        supply_chain_verifications,
    ) = {
        let guard = state.lock();
        (
            guard.dsh_harness.clone(),
            guard.pi_harness.clone(),
            guard.execution_environments.attestations(),
            guard.build_identity_ref.clone(),
            guard.evidence_ledger.clone(),
            guard.supply_chain_verifications.clone(),
        )
    };
    let (
        dsh_health,
        dsh_mode,
        dsh_environment_ref,
        dsh_profile_configuration_ref,
        dsh_runtime_version,
        dsh_runtime_digest,
        dsh_wire_server_version,
    ) = {
        let provider = dsh_harness.lock().expect("dsh harness poisoned");
        (
            provider.runtime_health().clone(),
            match provider.mode() {
                morn_harness::provider::DshMode::Fixture => "fixture",
                morn_harness::provider::DshMode::Real => "real",
            },
            provider
                .configured_execution_environment_ref()
                .map(str::to_string),
            provider
                .configured_profile_configuration_ref()
                .map(str::to_string),
            provider.configured_runtime_version().map(str::to_string),
            provider.configured_runtime_digest().map(str::to_string),
            provider.wire_server_version().map(str::to_string),
        )
    };
    let (pi_health, pi_mode, pi_environment_ref, pi_runtime_version, pi_runtime_digest) = {
        let provider = pi_harness.lock().expect("pi harness poisoned");
        (
            provider.runtime_health().clone(),
            match provider.mode() {
                morn_harness::PiMode::Fixture => "fixture",
                morn_harness::PiMode::Real => "real",
            },
            provider
                .configured_execution_environment_ref()
                .map(str::to_string),
            provider.configured_runtime_version().map(str::to_string),
            provider.configured_runtime_digest().map(str::to_string),
        )
    };
    let now = morn_kernel::time::Timestamp::now();
    let mut project_runtime_health = |provider_id: &str,
                                      health: &morn_harness::HarnessRuntimeHealth|
     -> Result<(), Error> {
        use morn_harness::HarnessRuntimeHealthState;
        use morn_runtime::provider_registry::ProviderStatus;

        let (status, reason, ttl_ms) = match health.state {
            HarnessRuntimeHealthState::Fixture | HarnessRuntimeHealthState::Unconfigured => {
                return Ok(());
            }
            HarnessRuntimeHealthState::Configured => {
                (ProviderStatus::Registered, health.reason.clone(), None)
            }
            HarnessRuntimeHealthState::Initialized if health.ready_for_new_turn_at(now) => (
                ProviderStatus::Registered,
                health.reason.clone(),
                health.remaining_lease_ms(now),
            ),
            HarnessRuntimeHealthState::Initialized => (
                ProviderStatus::Degraded,
                "runtime initialization lease expired before a settled live turn; fresh preflight is required".to_string(),
                None,
            ),
            HarnessRuntimeHealthState::Healthy if health.selectable_at(now) => (
                ProviderStatus::Healthy,
                health.reason.clone(),
                health.remaining_lease_ms(now),
            ),
            HarnessRuntimeHealthState::Healthy => (
                ProviderStatus::Degraded,
                "live runtime health lease expired; a fresh settled turn is required".to_string(),
                None,
            ),
            HarnessRuntimeHealthState::Degraded => {
                (ProviderStatus::Degraded, health.reason.clone(), None)
            }
            HarnessRuntimeHealthState::Closed => {
                (ProviderStatus::Unavailable, health.reason.clone(), None)
            }
        };
        provider_catalog.observe_status_with_ttl(
            provider_id,
            status,
            reason,
            health.evidence_refs.clone(),
            ttl_ms,
        )?;
        Ok(())
    };
    project_runtime_health("deepseek-harness", &dsh_health)?;
    project_runtime_health("pi", &pi_health)?;
    drop(project_runtime_health);
    let providers = provider_catalog.list();
    let provider_observations = provider_catalog.observations();

    let dsh_readiness = if dsh_mode == "real" && dsh_health.selectable_at(now) {
        morn_assurance::RuntimeReadinessEvidence::healthy(
            dsh_health.evidence_refs.clone(),
            dsh_health.reason.clone(),
        )
    } else {
        morn_assurance::RuntimeReadinessEvidence::unavailable(format!(
            "mode={dsh_mode}; health={:?}; {}",
            dsh_health.state, dsh_health.reason
        ))
    };
    let pi_readiness = if pi_mode == "real" && pi_health.selectable_at(now) {
        morn_assurance::RuntimeReadinessEvidence::healthy(
            pi_health.evidence_refs.clone(),
            pi_health.reason.clone(),
        )
    } else {
        morn_assurance::RuntimeReadinessEvidence::unavailable(format!(
            "mode={pi_mode}; health={:?}; {}",
            pi_health.state, pi_health.reason
        ))
    };
    let release_readiness = morn_assurance::release_readiness_axes_for_build(
        &evidence_ledger,
        dsh_readiness,
        pi_readiness,
        build_identity_ref.as_deref(),
    );

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
            "semantic_invariants": protocol.invariants,
            "control_model": "desired/observed + controllers + reconciliation",
            "composition_runtime": {
                "name": "Cordis",
                "role": "node-local composition runtime",
                "reference_version": "4.0.4",
                "business_truth": false
            }
        },
        "provider_catalog": providers,
        "provider_observations": provider_observations,
        "harness_runtime": {
            "dsh": {
                "mode": dsh_mode,
                "health": dsh_health.state,
                "configured_execution_environment_ref": dsh_environment_ref,
                "profile_configuration_ref": dsh_profile_configuration_ref,
                "runtime_version": dsh_runtime_version,
                "runtime_digest": dsh_runtime_digest,
                "wire_server_version": dsh_wire_server_version
            },
            "pi": {
                "mode": pi_mode,
                "health": pi_health.state,
                "configured_execution_environment_ref": pi_environment_ref,
                "runtime_version": pi_runtime_version,
                "runtime_digest": pi_runtime_digest
            }
        },
        "execution_environment_attestations": execution_environment_attestations.iter().map(|attestation| json!({
            "environment_ref": attestation.environment_ref,
            "provider": attestation.provider,
            "isolation": attestation.isolation,
            "required_guarantees": attestation.attested_spec.required_guarantees,
            "runtime": attestation.attested_spec.runtime,
            "runtime_identities": attestation.runtime_identities,
            "evidence_refs": attestation.evidence_refs,
            "observed_at": attestation.observed_at,
            "valid_until": attestation.valid_until,
            "active": attestation.active_at(now)
        })).collect::<Vec<_>>(),
        "provider_status_semantics": {
            "registered": "configured/known but not selectable until live health evidence exists",
            "healthy": "live evidence-backed and selectable within its feature/effect constraints"
        },
        "providers": {
            "harness": [
                { "id": "morn-native", "status": "reference" },
                {
                    "id": "deepseek-harness",
                    "status": "fixture contract + real SDK stdio wire client; external runtime/model credentials required",
                    "wire_methods": ["initialize", "session/prompt", "shutdown"],
                    "cancel_method": false
                },
                {
                    "id": "pi",
                    "status": "fixture contract + real JSONL RPC subprocess client; external Pi binary/model credentials required",
                    "wire_mode": "pi --mode rpc --no-session",
                    "commands": ["prompt", "get_state", "abort"],
                    "settled_event": "agent_settled"
                }
            ],
            "execution_environment": ["process", "container", "microvm", "full-vm", "remote", "physical"],
            "workload_identity": "provider-neutral; SPIFFE-compatible reference shape; no secret material in identity record",
            "authority": "provider-neutral; native policy reference, OPA/Cedar/customer IAM compatible by contract"
        },
        "capability_supply_chain": {
            "stages": ["Declared", "Observed", "Qualified", "Released", "Admitted", "Suspended", "Retired"],
            "artifact_compilers": ["OpenAPI2Capability", "SOP2ProcedureCapability", "Repo2Capability", "ReviewedPaper2Capability", "Model2Capability", "Workflow2Capability"],
            "qualification_is_not_admission": true,
            "site_admission_requires_verified_signature_and_provenance": true,
            "caller_can_self_assert_verification": false,
            "verification_evidence": supply_chain_verifications.iter().map(|evidence| json!({
                "subject_digest": evidence.subject_digest,
                "verifier_ref": evidence.verifier_ref,
                "signature_verified": evidence.signature_verified,
                "provenance_verified": evidence.provenance_verified,
                "evidence_refs": evidence.evidence_refs
            })).collect::<Vec<_>>()
        },
        "profiles": profiles,
        "factory_profile": {
            "id": profile.id,
            "version": profile.version,
            "minimum_isolation": profile.minimum_isolation,
            "required_execution_guarantees": profile.required_execution_guarantees,
            "required_guarantees": required_guarantees,
            "production_write": false,
            "first_wedge": "outage/insert-order -> capacity -> delivery-impact review"
        },
        "evidence_policy": {
            "classes": ["design-spec", "local-fixture", "ci-conformance", "real-runtime", "real-site", "production-write"],
            "classes_are_categorical": true,
            "no_implicit_promotion": true,
            "claims": evidence_ledger.claims()
        },
        "release_readiness": {
            "build_identity_ref": build_identity_ref,
            "axes": release_readiness,
            "semantics": {
                "aggregate_ready": false,
                "reason": "readiness axes are independent; deployments must choose exact provider/effect scope and cannot infer production authorization from local engineering"
            }
        },
        "claims": {
            "local_engineering": "reference implementation + fixture/conformance tests",
            "real_dsh": "SDK wire adapter plus ACP lifecycle/control are locally implemented; real mode remains E0-only; authenticated runtime/model smoke remains external-blocked",
            "real_pi": "JSONL RPC adapter implemented; real mode is E0-only with scrubbed child environment; installed Pi binary/model/credential smoke remains external-blocked",
            "real_factory": "external-blocked until lawful site data/authority exists",
            "production_write": "not authorized; not entered"
        }
    })))
}

async fn v115_work_resolve(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_capability::{CapabilityResolver, EffectClass, WorkcellRequest};
    use morn_control_plane::{
        capability_resolution_evidence, provenance_condition_evidence,
        workcell_qualification_evidence, CapabilityResolutionDecision, ControlPlaneStore,
        DurableWorkControllerRuntime,
    };
    use morn_foundry::solution_package_ref;
    use morn_kernel::time::Timestamp;
    use morn_work::control::AutonomyPosture;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let now = Timestamp::now();
    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if !work_phase_allows_requirement_resolution(work.status.phase) {
        return Err(AppError(Error::invalid_state(format!(
            "Work in phase {:?} cannot rerun requirement resolution; preserve current execution/outcome reconciliation state or create an explicit new generation",
            work.status.phase
        ))));
    }

    let solution_ref = work.spec.source_solution_ref.as_deref().ok_or_else(|| {
        AppError(Error::invalid_state(
            "automatic capability resolution requires Work.source_solution_ref",
        ))
    })?;
    let package_id = solution_ref
        .strip_prefix("solution://")
        .and_then(|rest| rest.rsplit_once('@').map(|(id, _)| id))
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| {
            AppError(Error::validation(
                "Work.source_solution_ref is not a canonical solution://<id>@<version> reference",
            ))
        })?;
    let package = guard
        .store
        .load_record::<morn_foundry::SolutionPackage>("solution_package_v115", package_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("SolutionPackage {package_id}"))))?;
    if solution_package_ref(&package) != solution_ref {
        return Err(AppError(Error::conflict(
            "persisted SolutionPackage identity/version does not match Work.source_solution_ref",
        )));
    }
    let policy = package.policy_v115().ok_or_else(|| {
        AppError(Error::invalid_state(
            "automatic resolver requires a reviewed morn.solution-package/v11.5 policy",
        ))
    })?;
    if policy.required_capabilities.is_empty() {
        return Err(AppError(Error::validation(
            "SolutionPackage declares no required_capabilities to resolve",
        )));
    }
    let profile =
        morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
            AppError(Error::validation(format!(
                "unsupported Work profile {}",
                work.spec.profile_ref
            )))
        })?;

    let mut unavailable_providers = Vec::new();
    {
        let dsh = guard
            .dsh_harness
            .lock()
            .map_err(|_| AppError(Error::internal("dsh harness lock poisoned")))?;
        if dsh.mode() == morn_harness::provider::DshMode::Real
            && !dsh.runtime_health().selectable_at(now)
        {
            unavailable_providers.push("deepseek-harness".to_string());
        }
    }
    {
        let pi = guard
            .pi_harness
            .lock()
            .map_err(|_| AppError(Error::internal("pi harness lock poisoned")))?;
        if pi.mode() == morn_harness::PiMode::Real && !pi.runtime_health().selectable_at(now) {
            unavailable_providers.push("pi".to_string());
        }
    }

    let strict_admission = profile.requires("CapabilityQualification");
    let maximum_effect = if work.spec.autonomy_posture == AutonomyPosture::Assist
        || profile.forbids("ProductionWrite")
    {
        Some(EffectClass::E0LifecycleReversible)
    } else {
        None
    };
    let request = WorkcellRequest {
        required_provides: policy.required_capabilities.clone(),
        minimum_isolation: morn_kernel::ExecutionClass::parse(&profile.minimum_isolation),
        maximum_effect,
        required_execution_guarantees: profile.required_execution_guarantees.clone(),
        site_ref: strict_admission
            .then(|| work.spec.site_ref.clone())
            .flatten(),
        profile_ref: strict_admission.then(|| work.spec.profile_ref.clone()),
        unavailable_providers,
        ..Default::default()
    };
    let plan = CapabilityResolver.resolve_minimum_workcell(&request, &guard.v115_capabilities);
    if !plan.is_complete() {
        return Ok(Json(json!({
            "resolved": false,
            "plan": plan,
            "work": work,
            "evidence_blockers": ["required capability coverage is incomplete"],
            "note": "No positive CapabilityResolved witness was written."
        })));
    }

    let selected = plan
        .members
        .iter()
        .map(|member| {
            guard
                .v115_capabilities
                .iter()
                .find(|record| record.manifest.id == member.capability.manifest_id)
                .cloned()
                .ok_or_else(|| {
                    AppError(Error::internal(format!(
                        "resolved capability {} disappeared from the canonical catalog",
                        member.capability.manifest_id
                    )))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let decision = CapabilityResolutionDecision::new(&work, plan.clone())?;
    let mut resolution = capability_resolution_evidence(&work, &plan)?;
    resolution.evidence_refs.push(decision.id.to_string());
    resolution.evidence_refs.sort();
    resolution.evidence_refs.dedup();

    let mut additional = Vec::new();
    let mut blockers = Vec::new();
    if profile.requires("CapabilityQualification") {
        match workcell_qualification_evidence(&work, &selected, &guard.v115_admission, now) {
            Ok(evidence) => additional.push(evidence),
            Err(error) => blockers.push(error.to_string()),
        }
    }
    if profile.requires("Provenance") {
        match provenance_condition_evidence(&work, &selected) {
            Ok(evidence) => additional.push(evidence),
            Err(error) => blockers.push(error.to_string()),
        }
    }

    guard.store.save_capability_resolution(&work, &decision)?;
    guard.store.save_condition_evidence(&work, &resolution)?;
    for evidence in &additional {
        guard.store.save_condition_evidence(&work, evidence)?;
    }
    // Readiness evidence is stamped when it is constructed above. Reconcile
    // after persistence with a fresh clock so newly-created evidence is never
    // rejected as future-dated merely because provider/catalog checks ran first.
    let reconcile_at = Timestamp::now();
    let tick = DurableWorkControllerRuntime::new("api-v115-capability-resolver")
        .reconcile_from_evidence(&guard.store, work.id.as_str(), &profile, reconcile_at)?;
    let current = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work.id.as_str())?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {}", work.id))))?;

    Ok(Json(json!({
        "resolved": true,
        "decision": decision,
        "plan": plan,
        "evidence_blockers": blockers,
        "controller_tick": tick,
        "work": current,
        "note": "Resolution is generation-scoped evidence only. It grants neither ExecutionBinding nor authority."
    })))
}

async fn v115_source_of_truth_catalog(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({
        "bindings": guard.source_of_truth_catalog,
        "deployment_owned": true,
        "caller_can_create_authority": false,
        "note": "Catalog entries are loaded at server startup from MORN_SOURCE_OF_TRUTH_BINDINGS_FILE."
    })))
}

async fn v115_work_bind_source_of_truth(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::{
        source_of_truth_condition_evidence, ControlPlaneStore, DurableWorkControllerRuntime,
    };
    use morn_integration::SourceOfTruthBindingId;
    use morn_kernel::time::Timestamp;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let catalog_binding_id = body
        .get("catalog_binding_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("catalog_binding_id is required")))?;

    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase.is_terminal() {
        return Err(AppError(Error::invalid_state(
            "terminal Work cannot attach a new source-of-truth binding without a new generation",
        )));
    }

    let catalog = guard
        .source_of_truth_catalog
        .iter()
        .find(|binding| binding.id.as_str() == catalog_binding_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "deployment source-of-truth binding {catalog_binding_id}"
            )))
        })?;
    catalog.validate()?;
    if catalog.site_ref != work.spec.site_ref {
        return Err(AppError(Error::validation(
            "deployment source-of-truth binding site does not match the Work site",
        )));
    }

    // The deployment catalog entry is reusable configuration. Persist a unique
    // Work-scoped immutable copy so the canonical store never aliases one
    // binding record across unrelated Work resources.
    let mut binding = catalog.clone();
    binding.id = SourceOfTruthBindingId::generate_with("sot-work");
    binding.created_at = Timestamp::now();
    guard.store.save_source_of_truth_binding(&work, &binding)?;

    let mut evidence = source_of_truth_condition_evidence(&work, &binding)?;
    evidence
        .evidence_refs
        .push(format!("deployment-source-binding:{catalog_binding_id}"));
    evidence.evidence_refs.sort();
    evidence.evidence_refs.dedup();
    guard.store.save_condition_evidence(&work, &evidence)?;

    let profile =
        morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
            AppError(Error::validation(format!(
                "unsupported Work profile {}",
                work.spec.profile_ref
            )))
        })?;
    let tick = DurableWorkControllerRuntime::new("api-v115-source-of-truth")
        .reconcile_from_evidence(&guard.store, work.id.as_str(), &profile, Timestamp::now())?;
    let current = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work.id.as_str())?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {}", work.id))))?;

    Ok(Json(json!({
        "binding": binding,
        "catalog_binding_ref": catalog_binding_id,
        "condition_evidence": evidence,
        "controller_tick": tick,
        "work": current,
        "note": "Deployment-owned authority was attached to this Work. No world outcome or acceptance was created."
    })))
}

async fn v115_source_observations(State(state): State<AppState>) -> ApiResult {
    use morn_integration::SourceOfTruthBinding;
    use morn_work::control::WorkResource;

    let guard = state.lock();
    let now = morn_kernel::time::Timestamp::now();
    let mut observations = Vec::new();
    for attestation in &guard.source_observation_attestations {
        if attestation.workspace_id != guard.workspace.id
            || guard
                .store
                .load_record::<Value>(
                    "source_observation_attestation_consumed_v115",
                    &attestation.attestation_id,
                )?
                .is_some()
        {
            continue;
        }

        let Some(work) = guard.store.load_record::<WorkResource>(
            "work_resource_v115",
            attestation.work_package_id.as_str(),
        )?
        else {
            continue;
        };
        let Some(binding) = guard.store.load_record::<SourceOfTruthBinding>(
            "source_of_truth_binding_v115",
            attestation.source_binding_id.as_str(),
        )?
        else {
            continue;
        };
        if work.workspace_id != guard.workspace.id
            || !attestation.active_for(
                &work.workspace_id,
                &work.id,
                work.generation,
                &binding.id,
                &attestation.fact_type,
                now,
            )
            || !binding.authoritative_for(&attestation.fact_type)
            || binding.site_ref != work.spec.site_ref
            || !source_ref_within_binding(&binding.source_ref, &attestation.source_ref)
            || !binding.observation_is_fresh(attestation.observed_at, now)
        {
            continue;
        }
        observations.push(attestation.clone());
    }
    Ok(Json(json!({
        "observations": observations,
        "deployment_attested": true,
        "caller_can_submit_world_facts": false,
        "consumed_attestations_are_listed": false,
        "only_currently_consumable": true
    })))
}

fn source_ref_within_binding(binding_root: &str, source_ref: &str) -> bool {
    let root = binding_root.trim_end_matches('/');
    source_ref == root
        || source_ref.strip_prefix(root).is_some_and(|suffix| {
            suffix.starts_with('/') || suffix.starts_with('#') || suffix.starts_with('?')
        })
}

async fn v115_work_observe_outcome(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::{ControlPlaneStore, WorkProgressController, WorkProgressInputs};
    use morn_integration::{SourceOfTruthBinding, TruthAuthorityKind};
    use morn_world::{ObservedOutcome, OutcomeSourceKind};

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let attestation_id = body
        .get("observation_attestation_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError(Error::validation(
                "observation_attestation_id is required; callers cannot submit world facts",
            ))
        })?;
    for forbidden in [
        "source_binding_id",
        "fact_type",
        "objective",
        "source_ref",
        "observed_facts",
        "evidence_refs",
    ] {
        if body.get(forbidden).is_some() {
            return Err(AppError(Error::validation(format!(
                "{forbidden} is deployment-attested observation data and cannot be asserted by the caller"
            ))));
        }
    }

    let guard = state.lock();
    let now = morn_kernel::time::Timestamp::now();
    let mut work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase.is_terminal() {
        return Err(AppError(Error::invalid_state(
            "terminal Work cannot receive a new observed outcome without a new generation",
        )));
    }

    let attestation = guard
        .source_observation_attestations
        .iter()
        .find(|candidate| candidate.attestation_id == attestation_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_authorized(
                "observation attestation is not present in the deployment trust set",
            ))
        })?;
    if attestation.workspace_id != work.workspace_id || attestation.work_package_id != work.id {
        return Err(AppError(Error::not_authorized(
            "observation attestation does not belong to this exact Work and workspace",
        )));
    }

    let binding = guard
        .store
        .load_record::<SourceOfTruthBinding>(
            "source_of_truth_binding_v115",
            attestation.source_binding_id.as_str(),
        )?
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "SourceOfTruthBinding {}",
                attestation.source_binding_id
            )))
        })?;
    binding.validate()?;
    if !attestation.active_for(
        &work.workspace_id,
        &work.id,
        work.generation,
        &binding.id,
        &attestation.fact_type,
        now,
    ) {
        return Err(AppError(Error::not_authorized(
            "source observation attestation is stale or does not match the Work/source binding",
        )));
    }
    if binding.site_ref != work.spec.site_ref
        || !binding.authoritative_for(&attestation.fact_type)
        || !source_ref_within_binding(&binding.source_ref, &attestation.source_ref)
    {
        return Err(AppError(Error::validation(
            "attested observation falls outside the bound source authority",
        )));
    }
    if !binding.observation_is_fresh(attestation.observed_at, now) {
        return Err(AppError(Error::not_authorized(
            "attested observation violates the source-of-truth freshness SLA",
        )));
    }
    let bound_for_work = guard
        .store
        .load_records_in_workspace::<morn_control_plane::ConditionEvidence>(
            "condition_evidence_v115",
            guard.workspace.id.as_str(),
        )?
        .into_iter()
        .any(|evidence| {
            evidence.active_for(&work, now)
                && evidence.condition_type == "SourceOfTruthBound"
                && evidence.satisfied
                && evidence
                    .evidence_refs
                    .iter()
                    .any(|reference| reference == binding.id.as_str())
        });
    if !bound_for_work {
        return Err(AppError(Error::not_authorized(
            "source-of-truth binding is not attached to this Work generation",
        )));
    }
    let consumed_kind = "source_observation_attestation_consumed_v115";
    if guard
        .store
        .load_record::<Value>(consumed_kind, attestation_id)?
        .is_some()
    {
        return Err(AppError(Error::conflict(
            "source observation attestation was already consumed",
        )));
    }

    let source_kind = match binding.authority_kind {
        TruthAuthorityKind::SystemOfRecord => OutcomeSourceKind::ExternalSystem,
        TruthAuthorityKind::Sensor => OutcomeSourceKind::Sensor,
        TruthAuthorityKind::HumanAuthority => OutcomeSourceKind::HumanObservation,
        TruthAuthorityKind::ValidatedComputation => OutcomeSourceKind::ValidatedComputation,
    };
    let mut outcome = ObservedOutcome::new(
        work.workspace_id.clone(),
        work.id.clone(),
        attestation.objective.clone(),
        source_kind,
        attestation.source_ref.clone(),
        attestation.observed_facts.clone(),
    );
    outcome.pin_work_generation(work.generation)?;
    outcome.pin_source_provenance(
        binding.id.to_string(),
        attestation.attestation_id.clone(),
        attestation.fact_type.clone(),
    )?;
    outcome.evidence_refs = attestation.evidence_refs.clone();
    outcome.observed_at = attestation.observed_at;

    // Consume first. A later storage/controller failure burns the attestation
    // fail-closed instead of making authoritative world evidence replayable.
    guard.store.save_record_immutable(
        consumed_kind,
        attestation_id,
        work.workspace_id.as_str(),
        now.millis(),
        &json!({
            "attestation_id": attestation_id,
            "work_id": work.id,
            "source_binding_id": binding.id,
            "fact_type": attestation.fact_type,
            "outcome_id": outcome.id,
            "consumed_at": now
        }),
    )?;
    guard.store.save_observed_outcome(&work, &outcome)?;

    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            outcome: Some(&outcome),
            ..Default::default()
        },
    );
    if let Err(cas_error) = guard.store.save_work_resource_cas(&mut work) {
        let recovery = morn_control_plane::DurableBusinessEvidenceControllerRuntime::new(
            "morn-app-outcome-projection-recovery",
        )
        .reconcile_from_persisted_business_evidence(&guard.store, work_id, now);
        if recovery.is_err() {
            return Err(AppError(cas_error));
        }
        work = guard
            .store
            .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
            .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    }

    Ok(Json(json!({
        "outcome": outcome,
        "work": work,
        "source_binding": binding.id,
        "observation_attestation_id": attestation_id,
        "business_outcome_source_grounded": true,
        "caller_supplied_world_facts": false,
        "independent_acceptance": false,
        "note": "Deployment-attested authoritative observation advances Work to Delivered, not Accepted. Independent review remains required."
    })))
}

fn value_claim_subject(
    work_id: &morn_kernel::ids::WorkPackageId,
    generation: u64,
    outcome_id: &morn_kernel::ids::OutcomeRecordId,
) -> String {
    format!("value:{}:g{}:{}", work_id, generation, outcome_id)
}

async fn v115_work_assess_value(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_assurance::EvidenceClass;
    use morn_control_plane::ControlPlaneStore;
    use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
    use morn_work::control::{WorkPhase, WorkResource};
    use morn_work::value::{ValueAssessment, ValueEvidenceClass};
    use morn_world::ObservedOutcome;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let outcome_id = body
        .get("outcome_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("outcome_id is required")))?;
    let acceptance_id = body
        .get("acceptance_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("acceptance_id is required")))?;
    let evidence_class = match body
        .get("evidence_class")
        .and_then(Value::as_str)
        .unwrap_or("observed-operational")
    {
        "fixture" => ValueEvidenceClass::Fixture,
        "simulation" => ValueEvidenceClass::Simulation,
        "shadow" => ValueEvidenceClass::Shadow,
        "observed-operational" => ValueEvidenceClass::ObservedOperational,
        "customer-validated" => ValueEvidenceClass::CustomerValidated,
        other => {
            return Err(AppError(Error::validation(format!(
                "unsupported value evidence class {other:?}"
            ))))
        }
    };
    let customer_value_attestation_id = body
        .get("customer_value_attestation_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let caller_evidence_refs: Vec<String> = body
        .get("evidence_refs")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    if evidence_class == ValueEvidenceClass::CustomerValidated {
        if customer_value_attestation_id.is_none() {
            return Err(AppError(Error::validation(
                "CustomerValidated requires customer_value_attestation_id",
            )));
        }
        for forbidden in ["evidence_refs", "baseline_ref", "kpis"] {
            if body.get(forbidden).is_some() {
                return Err(AppError(Error::validation(format!(
                    "{forbidden} is deployment-attested for CustomerValidated and cannot be asserted by the caller"
                ))));
            }
        }
    } else {
        if customer_value_attestation_id.is_some() {
            return Err(AppError(Error::validation(
                "customer_value_attestation_id is only valid for CustomerValidated",
            )));
        }
        if caller_evidence_refs.is_empty() {
            return Err(AppError(Error::validation(
                "value assessment requires explicit evidence references",
            )));
        }
    }

    let guard = state.lock();
    let work = guard
        .store
        .load_record::<WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase != WorkPhase::Accepted {
        return Err(AppError(Error::invalid_state(
            "value assessment requires an independently Accepted Work generation",
        )));
    }
    let outcome = guard
        .store
        .load_record::<ObservedOutcome>("observed_outcome_v115", outcome_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("ObservedOutcome {outcome_id}"))))?;
    if outcome.work_package_id != work.id
        || outcome.workspace_id != work.workspace_id
        || outcome.work_generation != work.generation
        || !outcome.is_source_grounded()
    {
        return Err(AppError(Error::validation(
            "value assessment requires a source-grounded outcome from the exact current Work generation",
        )));
    }
    let acceptance = guard
        .store
        .load_record::<AcceptanceDecision>("acceptance_decision_v115", acceptance_id)?
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "AcceptanceDecision {acceptance_id}"
            )))
        })?;
    if acceptance.work_package_id != work.id
        || acceptance.work_generation != work.generation
        || acceptance.disposition != AcceptanceDisposition::Accept
        || !acceptance.is_final_acceptance()
        || !acceptance.outcome_refs.contains(&outcome.id)
    {
        return Err(AppError(Error::validation(
            "value assessment requires final independent acceptance of the exact outcome",
        )));
    }

    let subject = value_claim_subject(&work.id, work.generation, &outcome.id);
    let mut assessment = ValueAssessment::new(work.id.clone(), outcome.id.clone(), evidence_class);
    assessment.pin_work_generation(work.generation)?;
    assessment.acceptance_ref = Some(acceptance.id.clone());

    let mut real_site_evidence = Vec::new();
    let mut attestation_evidence = Vec::new();

    if evidence_class == ValueEvidenceClass::CustomerValidated {
        let now = morn_kernel::time::Timestamp::now();
        assessment.assessed_at = now;
        let attestation_id = customer_value_attestation_id
            .as_deref()
            .expect("validated customer value attestation id");
        let attestation = guard
            .customer_value_attestations
            .iter()
            .find(|candidate| candidate.attestation_id == attestation_id)
            .filter(|candidate| {
                candidate.active_for(
                    &work.workspace_id,
                    (&work.id, work.generation),
                    &outcome.id,
                    &acceptance.id,
                    now,
                )
            })
            .cloned()
            .ok_or_else(|| {
                AppError(Error::not_authorized(
                    "customer value attestation is missing, expired or does not match the exact Work, Outcome and Acceptance",
                ))
            })?;

        let consumed_kind = "customer_value_attestation_consumed_v115";
        if guard
            .store
            .load_record::<Value>(consumed_kind, attestation_id)?
            .is_some()
        {
            return Err(AppError(Error::conflict(
                "customer value attestation was already consumed",
            )));
        }

        let claims: Vec<_> = guard
            .evidence_ledger
            .proven_for(&subject)
            .into_iter()
            .filter(|claim| claim.class == EvidenceClass::RealSite)
            .collect();
        if claims.is_empty() {
            return Err(AppError(Error::not_authorized(format!(
                "CustomerValidated value requires deployment-owned RealSite evidence for subject {subject}"
            ))));
        }
        for claim in claims {
            real_site_evidence.extend(claim.evidence_refs.iter().cloned());
        }

        assessment.customer_value_attestation_ref = Some(attestation.attestation_id.clone());
        assessment.baseline_ref = attestation.baseline_ref.clone();
        assessment.kpis = attestation
            .kpis
            .iter()
            .map(|metric| (metric.name.clone(), metric.value))
            .collect();
        assessment.evidence_refs = attestation.evidence_refs.clone();
        attestation_evidence = attestation.evidence_refs.clone();
        assessment.evidence_refs.extend(real_site_evidence.clone());
        assessment.evidence_refs.push(format!(
            "customer-value-attestation:{}",
            attestation.attestation_id
        ));
        assessment.evidence_refs.sort();
        assessment.evidence_refs.dedup();

        let attestation_kind = "customer_value_attestation_v115";
        match guard
            .store
            .load_record::<morn_assurance::CustomerValueAttestation>(
                attestation_kind,
                attestation_id,
            )? {
            Some(persisted) if persisted != attestation => {
                return Err(AppError(Error::conflict(
                    "persisted customer value attestation differs from deployment trust input",
                )));
            }
            Some(_) => {}
            None => guard.store.save_record_immutable(
                attestation_kind,
                attestation_id,
                work.workspace_id.as_str(),
                attestation.observed_at.millis(),
                &attestation,
            )?,
        }

        guard.store.save_record_immutable(
            consumed_kind,
            attestation_id,
            work.workspace_id.as_str(),
            now.millis(),
            &json!({
                "attestation_id": attestation.attestation_id,
                "workspace_id": work.workspace_id,
                "work_id": work.id,
                "work_generation": work.generation,
                "outcome_id": outcome.id,
                "acceptance_id": acceptance.id,
                "assessment_id": assessment.id,
                "issuer": attestation.issuer,
                "consumed_at": now
            }),
        )?;
    } else {
        assessment.evidence_refs = caller_evidence_refs;
        assessment.baseline_ref = body
            .get("baseline_ref")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        if let Some(kpis) = body.get("kpis").and_then(Value::as_object) {
            for (name, value) in kpis {
                if name.trim().is_empty() {
                    return Err(AppError(Error::validation("KPI name must not be empty")));
                }
                let value = value
                    .as_f64()
                    .ok_or_else(|| AppError(Error::validation("KPI values must be numeric")))?;
                if !value.is_finite() {
                    return Err(AppError(Error::validation("KPI values must be finite")));
                }
                assessment.kpis.push((name.clone(), value));
            }
        }
    }

    guard.store.save_value_assessment(&work, &assessment)?;

    Ok(Json(json!({
        "value_assessment": assessment,
        "value_subject": subject,
        "customer_value_attestation_id": customer_value_attestation_id,
        "customer_value_attestation_evidence_refs": attestation_evidence,
        "real_site_evidence_refs": real_site_evidence,
        "customer_validated": evidence_class == ValueEvidenceClass::CustomerValidated,
        "note": "CustomerValidated KPI/baseline data comes only from a single-use deployment attestation and also requires current RealSite evidence."
    })))
}

fn execution_spec_for_capability_and_profile(
    capability: &morn_capability::CapabilityRecord,
    profile: &morn_profile::DomainProfile,
) -> Result<morn_runtime::ExecutionEnvironmentSpec, Error> {
    use morn_kernel::ExecutionClass;

    let profile_isolation = ExecutionClass::parse(&profile.minimum_isolation).ok_or_else(|| {
        Error::validation(format!(
            "unsupported profile isolation {}",
            profile.minimum_isolation
        ))
    })?;
    let capability_isolation = capability.manifest.execution.minimum_isolation;
    let minimum_isolation = if capability_isolation.satisfies(profile_isolation) {
        capability_isolation
    } else {
        profile_isolation
    };
    let mut required_guarantees = capability.manifest.execution.required_guarantees.clone();
    required_guarantees.extend(profile.required_execution_guarantees.iter().copied());
    if matches!(
        capability.manifest.provider_ref.as_str(),
        "deepseek-harness" | "pi"
    ) {
        required_guarantees.push(morn_kernel::ExecutionGuarantee::ToolMediation);
    }
    required_guarantees.sort();
    required_guarantees.dedup();

    Ok(morn_runtime::ExecutionEnvironmentSpec {
        minimum_isolation,
        os: capability.manifest.execution.os.clone(),
        runtime: if capability.manifest.execution.runtime_kinds.len() == 1 {
            capability.manifest.execution.runtime_kinds.first().cloned()
        } else {
            None
        },
        required_guarantees,
        cpu_millis: capability.manifest.execution.cpu_millis,
        memory_mb: capability.manifest.execution.memory_mb,
        gpu_count: capability.manifest.execution.gpu_count,
        network_allowlist: capability.manifest.execution.network_allowlist.clone(),
        writable_paths: capability.manifest.execution.writable_paths.clone(),
        secret_refs: capability.manifest.execution.secret_refs.clone(),
        persistence_scope: capability.manifest.execution.persistence_scope.clone(),
        timeout_ms: capability.manifest.execution.timeout_ms,
        side_effect_policy: capability.manifest.execution.side_effect_policy.clone(),
    })
}

fn ensure_live_environment_for_binding(
    binding: &morn_runtime::ExecutionBinding,
    capability: &morn_capability::CapabilityRecord,
    profile: &morn_profile::DomainProfile,
    environments: &morn_runtime::AttestedExecutionEnvironmentProvider,
    now: morn_kernel::time::Timestamp,
) -> Result<(), Error> {
    let environment_ref = binding
        .execution_environment_ref
        .as_deref()
        .filter(|reference| !reference.trim().is_empty())
        .ok_or_else(|| {
            Error::invalid_state("real provider binding has no execution environment")
        })?;
    let attestation = environments.attestation(environment_ref).ok_or_else(|| {
        Error::invalid_state(
            "execution environment attestation no longer exists for the bound real provider",
        )
    })?;
    let requested = execution_spec_for_capability_and_profile(capability, profile)?;
    if !attestation.satisfies(&requested, now) {
        return Err(Error::invalid_state(
            "bound execution environment attestation expired or no longer satisfies the capability/profile contract",
        ));
    }
    if binding.execution_class != Some(attestation.isolation)
        || !binding.execution_guarantees.iter().all(|guarantee| {
            attestation
                .attested_spec
                .required_guarantees
                .contains(guarantee)
        })
    {
        return Err(Error::invalid_state(
            "current execution-environment attestation no longer covers the guarantees pinned by the binding",
        ));
    }
    let digest = binding.provider_digest.as_deref().ok_or_else(|| {
        Error::invalid_state("real provider binding is missing an exact runtime digest")
    })?;
    if !attestation.attests_runtime_identity(
        &binding.provider_ref,
        &binding.provider_version,
        digest,
        now,
    ) {
        return Err(Error::invalid_state(
            "execution environment no longer attests the exact provider runtime identity pinned by the binding",
        ));
    }
    Ok(())
}

async fn v115_work_bind_attested_e0(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_capability::{CapabilityStage, EffectClass};
    use morn_control_plane::{ConditionEvidence, ControlPlaneStore};
    use morn_kernel::time::Timestamp;
    use morn_runtime::{CompositionRuntimeRef, ExecutionBinding, ExecutionManifest};
    use morn_work::control::WorkPhase;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let capability_id = body
        .get("capability_manifest_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("capability_manifest_id is required")))?;
    let environment_ref = body
        .get("execution_environment_ref")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("execution_environment_ref is required")))?;

    let now = Timestamp::now();
    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase != WorkPhase::Ready || !work.required_conditions_satisfied() {
        return Err(AppError(Error::invalid_state(
            "attested binding requires a Ready Work with all current-generation conditions satisfied",
        )));
    }

    let capability = guard
        .v115_capabilities
        .iter()
        .find(|record| record.manifest.id.as_str() == capability_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {capability_id}"
            )))
        })?;
    capability.manifest.validate_governance()?;
    if !matches!(
        capability.stage,
        CapabilityStage::Qualified | CapabilityStage::Admitted
    ) {
        return Err(AppError(Error::invalid_state(
            "attested execution binding requires a qualified or admitted capability",
        )));
    }
    if capability.manifest.authority.maximum_effect != EffectClass::E0LifecycleReversible {
        return Err(AppError(Error::not_authorized(
            "bind-attested-e0 only permits an E0 lifecycle-reversible capability",
        )));
    }

    let resolved_evidence = guard
        .store
        .load_records_in_workspace::<ConditionEvidence>(
            "condition_evidence_v115",
            guard.workspace.id.as_str(),
        )?
        .into_iter()
        .any(|evidence| {
            evidence.condition_type == "CapabilityResolved"
                && evidence.satisfied
                && evidence.active_for(&work, now)
                && evidence
                    .evidence_refs
                    .iter()
                    .any(|reference| reference == capability.manifest.id.as_str())
        });
    if !resolved_evidence {
        return Err(AppError(Error::invalid_state(
            "capability is not witnessed by current-generation CapabilityResolved evidence",
        )));
    }

    let profile =
        morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
            AppError(Error::validation(format!(
                "unsupported Work profile {}",
                work.spec.profile_ref
            )))
        })?;
    if profile.requires("CapabilityQualification") {
        let site = work
            .spec
            .site_ref
            .as_deref()
            .ok_or_else(|| AppError(Error::validation("governed Work requires site_ref")))?;
        if !guard.v115_admission.site_profile_admission_active_at(
            &capability,
            site,
            &work.spec.profile_ref,
            now,
        ) {
            return Err(AppError(Error::invalid_state(
                "capability no longer has an active exact site/profile admission",
            )));
        }
    }

    let requested = execution_spec_for_capability_and_profile(&capability, &profile)?;
    let attestation = guard
        .execution_environments
        .attestation(environment_ref)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "trusted execution environment attestation {environment_ref}"
            )))
        })?;
    if !attestation.satisfies(&requested, now) {
        return Err(AppError(Error::invalid_state(
            "trusted execution environment attestation is stale or does not satisfy the capability/profile contract",
        )));
    }
    if !capability.manifest.execution.runtime_kinds.is_empty()
        && attestation
            .attested_spec
            .runtime
            .as_ref()
            .is_none_or(|runtime| {
                !capability
                    .manifest
                    .execution
                    .runtime_kinds
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(runtime))
            })
    {
        return Err(AppError(Error::invalid_state(
            "attested runtime kind is not one of the capability's declared runtime kinds",
        )));
    }

    let provider_ref = capability.manifest.provider_ref.clone();
    let (provider_version, provider_digest, runtime_ref) = match provider_ref.as_str() {
        "deepseek-harness" => {
            let mut provider = guard
                .dsh_harness
                .lock()
                .map_err(|_| AppError(Error::internal("dsh harness lock poisoned")))?;
            if provider.mode() != morn_harness::provider::DshMode::Real {
                return Err(AppError(Error::invalid_state(
                    "bind-attested-e0 is reserved for a real DSH runtime; fixture providers use bind-e0",
                )));
            }
            if provider.configured_execution_environment_ref() != Some(environment_ref) {
                return Err(AppError(Error::invalid_state(
                    "attested environment does not match the environment pinned by the real DSH launch configuration",
                )));
            }
            let version = provider
                .configured_runtime_version()
                .map(str::to_string)
                .ok_or_else(|| {
                    AppError(Error::invalid_state(
                        "real DSH runtime distribution version is not pinned",
                    ))
                })?;
            let digest = provider
                .configured_runtime_digest()
                .map(str::to_string)
                .ok_or_else(|| {
                    AppError(Error::invalid_state(
                        "real DSH runtime distribution digest is not pinned",
                    ))
                })?;
            if !attestation.attests_runtime_identity("deepseek-harness", &version, &digest, now) {
                return Err(AppError(Error::invalid_state(
                    "execution-environment attestation does not authorize the configured DSH runtime artifact",
                )));
            }
            let profile_configuration_ref = provider
                .configured_profile_configuration_ref()
                .ok_or_else(|| {
                    AppError(Error::invalid_state(
                        "real DSH Home/Profile composition identity is not pinned",
                    ))
                })?;
            if !attestation
                .runtime_identities
                .iter()
                .any(|identity| identity == profile_configuration_ref)
            {
                return Err(AppError(Error::invalid_state(
                    "execution-environment attestation does not authorize the configured DSH Home/Profile composition",
                )));
            }
            let initialized = provider.preflight_real_runtime()?;
            if initialized != version {
                return Err(AppError(Error::invalid_state(
                    "initialized DSH runtime identity differs from the pre-authorized deployment version",
                )));
            }
            let route_ref = provider.configured_route_ref().ok_or_else(|| {
                AppError(Error::invalid_state(
                    "real DSH execution route is not pinned",
                ))
            })?;
            (version, Some(digest), route_ref)
        }
        "pi" => {
            let mut provider = guard
                .pi_harness
                .lock()
                .map_err(|_| AppError(Error::internal("pi harness lock poisoned")))?;
            if provider.mode() != morn_harness::pi::PiMode::Real {
                return Err(AppError(Error::invalid_state(
                    "bind-attested-e0 is reserved for a real Pi runtime; fixture providers use bind-e0",
                )));
            }
            if provider.configured_execution_environment_ref() != Some(environment_ref) {
                return Err(AppError(Error::invalid_state(
                    "attested environment does not match the environment pinned by the real Pi launch configuration",
                )));
            }
            let version = provider
                .configured_runtime_version()
                .map(str::to_string)
                .ok_or_else(|| {
                    AppError(Error::invalid_state(
                        "real Pi runtime distribution version is not pinned",
                    ))
                })?;
            let digest = provider
                .configured_runtime_digest()
                .map(str::to_string)
                .ok_or_else(|| {
                    AppError(Error::invalid_state(
                        "real Pi runtime distribution digest is not pinned",
                    ))
                })?;
            if !attestation.attests_runtime_identity("pi", &version, &digest, now) {
                return Err(AppError(Error::invalid_state(
                    "execution-environment attestation does not authorize the configured Pi runtime artifact",
                )));
            }
            let initialized = provider.preflight_real_runtime()?;
            if initialized != version {
                return Err(AppError(Error::invalid_state(
                    "initialized Pi runtime identity differs from the pre-authorized deployment version",
                )));
            }
            let route_ref = provider.configured_route_ref().ok_or_else(|| {
                AppError(Error::invalid_state(
                    "real Pi execution route is not pinned",
                ))
            })?;
            (version, Some(digest), route_ref)
        }
        other => {
            return Err(AppError(Error::validation(format!(
                "provider {other:?} does not use the attested real-harness binding path"
            ))));
        }
    };

    let provider_digest_value = provider_digest.as_deref().ok_or_else(|| {
        AppError(Error::invalid_state(
            "real harness provider must pin an exact runtime digest",
        ))
    })?;
    if !attestation.attests_runtime_identity(
        &provider_ref,
        &provider_version,
        provider_digest_value,
        now,
    ) {
        return Err(AppError(Error::invalid_state(
            "execution-environment attestation does not bind the exact provider runtime artifact",
        )));
    }

    let mut binding = ExecutionBinding::for_work(
        &work,
        capability.manifest.id.to_string(),
        provider_ref,
        provider_version,
    );
    binding.provider_digest = provider_digest;
    binding.runtime_ref = Some(runtime_ref);
    binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
    binding.compensation_ref = capability.manifest.compensation_ref.clone();
    binding.idempotency_key_required = capability.manifest.idempotency_key_required;
    binding.pin_execution_environment(
        attestation.environment_ref.clone(),
        attestation.isolation,
        attestation.attested_spec.required_guarantees.clone(),
    )?;
    if !binding.environment_satisfies(requested.minimum_isolation, &requested.required_guarantees) {
        return Err(AppError(Error::invalid_state(
            "pinned execution environment does not satisfy the computed execution contract",
        )));
    }

    guard.store.save_execution_binding(&work, &binding)?;
    let manifest = ExecutionManifest::from_binding(
        &work,
        &binding,
        CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
    )?;
    guard.store.save_execution_manifest(&work, &manifest)?;

    Ok(Json(json!({
        "binding": binding,
        "execution_manifest": manifest,
        "attestation": {
            "environment_ref": attestation.environment_ref,
            "provider": attestation.provider,
            "isolation": attestation.isolation,
            "runtime_identities": attestation.runtime_identities,
            "evidence_refs": attestation.evidence_refs,
            "observed_at": attestation.observed_at,
            "valid_until": attestation.valid_until
        },
        "execution_started": false,
        "note": "Trusted deployment attestation plus deployment-pinned DSH distribution version/digest are recorded; SDK serverInfo remains wire identity only. No Harness turn, world outcome, authority grant or acceptance was created."
    })))
}

async fn v115_work_bind_e0(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_capability::{CapabilityStage, EffectClass};
    use morn_control_plane::{ConditionEvidence, ControlPlaneStore};
    use morn_kernel::time::Timestamp;
    use morn_runtime::{CompositionRuntimeRef, ExecutionBinding, ExecutionManifest};
    use morn_work::control::WorkPhase;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let capability_id = body
        .get("capability_manifest_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("capability_manifest_id is required")))?;

    let now = Timestamp::now();
    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase != WorkPhase::Ready || !work.required_conditions_satisfied() {
        return Err(AppError(Error::invalid_state(format!(
            "Work {} must be Ready with all generation-scoped conditions satisfied before binding",
            work.id
        ))));
    }

    let capability = guard
        .v115_capabilities
        .iter()
        .find(|record| record.manifest.id.as_str() == capability_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {capability_id}"
            )))
        })?;
    capability.manifest.validate_governance()?;
    if !matches!(
        capability.stage,
        CapabilityStage::Qualified | CapabilityStage::Admitted
    ) {
        return Err(AppError(Error::invalid_state(
            "execution binding requires a qualified or admitted capability",
        )));
    }
    if capability.manifest.authority.maximum_effect != EffectClass::E0LifecycleReversible {
        return Err(AppError(Error::not_authorized(
            "work/bind-e0 only accepts capabilities whose declared maximum effect is E0",
        )));
    }

    let resolved_evidence = guard
        .store
        .load_records_in_workspace::<ConditionEvidence>(
            "condition_evidence_v115",
            guard.workspace.id.as_str(),
        )?
        .into_iter()
        .any(|evidence| {
            evidence.condition_type == "CapabilityResolved"
                && evidence.satisfied
                && evidence.active_for(&work, now)
                && evidence
                    .evidence_refs
                    .iter()
                    .any(|reference| reference == capability.manifest.id.as_str())
        });
    if !resolved_evidence {
        return Err(AppError(Error::invalid_state(
            "capability is not witnessed by current-generation CapabilityResolved evidence",
        )));
    }

    let profile =
        morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
            AppError(Error::validation(format!(
                "unsupported Work profile {}",
                work.spec.profile_ref
            )))
        })?;
    if profile.requires("CapabilityQualification") {
        let site = work
            .spec
            .site_ref
            .as_deref()
            .ok_or_else(|| AppError(Error::validation("governed Work requires site_ref")))?;
        if !guard.v115_admission.site_profile_admission_active_at(
            &capability,
            site,
            &work.spec.profile_ref,
            now,
        ) {
            return Err(AppError(Error::invalid_state(
                "capability no longer has an active exact site/profile admission",
            )));
        }
    }

    let provider_ref = capability.manifest.provider_ref.clone();
    let provider_version = match provider_ref.as_str() {
        "morn-native" => guard
            .native_harness
            .lock()
            .map_err(|_| AppError(Error::internal("native harness lock poisoned")))?
            .runtime_version()
            .ok_or_else(|| AppError(Error::invalid_state("native runtime identity unavailable")))?,
        "deepseek-harness" => {
            let provider = guard
                .dsh_harness
                .lock()
                .map_err(|_| AppError(Error::internal("dsh harness lock poisoned")))?;
            if provider.mode() == morn_harness::provider::DshMode::Real {
                return Err(AppError(Error::invalid_state(
                    "real DSH binding requires a trusted fresh execution-environment attestation; use the deployment controller rather than the public bind-e0 endpoint",
                )));
            }
            provider
                .runtime_version()
                .ok_or_else(|| AppError(Error::invalid_state("DSH runtime identity unavailable")))?
        }
        "pi" => {
            let provider = guard
                .pi_harness
                .lock()
                .map_err(|_| AppError(Error::internal("pi harness lock poisoned")))?;
            if provider.mode() == morn_harness::PiMode::Real {
                return Err(AppError(Error::invalid_state(
                    "real Pi binding requires a trusted fresh execution-environment attestation and exact runtime identity; use the deployment controller rather than the public bind-e0 endpoint",
                )));
            }
            provider
                .runtime_version()
                .ok_or_else(|| AppError(Error::invalid_state("Pi runtime identity unavailable")))?
        }
        other => {
            return Err(AppError(Error::validation(format!(
                "capability provider {other:?} is not an executable HarnessProvider"
            ))))
        }
    };

    if capability.manifest.execution.minimum_isolation != morn_kernel::ExecutionClass::NoIsolation
        && capability.manifest.execution.minimum_isolation != morn_kernel::ExecutionClass::Process
    {
        return Err(AppError(Error::invalid_state(
            "reference bind-e0 cannot fabricate a stronger execution environment; use an attested deployment binding",
        )));
    }
    if !capability.manifest.execution.required_guarantees.is_empty()
        || !profile.required_execution_guarantees.is_empty()
    {
        return Err(AppError(Error::invalid_state(
            "reference bind-e0 cannot self-attest execution guarantees; use an attested deployment binding",
        )));
    }

    let mut binding = ExecutionBinding::for_work(
        &work,
        capability.manifest.id.to_string(),
        provider_ref,
        provider_version,
    );
    binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
    binding.compensation_ref = capability.manifest.compensation_ref.clone();
    binding.idempotency_key_required = capability.manifest.idempotency_key_required;
    guard.store.save_execution_binding(&work, &binding)?;
    let manifest = ExecutionManifest::from_binding(
        &work,
        &binding,
        CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
    )?;
    guard.store.save_execution_manifest(&work, &manifest)?;

    Ok(Json(json!({
        "binding": binding,
        "execution_manifest": manifest,
        "execution_started": false,
        "note": "Binding is immutable and generation-scoped. No Harness session, business outcome or acceptance was created."
    })))
}

fn classify_e0_turn_receipt(
    success: bool,
    snapshot_status: &str,
    scope_cleanup_ok: bool,
) -> (&'static str, bool) {
    if snapshot_status == "outcome-unknown"
        || snapshot_status.starts_with("outcome-unknown-")
        || snapshot_status == "policy-violation-runtime-reaped"
    {
        // A prohibited executor tool/subagent event means code outside Morn's
        // governed effect path may have run before containment. Reaping the
        // runtime stops further activity but cannot prove the world was
        // unchanged, so blind retry is unsafe.
        return ("outcome-unknown", false);
    }
    if !scope_cleanup_ok {
        return ("cleanup-failed", true);
    }
    if success {
        return ("completed", true);
    }
    ("failed", true)
}

fn work_phase_allows_requirement_resolution(phase: morn_work::control::WorkPhase) -> bool {
    matches!(
        phase,
        morn_work::control::WorkPhase::Proposed
            | morn_work::control::WorkPhase::Resolving
            | morn_work::control::WorkPhase::Blocked
            | morn_work::control::WorkPhase::Ready
    )
}

fn work_phase_allows_new_e0_execution(phase: morn_work::control::WorkPhase) -> bool {
    matches!(
        phase,
        morn_work::control::WorkPhase::Ready
            | morn_work::control::WorkPhase::Running
            | morn_work::control::WorkPhase::Waiting
    )
}

#[derive(Debug)]
struct E0HarnessTurnEvidence {
    output: Option<morn_harness::provider::HarnessOutput>,
    error: Option<String>,
    cleanup_error: Option<String>,
    receipt: morn_harness::ExecutionReceipt,
    events: Vec<morn_harness::ExecutionEvent>,
    snapshot_status: String,
}

fn run_e0_harness_turn<P: morn_harness::HarnessProvider>(
    provider: &mut P,
    work: &morn_work::control::WorkResource,
    binding: &morn_runtime::ExecutionBinding,
    input: &str,
) -> Result<E0HarnessTurnEvidence, Error> {
    use morn_harness::{CapabilityScope, RuntimeContext, ScopeKind};
    use morn_kernel::ids::ActorInstanceId;

    let scope = CapabilityScope::new(
        ScopeKind::ExecutionRun,
        None,
        work.workspace_id.clone(),
        format!("work:{}:generation:{}:e0", work.id, work.generation),
    )
    .with_restriction("morn.effects<=E0");
    let handle = provider.mount(scope)?;

    let build = (|| -> Result<(morn_harness::provider::HarnessSession, RuntimeContext), Error> {
        let mut ctx = RuntimeContext::new(
            work.workspace_id.clone(),
            ActorInstanceId::generate_with("work-executor"),
            work.id.clone(),
        )
        .with_work_binding(work.generation, binding.id.clone())
        .map_err(Error::validation)?
        .with_scope_id(handle.scope_id.clone())
        .map_err(Error::validation)?;
        if let (Some(environment_ref), Some(class)) = (
            binding.execution_environment_ref.clone(),
            binding.execution_class,
        ) {
            ctx = ctx
                .with_execution_environment(
                    environment_ref,
                    class,
                    binding.execution_guarantees.clone(),
                )
                .map_err(Error::validation)?;
        }
        ctx.provenance_refs = vec![
            binding.id.to_string(),
            binding.capability_manifest_ref.clone(),
        ];
        let session = provider.start(&ctx)?;
        Ok((session, ctx))
    })();

    let (session, ctx) = match build {
        Ok(value) => value,
        Err(error) => {
            let cleanup = provider.unmount(&handle).err();
            return Err(match cleanup {
                Some(cleanup) => Error::external(format!(
                    "{error}; execution scope cleanup also failed: {cleanup}"
                )),
                None => error,
            });
        }
    };

    let result = provider.send(&session.id, input);
    let mut events = provider.stream_events(&session.id);
    let snapshot_status = provider
        .inspect(&session.id)
        .map(|snapshot| snapshot.status)
        .unwrap_or_else(|_| "unknown".to_string());

    let cleanup_error = provider
        .unmount(&handle)
        .err()
        .map(|error| error.to_string());
    if let Some(error) = &cleanup_error {
        let mut event = morn_harness::ExecutionEvent::new(
            work.workspace_id.clone(),
            session.id.clone(),
            morn_harness::ExecutionEventKind::Failed,
            "E0 execution scope cleanup failed; executor result is not considered fully completed",
        );
        event.refs.push(handle.scope_id.clone());
        event.refs.push(format!("cleanup-error:{error}"));
        events.push(event);
    }

    let mut receipt = morn_harness::ExecutionReceipt::from_runtime_context(
        &ctx,
        provider.provider_name(),
        session.id.clone(),
    );
    receipt.event_ids = events.iter().map(|event| event.id.to_string()).collect();
    receipt.trace_refs = receipt.event_ids.clone();
    if let Some(started_at) = events.iter().map(|event| event.created_at).min() {
        receipt.started_at = started_at;
    }
    receipt.runtime_version = provider.runtime_version();
    receipt.runtime_digest = provider.runtime_digest();
    let (receipt_outcome, terminally_settled) =
        classify_e0_turn_receipt(result.is_ok(), &snapshot_status, cleanup_error.is_none());
    receipt.outcome = receipt_outcome.to_string();
    receipt.ended_at = terminally_settled.then(morn_kernel::time::Timestamp::now);

    Ok(E0HarnessTurnEvidence {
        output: result.as_ref().ok().cloned(),
        error: result.err().map(|error| error.to_string()),
        cleanup_error,
        receipt,
        events,
        snapshot_status,
    })
}

async fn v115_work_execute_e0(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_capability::EffectClass;
    use morn_control_plane::{ControlPlaneStore, WorkProgressController, WorkProgressInputs};

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let binding_id = body
        .get("binding_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("binding_id is required")))?;
    let input = body
        .get("input")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("input is required")))?
        .to_string();

    let (work, binding, native, dsh, pi) = {
        let guard = state.lock();
        let work = guard
            .store
            .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
            .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
        if work.workspace_id != guard.workspace.id {
            return Err(AppError(Error::not_found(format!(
                "WorkResource {work_id}"
            ))));
        }
        let binding = guard
            .store
            .load_record::<morn_runtime::ExecutionBinding>("execution_binding_v115", binding_id)?
            .ok_or_else(|| AppError(Error::not_found(format!("ExecutionBinding {binding_id}"))))?;

        // An immutable binding is historical provenance, not a perpetual lease
        // to issue new effects. Re-check the current capability/admission and
        // exact provider identity immediately before every new E0 execution.
        let capability = guard
            .v115_capabilities
            .iter()
            .find(|record| record.manifest.id.as_str() == binding.capability_manifest_ref)
            .ok_or_else(|| {
                AppError(Error::invalid_state(
                    "bound capability is no longer present in the canonical capability catalog",
                ))
            })?;
        if !matches!(
            capability.stage,
            morn_capability::CapabilityStage::Qualified
                | morn_capability::CapabilityStage::Admitted
        ) {
            return Err(AppError(Error::invalid_state(
                "bound capability is suspended, retired, or no longer qualified for new execution",
            )));
        }
        capability.manifest.validate_governance()?;
        if capability.manifest.authority.maximum_effect
            != morn_capability::EffectClass::E0LifecycleReversible
        {
            return Err(AppError(Error::not_authorized(
                "bound capability no longer satisfies the E0 execution ceiling",
            )));
        }
        let profile =
            morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
                AppError(Error::validation(format!(
                    "unsupported Work profile {}",
                    work.spec.profile_ref
                )))
            })?;
        let now = morn_kernel::time::Timestamp::now();
        if profile.requires("CapabilityQualification") {
            let site =
                work.spec.site_ref.as_deref().ok_or_else(|| {
                    AppError(Error::validation("governed Work requires site_ref"))
                })?;
            if !guard.v115_admission.site_profile_admission_active_at(
                capability,
                site,
                &work.spec.profile_ref,
                now,
            ) {
                return Err(AppError(Error::invalid_state(
                    "bound capability admission was revoked or expired; new execution is blocked",
                )));
            }
        }

        match binding.provider_ref.as_str() {
            "morn-native" => {
                let provider = guard
                    .native_harness
                    .lock()
                    .map_err(|_| AppError(Error::internal("native harness lock poisoned")))?;
                if provider.runtime_version().as_deref() != Some(binding.provider_version.as_str())
                {
                    return Err(AppError(Error::invalid_state(
                        "bound native provider identity no longer matches the active runtime",
                    )));
                }
            }
            "deepseek-harness" => {
                let provider = guard
                    .dsh_harness
                    .lock()
                    .map_err(|_| AppError(Error::internal("dsh harness lock poisoned")))?;
                if provider.mode() == morn_harness::provider::DshMode::Real {
                    ensure_live_environment_for_binding(
                        &binding,
                        capability,
                        &profile,
                        &guard.execution_environments,
                        now,
                    )?;
                    let environment_ref =
                        binding
                            .execution_environment_ref
                            .as_deref()
                            .ok_or_else(|| {
                                AppError(Error::invalid_state(
                                    "bound DSH execution has no environment identity",
                                ))
                            })?;
                    let attestation = guard
                        .execution_environments
                        .attestation(environment_ref)
                        .ok_or_else(|| {
                            AppError(Error::invalid_state(
                                "bound DSH execution environment attestation no longer exists",
                            ))
                        })?;
                    let profile_configuration_ref = provider
                        .configured_profile_configuration_ref()
                        .ok_or_else(|| {
                            AppError(Error::invalid_state(
                                "configured DSH Home/Profile composition identity is missing",
                            ))
                        })?;
                    if !attestation
                        .runtime_identities
                        .iter()
                        .any(|identity| identity == profile_configuration_ref)
                    {
                        return Err(AppError(Error::invalid_state(
                            "execution environment no longer attests the DSH Home/Profile composition pinned by the active route",
                        )));
                    }
                }
                if provider.runtime_version().as_deref() != Some(binding.provider_version.as_str())
                {
                    return Err(AppError(Error::invalid_state(
                        "bound DSH provider identity no longer matches the active runtime",
                    )));
                }
                if provider.mode() == morn_harness::provider::DshMode::Real
                    && binding.provider_digest.as_deref() != provider.configured_runtime_digest()
                {
                    return Err(AppError(Error::invalid_state(
                        "bound DSH provider digest no longer matches the configured runtime distribution",
                    )));
                }
                if provider.mode() == morn_harness::provider::DshMode::Real
                    && binding.runtime_ref != provider.configured_route_ref()
                {
                    return Err(AppError(Error::invalid_state(
                        "bound DSH execution route no longer matches the configured profile/model route",
                    )));
                }
                if provider.mode() == morn_harness::provider::DshMode::Real
                    && !provider.runtime_health().ready_for_new_turn_at(now)
                {
                    return Err(AppError(Error::invalid_state(
                        "real DSH provider lacks a fresh initialization/healthy lease for a new turn",
                    )));
                }
            }
            "pi" => {
                let provider = guard
                    .pi_harness
                    .lock()
                    .map_err(|_| AppError(Error::internal("pi harness lock poisoned")))?;
                if provider.mode() == morn_harness::PiMode::Real {
                    ensure_live_environment_for_binding(
                        &binding,
                        capability,
                        &profile,
                        &guard.execution_environments,
                        now,
                    )?;
                }
                if provider.runtime_version().as_deref() != Some(binding.provider_version.as_str())
                {
                    return Err(AppError(Error::invalid_state(
                        "bound Pi provider identity no longer matches the active runtime",
                    )));
                }
                if provider.mode() == morn_harness::PiMode::Real
                    && binding.provider_digest.as_deref() != provider.configured_runtime_digest()
                {
                    return Err(AppError(Error::invalid_state(
                        "bound Pi provider digest no longer matches the configured runtime distribution",
                    )));
                }
                if provider.mode() == morn_harness::PiMode::Real
                    && binding.runtime_ref != provider.configured_route_ref()
                {
                    return Err(AppError(Error::invalid_state(
                        "bound Pi execution route no longer matches the configured provider/model route",
                    )));
                }
                if provider.mode() == morn_harness::PiMode::Real
                    && !provider.runtime_health().ready_for_new_turn_at(now)
                {
                    return Err(AppError(Error::invalid_state(
                        "real Pi provider lacks a fresh initialization/healthy lease for a new turn",
                    )));
                }
            }
            other => {
                return Err(AppError(Error::validation(format!(
                    "binding provider {other:?} is not an executable harness provider"
                ))))
            }
        }

        (
            work,
            binding,
            guard.native_harness.clone(),
            guard.dsh_harness.clone(),
            guard.pi_harness.clone(),
        )
    };

    if !work_phase_allows_new_e0_execution(work.status.phase)
        || !work.required_conditions_satisfied()
    {
        return Err(AppError(Error::invalid_state(format!(
            "Work {} is not execution-ready in phase {:?}",
            work.id, work.status.phase
        ))));
    }
    if !binding.matches_work_generation(&work)
        || binding.profile_ref != work.spec.profile_ref
        || binding.site_ref != work.spec.site_ref
        || binding.autonomy_posture != work.spec.autonomy_posture
    {
        return Err(AppError(Error::validation(
            "ExecutionBinding does not match the exact canonical Work generation/profile/site/autonomy",
        )));
    }
    if binding.effect_ceiling != Some(EffectClass::E0LifecycleReversible) {
        return Err(AppError(Error::not_authorized(
            "work/execute-e0 only permits an E0 lifecycle-reversible binding; E1/E2/E3 must use ExternalAction",
        )));
    }

    let work_for_turn = work.clone();
    let binding_for_turn = binding.clone();
    let provider_ref = binding.provider_ref.clone();
    let turn = match provider_ref.as_str() {
        "morn-native" => tokio::task::spawn_blocking(move || {
            let mut provider = native
                .lock()
                .map_err(|_| Error::internal("native harness lock poisoned"))?;
            run_e0_harness_turn(&mut *provider, &work_for_turn, &binding_for_turn, &input)
        })
        .await
        .map_err(|error| {
            AppError(Error::internal(format!(
                "native harness task failed: {error}"
            )))
        })??,
        "deepseek-harness" => tokio::task::spawn_blocking(move || {
            let mut provider = dsh
                .lock()
                .map_err(|_| Error::internal("dsh harness lock poisoned"))?;
            run_e0_harness_turn(&mut *provider, &work_for_turn, &binding_for_turn, &input)
        })
        .await
        .map_err(|error| {
            AppError(Error::internal(format!("DSH harness task failed: {error}")))
        })??,
        "pi" => tokio::task::spawn_blocking(move || {
            let mut provider = pi
                .lock()
                .map_err(|_| Error::internal("pi harness lock poisoned"))?;
            run_e0_harness_turn(&mut *provider, &work_for_turn, &binding_for_turn, &input)
        })
        .await
        .map_err(|error| AppError(Error::internal(format!("Pi harness task failed: {error}"))))??,
        other => {
            return Err(AppError(Error::validation(format!(
                "binding provider {other:?} is not an executable harness provider"
            ))))
        }
    };

    let current = {
        let guard = state.lock();
        for event in &turn.events {
            guard.store.save_execution_event(&work, event)?;
        }
        guard.store.save_execution_receipt(&work, &turn.receipt)?;
        let mut current = guard
            .store
            .load_record::<morn_work::control::WorkResource>(
                "work_resource_v115",
                work.id.as_str(),
            )?
            .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {}", work.id))))?;
        if current.generation == work.generation {
            WorkProgressController.reconcile(
                &mut current,
                &WorkProgressInputs {
                    binding: Some(&binding),
                    receipt: Some(&turn.receipt),
                    ..Default::default()
                },
            );
            guard.store.save_work_resource_cas(&mut current)?;
        }
        current
    };

    Ok(Json(json!({
        "executor_status": match turn.receipt.outcome.as_str() {
            "completed" => "completed",
            "outcome-unknown" => "outcome-unknown",
            _ => "failed",
        },
        "provider_ref": provider_ref,
        "output": turn.output,
        "executor_error": turn.error,
        "scope_cleanup_error": turn.cleanup_error,
        "snapshot_status": turn.snapshot_status,
        "receipt": turn.receipt,
        "normalized_events": turn.events,
        "work": current,
        "business_outcome_observed": false,
        "independent_acceptance": false,
        "note": "E0 Harness execution evidence only. Provider completion never creates ObservedOutcome or AcceptanceDecision."
    })))
}

async fn v115_acceptance_reviewers(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let now = morn_kernel::time::Timestamp::now();
    let reviewers: Vec<_> = guard
        .acceptance_reviewers
        .iter()
        .filter(|reviewer| {
            reviewer.validate().is_ok()
                && reviewer.observed_at <= now
                && reviewer.valid_until.is_none_or(|until| now <= until)
                && reviewer.principal_id != guard.workspace.owner
        })
        .cloned()
        .collect();
    Ok(Json(json!({
        "reviewers": reviewers,
        "deployment_attested": true,
        "caller_can_self_assert_identity": false,
        "final_review_requires_exact_out_of_band_authorization": true,
        "authorization_ids_are_listed": false
    })))
}

async fn v115_work_review_outcome(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::{ControlPlaneStore, WorkProgressController, WorkProgressInputs};
    use morn_kernel::ids::{AcceptanceSpecId, OutcomeRecordId};
    use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
    use morn_work::control::WorkPhase;
    use morn_world::ObservedOutcome;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let outcome_id = body
        .get("outcome_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("outcome_id is required")))?;
    let review_authorization_id = body
        .get("review_authorization_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("review_authorization_id is required")))?;
    let reviewer_principal_id = body
        .get("reviewer_principal_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("reviewer_principal_id is required")))?;
    let acting_role = body
        .get("acting_role")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("acting_role is required")))?;
    let reason = body
        .get("reason")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("reason is required")))?;
    let evidence_refs: Vec<String> = body
        .get("evidence_refs")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if evidence_refs.is_empty() {
        return Err(AppError(Error::validation(
            "independent review requires at least one evidence reference",
        )));
    }
    let disposition = match body
        .get("disposition")
        .and_then(Value::as_str)
        .unwrap_or("accept")
        .to_ascii_lowercase()
        .as_str()
    {
        "accept" => AcceptanceDisposition::Accept,
        "reject" => AcceptanceDisposition::Reject,
        "conditional" => AcceptanceDisposition::Conditional,
        "request-more-evidence" | "request_more_evidence" => {
            AcceptanceDisposition::RequestMoreEvidence
        }
        other => {
            return Err(AppError(Error::validation(format!(
                "unsupported acceptance disposition {other:?}"
            ))))
        }
    };

    let guard = state.lock();
    let now = morn_kernel::time::Timestamp::now();
    let reviewer = guard
        .acceptance_reviewers
        .iter()
        .find(|reviewer| reviewer.principal_id.as_str() == reviewer_principal_id)
        .filter(|reviewer| reviewer.active_for(acting_role, now))
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_authorized(
                "reviewer principal/role is not covered by an active deployment attestation",
            ))
        })?;
    if reviewer.principal_id == guard.workspace.owner {
        return Err(AppError(Error::not_authorized(
            "independent acceptance reviewer must differ from the workspace owner",
        )));
    }
    let mut work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase.is_terminal() {
        return Err(AppError(Error::invalid_state(
            "terminal Work cannot receive a new acceptance decision without a new generation",
        )));
    }
    if !matches!(work.status.phase, WorkPhase::Delivered | WorkPhase::Waiting) {
        return Err(AppError(Error::invalid_state(
            "outcome review requires Work to be Delivered or Waiting",
        )));
    }

    let outcome = guard
        .store
        .load_record::<ObservedOutcome>(
            "observed_outcome_v115",
            OutcomeRecordId::new(outcome_id).as_str(),
        )?
        .ok_or_else(|| AppError(Error::not_found(format!("ObservedOutcome {outcome_id}"))))?;
    if outcome.workspace_id != work.workspace_id || outcome.work_package_id != work.id {
        return Err(AppError(Error::validation(
            "reviewed outcome must belong to the exact Work and workspace",
        )));
    }
    if !outcome.is_source_grounded() {
        return Err(AppError(Error::validation(
            "independent review requires a source-grounded observed outcome",
        )));
    }
    if disposition == AcceptanceDisposition::Accept && !outcome.is_attested_source_grounded() {
        return Err(AppError(Error::validation(
            "final acceptance requires deployment-attested source provenance; manually/imported evidence may be reviewed but cannot be accepted as business truth",
        )));
    }

    let authorization = guard
        .acceptance_review_authorizations
        .iter()
        .find(|authorization| authorization.authorization_id == review_authorization_id)
        .filter(|authorization| {
            authorization.authorizes(
                &reviewer.principal_id,
                acting_role,
                (&work.id, work.generation),
                &outcome.id,
                disposition,
                now,
            )
        })
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_authorized(
                "review request lacks an active deployment authorization for this exact reviewer, role, Work, outcome and disposition",
            ))
        })?;
    let consumed_kind = "acceptance_review_authorization_consumed_v115";
    if guard
        .store
        .load_record::<Value>(consumed_kind, review_authorization_id)?
        .is_some()
    {
        return Err(AppError(Error::conflict(
            "acceptance review authorization was already consumed",
        )));
    }

    let acceptance_spec_id = work
        .spec
        .acceptance_ref
        .as_deref()
        .map(AcceptanceSpecId::new)
        .unwrap_or_else(|| AcceptanceSpecId::generate_with("acceptance"));
    let mut decision = AcceptanceDecision::new(
        work.id.clone(),
        acceptance_spec_id,
        disposition,
        reviewer.principal_id.clone(),
        acting_role,
        reason,
    );
    decision.pin_work_generation(work.generation)?;
    decision.outcome_refs.push(outcome.id.clone());
    decision.evidence_refs = evidence_refs;
    decision
        .evidence_refs
        .extend(reviewer.evidence_refs.clone());
    decision
        .evidence_refs
        .extend(authorization.evidence_refs.clone());
    decision.evidence_refs.sort();
    decision.evidence_refs.dedup();
    if let Some(conditions) = body.get("conditions").and_then(Value::as_array) {
        decision.conditions = conditions
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }

    // Consume before writing the decision: if a later persistence step fails,
    // the authorization fails closed and must be re-issued rather than becoming replayable.
    guard.store.save_record_immutable(
        consumed_kind,
        review_authorization_id,
        work.workspace_id.as_str(),
        now.millis(),
        &json!({
            "authorization_id": review_authorization_id,
            "work_id": work.id,
            "outcome_id": outcome.id,
            "reviewer_principal_id": reviewer.principal_id,
            "acting_role": acting_role,
            "disposition": disposition,
            "consumed_at": now
        }),
    )?;
    ControlPlaneStore::save_acceptance_decision(&guard.store, &work, &decision)?;
    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            outcome: Some(&outcome),
            acceptance: Some(&decision),
            ..Default::default()
        },
    );
    if let Err(cas_error) = guard.store.save_work_resource_cas(&mut work) {
        let recovery = morn_control_plane::DurableBusinessEvidenceControllerRuntime::new(
            "morn-app-acceptance-projection-recovery",
        )
        .reconcile_from_persisted_business_evidence(&guard.store, work_id, now);
        if recovery.is_err() {
            return Err(AppError(cas_error));
        }
        work = guard
            .store
            .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
            .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    }

    Ok(Json(json!({
        "decision": decision,
        "work": work,
        "reviewed_outcome": outcome.id,
        "reviewer_principal_id": reviewer.principal_id,
        "reviewer_identity_evidence": reviewer.evidence_refs,
        "review_authorization_evidence": authorization.evidence_refs,
        "review_authorization_consumed": true,
        "business_outcome_source_grounded": true,
        "note": "Review consumes an already-persisted source-grounded outcome. It never promotes a Harness receipt or model output into business truth."
    })))
}

async fn v115_work_reconcile(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_control_plane::DurableWorkControllerRuntime;
    use morn_kernel::time::Timestamp;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;

    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if !work_phase_allows_requirement_resolution(work.status.phase) {
        return Err(AppError(Error::invalid_state(format!(
            "readiness reconciliation is not valid in Work phase {:?}; execution/outcome reconciliation requires its own evidence path",
            work.status.phase
        ))));
    }
    let profile =
        morn_profile::DomainProfile::from_ref(&work.spec.profile_ref).ok_or_else(|| {
            AppError(Error::validation(format!(
                "unsupported Work profile {}",
                work.spec.profile_ref
            )))
        })?;
    let holder = body
        .get("controller_holder")
        .and_then(Value::as_str)
        .unwrap_or("morn-app-api");
    let runtime = DurableWorkControllerRuntime::new(holder);
    for forbidden in [
        "capability_resolved",
        "capability_qualified",
        "authority_satisfied",
        "source_of_truth_bound",
        "provenance_ready",
    ] {
        if body.get(forbidden).is_some() {
            return Err(AppError(Error::validation(format!(
                "{forbidden} is derived from durable condition evidence and cannot be asserted by the reconcile caller"
            ))));
        }
    }
    let tick =
        runtime.reconcile_from_evidence(&guard.store, work_id, &profile, Timestamp::now())?;
    let current = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;

    Ok(Json(json!({
        "tick": tick,
        "work": current,
        "note": "controller tick derived readiness from durable generation-scoped evidence, then used lease/fencing + CAS + atomic durable semantic outbox"
    })))
}

async fn v115_work_reconcile_business_evidence(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::DurableBusinessEvidenceControllerRuntime;
    use morn_kernel::time::Timestamp;

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    for forbidden in [
        "outcome_id",
        "acceptance_id",
        "disposition",
        "source_ref",
        "evidence_refs",
    ] {
        if body.get(forbidden).is_some() {
            return Err(AppError(Error::validation(format!(
                "{forbidden} is derived from durable business evidence and cannot be asserted by the recovery caller"
            ))));
        }
    }

    let guard = state.lock();
    let work = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    let holder = body
        .get("controller_holder")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("morn-app-business-evidence-recovery");
    let runtime = DurableBusinessEvidenceControllerRuntime::new(holder);
    let tick = runtime.reconcile_from_persisted_business_evidence(
        &guard.store,
        work_id,
        Timestamp::now(),
    )?;
    let current = guard
        .store
        .load_record::<morn_work::control::WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;

    Ok(Json(json!({
        "tick": tick,
        "work": current,
        "replayed_external_execution": false,
        "reconsumed_authorization": false,
        "note": "Recovered only the mutable Work projection from already-persisted Outcome/Acceptance evidence."
    })))
}

async fn v115_work_reconcile_executor_outcome(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::ControlPlaneStore;
    use morn_harness::{
        ExecutionReceipt, ExecutorOutcomeDisposition, ExecutorOutcomeReconciliation,
    };
    use morn_kernel::time::Timestamp;
    use morn_runtime::ExecutionBinding;
    use morn_work::control::{ConditionStatus, WorkCondition, WorkPhase, WorkResource};

    let work_id = body
        .get("work_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("work_id is required")))?;
    let receipt_id = body
        .get("execution_receipt_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("execution_receipt_id is required")))?;
    let authorization_id = body
        .get("reconciliation_authorization_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AppError(Error::validation(
                "reconciliation_authorization_id is required; callers cannot self-assert executor settlement",
            ))
        })?;
    let reason = body
        .get("reason")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError(Error::validation("reason is required")))?;

    for forbidden in [
        "disposition",
        "evidence_refs",
        "principal_id",
        "execution_binding_ref",
        "work_generation",
    ] {
        if body.get(forbidden).is_some() {
            return Err(AppError(Error::validation(format!(
                "{forbidden} is deployment-authorized reconciliation data and cannot be asserted by the caller"
            ))));
        }
    }

    let guard = state.lock();
    let now = Timestamp::now();
    let mut work = guard
        .store
        .load_record::<WorkResource>("work_resource_v115", work_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("WorkResource {work_id}"))))?;
    if work.workspace_id != guard.workspace.id {
        return Err(AppError(Error::not_found(format!(
            "WorkResource {work_id}"
        ))));
    }
    if work.status.phase != WorkPhase::Reconciling {
        return Err(AppError(Error::invalid_state(
            "executor outcome reconciliation requires Work phase Reconciling",
        )));
    }

    let receipt = guard
        .store
        .load_record::<ExecutionReceipt>("execution_receipt_v115", receipt_id)?
        .ok_or_else(|| AppError(Error::not_found(format!("ExecutionReceipt {receipt_id}"))))?;
    if receipt.workspace_id != work.workspace_id
        || receipt.work_package_id.as_ref() != Some(&work.id)
        || receipt.work_generation != Some(work.generation)
        || receipt.outcome != "outcome-unknown"
        || receipt.ended_at.is_some()
    {
        return Err(AppError(Error::validation(
            "executor reconciliation requires the exact unsettled outcome-unknown receipt for this Work generation",
        )));
    }
    let binding_id = receipt.execution_binding_ref.as_ref().ok_or_else(|| {
        AppError(Error::validation(
            "outcome-unknown receipt has no pinned ExecutionBinding",
        ))
    })?;
    let binding = guard
        .store
        .load_record::<ExecutionBinding>("execution_binding_v115", binding_id.as_str())?
        .ok_or_else(|| AppError(Error::not_found(format!("ExecutionBinding {binding_id}"))))?;
    if !binding.matches_work_generation(&work) {
        return Err(AppError(Error::validation(
            "executor reconciliation receipt binding does not match the canonical Work generation",
        )));
    }

    if guard
        .store
        .load_records_in_workspace::<ExecutorOutcomeReconciliation>(
            "executor_outcome_reconciliation_v115",
            work.workspace_id.as_str(),
        )?
        .iter()
        .any(|record| record.execution_receipt_id == receipt.id)
    {
        return Err(AppError(Error::conflict(
            "execution receipt already has a durable executor reconciliation",
        )));
    }

    let authorization = guard
        .executor_reconciliation_authorizations
        .iter()
        .find(|candidate| candidate.authorization_id == authorization_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_authorized(
                "executor reconciliation authorization is not present in the deployment trust set",
            ))
        })?;
    if !authorization.authorizes(&work.id, work.generation, &binding.id, &receipt.id, now) {
        return Err(AppError(Error::not_authorized(
            "executor reconciliation authorization is stale or does not match the exact Work/binding/receipt",
        )));
    }

    let consumed_kind = "executor_reconciliation_authorization_consumed_v115";
    if guard
        .store
        .load_record::<Value>(consumed_kind, authorization_id)?
        .is_some()
    {
        return Err(AppError(Error::conflict(
            "executor reconciliation authorization was already consumed",
        )));
    }
    let reconciliation =
        ExecutorOutcomeReconciliation::from_authorization(&authorization, reason, now)?;

    // Consume the deployment authorization first. A later persistence failure
    // burns it fail-closed rather than making a bearer reconciliation replayable.
    guard.store.save_record_immutable(
        consumed_kind,
        authorization_id,
        work.workspace_id.as_str(),
        now.millis(),
        &json!({
            "authorization_id": authorization_id,
            "work_id": work.id,
            "work_generation": work.generation,
            "execution_binding_ref": binding.id,
            "execution_receipt_id": receipt.id,
            "disposition": authorization.disposition,
            "consumed_at": now
        }),
    )?;
    guard.store.save_record_immutable(
        "executor_outcome_reconciliation_v115",
        reconciliation.id.as_str(),
        work.workspace_id.as_str(),
        now.millis(),
        &reconciliation,
    )?;

    let (known_status, retry_status, next_phase, known_reason, retry_reason) =
        match reconciliation.disposition {
            ExecutorOutcomeDisposition::NoEffectConfirmed => (
                ConditionStatus::True,
                ConditionStatus::True,
                WorkPhase::Blocked,
                "deployment-authorized reconciliation confirmed the ambiguous executor turn caused no external effect",
                "retry may proceed only after readiness is re-evaluated from current durable evidence",
            ),
            ExecutorOutcomeDisposition::EffectObserved => (
                ConditionStatus::True,
                ConditionStatus::False,
                WorkPhase::Reconciling,
                "deployment-authorized reconciliation confirmed an executor-side effect occurred",
                "blind retry remains forbidden; wait for authoritative business outcome observation",
            ),
            ExecutorOutcomeDisposition::StillUnknown => (
                ConditionStatus::False,
                ConditionStatus::False,
                WorkPhase::Reconciling,
                "deployment-authorized reconciliation could not establish definitive executor settlement",
                "blind retry remains forbidden while executor outcome is unknown",
            ),
        };
    let refs = std::iter::once(reconciliation.id.to_string())
        .chain(reconciliation.evidence_refs.iter().cloned())
        .collect::<Vec<_>>();
    let mut known = WorkCondition::new("ExecutorOutcomeKnown", known_status);
    known.reason = known_reason.to_string();
    known.evidence_refs = refs.clone();
    work.set_condition(known);
    let mut retry = WorkCondition::new("ExecutorRetrySafe", retry_status);
    retry.reason = retry_reason.to_string();
    retry.evidence_refs = refs;
    work.set_condition(retry);
    work.status.phase = next_phase;
    work.mark_observed();
    guard.store.save_work_resource_cas(&mut work)?;

    Ok(Json(json!({
        "reconciliation": reconciliation,
        "work": work,
        "authorization_consumed": true,
        "business_outcome_observed": false,
        "independent_acceptance": false,
        "note": "Executor reconciliation resolves only ambiguous execution settlement. It cannot create business Outcome or Acceptance."
    })))
}

async fn v115_control_plane(State(state): State<AppState>) -> ApiResult {
    use morn_assurance::{EvidenceClaimState, EvidenceClass};
    use morn_work::value::{ValueAssessment, ValueEvidenceClass};

    let guard = state.lock();
    let store = &guard.store;
    let load = |kind: &str| -> Result<Vec<Value>, Error> {
        store.load_records_in_workspace::<Value>(kind, guard.workspace.id.as_str())
    };
    let value_assessments = store.load_records_in_workspace::<ValueAssessment>(
        "value_assessment_v115",
        guard.workspace.id.as_str(),
    )?;
    let value_assessment_support: Vec<Value> = value_assessments
        .iter()
        .filter(|assessment| assessment.evidence_class == ValueEvidenceClass::CustomerValidated)
        .map(|assessment| {
            let subject = value_claim_subject(
                &assessment.work_package_id,
                assessment.work_generation,
                &assessment.outcome_ref,
            );
            let current = guard
                .evidence_ledger
                .current_claim(&subject, EvidenceClass::RealSite);
            let current_state = current.map(|claim| match claim.state {
                EvidenceClaimState::Proven => "proven",
                EvidenceClaimState::BlockedExternal => "blocked-external",
                EvidenceClaimState::Revoked => "revoked",
            });
            let attestation = assessment
                .customer_value_attestation_ref
                .as_deref()
                .and_then(|attestation_id| {
                    guard
                        .customer_value_attestations
                        .iter()
                        .find(|candidate| candidate.attestation_id == attestation_id)
                });
            let attestation_active = assessment.acceptance_ref.as_ref().is_some_and(|acceptance_id| {
                attestation.is_some_and(|attestation| {
                    attestation.active_for(
                        &guard.workspace.id,
                        (&assessment.work_package_id, assessment.work_generation),
                        &assessment.outcome_ref,
                        acceptance_id,
                        morn_kernel::time::Timestamp::now(),
                    )
                })
            });
            json!({
                "assessment_id": assessment.id,
                "subject": subject,
                "requires_real_site": true,
                "customer_value_attestation_ref": assessment.customer_value_attestation_ref.clone(),
                "customer_value_attestation_active": attestation_active,
                "currently_supported": current.is_some_and(|claim| claim.state == EvidenceClaimState::Proven)
                    && attestation_active,
                "current_real_site_state": current_state,
                "current_real_site_claim_id": current.map(|claim| claim.id.to_string()),
                "current_real_site_evidence_refs": current.map(|claim| claim.evidence_refs.clone()).unwrap_or_default()
            })
        })
        .collect();

    Ok(Json(json!({
        "work": load("work_resource_v115")?,
        "condition_evidence": load("condition_evidence_v115")?,
        "capability_resolutions": load("capability_resolution_v115")?,
        "profile_conformance_attestations": load("profile_conformance_attestation_v115")?,
        "source_of_truth_bindings": load("source_of_truth_binding_v115")?,
        "execution_bindings": load("execution_binding_v115")?,
        "execution_manifests": load("execution_manifest_v115")?,
        "execution_events": load("execution_event_v115")?,
        "execution_receipts": load("execution_receipt_v115")?,
        "executor_outcome_reconciliations": load("executor_outcome_reconciliation_v115")?,
        "interop_bindings": load("interop_binding_v115")?,
        "external_task_observations": load("external_task_observation_v115")?,
        "binding_migrations": load("binding_migration_v115")?,
        "durable_workflow_bindings": load("durable_workflow_binding_v115")?,
        "attempts": load("action_attempt_v115")?,
        "reconciliations": load("reconciliation_v115")?,
        "outcomes": load("observed_outcome_v115")?,
        "acceptance_decisions": load("acceptance_decision_v115")?,
        "value_assessments": value_assessments,
        "value_assessment_support": value_assessment_support,
        "note": "canonical persisted v11.5 records; CustomerValidated support is re-evaluated from the latest RealSite evidence claim without rewriting historical assessments"
    })))
}

async fn v115_solutions(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    let packages = guard
        .store
        .load_records_in_workspace::<morn_foundry::SolutionPackage>(
            "solution_package_v115",
            guard.workspace.id.as_str(),
        )?;
    Ok(Json(json!({
        "solution_packages": packages,
        "note": "approved reusable blueprints persisted independently of runtime Work"
    })))
}

fn json_strings(body: &Value, key: &str) -> Vec<String> {
    body.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn register_v115_candidate(
    state: &AppState,
    record: &morn_capability::CapabilityRecord,
) -> Result<(), Error> {
    let mut guard = state.lock();
    guard.store.save_record(
        "capability_record_v115",
        record.manifest.id.as_str(),
        guard.workspace.id.as_str(),
        record.manifest.declared_at.millis(),
        record,
    )?;
    guard
        .v115_capabilities
        .retain(|existing| existing.manifest.id != record.manifest.id);
    guard.v115_capabilities.push(record.clone());
    Ok(())
}

async fn v115_capabilities(State(state): State<AppState>) -> ApiResult {
    let guard = state.lock();
    Ok(Json(json!({
        "capabilities": guard.v115_capabilities,
        "observations": guard.v115_admission.observations,
        "qualifications": guard.v115_admission.qualifications,
        "releases": guard.v115_admission.releases,
        "admissions": guard.v115_admission.admissions,
        "lifecycle_events": guard.v115_admission.events,
        "invariant": "compile != observe != qualify != release != site admission; lifecycle projection changes append events"
    })))
}

async fn v115_discovery(State(state): State<AppState>) -> ApiResult {
    use morn_capability::{
        project_a2a_agent_card, project_oasf, project_xregistry, CapabilityKind,
    };

    let guard = state.lock();
    let mut xregistry = Vec::new();
    let mut a2a_agent_cards = Vec::new();
    let mut oasf = Vec::new();
    let mut rejected = Vec::new();

    for record in &guard.v115_capabilities {
        match project_xregistry(record) {
            Ok(projection) => xregistry.push(projection),
            Err(error) => rejected.push(json!({
                "manifest_id": record.manifest.id,
                "projection": "xregistry",
                "reason": error.to_string()
            })),
        }

        if record.manifest.kind == CapabilityKind::Agent {
            let a2a_card_url = record
                .manifest
                .interfaces
                .iter()
                .find(|interface| interface.protocol.eq_ignore_ascii_case("a2a"))
                .map(|interface| interface.input_schema_ref.clone());
            if let Some(card_url) = a2a_card_url {
                match project_a2a_agent_card(record, card_url, "1.0") {
                    Ok(projection) => a2a_agent_cards.push(projection),
                    Err(error) => rejected.push(json!({
                        "manifest_id": record.manifest.id,
                        "projection": "a2a-agent-card",
                        "reason": error.to_string()
                    })),
                }
            }
            match project_oasf(record, "declared", Vec::new()) {
                Ok(projection) => oasf.push(projection),
                Err(error) => rejected.push(json!({
                    "manifest_id": record.manifest.id,
                    "projection": "oasf",
                    "reason": error.to_string()
                })),
            }
        }
    }

    Ok(Json(json!({
        "xregistry": xregistry,
        "a2a_agent_cards": a2a_agent_cards,
        "oasf": oasf,
        "rejected": rejected,
        "metadata_class": "declared",
        "business_truth": false,
        "invariant": "registry projections are discovery metadata only; qualification, release, site admission, authority and accepted outcomes remain canonical Morn records"
    })))
}

async fn v115_capability_observe(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    let manifest_id = body
        .get("manifest_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("manifest_id is required")))?;
    let evidence_refs = json_strings(&body, "evidence_refs");
    let evaluator_identity = body
        .get("evaluator_identity")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("evaluator_identity is required")))?;

    let mut guard = state.lock();
    let index = guard
        .v115_capabilities
        .iter()
        .position(|capability| capability.manifest.id.as_str() == manifest_id)
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {manifest_id}"
            )))
        })?;
    let observation = {
        let inner = &mut *guard;
        let capability = &mut inner.v115_capabilities[index];
        inner
            .v115_admission
            .observe(capability, evidence_refs, evaluator_identity)?
    };
    guard.persist_all()?;
    Ok(Json(json!({
        "observation": observation,
        "stage": guard.v115_capabilities[index].stage,
        "next": ["strict-qualification"]
    })))
}

async fn v115_capability_qualify(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_assurance::{QualificationEvidence, StrictQualificationRequest};
    use morn_kernel::time::Timestamp;

    let manifest_id = body
        .get("manifest_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("manifest_id is required")))?;
    let candidate_ref = body
        .get("candidate_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("candidate_ref is required")))?;
    let decision_ref = body
        .get("decision_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("decision_ref is required")))?;
    let evaluator_identity = body
        .get("evaluator_identity")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("evaluator_identity is required")))?;
    let environment_digest = body
        .get("environment_digest")
        .and_then(Value::as_str)
        .map(str::to_string);
    let valid_until = body
        .get("valid_until_millis")
        .and_then(Value::as_i64)
        .map(Timestamp::from_millis);

    let request = StrictQualificationRequest {
        candidate_ref: candidate_ref.to_string(),
        decision_ref: decision_ref.to_string(),
        evidence_refs: json_strings(&body, "evidence_refs"),
        qualification_evidence: QualificationEvidence {
            test_suite_refs: json_strings(&body, "test_suite_refs"),
            environment_digest,
            input_scope: json_strings(&body, "input_scope"),
            expected_properties: json_strings(&body, "expected_properties"),
            known_failure_modes: json_strings(&body, "known_failure_modes"),
            cost_evidence_ref: body
                .get("cost_evidence_ref")
                .and_then(Value::as_str)
                .map(str::to_string),
            latency_evidence_ref: body
                .get("latency_evidence_ref")
                .and_then(Value::as_str)
                .map(str::to_string),
            evaluator_identity: Some(evaluator_identity.to_string()),
        },
        context_of_use: json_strings(&body, "context_of_use"),
        valid_until,
    };

    let mut guard = state.lock();
    let index = guard
        .v115_capabilities
        .iter()
        .position(|capability| capability.manifest.id.as_str() == manifest_id)
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {manifest_id}"
            )))
        })?;
    let qualification = {
        let inner = &mut *guard;
        let capability = &mut inner.v115_capabilities[index];
        inner
            .v115_admission
            .qualify_with_evidence(capability, request)?
    };
    guard.persist_all()?;
    Ok(Json(json!({
        "qualification": qualification,
        "stage": guard.v115_capabilities[index].stage,
        "next": ["content-addressed-release"]
    })))
}

async fn v115_capability_release(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    let manifest_id = body
        .get("manifest_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("manifest_id is required")))?;
    let qualification_id = body
        .get("qualification_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("qualification_id is required")))?;
    let package_ref = body
        .get("package_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("package_ref is required")))?;
    let content_digest = body
        .get("content_digest")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("content_digest is required")))?;
    let signature_ref = body
        .get("signature_ref")
        .and_then(Value::as_str)
        .map(str::to_string);
    let provenance_ref = body
        .get("provenance_ref")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut guard = state.lock();
    let qualification = guard
        .v115_admission
        .qualifications
        .iter()
        .find(|item| item.id.as_str() == qualification_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "Qualification {qualification_id}"
            )))
        })?;
    let index = guard
        .v115_capabilities
        .iter()
        .position(|capability| capability.manifest.id.as_str() == manifest_id)
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {manifest_id}"
            )))
        })?;
    let verification = guard
        .supply_chain_verifications
        .iter()
        .find(|evidence| {
            evidence
                .validate_subject_digest(content_digest, true, true)
                .is_ok()
        })
        .cloned();
    let release = {
        let inner = &mut *guard;
        let capability = &mut inner.v115_capabilities[index];
        match verification {
            Some(verification) => {
                let signature_ref = signature_ref.ok_or_else(|| {
                    AppError(Error::validation(
                        "verified release requires signature_ref from the publication record",
                    ))
                })?;
                let provenance_ref = provenance_ref.ok_or_else(|| {
                    AppError(Error::validation(
                        "verified release requires provenance_ref from the publication record",
                    ))
                })?;
                inner.v115_admission.record_verified_release(
                    capability,
                    &qualification,
                    VerifiedReleaseRequest::new(
                        package_ref,
                        content_digest,
                        signature_ref,
                        provenance_ref,
                        verification,
                    ),
                )?
            }
            None => inner.v115_admission.record_release(
                capability,
                &qualification,
                package_ref,
                content_digest,
                signature_ref,
                provenance_ref,
            )?,
        }
    };
    guard.persist_all()?;
    let verified = release.supply_chain_verification.is_some();
    Ok(Json(json!({
        "release": release,
        "verified": verified,
        "stage": guard.v115_capabilities[index].stage,
        "next": if verified {
            vec!["profile-conformance", "site-admission"]
        } else {
            vec!["deployment-supply-chain-verification"]
        }
    })))
}

async fn v115_capability_admit(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    let manifest_id = body
        .get("manifest_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("manifest_id is required")))?;
    let qualification_id = body
        .get("qualification_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("qualification_id is required")))?;
    let conformance_attestation_id = body
        .get("conformance_attestation_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError(Error::validation(
                "conformance_attestation_id is required; raw conformance booleans are not accepted",
            ))
        })?;
    let approved_by = body
        .get("approved_by")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("approved_by is required")))?;

    for forbidden in [
        "satisfied_semantics",
        "forbidden_semantics_present",
        "execution_guarantees",
        "isolation",
        "durable_work_state",
        "source_of_truth_bound",
        "provenance_ready",
    ] {
        if body.get(forbidden).is_some() {
            return Err(AppError(Error::validation(format!(
                "{forbidden} must come from a persisted ProfileConformanceAttestation, not the admission caller"
            ))));
        }
    }

    let mut guard = state.lock();
    let qualification = guard
        .v115_admission
        .qualifications
        .iter()
        .find(|item| item.id.as_str() == qualification_id)
        .cloned()
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "Qualification {qualification_id}"
            )))
        })?;
    let attestation = guard
        .store
        .load_record::<morn_assurance::ProfileConformanceAttestation>(
            "profile_conformance_attestation_v115",
            conformance_attestation_id,
        )?
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "ProfileConformanceAttestation {conformance_attestation_id}"
            )))
        })?;
    let index = guard
        .v115_capabilities
        .iter()
        .position(|capability| capability.manifest.id.as_str() == manifest_id)
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {manifest_id}"
            )))
        })?;

    let admission = {
        let inner = &mut *guard;
        let capability = &mut inner.v115_capabilities[index];
        inner.v115_admission.admit_with_attestation(
            capability,
            &qualification,
            &attestation,
            approved_by,
        )?
    };
    guard.persist_all()?;
    Ok(Json(json!({
        "admitted": true,
        "admission": admission,
        "conformance_attestation": attestation,
        "stage": guard.v115_capabilities[index].stage
    })))
}

async fn v115_capability_revoke_release(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_assurance::CapabilityDistributionReleaseId;

    let manifest_id = body
        .get("manifest_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("manifest_id is required")))?;
    let release_id = body
        .get("release_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("release_id is required")))?;

    let mut guard = state.lock();
    let index = guard
        .v115_capabilities
        .iter()
        .position(|capability| capability.manifest.id.as_str() == manifest_id)
        .ok_or_else(|| {
            AppError(Error::not_found(format!(
                "CapabilityManifest {manifest_id}"
            )))
        })?;
    {
        let inner = &mut *guard;
        let capability = &mut inner.v115_capabilities[index];
        inner.v115_admission.revoke_release(
            capability,
            &CapabilityDistributionReleaseId::new(release_id),
        )?;
    }
    guard.persist_all()?;
    Ok(Json(json!({
        "revoked": true,
        "release_id": release_id,
        "stage": guard.v115_capabilities[index].stage,
        "admission_refs": guard.v115_capabilities[index].admission_refs
    })))
}

async fn v115_creator_draft(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_foundry::{draft_creator_solution, CreatorAutonomy, CreatorRequest};

    let guard = state.lock();
    let autonomy = match body
        .get("autonomy")
        .and_then(Value::as_str)
        .unwrap_or("governed")
    {
        "assist" => CreatorAutonomy::Assist,
        "governed" => CreatorAutonomy::Governed,
        "autonomous-within-policy" => CreatorAutonomy::AutonomousWithinPolicy,
        other => {
            return Err(AppError(Error::validation(format!(
                "unsupported creator autonomy {other}"
            ))))
        }
    };
    let strings = |key: &str| -> Vec<String> {
        body.get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };

    let request = CreatorRequest {
        workspace_id: guard.workspace.id.clone(),
        name: body
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("untitled-solution")
            .to_string(),
        goal: body
            .get("goal")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError(Error::validation("creator goal is required")))?
            .to_string(),
        profile_ref: body
            .get("profile_ref")
            .and_then(Value::as_str)
            .unwrap_or("morn.lite@1.0.0")
            .to_string(),
        site_ref: body
            .get("site_ref")
            .and_then(Value::as_str)
            .map(str::to_string),
        required_capabilities: strings("required_capabilities"),
        constraints: strings("constraints"),
        acceptance: strings("acceptance"),
        autonomy,
    };
    drop(guard);

    let draft = draft_creator_solution(request)?;
    Ok(Json(json!({
        "draft": draft,
        "canonical_write": false,
        "next": ["review", "approve", "compile-solution-package", "instantiate-work"]
    })))
}

async fn v115_compile_openapi(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, OpenApiJsonCompiler};

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
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["evaluate", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_compile_procedure(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, ProcedureJsonCompiler};

    let name = body
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("procedure-capability");
    let source_ref = body
        .get("source_ref")
        .and_then(Value::as_str)
        .unwrap_or("inline://procedure");
    let source_digest = body
        .get("source_digest")
        .and_then(Value::as_str)
        .map(str::to_string);
    let content = body
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError(Error::validation("content must contain procedure JSON")))?;

    let source = ArtifactSource {
        kind: ArtifactKind::Procedure,
        name: name.to_string(),
        source_ref: source_ref.to_string(),
        source_digest,
        content: content.to_string(),
    };
    let candidate = ProcedureJsonCompiler.compile(&source)?;
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["evaluate", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_compile_repository(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_foundry::{
        ArtifactCompiler, ArtifactKind, ArtifactSource, RepositoryManifestCompiler,
    };

    let source = ArtifactSource {
        kind: ArtifactKind::Repository,
        name: body
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("repository-capability")
            .to_string(),
        source_ref: body
            .get("source_ref")
            .and_then(Value::as_str)
            .unwrap_or("repo://inline-manifest")
            .to_string(),
        source_digest: body
            .get("source_digest")
            .and_then(Value::as_str)
            .map(str::to_string),
        content: body
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError(Error::validation(
                    "content must contain repository manifest JSON",
                ))
            })?
            .to_string(),
    };
    let candidate = RepositoryManifestCompiler.compile(&source)?;
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["evaluate", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_compile_paper(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_foundry::{
        ArtifactCompiler, ArtifactKind, ArtifactSource, ReviewedPaperManifestCompiler,
    };

    let source = ArtifactSource {
        kind: ArtifactKind::Paper,
        name: body
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("paper-derived-capability")
            .to_string(),
        source_ref: body
            .get("source_ref")
            .and_then(Value::as_str)
            .unwrap_or("paper://reviewed-manifest")
            .to_string(),
        source_digest: body
            .get("source_digest")
            .and_then(Value::as_str)
            .map(str::to_string),
        content: body
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError(Error::validation(
                    "content must contain reviewed paper manifest JSON",
                ))
            })?
            .to_string(),
    };
    let candidate = ReviewedPaperManifestCompiler.compile(&source)?;
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["independent-evaluation", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_compile_model(State(state): State<AppState>, Json(body): Json<Value>) -> ApiResult {
    use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, ModelManifestCompiler};

    let source = ArtifactSource {
        kind: ArtifactKind::Model,
        name: body
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("model-capability")
            .to_string(),
        source_ref: body
            .get("source_ref")
            .and_then(Value::as_str)
            .unwrap_or("model://declared-manifest")
            .to_string(),
        source_digest: body
            .get("source_digest")
            .and_then(Value::as_str)
            .map(str::to_string),
        content: body
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError(Error::validation(
                    "content must contain structured model manifest JSON",
                ))
            })?
            .to_string(),
    };
    let candidate = ModelManifestCompiler.compile(&source)?;
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["independent-evaluation", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_compile_workflow(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, WorkflowManifestCompiler};

    let source = ArtifactSource {
        kind: ArtifactKind::Workflow,
        name: body
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("workflow-capability")
            .to_string(),
        source_ref: body
            .get("source_ref")
            .and_then(Value::as_str)
            .unwrap_or("workflow://declared-manifest")
            .to_string(),
        source_digest: body
            .get("source_digest")
            .and_then(Value::as_str)
            .map(str::to_string),
        content: body
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError(Error::validation(
                    "content must contain structured workflow manifest JSON",
                ))
            })?
            .to_string(),
    };
    let candidate = WorkflowManifestCompiler.compile(&source)?;
    register_v115_candidate(&state, &candidate.record)?;
    Ok(Json(json!({
        "candidate": candidate,
        "admission": "not-qualified-not-admitted",
        "next": ["evaluate-workflow-contract", "qualify", "release", "site-conformance", "admit"]
    })))
}

async fn v115_instantiate_solution(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> ApiResult {
    use morn_control_plane::ControlPlaneStore;
    use morn_foundry::{instantiate_approved_solution, SolutionInstantiationRequest};

    let guard = state.lock();
    let package = if let Some(package_id) = body
        .get("solution_package_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        guard
            .store
            .load_record::<morn_foundry::SolutionPackage>("solution_package_v115", package_id)?
            .ok_or_else(|| AppError(Error::not_found(format!("SolutionPackage {package_id}"))))?
    } else {
        guard.last_package.clone().ok_or_else(|| {
            AppError(Error::validation(
                "compile an approved SolutionPackage first or provide solution_package_id",
            ))
        })?
    };
    let default_goal = guard
        .last_problem
        .as_ref()
        .map(|problem| problem.objective.clone())
        .unwrap_or_else(|| package.name.clone());

    let goal = body
        .get("goal")
        .and_then(Value::as_str)
        .unwrap_or(&default_goal)
        .to_string();
    let profile_ref = body
        .get("profile_ref")
        .and_then(Value::as_str)
        .unwrap_or("morn.lite@1.0.0")
        .to_string();
    let profile = morn_profile::DomainProfile::from_ref(&profile_ref).ok_or_else(|| {
        AppError(Error::validation(format!(
            "unknown or unsupported Profile reference {profile_ref}"
        )))
    })?;

    let mut request =
        SolutionInstantiationRequest::new(guard.workspace.id.clone(), goal, profile_ref);
    request.site_ref = body
        .get("site_ref")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);
    request.acceptance_ref = body
        .get("acceptance_ref")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);
    request.constraints = body
        .get("constraints")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    request.required_conditions = profile.pre_execution_work_conditions();
    request.required_conditions.extend(
        body.get("required_conditions")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );

    let plan = instantiate_approved_solution(&package, request)?;
    guard.store.save_work_resource(&plan.work)?;

    Ok(Json(json!({
        "plan": plan,
        "state": "proposed",
        "execution_started": false,
        "note": "SolutionPackage was instantiated as canonical Work; capability resolution, qualification/site admission, authority, binding and execution remain explicit gates"
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
    let native_harness = guard
        .native_harness
        .lock()
        .expect("native harness poisoned");
    let dsh_harness = guard.dsh_harness.lock().expect("dsh harness poisoned");
    let pi_harness = guard.pi_harness.lock().expect("pi harness poisoned");
    let dsh_status = match dsh_harness.mode() {
        morn_harness::provider::DshMode::Fixture => "fixture-mode",
        morn_harness::provider::DshMode::Real => "real-sdk-mode",
    };
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
            "native": { "provider": native_harness.provider_name(), "status": "mounted" },
            "dsh": {
                "provider": dsh_harness.provider_name(),
                "status": dsh_status,
                "features": dsh_harness.features()
            },
            "pi": {
                "provider": pi_harness.provider_name(),
                "mode": match pi_harness.mode() {
                    morn_harness::PiMode::Fixture => "fixture",
                    morn_harness::PiMode::Real => "real-rpc",
                },
                "features": pi_harness.features()
            }
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
    let native_harness = guard
        .native_harness
        .lock()
        .expect("native harness poisoned");
    let dsh_harness = guard.dsh_harness.lock().expect("dsh harness poisoned");
    let pi_harness = guard.pi_harness.lock().expect("pi harness poisoned");
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
            "native": native_harness.provider_name(),
            "dsh": {
                "provider": dsh_harness.provider_name(),
                "mode": match dsh_harness.mode() {
                    morn_harness::provider::DshMode::Fixture => "fixture",
                    morn_harness::provider::DshMode::Real => "real-sdk",
                },
                "features": dsh_harness.features(),
                "effect_ceiling": match dsh_harness.mode() {
                    morn_harness::provider::DshMode::Fixture => "fixture-reference",
                    morn_harness::provider::DshMode::Real => "E0-only; E1/E2/E3 via Morn ExternalAction",
                },
                "credential_boundary": match dsh_harness.mode() {
                    morn_harness::provider::DshMode::Fixture => "no live provider credential",
                    morn_harness::provider::DshMode::Real => "scrubbed child environment; explicit MORN_DSH_ENV_PASSTHROUGH only",
                },
                "runtime_health": dsh_harness.runtime_health(),
                "runtime_version": dsh_harness.runtime_version(),
                "runtime_digest": dsh_harness.configured_runtime_digest(),
                "profile_configuration_ref": dsh_harness.configured_profile_configuration_ref(),
                "route_ref": dsh_harness.configured_route_ref(),
                "wire_server_version": dsh_harness.wire_server_version(),
                "execution_environment_ref": dsh_harness.configured_execution_environment_ref(),
                "tool_mediation_required": dsh_harness.mode() == morn_harness::provider::DshMode::Real
            },
            "pi": {
                "provider": pi_harness.provider_name(),
                "mode": match pi_harness.mode() {
                    morn_harness::PiMode::Fixture => "fixture",
                    morn_harness::PiMode::Real => "real-rpc",
                },
                "features": pi_harness.features(),
                "effect_ceiling": match pi_harness.mode() {
                    morn_harness::PiMode::Fixture => "fixture-reference",
                    morn_harness::PiMode::Real => "E0-only; E1/E2/E3 via Morn ExternalAction",
                },
                "credential_boundary": match pi_harness.mode() {
                    morn_harness::PiMode::Fixture => "no live provider credential",
                    morn_harness::PiMode::Real => "scrubbed child environment; explicit MORN_PI_ENV_PASSTHROUGH only",
                },
                "runtime_health": pi_harness.runtime_health(),
                "runtime_version": pi_harness.runtime_version(),
                "runtime_digest": pi_harness.configured_runtime_digest(),
                "route_ref": pi_harness.configured_route_ref(),
                "execution_environment_ref": pi_harness.configured_execution_environment_ref(),
                "tool_mediation_required": pi_harness.mode() == morn_harness::PiMode::Real
            }
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
    let provider_catalog = morn_runtime::reference_provider_catalog();
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
        "provider_catalog": provider_catalog.list(),
        "actor_templates": [ { "id": "generic-actor@1.0", "name": "Generic Actor", "trust": "Verified" } ],
        "harness_templates": [
            { "id": "morn-native@1.0", "name": "Morn Native Harness", "trust": "Verified" },
            { "id": "deepseek-harness@fixture", "name": "DeepSeek Harness Provider", "trust": "ContractVerified" },
            { "id": "pi@fixture", "name": "Pi Harness Provider", "trust": "ContractVerified" }
        ],
        "composition_runtimes": [
            { "id": "cordis@4.0.4", "name": "Cordis Reference Composition Runtime", "trust": "Reference" }
        ],
        "capability_compilers": [
            { "id": "openapi2capability@v11.5", "name": "OpenAPI → Capability Candidate", "trust": "LocalVerified" },
            { "id": "sop2capability@v11.5", "name": "SOP / Procedure → Capability Candidate", "trust": "LocalVerified" },
            { "id": "repo2capability@v11.5", "name": "Repository Manifest → Capability Candidate", "trust": "LocalVerified" },
            { "id": "reviewed-paper2capability@v11.5", "name": "Reviewed Paper Manifest → Capability Candidate", "trust": "LocalVerified" },
            { "id": "model2capability@v11.5", "name": "Pinned Model Manifest → Capability Candidate", "trust": "LocalVerified" },
            { "id": "workflow2capability@v11.5", "name": "Durable Workflow Manifest → Capability Candidate", "trust": "LocalVerified" }
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
    guard.store.save_record_immutable(
        "solution_package_v115",
        pkg.id.as_str(),
        guard.workspace.id.as_str(),
        pkg.created_at.millis(),
        &pkg,
    )?;
    guard.last_package = Some(pkg.clone());
    Ok(Json(json!({
        "package": pkg,
        "manifest": pkg.manifest,
        "persisted": true
    })))
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

#[cfg(test)]
mod workspace_boundary_tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::{WorkResource, WorkSpec};

    #[tokio::test]
    async fn v115_status_exposes_runtime_mode_and_only_deployment_attestations() {
        let state = AppState::new(":memory:").unwrap();
        let Json(body) = v115_status(State(state)).await.unwrap();
        assert_eq!(body["harness_runtime"]["dsh"]["mode"], "fixture");
        assert_eq!(body["harness_runtime"]["pi"]["mode"], "fixture");
        assert_eq!(
            body["execution_environment_attestations"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn v115_work_projection_only_returns_current_workspace_records() {
        let state = AppState::new(":memory:").unwrap();
        {
            let guard = state.lock();
            guard
                .store
                .save_record(
                    "work_resource_v115",
                    "visible",
                    guard.workspace.id.as_str(),
                    1,
                    &json!({"id":"visible"}),
                )
                .unwrap();
            guard
                .store
                .save_record(
                    "work_resource_v115",
                    "foreign",
                    "foreign-workspace",
                    2,
                    &json!({"id":"foreign"}),
                )
                .unwrap();
        }

        let Json(body) = v115_control_plane(State(state)).await.unwrap();
        let works = body["work"].as_array().unwrap();
        assert_eq!(works.len(), 1);
        assert_eq!(works[0]["id"], "visible");
    }

    #[tokio::test]
    async fn resolve_uses_approved_solution_requirements_and_writes_generation_scoped_evidence() {
        use morn_capability::{
            CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityStage, EffectClass,
        };
        use morn_control_plane::ControlPlaneStore;
        use morn_kernel::ids::{
            ApprovedSolutionId, CapabilityId, ProposedSolutionId, SolutionPackageId,
        };
        use morn_kernel::version::Version;

        let state = AppState::new(":memory:").unwrap();
        let work_id = {
            let mut guard = state.lock();
            let mut capability = CapabilityRecord::new(CapabilityManifest::new(
                CapabilityId::generate_with("capability"),
                "draft-agent",
                "deepseek-harness",
                CapabilityKind::Agent,
                EffectClass::E0LifecycleReversible,
            ));
            capability.manifest.provides = vec!["draft.generate".to_string()];
            capability.manifest.provenance.source_ref = "repo://draft-agent".to_string();
            capability.stage = CapabilityStage::Qualified;
            guard.v115_capabilities.push(capability);
            guard.persist_all().unwrap();

            let package = morn_foundry::SolutionPackage {
                id: SolutionPackageId::generate_with("solution-package"),
                name: "draft solution".to_string(),
                version: Version::new(1, 0, 0),
                manifest: json!({
                    "schema":"morn.solution-package/v11.5",
                    "profile_ref":"morn.lite@1.0.0",
                    "site_ref":null,
                    "acceptance_criteria":["reviewed"],
                    "required_capabilities":["draft.generate"],
                    "harness_policy":"provider-neutral",
                    "production_write_allowed":false
                }),
                proposed_solution_id: ProposedSolutionId::generate_with("proposed"),
                approved_solution_id: Some(ApprovedSolutionId::generate_with("approved")),
                created_at: morn_kernel::time::Timestamp::now(),
            };
            guard
                .store
                .save_record(
                    "solution_package_v115",
                    package.id.as_str(),
                    guard.workspace.id.as_str(),
                    package.created_at.millis(),
                    &package,
                )
                .unwrap();
            let plan = morn_foundry::instantiate_approved_solution(
                &package,
                morn_foundry::SolutionInstantiationRequest::new(
                    guard.workspace.id.clone(),
                    "create reviewed draft",
                    "morn.lite@1.0.0",
                ),
            )
            .unwrap();
            let mut work = plan.work;
            guard.store.save_work_resource_cas(&mut work).unwrap();
            work.id.to_string()
        };

        let Json(response) =
            v115_work_resolve(State(state.clone()), Json(json!({"work_id": work_id})))
                .await
                .unwrap();
        assert_eq!(response["resolved"], true);
        assert_eq!(response["work"]["status"]["phase"], "Ready");
        assert_eq!(response["plan"]["uncovered"], json!([]));

        let guard = state.lock();
        let decisions = guard
            .store
            .load_records_in_workspace::<morn_control_plane::CapabilityResolutionDecision>(
                "capability_resolution_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap();
        assert_eq!(decisions.len(), 1);
        let evidence = guard
            .store
            .load_records_in_workspace::<morn_control_plane::ConditionEvidence>(
                "condition_evidence_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap();
        assert!(evidence.iter().any(|item| {
            item.work_ref == work_id
                && item.condition_type == "CapabilityResolved"
                && item.satisfied
                && item
                    .evidence_refs
                    .iter()
                    .any(|reference| reference == decisions[0].id.as_str())
        }));
        assert!(guard
            .store
            .load_records_in_workspace::<morn_runtime::ExecutionBinding>(
                "execution_binding_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn bind_e0_requires_current_resolution_evidence_and_never_starts_execution() {
        use morn_capability::{
            CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityStage, EffectClass,
        };
        use morn_control_plane::{ConditionEvidence, ControlPlaneStore};
        use morn_kernel::ids::CapabilityId;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let (work, capability) = {
            let mut guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "draft a reversible plan",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.spec.required_conditions = vec!["CapabilityResolved".to_string()];
            let mut capability = CapabilityRecord::new(CapabilityManifest::new(
                CapabilityId::generate_with("capability"),
                "fixture-agent",
                "deepseek-harness",
                CapabilityKind::Agent,
                EffectClass::E0LifecycleReversible,
            ));
            capability.manifest.provenance.source_ref = "repo://fixture-agent".to_string();
            capability.stage = CapabilityStage::Qualified;
            guard.v115_capabilities.push(capability.clone());
            guard.persist_all().unwrap();
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let evidence = ConditionEvidence::new(
                &work,
                "CapabilityResolved",
                true,
                "controller://resolver",
                vec![capability.manifest.id.to_string()],
            )
            .unwrap();
            guard
                .store
                .save_condition_evidence(&work, &evidence)
                .unwrap();
            work.status.phase = WorkPhase::Ready;
            work.set_condition(morn_work::control::WorkCondition {
                condition_type: "CapabilityResolved".to_string(),
                status: morn_work::control::ConditionStatus::True,
                reason: "resolved from durable evidence".to_string(),
                evidence_refs: vec![capability.manifest.id.to_string()],
                observed_at: morn_kernel::time::Timestamp::now(),
            });
            guard.store.save_work_resource_cas(&mut work).unwrap();
            (work, capability)
        };

        let Json(response) = v115_work_bind_e0(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "capability_manifest_id": capability.manifest.id.to_string()
            })),
        )
        .await
        .unwrap();

        assert_eq!(response["execution_started"], false);
        assert_eq!(response["binding"]["work_generation"], work.generation);
        assert_eq!(response["binding"]["provider_ref"], "deepseek-harness");
        assert_eq!(response["binding"]["provider_version"], "fixture");

        let guard = state.lock();
        assert_eq!(
            guard
                .store
                .load_records_in_workspace::<morn_runtime::ExecutionBinding>(
                    "execution_binding_v115",
                    guard.workspace.id.as_str(),
                )
                .unwrap()
                .len(),
            1
        );
        assert!(guard
            .store
            .load_records_in_workspace::<morn_harness::ExecutionReceipt>(
                "execution_receipt_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
    }

    #[test]
    fn e0_receipt_classification_fails_closed_on_unknown_outcome_or_scope_cleanup() {
        assert_eq!(
            classify_e0_turn_receipt(false, "outcome-unknown-runtime-reaped", true),
            ("outcome-unknown", false)
        );
        assert_eq!(
            classify_e0_turn_receipt(true, "outcome-unknown", true),
            ("outcome-unknown", false),
            "ambiguous provider state must override an optimistic return value"
        );
        assert_eq!(
            classify_e0_turn_receipt(false, "policy-violation-runtime-reaped", true),
            ("outcome-unknown", false),
            "prohibited executor activity cannot be treated as a definitive no-effect failure"
        );
        assert_eq!(
            classify_e0_turn_receipt(false, "idle-non-success", true),
            ("failed", true)
        );
        assert_eq!(
            classify_e0_turn_receipt(true, "idle", false),
            ("cleanup-failed", true)
        );
        assert_eq!(
            classify_e0_turn_receipt(true, "idle", true),
            ("completed", true)
        );
    }

    #[tokio::test]
    async fn trusted_no_effect_reconciliation_burns_authorization_and_requires_fresh_readiness() {
        use morn_capability::EffectClass;
        use morn_control_plane::ControlPlaneStore;
        use morn_harness::{
            ExecutionReceipt, ExecutorOutcomeDisposition,
            ExecutorOutcomeReconciliationAuthorization,
        };
        use morn_kernel::ids::{PrincipalId, WorkPackageId};
        use morn_runtime::ExecutionBinding;
        use morn_work::control::{ConditionStatus, WorkPhase, WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let (work_id, receipt_id, authorization_id) = {
            let mut guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "ambiguous E0 executor turn",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.status.phase = WorkPhase::Reconciling;
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let mut binding =
                ExecutionBinding::for_work(&work, "cap:test", "deepseek-harness", "fixture");
            binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
            guard.store.save_execution_binding(&work, &binding).unwrap();

            let mut receipt = ExecutionReceipt::new(work.workspace_id.clone(), "session-unknown");
            receipt.work_package_id = Some(work.id.clone());
            receipt.work_generation = Some(work.generation);
            receipt.execution_binding_ref = Some(binding.id.clone());
            receipt.provider_ref = Some(binding.provider_ref.clone());
            receipt.scope_ref = Some("scope://unknown".to_string());
            receipt.outcome = "outcome-unknown".to_string();
            guard.store.save_execution_receipt(&work, &receipt).unwrap();

            let authorization = ExecutorOutcomeReconciliationAuthorization {
                authorization_id: "exec-reconcile-auth-1".to_string(),
                principal_id: PrincipalId::generate_with("deployment-operator"),
                work_package_id: work.id.clone(),
                work_generation: work.generation,
                execution_binding_ref: binding.id,
                execution_receipt_id: receipt.id.clone(),
                disposition: ExecutorOutcomeDisposition::NoEffectConfirmed,
                evidence_refs: vec!["audit://runtime/no-effect/1".to_string()],
                issued_at: morn_kernel::time::Timestamp::from_millis(
                    morn_kernel::time::Timestamp::now()
                        .millis()
                        .saturating_sub(1),
                ),
                valid_until: None,
            };
            let authorization_id = authorization.authorization_id.clone();
            guard
                .executor_reconciliation_authorizations
                .push(authorization);
            (
                work.id.to_string(),
                receipt.id.to_string(),
                authorization_id,
            )
        };

        let Json(response) = v115_work_reconcile_executor_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work_id,
                "execution_receipt_id": receipt_id,
                "reconciliation_authorization_id": authorization_id,
                "reason": "deployment audit confirmed no external effect"
            })),
        )
        .await
        .unwrap();
        assert_eq!(response["authorization_consumed"], true);
        assert_eq!(response["work"]["status"]["phase"], "Blocked");

        {
            let guard = state.lock();
            let work: WorkResource = guard
                .store
                .load_record("work_resource_v115", &work_id)
                .unwrap()
                .unwrap();
            let retry = work
                .status
                .conditions
                .iter()
                .find(|condition| condition.condition_type == "ExecutorRetrySafe")
                .unwrap();
            assert_eq!(retry.status, ConditionStatus::True);
        }

        let replay = v115_work_reconcile_executor_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work_id,
                "execution_receipt_id": receipt_id,
                "reconciliation_authorization_id": authorization_id,
                "reason": "replay"
            })),
        )
        .await;
        assert!(replay.is_err());

        // No-effect reconciliation does not itself bypass current readiness.
        // With no fresh CapabilityResolved witness, a new controller tick must
        // remain fail-closed instead of turning ExecutorRetrySafe into Ready.
        let Json(readiness) =
            v115_work_reconcile(State(state.clone()), Json(json!({ "work_id": work_id })))
                .await
                .unwrap();
        assert_eq!(readiness["work"]["status"]["phase"], "Blocked");
        let guard = state.lock();
        let work: WorkResource = guard
            .store
            .load_record("work_resource_v115", &work_id)
            .unwrap()
            .unwrap();
        assert!(!work.condition_is_true("CapabilityResolved"));
    }

    #[tokio::test]
    async fn observed_or_still_unknown_executor_reconciliation_never_unlocks_retry() {
        use morn_capability::EffectClass;
        use morn_control_plane::ControlPlaneStore;
        use morn_harness::{
            ExecutionReceipt, ExecutorOutcomeDisposition,
            ExecutorOutcomeReconciliationAuthorization,
        };
        use morn_kernel::ids::{PrincipalId, WorkPackageId};
        use morn_runtime::ExecutionBinding;
        use morn_work::control::{ConditionStatus, WorkPhase, WorkResource, WorkSpec};

        for disposition in [
            ExecutorOutcomeDisposition::EffectObserved,
            ExecutorOutcomeDisposition::StillUnknown,
        ] {
            let state = AppState::new(":memory:").unwrap();
            let (work_id, receipt_id, authorization_id) = {
                let mut guard = state.lock();
                let mut work = WorkResource::new(
                    guard.workspace.id.clone(),
                    WorkSpec::new(
                        WorkPackageId::generate_with("work"),
                        "ambiguous executor turn",
                        morn_profile::DomainProfile::lite_v1().canonical_ref(),
                    ),
                );
                work.status.phase = WorkPhase::Reconciling;
                guard.store.save_work_resource_cas(&mut work).unwrap();
                let mut binding = ExecutionBinding::for_work(&work, "cap:test", "pi", "fixture");
                binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
                guard.store.save_execution_binding(&work, &binding).unwrap();
                let mut receipt =
                    ExecutionReceipt::new(work.workspace_id.clone(), "session-unknown");
                receipt.work_package_id = Some(work.id.clone());
                receipt.work_generation = Some(work.generation);
                receipt.execution_binding_ref = Some(binding.id.clone());
                receipt.provider_ref = Some(binding.provider_ref.clone());
                receipt.scope_ref = Some("scope://unknown".to_string());
                receipt.outcome = "outcome-unknown".to_string();
                guard.store.save_execution_receipt(&work, &receipt).unwrap();
                let authorization = ExecutorOutcomeReconciliationAuthorization {
                    authorization_id: format!("auth-{:?}", disposition),
                    principal_id: PrincipalId::generate_with("deployment-operator"),
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    execution_binding_ref: binding.id,
                    execution_receipt_id: receipt.id.clone(),
                    disposition,
                    evidence_refs: vec!["audit://runtime/effect-check".to_string()],
                    issued_at: morn_kernel::time::Timestamp::from_millis(
                        morn_kernel::time::Timestamp::now()
                            .millis()
                            .saturating_sub(1),
                    ),
                    valid_until: None,
                };
                let authorization_id = authorization.authorization_id.clone();
                guard
                    .executor_reconciliation_authorizations
                    .push(authorization);
                (
                    work.id.to_string(),
                    receipt.id.to_string(),
                    authorization_id,
                )
            };
            let Json(response) = v115_work_reconcile_executor_outcome(
                State(state.clone()),
                Json(json!({
                    "work_id": work_id,
                    "execution_receipt_id": receipt_id,
                    "reconciliation_authorization_id": authorization_id,
                    "reason": "authoritative executor reconciliation"
                })),
            )
            .await
            .unwrap();
            assert_eq!(response["work"]["status"]["phase"], "Reconciling");
            let guard = state.lock();
            let work: WorkResource = guard
                .store
                .load_record("work_resource_v115", &work_id)
                .unwrap()
                .unwrap();
            let retry = work
                .status
                .conditions
                .iter()
                .find(|condition| condition.condition_type == "ExecutorRetrySafe")
                .unwrap();
            assert_eq!(retry.status, ConditionStatus::False);
        }
    }

    #[tokio::test]
    async fn readiness_reconcile_cannot_escape_executor_outcome_reconciliation() {
        use morn_control_plane::ControlPlaneStore;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let work_id = {
            let guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "ambiguous executor turn",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.status.phase = WorkPhase::Reconciling;
            guard.store.save_work_resource_cas(&mut work).unwrap();
            work.id.to_string()
        };
        let result =
            v115_work_reconcile(State(state.clone()), Json(json!({ "work_id": work_id }))).await;
        assert!(result.is_err());
        let guard = state.lock();
        let persisted: WorkResource = guard
            .store
            .load_record("work_resource_v115", &work_id)
            .unwrap()
            .unwrap();
        assert_eq!(persisted.status.phase, WorkPhase::Reconciling);
    }

    #[tokio::test]
    async fn requirement_resolution_cannot_escape_running_waiting_or_reconciling_work() {
        use morn_control_plane::ControlPlaneStore;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};

        for phase in [
            WorkPhase::Running,
            WorkPhase::Waiting,
            WorkPhase::Reconciling,
        ] {
            let state = AppState::new(":memory:").unwrap();
            let work_id = {
                let guard = state.lock();
                let mut work = WorkResource::new(
                    guard.workspace.id.clone(),
                    WorkSpec::new(
                        WorkPackageId::generate_with("work"),
                        "preserve execution state",
                        morn_profile::DomainProfile::lite_v1().canonical_ref(),
                    ),
                );
                work.status.phase = phase;
                guard.store.save_work_resource_cas(&mut work).unwrap();
                work.id.to_string()
            };
            let result = v115_work_resolve(State(state), Json(json!({ "work_id": work_id }))).await;
            assert!(result.is_err(), "phase {phase:?} must not be re-resolved");
        }
    }

    #[test]
    fn policy_violation_receipt_forces_reconciliation_and_blocks_blind_e0_retry() {
        use morn_capability::EffectClass;
        use morn_control_plane::{WorkProgressController, WorkProgressInputs};
        use morn_harness::ExecutionReceipt;
        use morn_kernel::ids::{RuntimeBindingId, WorkspaceId};
        use morn_runtime::ExecutionBinding;
        use morn_work::control::{ConditionStatus, WorkPhase, WorkResource, WorkSpec};

        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "governed E0 analysis",
                morn_profile::DomainProfile::lite_v1().canonical_ref(),
            ),
        );
        work.status.phase = WorkPhase::Ready;

        let mut binding =
            ExecutionBinding::for_work(&work, "capability:test", "deepseek-harness", "runtime-1");
        binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);

        let mut receipt = ExecutionReceipt::new(work.workspace_id.clone(), "session-1");
        receipt.work_package_id = Some(work.id.clone());
        receipt.work_generation = Some(work.generation);
        receipt.execution_binding_ref = Some(binding.id.clone());
        receipt.provider_ref = Some(binding.provider_ref.clone());
        receipt.scope_ref = Some("scope://e0".to_string());
        let (outcome, settled) =
            classify_e0_turn_receipt(false, "policy-violation-runtime-reaped", true);
        receipt.outcome = outcome.to_string();
        receipt.ended_at = settled.then(morn_kernel::time::Timestamp::now);

        WorkProgressController.reconcile(
            &mut work,
            &WorkProgressInputs {
                binding: Some(&binding),
                receipt: Some(&receipt),
                ..Default::default()
            },
        );

        assert_eq!(work.status.phase, WorkPhase::Reconciling);
        assert!(!work_phase_allows_new_e0_execution(work.status.phase));
        let unknown = work
            .status
            .conditions
            .iter()
            .find(|condition| condition.condition_type == "ExecutorOutcomeKnown")
            .expect("outcome-known condition must be projected");
        assert_eq!(unknown.status, ConditionStatus::False);
        assert!(receipt.ended_at.is_none());
        assert_eq!(receipt.outcome, "outcome-unknown");

        // Guard against accidentally treating a reconstructed/foreign binding
        // as the original pinned identity in future refactors.
        assert_ne!(
            binding.id,
            RuntimeBindingId::generate_with("different-binding")
        );
    }

    #[test]
    fn real_binding_requires_a_fresh_environment_attestation_at_execution_time() {
        use morn_capability::{CapabilityKind, CapabilityManifest, CapabilityRecord, EffectClass};
        use morn_kernel::ids::{CapabilityId, WorkspaceId};
        use morn_kernel::{ExecutionClass, ExecutionGuarantee};
        use morn_runtime::{
            AttestedExecutionEnvironmentProvider, ExecutionBinding,
            ExecutionEnvironmentAttestation, ExecutionEnvironmentSpec,
        };
        use morn_work::control::{WorkResource, WorkSpec};

        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "run isolated analysis",
                morn_profile::DomainProfile::lite_v1().canonical_ref(),
            ),
        );
        let mut capability = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("capability"),
            "real-agent",
            "deepseek-harness",
            CapabilityKind::Agent,
            EffectClass::E0LifecycleReversible,
        ));
        capability.manifest.execution.minimum_isolation = ExecutionClass::Container;
        capability.manifest.execution.required_guarantees = vec![
            ExecutionGuarantee::ProcessBoundary,
            ExecutionGuarantee::RuntimeAttestation,
            ExecutionGuarantee::ToolMediation,
        ];
        let profile = morn_profile::DomainProfile::lite_v1();

        let now = morn_kernel::time::Timestamp::from_millis(100);
        let mut environments =
            AttestedExecutionEnvironmentProvider::new("deployment-attestor").unwrap();
        environments
            .register_attestation(ExecutionEnvironmentAttestation {
                environment_ref: "env://real/a".to_string(),
                provider: "deployment-attestor".to_string(),
                isolation: ExecutionClass::Container,
                attested_spec: ExecutionEnvironmentSpec {
                    minimum_isolation: ExecutionClass::Container,
                    required_guarantees: vec![
                        ExecutionGuarantee::ProcessBoundary,
                        ExecutionGuarantee::RuntimeAttestation,
                        ExecutionGuarantee::ToolMediation,
                    ],
                    ..Default::default()
                },
                runtime_identities: vec![format!(
                    "deepseek-harness@runtime-1#sha256:{}",
                    "d".repeat(64)
                )],
                evidence_refs: vec!["attestation://real/a".to_string()],
                observed_at: morn_kernel::time::Timestamp::from_millis(10),
                valid_until: morn_kernel::time::Timestamp::from_millis(110),
            })
            .unwrap();

        let mut binding = ExecutionBinding::for_work(
            &work,
            capability.manifest.id.to_string(),
            "deepseek-harness",
            "runtime-1",
        );
        binding.provider_digest = Some(format!("sha256:{}", "d".repeat(64)));
        binding
            .pin_execution_environment(
                "env://real/a",
                ExecutionClass::Container,
                vec![
                    ExecutionGuarantee::ProcessBoundary,
                    ExecutionGuarantee::RuntimeAttestation,
                    ExecutionGuarantee::ToolMediation,
                ],
            )
            .unwrap();

        ensure_live_environment_for_binding(&binding, &capability, &profile, &environments, now)
            .unwrap();
        assert!(ensure_live_environment_for_binding(
            &binding,
            &capability,
            &profile,
            &environments,
            morn_kernel::time::Timestamp::from_millis(111),
        )
        .is_err());
    }

    #[tokio::test]
    async fn execute_e0_persists_harness_receipt_without_inventing_outcome_or_acceptance() {
        use morn_capability::{
            CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityStage, EffectClass,
        };
        use morn_control_plane::ControlPlaneStore;
        use morn_kernel::ids::CapabilityId;
        use morn_runtime::ExecutionBinding;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let (work, binding) = {
            let mut guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "prepare an evidence-backed draft",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.status.phase = WorkPhase::Ready;
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let mut capability = CapabilityRecord::new(CapabilityManifest::new(
                CapabilityId::generate_with("capability"),
                "fixture-agent",
                "deepseek-harness",
                CapabilityKind::Agent,
                EffectClass::E0LifecycleReversible,
            ));
            capability.manifest.provenance.source_ref = "repo://fixture-agent".to_string();
            capability.stage = CapabilityStage::Qualified;
            let manifest_id = capability.manifest.id.to_string();
            guard.v115_capabilities.push(capability);
            guard.persist_all().unwrap();

            let mut binding =
                ExecutionBinding::for_work(&work, manifest_id, "deepseek-harness", "fixture");
            binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
            guard.store.save_execution_binding(&work, &binding).unwrap();
            (work, binding)
        };

        let Json(response) = v115_work_execute_e0(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "binding_id": binding.id.to_string(),
                "input": "produce a draft only"
            })),
        )
        .await
        .unwrap();

        assert_eq!(response["executor_status"], "completed");
        assert_eq!(response["business_outcome_observed"], false);
        assert_eq!(response["independent_acceptance"], false);
        assert_eq!(response["work"]["status"]["phase"], "Waiting");

        let guard = state.lock();
        let receipts = guard
            .store
            .load_records_in_workspace::<morn_harness::ExecutionReceipt>(
                "execution_receipt_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            receipts[0].execution_binding_ref.as_ref(),
            Some(&binding.id)
        );
        assert_eq!(receipts[0].outcome, "completed");
        assert!(guard
            .store
            .load_records_in_workspace::<Value>(
                "observed_outcome_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
        assert!(guard
            .store
            .load_records_in_workspace::<Value>(
                "acceptance_decision_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn execute_e0_rechecks_capability_revocation_before_new_turn() {
        use morn_capability::{
            CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityStage, EffectClass,
        };
        use morn_control_plane::ControlPlaneStore;
        use morn_kernel::ids::CapabilityId;
        use morn_runtime::ExecutionBinding;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let (work, binding, manifest_id) = {
            let mut guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "draft only",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.status.phase = WorkPhase::Ready;
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let mut capability = CapabilityRecord::new(CapabilityManifest::new(
                CapabilityId::generate_with("capability"),
                "revocable-fixture-agent",
                "deepseek-harness",
                CapabilityKind::Agent,
                EffectClass::E0LifecycleReversible,
            ));
            capability.manifest.provenance.source_ref = "repo://fixture-agent".to_string();
            capability.stage = CapabilityStage::Qualified;
            let manifest_id = capability.manifest.id.to_string();
            guard.v115_capabilities.push(capability);

            let mut binding = ExecutionBinding::for_work(
                &work,
                manifest_id.clone(),
                "deepseek-harness",
                "fixture",
            );
            binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
            guard.store.save_execution_binding(&work, &binding).unwrap();
            (work, binding, manifest_id)
        };

        {
            let mut guard = state.lock();
            let capability = guard
                .v115_capabilities
                .iter_mut()
                .find(|record| record.manifest.id.as_str() == manifest_id)
                .unwrap();
            capability.stage = CapabilityStage::Suspended;
        }

        let result = v115_work_execute_e0(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "binding_id": binding.id.to_string(),
                "input": "must not run after revocation"
            })),
        )
        .await;
        assert!(result.is_err());

        let guard = state.lock();
        assert!(guard
            .store
            .load_records_in_workspace::<morn_harness::ExecutionReceipt>(
                "execution_receipt_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn deployment_source_binding_is_required_before_authoritative_outcome_observation() {
        use morn_control_plane::ControlPlaneStore;
        use morn_integration::{
            ConflictPolicy, SourceObservationAttestation, SourceOfTruthBinding,
            SourceOfTruthBindingId, TruthAuthorityKind,
        };
        use morn_kernel::time::Timestamp;
        use morn_work::control::{WorkResource, WorkSpec};

        let state = AppState::new(":memory:").unwrap();
        let (work, catalog_id) = {
            let mut guard = state.lock();
            let binding = SourceOfTruthBinding {
                id: SourceOfTruthBindingId::generate_with("sot-catalog"),
                site_ref: None,
                source_ref: "system://orders".to_string(),
                authority_kind: TruthAuthorityKind::SystemOfRecord,
                authoritative_fact_types: vec!["delivery.status".to_string()],
                key_mapping_ref: "mapping://orders@1".to_string(),
                query_capability_ref: "capability://orders.read@1".to_string(),
                freshness_sla_ms: Some(30_000),
                conflict_policy: ConflictPolicy::ReconcileBeforeUse,
                version_ref: "binding:v1".to_string(),
                created_at: Timestamp::now(),
            };
            let catalog_id = binding.id.to_string();
            guard.source_of_truth_catalog.push(binding);
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "verify delivery outcome",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            guard.store.save_work_resource_cas(&mut work).unwrap();
            (work, catalog_id)
        };

        let unauthorized = v115_work_observe_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "observation_attestation_id": "missing-observation-attestation"
            })),
        )
        .await;
        assert!(unauthorized.is_err());

        let Json(bound) = v115_work_bind_source_of_truth(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "catalog_binding_id": catalog_id
            })),
        )
        .await
        .unwrap();
        let work_binding_id = bound["binding"]["id"].as_str().unwrap().to_string();
        assert_ne!(work_binding_id, catalog_id);
        assert_eq!(
            bound["condition_evidence"]["condition_type"],
            "SourceOfTruthBound"
        );

        {
            let mut guard = state.lock();
            let binding_id = SourceOfTruthBindingId::new(work_binding_id.clone());
            let workspace_id = guard.workspace.id.clone();
            guard
                .source_observation_attestations
                .push(SourceObservationAttestation {
                    attestation_id: "obs-outside".to_string(),
                    workspace_id: workspace_id.clone(),
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    source_binding_id: binding_id.clone(),
                    fact_type: "delivery.status".to_string(),
                    objective: "observe delivery".to_string(),
                    source_ref: "system://orders-evil/42".to_string(),
                    observed_facts: json!({"status":"complete"}),
                    evidence_refs: vec!["system://orders-evil/42/receipt".to_string()],
                    observed_at: Timestamp::now(),
                    valid_until: None,
                });
            guard
                .source_observation_attestations
                .push(SourceObservationAttestation {
                    attestation_id: "obs-grounded".to_string(),
                    workspace_id,
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    source_binding_id: binding_id,
                    fact_type: "delivery.status".to_string(),
                    objective: "observe delivery".to_string(),
                    source_ref: "system://orders/42".to_string(),
                    observed_facts: json!({"status":"complete"}),
                    evidence_refs: vec!["system://orders/42/receipt".to_string()],
                    observed_at: Timestamp::now(),
                    valid_until: None,
                });
        }

        {
            let mut guard = state.lock();
            let binding_id = SourceOfTruthBindingId::new(work_binding_id.clone());
            let workspace_id = guard.workspace.id.clone();
            guard
                .source_observation_attestations
                .push(SourceObservationAttestation {
                    attestation_id: "obs-stale".to_string(),
                    workspace_id,
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    source_binding_id: binding_id,
                    fact_type: "delivery.status".to_string(),
                    objective: "observe stale delivery".to_string(),
                    source_ref: "system://orders/42".to_string(),
                    observed_facts: json!({"status":"complete"}),
                    evidence_refs: vec!["system://orders/42/stale-receipt".to_string()],
                    observed_at: Timestamp::from_millis(
                        Timestamp::now().millis().saturating_sub(30_001),
                    ),
                    valid_until: Some(Timestamp::from_millis(
                        Timestamp::now().millis().saturating_add(60_000),
                    )),
                });
        }
        let stale = v115_work_observe_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "observation_attestation_id": "obs-stale"
            })),
        )
        .await;
        assert!(stale.is_err());

        let Json(catalog) = v115_source_observations(State(state.clone()))
            .await
            .unwrap();
        let listed_ids: Vec<_> = catalog["observations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["attestation_id"].as_str())
            .collect();
        assert!(!listed_ids.contains(&"obs-stale"));
        assert!(listed_ids.contains(&"obs-grounded"));

        let outside = v115_work_observe_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "observation_attestation_id": "obs-outside"
            })),
        )
        .await;
        assert!(outside.is_err());

        let Json(observed) = v115_work_observe_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "observation_attestation_id": "obs-grounded"
            })),
        )
        .await
        .unwrap();
        assert_eq!(observed["work"]["status"]["phase"], "Delivered");
        assert_eq!(observed["independent_acceptance"], false);
        assert_eq!(observed["caller_supplied_world_facts"], false);
        assert_eq!(observed["observation_attestation_id"], "obs-grounded");
        assert_eq!(observed["outcome"]["source_binding_ref"], work_binding_id);
        assert_eq!(
            observed["outcome"]["source_attestation_ref"],
            "obs-grounded"
        );
        assert_eq!(observed["outcome"]["fact_type"], "delivery.status");

        let guard = state.lock();
        assert_eq!(
            guard
                .store
                .load_records_in_workspace::<morn_world::ObservedOutcome>(
                    "observed_outcome_v115",
                    guard.workspace.id.as_str(),
                )
                .unwrap()
                .len(),
            1
        );
        assert!(guard
            .store
            .load_records_in_workspace::<morn_work::acceptance_decision::AcceptanceDecision>(
                "acceptance_decision_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap()
            .is_empty());
        assert!(guard
            .store
            .load_record::<Value>(
                "source_observation_attestation_consumed_v115",
                "obs-grounded",
            )
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn review_outcome_accepts_only_existing_source_grounded_work_outcome() {
        use morn_control_plane::ControlPlaneStore;
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};

        let state = AppState::new(":memory:").unwrap();
        let (work, grounded, ungrounded, reviewer_id, authorization_id) = {
            let mut guard = state.lock();
            let reviewer_id = morn_kernel::ids::PrincipalId::generate_with("reviewer");
            guard
                .acceptance_reviewers
                .push(morn_work::acceptance::AcceptanceReviewerAttestation {
                    principal_id: reviewer_id.clone(),
                    acting_roles: vec!["independent-reviewer".to_string()],
                    evidence_refs: vec!["iam://reviewers/alice".to_string()],
                    observed_at: morn_kernel::time::Timestamp::now(),
                    valid_until: None,
                });
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "verify delivered result",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.status.phase = WorkPhase::Waiting;
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let mut grounded = ObservedOutcome::new(
                guard.workspace.id.clone(),
                work.id.clone(),
                "reviewed external result",
                OutcomeSourceKind::ExternalSystem,
                "system://authoritative/result-1",
                json!({"status":"complete"}),
            );
            grounded
                .evidence_refs
                .push("system://authoritative/result-1/receipt".to_string());
            let source_binding = morn_integration::SourceOfTruthBinding {
                id: morn_integration::SourceOfTruthBindingId::generate_with("sot"),
                site_ref: work.spec.site_ref.clone(),
                source_ref: "system://authoritative".to_string(),
                authority_kind: morn_integration::TruthAuthorityKind::SystemOfRecord,
                authoritative_fact_types: vec!["delivery.status".to_string()],
                key_mapping_ref: "mapping://authoritative@1".to_string(),
                query_capability_ref: "capability://authoritative.read@1".to_string(),
                freshness_sla_ms: None,
                conflict_policy: morn_integration::ConflictPolicy::ReconcileBeforeUse,
                version_ref: "binding:test-v1".to_string(),
                created_at: morn_kernel::time::Timestamp::now(),
            };
            guard
                .store
                .save_source_of_truth_binding(&work, &source_binding)
                .unwrap();
            let observation_attestation_id = "observation-attestation://result-1";
            grounded
                .pin_source_provenance(
                    source_binding.id.to_string(),
                    observation_attestation_id,
                    "delivery.status",
                )
                .unwrap();
            guard
                .store
                .save_record_immutable(
                    "source_observation_attestation_consumed_v115",
                    observation_attestation_id,
                    work.workspace_id.as_str(),
                    grounded.observed_at.millis(),
                    &json!({
                        "attestation_id": observation_attestation_id,
                        "work_id": work.id,
                        "source_binding_id": source_binding.id,
                        "fact_type": "delivery.status",
                        "outcome_id": grounded.id,
                        "consumed_at": grounded.observed_at
                    }),
                )
                .unwrap();
            guard.store.save_observed_outcome(&work, &grounded).unwrap();

            let ungrounded = ObservedOutcome::new(
                guard.workspace.id.clone(),
                work.id.clone(),
                "model assertion only",
                OutcomeSourceKind::ValidatedComputation,
                "model://claim",
                json!({"status":"complete"}),
            );
            guard
                .store
                .save_observed_outcome(&work, &ungrounded)
                .unwrap();

            let authorization_id = "review-auth-grounded-1".to_string();
            guard.acceptance_review_authorizations.push(
                morn_work::acceptance::AcceptanceReviewAuthorization {
                    authorization_id: authorization_id.clone(),
                    principal_id: reviewer_id.clone(),
                    acting_role: "independent-reviewer".to_string(),
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    outcome_id: grounded.id.clone(),
                    disposition: morn_work::acceptance_decision::AcceptanceDisposition::Accept,
                    evidence_refs: vec![
                        "iam://review-authorizations/review-auth-grounded-1".to_string()
                    ],
                    issued_at: morn_kernel::time::Timestamp::now(),
                    valid_until: None,
                },
            );
            (work, grounded, ungrounded, reviewer_id, authorization_id)
        };

        let weak = v115_work_review_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": ungrounded.id.to_string(),
                "disposition": "accept",
                "review_authorization_id": "review-auth-not-for-ungrounded",
                "reviewer_principal_id": reviewer_id.to_string(),
                "acting_role": "independent-reviewer",
                "reason": "must not accept an ungrounded assertion",
                "evidence_refs": ["review://ticket-1"]
            })),
        )
        .await;
        assert!(weak.is_err());

        let Json(response) = v115_work_review_outcome(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": grounded.id.to_string(),
                "disposition": "accept",
                "review_authorization_id": authorization_id,
                "reviewer_principal_id": reviewer_id.to_string(),
                "acting_role": "independent-reviewer",
                "reason": "authoritative source evidence satisfies the reviewed acceptance criteria",
                "evidence_refs": ["review://ticket-2"]
            })),
        )
        .await
        .unwrap();
        assert_eq!(response["work"]["status"]["phase"], "Accepted");
        assert_eq!(response["business_outcome_source_grounded"], true);

        let guard = state.lock();
        let decisions = guard
            .store
            .load_records_in_workspace::<morn_work::acceptance_decision::AcceptanceDecision>(
                "acceptance_decision_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].outcome_refs, vec![grounded.id]);
        assert_eq!(decisions[0].decided_by, reviewer_id);
        assert!(decisions[0]
            .evidence_refs
            .contains(&"iam://reviewers/alice".to_string()));
        assert!(decisions[0]
            .evidence_refs
            .contains(&"iam://review-authorizations/review-auth-grounded-1".to_string()));
        assert!(guard
            .store
            .load_record::<Value>(
                "acceptance_review_authorization_consumed_v115",
                "review-auth-grounded-1",
            )
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn business_evidence_recovery_projects_persisted_acceptance_without_replaying_execution()
    {
        use morn_control_plane::ControlPlaneStore;
        use morn_integration::{
            ConflictPolicy, SourceOfTruthBinding, SourceOfTruthBindingId, TruthAuthorityKind,
        };
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_work::control::{WorkResource, WorkSpec};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};

        let state = AppState::new(":memory:").unwrap();
        let work_id = {
            let guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "recover accepted delivery projection",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let binding = SourceOfTruthBinding {
                id: SourceOfTruthBindingId::generate_with("sot"),
                site_ref: work.spec.site_ref.clone(),
                source_ref: "erp://delivery".to_string(),
                authority_kind: TruthAuthorityKind::SystemOfRecord,
                authoritative_fact_types: vec!["delivery.status".to_string()],
                key_mapping_ref: "mapping://delivery@1".to_string(),
                query_capability_ref: "capability://delivery.read@1".to_string(),
                freshness_sla_ms: None,
                conflict_policy: ConflictPolicy::ReconcileBeforeUse,
                version_ref: "binding:test-v1".to_string(),
                created_at: morn_kernel::time::Timestamp::now(),
            };
            guard
                .store
                .save_source_of_truth_binding(&work, &binding)
                .unwrap();

            let mut outcome = ObservedOutcome::new(
                work.workspace_id.clone(),
                work.id.clone(),
                "authoritative delivery completed",
                OutcomeSourceKind::ExternalSystem,
                "erp://delivery/42",
                json!({"status":"delivered"}),
            );
            outcome.pin_work_generation(work.generation).unwrap();
            outcome
                .evidence_refs
                .push("erp://delivery/42/receipt".to_string());
            let observation_attestation_id = "observation-attestation://recovery-42";
            outcome
                .pin_source_provenance(
                    binding.id.to_string(),
                    observation_attestation_id,
                    "delivery.status",
                )
                .unwrap();
            guard
                .store
                .save_record_immutable(
                    "source_observation_attestation_consumed_v115",
                    observation_attestation_id,
                    work.workspace_id.as_str(),
                    outcome.observed_at.millis(),
                    &json!({
                        "attestation_id": observation_attestation_id,
                        "work_id": work.id,
                        "source_binding_id": binding.id,
                        "fact_type": "delivery.status",
                        "outcome_id": outcome.id,
                        "consumed_at": outcome.observed_at
                    }),
                )
                .unwrap();
            guard.store.save_observed_outcome(&work, &outcome).unwrap();

            let mut acceptance = AcceptanceDecision::new(
                work.id.clone(),
                morn_kernel::ids::AcceptanceSpecId::generate_with("acceptance"),
                AcceptanceDisposition::Accept,
                morn_kernel::ids::PrincipalId::generate_with("reviewer"),
                "independent-reviewer",
                "deployment-attested result accepted",
            );
            acceptance.pin_work_generation(work.generation).unwrap();
            acceptance.outcome_refs.push(outcome.id.clone());
            acceptance
                .evidence_refs
                .push("review://signed/recovery-42".to_string());
            ControlPlaneStore::save_acceptance_decision(&guard.store, &work, &acceptance).unwrap();

            // Intentionally do not mutate Work: this models the post-evidence,
            // pre-projection crash/CAS window.
            work.id.to_string()
        };

        let Json(response) = v115_work_reconcile_business_evidence(
            State(state.clone()),
            Json(json!({"work_id": work_id})),
        )
        .await
        .unwrap();
        assert_eq!(response["work"]["status"]["phase"], "Accepted");
        assert_eq!(response["replayed_external_execution"], false);
        assert_eq!(response["reconsumed_authorization"], false);

        let forbidden = v115_work_reconcile_business_evidence(
            State(state),
            Json(json!({
                "work_id": work_id,
                "outcome_id": "caller-selected-outcome"
            })),
        )
        .await;
        assert!(forbidden.is_err());
    }

    #[tokio::test]
    async fn value_assessment_requires_exact_acceptance_and_real_site_for_customer_claim() {
        use morn_assurance::{
            CustomerValueAttestation, CustomerValueMetric, EvidenceClaim, EvidenceClass,
        };
        use morn_control_plane::ControlPlaneStore;
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
        use morn_work::control::{WorkPhase, WorkResource, WorkSpec};
        use morn_world::{ObservedOutcome, OutcomeSourceKind};

        let state = AppState::new(":memory:").unwrap();
        let (work, outcome, acceptance) = {
            let guard = state.lock();
            let mut work = WorkResource::new(
                guard.workspace.id.clone(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "measure accepted delivery outcome",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            // Persist the initial Work before evidence. Accepted is a
            // projection derived only after the outcome and decision exist.
            guard.store.save_work_resource_cas(&mut work).unwrap();

            let mut outcome = ObservedOutcome::new(
                guard.workspace.id.clone(),
                work.id.clone(),
                "accepted authoritative delivery result",
                OutcomeSourceKind::ExternalSystem,
                "erp://delivery/42",
                json!({"late_minutes":0}),
            );
            outcome.pin_work_generation(work.generation).unwrap();
            outcome
                .evidence_refs
                .push("erp://delivery/42/receipt".to_string());
            let source_binding = morn_integration::SourceOfTruthBinding {
                id: morn_integration::SourceOfTruthBindingId::generate_with("sot"),
                site_ref: work.spec.site_ref.clone(),
                source_ref: "erp://delivery".to_string(),
                authority_kind: morn_integration::TruthAuthorityKind::SystemOfRecord,
                authoritative_fact_types: vec!["delivery.metrics".to_string()],
                key_mapping_ref: "mapping://delivery@1".to_string(),
                query_capability_ref: "capability://delivery.read@1".to_string(),
                freshness_sla_ms: None,
                conflict_policy: morn_integration::ConflictPolicy::ReconcileBeforeUse,
                version_ref: "binding:test-v1".to_string(),
                created_at: morn_kernel::time::Timestamp::now(),
            };
            guard
                .store
                .save_source_of_truth_binding(&work, &source_binding)
                .unwrap();
            let observation_attestation_id = "observation-attestation://delivery-42";
            outcome
                .pin_source_provenance(
                    source_binding.id.to_string(),
                    observation_attestation_id,
                    "delivery.metrics",
                )
                .unwrap();
            guard
                .store
                .save_record_immutable(
                    "source_observation_attestation_consumed_v115",
                    observation_attestation_id,
                    work.workspace_id.as_str(),
                    outcome.observed_at.millis(),
                    &json!({
                        "attestation_id": observation_attestation_id,
                        "work_id": work.id,
                        "source_binding_id": source_binding.id,
                        "fact_type": "delivery.metrics",
                        "outcome_id": outcome.id,
                        "consumed_at": outcome.observed_at
                    }),
                )
                .unwrap();
            guard.store.save_observed_outcome(&work, &outcome).unwrap();

            let mut acceptance = AcceptanceDecision::new(
                work.id.clone(),
                morn_kernel::ids::AcceptanceSpecId::generate_with("acceptance"),
                AcceptanceDisposition::Accept,
                morn_kernel::ids::PrincipalId::generate_with("reviewer"),
                "independent-reviewer",
                "source-grounded outcome accepted",
            );
            acceptance.pin_work_generation(work.generation).unwrap();
            acceptance.outcome_refs.push(outcome.id.clone());
            acceptance
                .evidence_refs
                .push("review://signed/42".to_string());
            ControlPlaneStore::save_acceptance_decision(&guard.store, &work, &acceptance).unwrap();
            work.status.phase = WorkPhase::Accepted;
            guard.store.save_work_resource_cas(&mut work).unwrap();
            (work, outcome, acceptance)
        };

        let Json(operational) = v115_work_assess_value(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": outcome.id.to_string(),
                "acceptance_id": acceptance.id.to_string(),
                "evidence_class": "observed-operational",
                "evidence_refs": ["metric://delivery/42"],
                "kpis": {"late_minutes_delta": -12.0}
            })),
        )
        .await
        .unwrap();
        assert_eq!(
            operational["value_assessment"]["work_generation"],
            work.generation
        );
        assert_eq!(operational["customer_validated"], false);

        let subject = value_claim_subject(&work.id, work.generation, &outcome.id);
        let value_attestation_id = "customer-value-attestation-42";
        {
            let mut guard = state.lock();
            let workspace_id = guard.workspace.id.clone();
            guard
                .customer_value_attestations
                .push(CustomerValueAttestation {
                    attestation_id: value_attestation_id.to_string(),
                    workspace_id,
                    work_package_id: work.id.clone(),
                    work_generation: work.generation,
                    outcome_id: outcome.id.clone(),
                    acceptance_id: acceptance.id.clone(),
                    baseline_ref: Some("baseline://customer/42".to_string()),
                    kpis: vec![CustomerValueMetric {
                        name: "late_minutes_delta".to_string(),
                        value: -12.0,
                    }],
                    evidence_refs: vec!["metric://customer/42/signed".to_string()],
                    issuer: "customer-site-owner".to_string(),
                    observed_at: morn_kernel::time::Timestamp::now(),
                    valid_until: None,
                });
        }

        let caller_kpi_override = v115_work_assess_value(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": outcome.id.to_string(),
                "acceptance_id": acceptance.id.to_string(),
                "evidence_class": "customer-validated",
                "customer_value_attestation_id": value_attestation_id,
                "kpis": {"late_minutes_delta": -999.0}
            })),
        )
        .await;
        assert!(caller_kpi_override.is_err());

        let blocked = v115_work_assess_value(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": outcome.id.to_string(),
                "acceptance_id": acceptance.id.to_string(),
                "evidence_class": "customer-validated",
                "customer_value_attestation_id": value_attestation_id
            })),
        )
        .await;
        assert!(blocked.is_err());

        {
            let mut guard = state.lock();
            guard
                .evidence_ledger
                .append(
                    EvidenceClaim::proven(
                        subject.clone(),
                        EvidenceClass::RealSite,
                        vec!["customer://site-pilot/42".to_string()],
                        "customer-site-owner",
                        "read-only customer pilot validated the KPI",
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let Json(customer) = v115_work_assess_value(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": outcome.id.to_string(),
                "acceptance_id": acceptance.id.to_string(),
                "evidence_class": "customer-validated",
                "customer_value_attestation_id": value_attestation_id
            })),
        )
        .await
        .unwrap();
        assert_eq!(customer["value_subject"], subject);
        assert_eq!(customer["customer_validated"], true);
        assert_eq!(
            customer["customer_value_attestation_id"],
            value_attestation_id
        );
        assert_eq!(
            customer["value_assessment"]["customer_value_attestation_ref"],
            value_attestation_id
        );
        assert_eq!(
            customer["value_assessment"]["baseline_ref"],
            "baseline://customer/42"
        );
        assert_eq!(
            customer["value_assessment"]["kpis"][0],
            json!(["late_minutes_delta", -12.0])
        );
        assert_eq!(
            customer["real_site_evidence_refs"],
            json!(["customer://site-pilot/42"])
        );

        let replay = v115_work_assess_value(
            State(state.clone()),
            Json(json!({
                "work_id": work.id.to_string(),
                "outcome_id": outcome.id.to_string(),
                "acceptance_id": acceptance.id.to_string(),
                "evidence_class": "customer-validated",
                "customer_value_attestation_id": value_attestation_id
            })),
        )
        .await;
        assert!(replay.is_err());

        let customer_assessment_id = customer["value_assessment"]["id"]
            .as_str()
            .unwrap()
            .to_string();

        let Json(control) = v115_control_plane(State(state.clone())).await.unwrap();
        let support = control["value_assessment_support"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["assessment_id"] == customer_assessment_id)
            .unwrap();
        assert_eq!(support["currently_supported"], true);
        assert_eq!(support["current_real_site_state"], "proven");

        {
            let mut guard = state.lock();
            guard
                .evidence_ledger
                .append(
                    EvidenceClaim::revoked(
                        subject.clone(),
                        EvidenceClass::RealSite,
                        vec!["customer://site-pilot/42/revocation".to_string()],
                        "customer-site-owner",
                        "site evidence authorization was withdrawn",
                    )
                    .unwrap(),
                )
                .unwrap();
        }

        let Json(control) = v115_control_plane(State(state.clone())).await.unwrap();
        let support = control["value_assessment_support"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["assessment_id"] == customer_assessment_id)
            .unwrap();
        assert_eq!(support["currently_supported"], false);
        assert_eq!(support["current_real_site_state"], "revoked");

        let guard = state.lock();
        let values = guard
            .store
            .load_records_in_workspace::<morn_work::value::ValueAssessment>(
                "value_assessment_v115",
                guard.workspace.id.as_str(),
            )
            .unwrap();
        assert_eq!(values.len(), 2);
        assert!(values
            .iter()
            .all(|value| value.work_generation == work.generation));
    }

    #[tokio::test]
    async fn reconcile_cannot_mutate_work_from_another_workspace() {
        let state = AppState::new(":memory:").unwrap();
        let work = {
            let guard = state.lock();
            let mut work = WorkResource::new(
                WorkspaceId::generate(),
                WorkSpec::new(
                    WorkPackageId::generate_with("work"),
                    "foreign workspace goal",
                    morn_profile::DomainProfile::lite_v1().canonical_ref(),
                ),
            );
            work.resource_version = 1;
            guard
                .store
                .save_record_cas(
                    "work_resource_v115",
                    work.id.as_str(),
                    work.workspace_id.as_str(),
                    work.created_at.millis(),
                    0,
                    &work,
                )
                .unwrap();
            work
        };

        let result = v115_work_reconcile(
            State(state.clone()),
            Json(json!({"work_id":work.id.to_string()})),
        )
        .await;
        assert_eq!(
            result.unwrap_err().into_response().status(),
            StatusCode::NOT_FOUND
        );
        let guard = state.lock();
        assert_eq!(
            guard
                .store
                .record_revision("work_resource_v115", work.id.as_str())
                .unwrap(),
            Some(1)
        );
        assert!(guard.store.pending_outbox_events(10).unwrap().is_empty());
    }
}
