//! One-shot live Harness provider evidence gate.
//!
//! This binary deliberately proves only the external executor path. It does not
//! create a customer Outcome, AcceptanceDecision, RealSite claim or production
//! authorization.

use std::process::ExitCode;

use morn_app::AppState;
use morn_harness::provider::{DshMode, HarnessProvider, HarnessRuntimeHealthState};
use morn_harness::{run_harness_smoke, PiMode, RuntimeContext};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ActorInstanceId, RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_runtime::ExecutionEnvironmentAttestation;
use serde_json::{json, Value};

fn live_context(
    workspace_id: morn_kernel::ids::WorkspaceId,
    attestation: &ExecutionEnvironmentAttestation,
) -> Result<RuntimeContext> {
    if !attestation.active_at(Timestamp::now()) {
        return Err(Error::external(
            "configured execution-environment attestation is missing, invalid, not-yet-valid or expired",
        ));
    }
    let mut context = RuntimeContext::new(
        workspace_id,
        ActorInstanceId::generate_with("provider-smoke"),
        WorkPackageId::generate_with("provider-smoke"),
    )
    .with_work_binding(1, RuntimeBindingId::generate_with("provider-smoke"))
    .map_err(Error::validation)?
    .with_execution_environment(
        attestation.environment_ref.clone(),
        attestation.isolation,
        attestation.attested_spec.required_guarantees.clone(),
    )
    .map_err(Error::validation)?;
    context.policy_snapshot_version = Some("morn-provider-live-smoke/v1".to_string());
    context.provenance_refs = attestation.evidence_refs.clone();
    context
        .provenance_refs
        .push("gate://morn/provider-live-smoke/v1".to_string());
    Ok(context)
}

fn find_attestation(
    attestations: &[ExecutionEnvironmentAttestation],
    environment_ref: &str,
) -> Result<ExecutionEnvironmentAttestation> {
    attestations
        .iter()
        .find(|item| item.environment_ref == environment_ref)
        .cloned()
        .ok_or_else(|| {
            Error::external(format!(
                "no deployment-owned execution attestation exists for {environment_ref:?}"
            ))
        })
}

