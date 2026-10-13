//! One-shot live DeepSeek ACP lifecycle evidence gate.
//!
//! This gate proves only the governed lifecycle/control transport. ACP's
//! `end_turn` stop reason is not an executor-success proof and this binary
//! never creates a Work Outcome, AcceptanceDecision or production authority.

use std::process::ExitCode;

use morn_app::AppState;
use morn_harness::provider::DSH_REAL_E0_SCOPE_RESTRICTION;
use morn_harness::{CapabilityScope, DshAcpLifecycleController, RuntimeContext, ScopeKind};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ActorInstanceId, RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_runtime::ExecutionEnvironmentAttestation;
use serde_json::{json, Value};

const CANARY: &str = "MORN_ACP_LIFECYCLE_OK";

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

fn live_context(
    workspace_id: morn_kernel::ids::WorkspaceId,
    scope_id: &str,
    attestation: &ExecutionEnvironmentAttestation,
    runtime_version: &str,
    runtime_digest: &str,
    route_ref: &str,
) -> Result<RuntimeContext> {
    if !attestation.active_at(Timestamp::now()) {
        return Err(Error::external(
            "ACP execution-environment attestation is missing, invalid, not-yet-valid or expired",
        ));
    }
    let mut context = RuntimeContext::new(
        workspace_id,
        ActorInstanceId::generate_with("dsh-acp-smoke"),
        WorkPackageId::generate_with("dsh-acp-smoke"),
    )
    .with_work_binding(1, RuntimeBindingId::generate_with("dsh-acp-smoke"))
    .map_err(Error::validation)?
    .with_provider_identity(
        "deepseek-harness",
        runtime_version,
        Some(runtime_digest.to_string()),
        Some(route_ref.to_string()),
    )
    .map_err(Error::validation)?
    .with_scope_id(scope_id.to_string())
    .map_err(Error::validation)?
    .with_execution_environment(
        attestation.environment_ref.clone(),
        attestation.isolation,
        attestation.attested_spec.required_guarantees.clone(),
    )
    .map_err(Error::validation)?;
    context.policy_snapshot_version = Some("morn-dsh-acp-live-smoke/v1".to_string());
    context.provenance_refs = attestation.evidence_refs.clone();
    context
        .provenance_refs
        .push("gate://morn/dsh-acp-live-smoke/v1".to_string());
    Ok(context)
}

