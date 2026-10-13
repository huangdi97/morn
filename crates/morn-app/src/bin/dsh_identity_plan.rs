//! Secret-free identity planning for deployment-attested DeepSeek Harness routes.
//!
//! This binary never launches DSH and never proves a live provider. It validates
//! the configured real route and prints the exact identities that an independent
//! execution-environment attestation must bind before a live smoke is eligible.

use std::process::ExitCode;

use morn_harness::{DshAcpConfig, DshSdkConfig};
use morn_kernel::error::{Error, Result};
use serde_json::{json, Value};

fn sdk_plan() -> Result<Value> {
    let config = DshSdkConfig::from_env()?;
    config.validate_for_real()?;
    let runtime_version = config
        .runtime_version
        .as_deref()
        .ok_or_else(|| Error::validation("DSH SDK runtime version missing after validation"))?;
    let runtime_digest = config
        .runtime_digest
        .as_deref()
        .ok_or_else(|| Error::validation("DSH SDK runtime digest missing after validation"))?;
    let environment_ref = config.execution_environment_ref.as_deref().ok_or_else(|| {
        Error::validation("DSH SDK execution environment missing after validation")
    })?;
    let profile_ref = config
        .profile_configuration_ref
        .as_deref()
        .ok_or_else(|| Error::validation("DSH SDK profile identity missing after validation"))?;
    let route_ref = config.route_attestation_ref()?;
    let runtime_identity = format!("deepseek-harness@{runtime_version}#{runtime_digest}");

    Ok(json!({
        "gate": "MORN_DSH_IDENTITY_PLAN",
        "status": "PLAN_ONLY",
        "transport": "sdk-jsonrpc-stdio",
        "provider": config.provider,
        "model": config.model,
        "execution_environment_ref": environment_ref,
        "required_runtime_identities": [
            runtime_identity,
            profile_ref,
            route_ref
        ],
        "claims": {
            "live_provider": false,
            "runtime_health": false,
            "canonical_work_outcome": false,
            "customer_acceptance": false,
            "production_write": false
        }
    }))
}

fn acp_plan() -> Result<Value> {
    let config = DshAcpConfig::from_env()?;
    config.validate_for_real()?;
    let runtime_version = config
        .runtime_version
        .as_deref()
        .ok_or_else(|| Error::validation("DSH ACP runtime version missing after validation"))?;
    let runtime_digest = config
        .runtime_digest
        .as_deref()
        .ok_or_else(|| Error::validation("DSH ACP runtime digest missing after validation"))?;
    let environment_ref = config.execution_environment_ref.as_deref().ok_or_else(|| {
        Error::validation("DSH ACP execution environment missing after validation")
    })?;
    let profile_ref = config
        .profile_configuration_ref
        .as_deref()
        .ok_or_else(|| Error::validation("DSH ACP profile identity missing after validation"))?;
    let route_ref = config.route_attestation_ref()?;
    let runtime_identity = format!("deepseek-harness@{runtime_version}#{runtime_digest}");

    Ok(json!({
        "gate": "MORN_DSH_IDENTITY_PLAN",
        "status": "PLAN_ONLY",
        "transport": "acp-v1-stdio",
        "provider": config.provider,
        "model": config.model,
        "execution_environment_ref": environment_ref,
        "required_runtime_identities": [
            runtime_identity,
            profile_ref,
            route_ref
        ],
        "claims": {
            "acp_lifecycle_live": false,
            "executor_success": false,
            "runtime_health": false,
            "canonical_work_outcome": false,
            "customer_acceptance": false,
            "production_write": false
        }
    }))
}

fn run() -> Result<Value> {
    match std::env::var("MORN_DSH_PLAN_MODE")
        .unwrap_or_else(|_| "sdk".to_string())
        .trim()
    {
        "sdk" => sdk_plan(),
        "acp" => acp_plan(),
        other => Err(Error::validation(format!(
            "unsupported MORN_DSH_PLAN_MODE {other:?}; expected sdk or acp"
        ))),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(plan) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&plan).expect("identity plan is serializable")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "gate": "MORN_DSH_IDENTITY_PLAN",
                    "status": "INVALID_CONFIGURATION",
                    "error": error.to_string(),
                    "claims": {
                        "live_provider": false,
                        "runtime_health": false,
                        "canonical_work_outcome": false,
                        "customer_acceptance": false,
                        "production_write": false
                    }
                }))
                .expect("identity-plan error is serializable")
            );
            ExitCode::from(2)
        }
    }
}
