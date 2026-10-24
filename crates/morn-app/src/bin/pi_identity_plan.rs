//! Secret-free identity planning for a deployment-attested Pi RPC route.
//!
//! This command validates configuration only. It never launches Pi and never
//! upgrades executor/runtime evidence into Work Outcome, Acceptance or write authority.

use std::process::ExitCode;

use morn_harness::PiRpcConfig;
use morn_kernel::error::{Error, Result};
use serde_json::{json, Value};

fn plan() -> Result<Value> {
    let config = PiRpcConfig::from_env()?;
    config.validate_for_real()?;
    let runtime_version = config
        .runtime_version
        .as_deref()
        .ok_or_else(|| Error::validation("Pi runtime version missing after validation"))?;
    let runtime_digest = config
        .runtime_digest
        .as_deref()
        .ok_or_else(|| Error::validation("Pi runtime digest missing after validation"))?;
    let environment_ref = config
        .execution_environment_ref
        .as_deref()
        .ok_or_else(|| Error::validation("Pi execution environment missing after validation"))?;
    let route_ref = config.route_attestation_ref()?;
    Ok(json!({
        "gate": "MORN_PI_IDENTITY_PLAN",
        "status": "PLAN_ONLY",
        "transport": "pi-jsonl-rpc",
        "provider": config.provider,
        "model": config.model,
        "execution_environment_ref": environment_ref,
        "required_runtime_identities": [
            format!("pi@{runtime_version}#{runtime_digest}"),
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

fn main() -> ExitCode {
    match plan() {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("Pi identity plan is serializable")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "gate": "MORN_PI_IDENTITY_PLAN",
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
                .expect("Pi identity-plan error is serializable")
            );
            ExitCode::from(2)
        }
    }
}