fn run_gate() -> Result<Value> {
    let db_path = std::env::var("MORN_DSH_ACP_SMOKE_DB").unwrap_or_else(|_| ":memory:".to_string());
    let state = AppState::new(&db_path)?;
    let (workspace_id, attestations) = {
        let guard = state.lock();
        (
            guard.workspace.id.clone(),
            guard.execution_environments.attestations(),
        )
    };

    let mut controller = DshAcpLifecycleController::from_env()?;
    let environment_ref = controller
        .configured_execution_environment_ref()
        .ok_or_else(|| Error::validation("ACP lifecycle has no execution environment ref"))?
        .to_string();
    let profile_configuration_ref = controller
        .configured_profile_configuration_ref()
        .ok_or_else(|| Error::validation("ACP lifecycle has no profile configuration ref"))?
        .to_string();
    let runtime_version = controller
        .configured_runtime_version()
        .ok_or_else(|| Error::validation("ACP lifecycle has no runtime version"))?
        .to_string();
    let runtime_digest = controller
        .configured_runtime_digest()
        .ok_or_else(|| Error::validation("ACP lifecycle has no runtime digest"))?
        .to_string();
    let route_ref = controller.configured_route_ref()?;

    let attestation = find_attestation(&attestations, &environment_ref)?;
    if !attestation.attests_runtime_identity(
        "deepseek-harness",
        &runtime_version,
        &runtime_digest,
        Timestamp::now(),
    ) {
        return Err(Error::external(
            "execution-environment attestation does not bind the exact ACP DSH runtime artifact",
        ));
    }
    if !attestation.attests_identity_ref(&profile_configuration_ref, Timestamp::now()) {
        return Err(Error::external(
            "execution-environment attestation does not bind the exact ACP Home/Profile composition",
        ));
    }
    if !attestation.attests_identity_ref(&route_ref, Timestamp::now()) {
        return Err(Error::external(
            "execution-environment attestation does not bind the exact ACP semantic route",
        ));
    }

    let scope = CapabilityScope::new(
        ScopeKind::ExecutionRun,
        None,
        workspace_id.clone(),
        "dsh-acp-live-smoke",
    )
    .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
    let scope_id = controller.mount_scope(scope)?;
    let context = live_context(
        workspace_id,
        &scope_id,
        &attestation,
        &runtime_version,
        &runtime_digest,
        &route_ref,
    )?;

    let session = match controller.open_session(&context) {
        Ok(session) => session,
        Err(error) => {
            let _ = controller.unmount_scope(&scope_id);
            let _ = controller.shutdown_transport();
            return Err(error);
        }
    };

    let prompt = format!("Reply with exactly {CANARY} and no other text.");
    let prompt_result = controller.prompt_text(&session.session_id, &context, &prompt);
    let session_after_prompt = controller.session(&session.session_id).cloned();
    let prompt_settled = prompt_result
        .as_ref()
        .is_ok_and(|result| result.reached_protocol_settlement());
    let canary_match = prompt_result
        .as_ref()
        .is_ok_and(|result| result.assistant_text.trim() == CANARY);
    let stop_reason = prompt_result
        .as_ref()
        .ok()
        .map(|result| result.stop_reason.clone());
    let prompt_error = prompt_result.err().map(|error| error.to_string());

    let close = controller.close_session(&session.session_id);
    let close_ok = close.is_ok();
    let close_error = close.err().map(|error| error.to_string());
    let unmount = controller.unmount_scope(&scope_id);
    let unmount_ok = unmount.is_ok();
    let unmount_error = unmount.err().map(|error| error.to_string());
    let wire_agent_version = controller.wire_agent_version().map(str::to_string);
    let shutdown = controller.shutdown_transport();
    let shutdown_ok = shutdown.is_ok();
    let shutdown_error = shutdown.err().map(|error| error.to_string());

    let identity_ok = session_after_prompt.as_ref().is_some_and(|observed| {
        observed.workspace_id == context.workspace_id
            && observed.work_package_id == context.work_package_id
            && Some(observed.work_generation) == context.work_generation
            && context.execution_binding_ref.as_ref() == Some(&observed.execution_binding_ref)
            && observed.execution_environment_ref == environment_ref
            && observed.scope_ref == scope_id
            && observed.route_ref == route_ref
            && context.proves_provider_identity(
                "deepseek-harness",
                &runtime_version,
                Some(&runtime_digest),
                Some(&route_ref),
            )
            && observed.runtime_version == runtime_version
            && observed.runtime_digest == runtime_digest
    });

    let pass =
        prompt_settled && canary_match && identity_ok && close_ok && unmount_ok && shutdown_ok;

    Ok(json!({
        "gate": "MORN_LIVE_DSH_ACP_LIFECYCLE_SMOKE",
        "status": if pass { "PASS" } else { "NOT_PROVEN" },
        "session_id": session.session_id,
        "route_ref": route_ref,
        "runtime_version": runtime_version,
        "runtime_digest": runtime_digest,
        "wire_agent_version": wire_agent_version,
        "execution_environment_ref": environment_ref,
        "profile_configuration_ref": profile_configuration_ref,
        "required_real_runtime_claim_refs": [
            format!("runtime://deepseek-harness/{runtime_version}#{runtime_digest}"),
            environment_ref,
            profile_configuration_ref,
            route_ref
        ],
        "execution_environment_runtime_identities": attestation.runtime_identities,
        "execution_environment_evidence_refs": attestation.evidence_refs,
        "prompt_settled": prompt_settled,
        "canary_match": canary_match,
        "stop_reason": stop_reason,
        "prompt_error": prompt_error,
        "identity_binding_ok": identity_ok,
        "session_cleanup_ok": close_ok,
        "session_cleanup_error": close_error,
        "scope_unmount_ok": unmount_ok,
        "scope_unmount_error": unmount_error,
        "transport_cleanup_ok": shutdown_ok,
        "transport_cleanup_error": shutdown_error,
        "claims": {
            "acp_lifecycle_live_evidence": pass,
            "executor_success": false,
            "provider_runtime_health": false,
            "canonical_work_outcome": false,
            "customer_acceptance": false,
            "production_write": false
        }
    }))
}

fn main() -> ExitCode {
    match run_gate() {
        Ok(report) => {
            let passed = report.get("status").and_then(Value::as_str) == Some("PASS");
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("ACP lifecycle smoke report is serializable")
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
                    "gate": "MORN_LIVE_DSH_ACP_LIFECYCLE_SMOKE",
                    "status": "NOT_PROVEN",
                    "error": error.to_string(),
                    "claims": {
                        "acp_lifecycle_live_evidence": false,
                        "executor_success": false,
                        "provider_runtime_health": false,
                        "canonical_work_outcome": false,
                        "customer_acceptance": false,
                        "production_write": false
                    }
                }))
                .expect("ACP lifecycle smoke error report is serializable")
            );
            ExitCode::from(2)
        }
    }
}
