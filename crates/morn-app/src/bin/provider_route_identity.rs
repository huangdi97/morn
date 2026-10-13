//! Secret-free provider route identity planner.
//!
//! This binary never launches a provider and never creates readiness evidence.
//! It derives the exact identities a deployment attestor must bind before a
//! live provider smoke can become eligible for RealRuntime evidence.

use std::process::ExitCode;

use morn_harness::{DshSdkConfig, PiRpcConfig};
use morn_kernel::error::{Error, Result};
use serde_json::{json, Value};

fn runtime_identity(provider: &str, version: &str, digest: &str) -> Result<String> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(Error::validation(
            "provider runtime digest must use sha256:<64-hex>",
        ));
    };
    if provider.trim().is_empty()
        || version.trim().is_empty()
        || hex.len() != 64
        || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(Error::validation(
            "provider runtime identity requires provider, version and sha256:<64-hex>",
        ));
    }
    Ok(format!("{provider}@{version}#{digest}"))
}

fn provider_selector() -> Result<String> {
    std::env::var("MORN_PROVIDER_ROUTE_PROVIDER")
        .or_else(|_| std::env::var("MORN_PROVIDER_SMOKE_PROVIDER"))
        .map_err(|_| {
            Error::validation(
                "MORN_PROVIDER_ROUTE_PROVIDER is required (deepseek-harness/dsh or pi)",
            )
        })
}

fn plan() -> Result<Value> {
    match provider_selector()?.as_str() {
        "deepseek-harness" | "dsh" => {
            let config = DshSdkConfig::from_env()?;
            let version = config
                .runtime_version
                .as_deref()
                .ok_or_else(|| Error::validation("DSH runtime version missing after validation"))?;
            let digest = config
                .runtime_digest
                .as_deref()
                .ok_or_else(|| Error::validation("DSH runtime digest missing after validation"))?;
            let profile = config.profile_configuration_ref.as_deref().ok_or_else(|| {
                Error::validation("DSH profile configuration identity missing after validation")
            })?;
            Ok(json!({
                "kind": "morn-provider-route-identity-plan/v1",
                "provider": "deepseek-harness",
                "creates_evidence": false,
                "starts_provider": false,
                "execution_environment_ref": config.execution_environment_ref,
                "runtime_identity": runtime_identity("deepseek-harness", version, digest)?,
                "profile_configuration_identity": profile,
                "route_ref": config.route_ref()?,
                "route_attestation_ref": config.route_attestation_ref()?,
                "required_attestation_identities": [
                    runtime_identity("deepseek-harness", version, digest)?,
                    profile,
                    config.route_attestation_ref()?
                ]
            }))
        }
        "pi" => {
            let config = PiRpcConfig::from_env()?;
            let version = config
                .runtime_version
                .as_deref()
                .ok_or_else(|| Error::validation("Pi runtime version missing after validation"))?;
            let digest = config
                .runtime_digest
                .as_deref()
                .ok_or_else(|| Error::validation("Pi runtime digest missing after validation"))?;
            Ok(json!({
                "kind": "morn-provider-route-identity-plan/v1",
                "provider": "pi",
                "creates_evidence": false,
                "starts_provider": false,
                "execution_environment_ref": config.execution_environment_ref,
                "runtime_identity": runtime_identity("pi", version, digest)?,
                "route_ref": config.route_ref()?,
                "route_attestation_ref": config.route_attestation_ref()?,
                "required_attestation_identities": [
                    runtime_identity("pi", version, digest)?,
                    config.route_attestation_ref()?
                ]
            }))
        }
        other => Err(Error::validation(format!(
            "unsupported provider {other:?}; expected deepseek-harness/dsh or pi"
        ))),
    }
}

fn main() -> ExitCode {
    match plan() {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value)
                    .expect("provider route identity plan is serializable")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_identity_requires_sha256_digest() {
        assert!(runtime_identity("pi", "1.0.0", "sha256:bad").is_err());
        let identity =
            runtime_identity("pi", "1.0.0", &format!("sha256:{}", "a".repeat(64))).unwrap();
        assert_eq!(identity, format!("pi@1.0.0#sha256:{}", "a".repeat(64)));
    }
}
