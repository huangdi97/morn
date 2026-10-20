//! One-shot external supply-chain verification gate.
//!
//! This gate verifies an already-published, digest-pinned OCI capability
//! against deployment-pinned Sigstore identity and SLSA provenance. It does
//! not publish, sign, grant SiteAdmission, or authorize production execution.

use std::collections::BTreeMap;
use std::env;
use std::process::ExitCode;

use morn_package::{CosignCliVerifier, OciPublishReceipt, SigstoreIdentityPolicy};
use serde_json::{json, Value};

fn required(name: &str) -> Result<String, String> {
    let value = env::var(name).map_err(|_| format!("{name} is required"))?;
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(format!("{name} must not be empty"));
    }
    Ok(value)
}

fn canonical_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        && hex.bytes().all(|byte| !byte.is_ascii_uppercase())
}

fn supply_chain_context_ref(kind: &str, value: &str) -> String {
    format!("morn-supply-chain://{kind}?value={value}")
}

fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_uppercase() || first == '_')
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn explicit_cosign_env() -> Result<BTreeMap<String, String>, String> {
    let raw = env::var("MORN_COSIGN_ENV_PASSTHROUGH").unwrap_or_default();
    let mut selected = BTreeMap::new();
    for name in raw
        .split([',', ';'])
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        if !valid_env_name(name) {
            return Err(format!(
                "MORN_COSIGN_ENV_PASSTHROUGH contains invalid environment name {name:?}"
            ));
        }
        let value = env::var(name)
            .map_err(|_| format!("requested Cosign environment variable {name} is not set"))?;
        selected.insert(name.to_string(), value);
    }
    Ok(selected)
}

fn receipt_from_env() -> Result<OciPublishReceipt, String> {
    let oci_ref = required("MORN_OCI_REF")?;
    let digest = oci_ref
        .rsplit_once('@')
        .map(|(_, digest)| digest.to_string())
        .ok_or_else(|| "MORN_OCI_REF must be digest-pinned".to_string())?;
    let manifest_media_type = env::var("MORN_OCI_MANIFEST_MEDIA_TYPE")
        .unwrap_or_else(|_| "application/vnd.oci.image.manifest.v1+json".to_string());
    let artifact_type = env::var("MORN_OCI_ARTIFACT_TYPE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let receipt = OciPublishReceipt {
        oci_ref,
        content_digest: digest,
        manifest_media_type,
        artifact_type,
    };
    receipt.validate()?;
    Ok(receipt)
}

fn run_gate() -> Result<Value, String> {
    let receipt = receipt_from_env()?;
    let policy = SigstoreIdentityPolicy {
        certificate_identity: required("MORN_COSIGN_CERTIFICATE_IDENTITY")?,
        certificate_oidc_issuer: required("MORN_COSIGN_OIDC_ISSUER")?,
    };
    policy.validate()?;

    let verifier = CosignCliVerifier {
        command: env::var("MORN_COSIGN_COMMAND").unwrap_or_else(|_| "cosign".to_string()),
        env: explicit_cosign_env()?,
    };
    if verifier.command.trim().is_empty() {
        return Err("MORN_COSIGN_COMMAND must not be empty".to_string());
    }

    let release_runner_ref = required("MORN_SUPPLY_CHAIN_RELEASE_RUNNER_REF")?;
    if release_runner_ref.chars().any(char::is_control) {
        return Err("MORN_SUPPLY_CHAIN_RELEASE_RUNNER_REF must be a single-line identity".to_string());
    }
    let cosign_digest = required("MORN_SUPPLY_CHAIN_COSIGN_DIGEST")?;
    if !canonical_sha256(&cosign_digest) {
        return Err(
            "MORN_SUPPLY_CHAIN_COSIGN_DIGEST must be canonical sha256:<64-lowercase-hex>"
                .to_string(),
        );
    }

    let signature = verifier.verify_signature(&receipt, &policy)?;
    let provenance = verifier.verify_slsa_provenance(&receipt, &policy)?;
    let mut merged = signature.merge(&provenance)?;
    for reference in [
        supply_chain_context_ref("release-runner", &release_runner_ref),
        supply_chain_context_ref("cosign-binary", &cosign_digest),
    ] {
        if !merged.evidence_refs.contains(&reference) {
            merged.evidence_refs.push(reference);
        }
    }
    merged.validate_subject_digest(&receipt.content_digest, true, true)?;

    Ok(json!({
        "gate": "MORN_LIVE_SUPPLY_CHAIN_VERIFY",
        "status": "PASS",
        "oci_ref": receipt.oci_ref,
        "subject_digest": receipt.content_digest,
        "manifest_media_type": receipt.manifest_media_type,
        "artifact_type": receipt.artifact_type,
        "verification_evidence": merged,
        "claims": {
            "remote_digest_pinned_artifact_verified": true,
            "signature_verified": true,
            "slsa_provenance_verified": true,
            "artifact_published_by_this_gate": false,
            "site_admitted": false,
            "customer_acceptance": false,
            "production_write": false
        }
    }))
}

fn main() -> ExitCode {
    match run_gate() {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("supply-chain verification report is serializable")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "gate": "MORN_LIVE_SUPPLY_CHAIN_VERIFY",
                    "status": "NOT_PROVEN",
                    "error": error,
                    "claims": {
                        "remote_digest_pinned_artifact_verified": false,
                        "signature_verified": false,
                        "slsa_provenance_verified": false,
                        "artifact_published_by_this_gate": false,
                        "site_admitted": false,
                        "customer_acceptance": false,
                        "production_write": false
                    }
                }))
                .expect("supply-chain verification error report is serializable")
            );
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_context_requires_canonical_cosign_digest() {
        assert!(canonical_sha256(&format!("sha256:{}", "a".repeat(64))));
        assert!(!canonical_sha256(&format!("sha256:{}", "A".repeat(64))));
        assert!(!canonical_sha256("sha256:not-a-digest"));
        assert_eq!(
            supply_chain_context_ref("release-runner", "runner://release-1"),
            "morn-supply-chain://release-runner?value=runner://release-1"
        );
    }

    #[test]
    fn environment_passthrough_names_are_strict_tokens() {
        for good in ["HOME", "DOCKER_CONFIG", "_PRIVATE_REGISTRY"] {
            assert!(valid_env_name(good), "{good}");
        }
        for bad in ["", "home", "A-B", "A=B", "A B", "GITHUB_TOKEN\nOTHER"] {
            assert!(!valid_env_name(bad), "{bad}");
        }
    }

    #[test]
    fn digest_pinned_reference_is_required_by_receipt_contract() {
        let receipt = OciPublishReceipt {
            oci_ref: "oci://registry.example/morn/cap:latest".to_string(),
            content_digest: format!("sha256:{}", "a".repeat(64)),
            manifest_media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            artifact_type: None,
        };
        assert!(receipt.validate().is_err());
    }
}