fn run_gate() -> Result<Value> {
    let provider_name = std::env::var("MORN_PROVIDER_SMOKE_PROVIDER").map_err(|_| {
        Error::validation("MORN_PROVIDER_SMOKE_PROVIDER is required (deepseek-harness or pi)")
    })?;
    let db_path =
        std::env::var("MORN_PROVIDER_SMOKE_DB").unwrap_or_else(|_| ":memory:".to_string());
    let state = AppState::new(&db_path)?;

    let (workspace_id, dsh, pi, attestations) = {
        let guard = state.lock();
        (
            guard.workspace.id.clone(),
            guard.dsh_harness.clone(),
            guard.pi_harness.clone(),
            guard.execution_environments.attestations(),
        )
    };

    match provider_name.as_str() {
        "deepseek-harness" | "dsh" => {
            let mut provider = dsh
                .lock()
                .map_err(|_| Error::internal("DSH provider lock poisoned"))?;
            if provider.mode() != DshMode::Real {
                return Err(Error::invalid_state(
                    "live DSH smoke requires MORN_DSH_MODE=real",
                ));
            }
            let environment_ref = provider
                .configured_execution_environment_ref()
                .ok_or_else(|| Error::validation("real DSH has no execution environment ref"))?
                .to_string();
            let attestation = find_attestation(&attestations, &environment_ref)?;
            let pinned_version = provider.preflight_real_runtime()?;
            let pinned_digest = provider
                .configured_runtime_digest()
                .ok_or_else(|| Error::validation("real DSH has no pinned runtime digest"))?
                .to_string();
            if !attestation.attests_runtime_identity(
                "deepseek-harness",
                &pinned_version,
                &pinned_digest,
                Timestamp::now(),
            ) {
                return Err(Error::external(
                    "execution-environment attestation does not bind the exact DSH runtime artifact",
                ));
            }
            let context = live_context(workspace_id, &attestation)?;
            let report = run_harness_smoke(&mut *provider, &context)?;
            let health = provider.runtime_health().clone();
            let runtime_version = provider.runtime_version();
            let runtime_digest = provider.runtime_digest();
            let wire_server_version = provider.wire_server_version().map(str::to_string);
            let selectable = health.selectable_at(Timestamp::now());
            let shutdown = provider.shutdown_real_runtime();
            let cleanup_ok = shutdown.is_ok();
            let pass = report.all_ok()
                && health.state == HarnessRuntimeHealthState::Healthy
                && selectable
                && cleanup_ok;
            Ok(json!({
                "gate": "MORN_LIVE_PROVIDER_SMOKE",
                "status": if pass { "PASS" } else { "NOT_PROVEN" },
                "provider": "deepseek-harness",
                "report": report,
                "runtime_health_before_shutdown": health,
                "runtime_version": runtime_version,
                "runtime_digest": runtime_digest,
                "wire_server_version": wire_server_version,
                "execution_environment_ref": attestation.environment_ref,
                "execution_environment_runtime_identities": attestation.runtime_identities,
                "execution_environment_evidence_refs": attestation.evidence_refs,
                "provider_cleanup_ok": cleanup_ok,
                "provider_cleanup_error": shutdown.err().map(|error| error.to_string()),
                "claims": {
                    "executor_live_evidence": pass,
                    "canonical_work_outcome": false,
                    "customer_acceptance": false,
                    "production_write": false
                }
            }))
        }
        "pi" => {
            let mut provider = pi
                .lock()
                .map_err(|_| Error::internal("Pi provider lock poisoned"))?;
            if provider.mode() != PiMode::Real {
                return Err(Error::invalid_state(
                    "live Pi smoke requires MORN_PI_MODE=real",
                ));
            }
            let environment_ref = provider
                .configured_execution_environment_ref()
                .ok_or_else(|| Error::validation("real Pi has no execution environment ref"))?
                .to_string();
            let attestation = find_attestation(&attestations, &environment_ref)?;
            let pinned_version = provider.preflight_real_runtime()?;
            let pinned_digest = provider
                .configured_runtime_digest()
                .ok_or_else(|| Error::validation("real Pi has no pinned runtime digest"))?
                .to_string();
            if !attestation.attests_runtime_identity(
                "pi",
                &pinned_version,
                &pinned_digest,
                Timestamp::now(),
            ) {
                return Err(Error::external(
                    "execution-environment attestation does not bind the exact Pi runtime artifact",
                ));
            }
            let context = live_context(workspace_id, &attestation)?;
            let report = run_harness_smoke(&mut *provider, &context)?;
            let health = provider.runtime_health().clone();
            let runtime_version = provider.runtime_version();
            let runtime_digest = provider.runtime_digest();
            let selectable = health.selectable_at(Timestamp::now());
            let shutdown = provider.shutdown_real_runtime();
            let cleanup_ok = shutdown.is_ok();
            let pass = report.all_ok()
                && health.state == HarnessRuntimeHealthState::Healthy
                && selectable
                && cleanup_ok;
            Ok(json!({
                "gate": "MORN_LIVE_PROVIDER_SMOKE",
                "status": if pass { "PASS" } else { "NOT_PROVEN" },
                "provider": "pi",
                "report": report,
                "runtime_health_before_shutdown": health,
                "runtime_version": runtime_version,
                "runtime_digest": runtime_digest,
                "execution_environment_ref": attestation.environment_ref,
                "execution_environment_runtime_identities": attestation.runtime_identities,
                "execution_environment_evidence_refs": attestation.evidence_refs,
                "provider_cleanup_ok": cleanup_ok,
                "provider_cleanup_error": shutdown.err().map(|error| error.to_string()),
                "claims": {
                    "executor_live_evidence": pass,
                    "canonical_work_outcome": false,
                    "customer_acceptance": false,
                    "production_write": false
                }
            }))
        }
        other => Err(Error::validation(format!(
            "unsupported MORN_PROVIDER_SMOKE_PROVIDER {other:?}; expected deepseek-harness or pi"
        ))),
    }
}

fn main() -> ExitCode {
    match run_gate() {
        Ok(report) => {
            let passed = report.get("status").and_then(Value::as_str) == Some("PASS");
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("live smoke report is serializable")
            );
            if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            }
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "gate": "MORN_LIVE_PROVIDER_SMOKE",
                    "status": "NOT_PROVEN",
                    "error": error.to_string(),
                    "claims": {
                        "executor_live_evidence": false,
                        "canonical_work_outcome": false,
                        "customer_acceptance": false,
                        "production_write": false
                    }
                }))
                .expect("error report is serializable")
            );
            ExitCode::from(2)
        }
    }
}
