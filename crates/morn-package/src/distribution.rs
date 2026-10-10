//! OCI artifact publication boundary for capability packages.
//!
//! Morn owns semantic admission and immutable publication evidence. ORAS owns
//! registry transport. A successful CLI exit is not enough: the publisher must
//! return a digest-pinned OCI reference that matches the registry receipt.
//! Credentials/environment are explicit deployment inputs and are never
//! inherited implicitly from the Morn process.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::supply_chain::SupplyChainVerificationEvidence;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciLayerInput {
    pub path: PathBuf,
    pub media_type: String,
}

impl OciLayerInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Err("OCI layer path required".to_string());
        }
        if self.media_type.trim().is_empty() || self.media_type.contains(char::is_whitespace) {
            return Err("OCI layer media_type must be a non-empty token".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciPublishRequest {
    /// Immutable registry repository namespace, for example
    /// `oci://ghcr.io/acme/morn/capability`. Tags are supplied separately.
    pub repository_ref: String,
    pub tag: String,
    pub artifact_type: String,
    pub layers: Vec<OciLayerInput>,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
}

impl OciPublishRequest {
    pub fn validate(&self) -> Result<(), String> {
        let repository = self
            .repository_ref
            .strip_prefix("oci://")
            .ok_or_else(|| "repository_ref must use oci:// scheme".to_string())?;
        if repository.trim().is_empty()
            || repository.contains('@')
            || repository.contains("://")
            || repository.chars().any(char::is_whitespace)
        {
            return Err("repository_ref must be an unpinned OCI repository path".to_string());
        }
        if self.tag.trim().is_empty()
            || self.tag.contains('@')
            || self.tag.contains('/')
            || self.tag.contains(char::is_whitespace)
        {
            return Err("OCI publish tag must be a simple non-empty tag".to_string());
        }
        if self.artifact_type.trim().is_empty() || self.artifact_type.contains(char::is_whitespace)
        {
            return Err("artifact_type must be a non-empty media type token".to_string());
        }
        if self.layers.is_empty() {
            return Err("at least one OCI layer is required".to_string());
        }
        for layer in &self.layers {
            layer.validate()?;
        }
        for (key, value) in &self.annotations {
            if key.trim().is_empty()
                || key.contains('=')
                || value.contains('\n')
                || value.contains('\r')
            {
                return Err(
                    "OCI annotations require a non-empty key and single-line value".to_string(),
                );
            }
        }
        Ok(())
    }

    fn oras_target(&self) -> Result<String, String> {
        self.validate()?;
        Ok(format!(
            "{}:{}",
            self.repository_ref.trim_start_matches("oci://"),
            self.tag
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciPublishReceipt {
    /// Digest-pinned canonical subject reference.
    pub oci_ref: String,
    pub content_digest: String,
    pub manifest_media_type: String,
    pub artifact_type: Option<String>,
}

impl OciPublishReceipt {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_digest(&self.content_digest) {
            return Err("publish receipt content_digest must be sha256:<64 hex>".to_string());
        }
        let reference = self
            .oci_ref
            .strip_prefix("oci://")
            .ok_or_else(|| "publish receipt oci_ref must use oci://".to_string())?;
        let (_, digest) = reference
            .rsplit_once('@')
            .ok_or_else(|| "publish receipt must be digest pinned".to_string())?;
        if !digest.eq_ignore_ascii_case(&self.content_digest) {
            return Err("publish receipt reference digest mismatch".to_string());
        }
        if self.manifest_media_type.trim().is_empty() {
            return Err("publish receipt media type required".to_string());
        }
        Ok(())
    }

    pub fn validate_for_request(&self, request: &OciPublishRequest) -> Result<(), String> {
        self.validate()?;
        request.validate()?;
        let expected_repository = request.repository_ref.trim_end_matches('/');
        let actual_repository = self
            .oci_ref
            .rsplit_once('@')
            .map(|(repository, _)| repository)
            .ok_or_else(|| "publish receipt must be digest pinned".to_string())?;
        if actual_repository != expected_repository {
            return Err("ORAS receipt repository does not match requested repository".to_string());
        }
        if let Some(artifact_type) = &self.artifact_type {
            if artifact_type != &request.artifact_type {
                return Err("ORAS receipt artifact type does not match publish request".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    /// Explicit child environment. The implementation clears the inherited
    /// environment first so parent secrets cannot leak accidentally.
    pub env: BTreeMap<String, String>,
}

pub trait OciArtifactPublisher {
    fn publish(&self, request: &OciPublishRequest) -> Result<OciPublishReceipt, String>;
}

/// ORAS CLI implementation. Authentication is deployment-owned: callers pass
/// only the exact environment variables needed by their registry client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrasCliPublisher {
    pub command: String,
    pub env: BTreeMap<String, String>,
}

impl Default for OrasCliPublisher {
    fn default() -> Self {
        Self {
            command: "oras".to_string(),
            env: BTreeMap::new(),
        }
    }
}

impl OrasCliPublisher {
    pub fn command_spec(&self, request: &OciPublishRequest) -> Result<ExternalCommandSpec, String> {
        request.validate()?;
        if self.command.trim().is_empty() {
            return Err("ORAS command required".to_string());
        }

        let mut args = vec![
            "push".to_string(),
            "--artifact-type".to_string(),
            request.artifact_type.clone(),
            "--format".to_string(),
            "json".to_string(),
        ];
        for (key, value) in &request.annotations {
            args.push("--annotation".to_string());
            args.push(format!("{key}={value}"));
        }
        args.push(request.oras_target()?);
        for layer in &request.layers {
            args.push(format!(
                "{}:{}",
                layer.path.to_string_lossy(),
                layer.media_type
            ));
        }

        Ok(ExternalCommandSpec {
            program: self.command.clone(),
            args,
            env: self.env.clone(),
        })
    }

    pub fn parse_receipt(stdout: &str) -> Result<OciPublishReceipt, String> {
        let value: Value = serde_json::from_str(stdout)
            .map_err(|error| format!("invalid ORAS JSON output: {error}"))?;
        let reference = value
            .get("reference")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "ORAS output missing reference".to_string())?;
        let digest = value
            .get("digest")
            .and_then(Value::as_str)
            .filter(|value| valid_digest(value))
            .ok_or_else(|| "ORAS output missing valid digest".to_string())?;
        let media_type = value
            .get("mediaType")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "ORAS output missing mediaType".to_string())?;
        let pinned = if reference.starts_with("oci://") {
            reference.to_string()
        } else {
            format!("oci://{reference}")
        };
        let receipt = OciPublishReceipt {
            oci_ref: pinned,
            content_digest: digest.to_string(),
            manifest_media_type: media_type.to_string(),
            artifact_type: value
                .get("artifactType")
                .and_then(Value::as_str)
                .map(str::to_string),
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

impl OciArtifactPublisher for OrasCliPublisher {
    fn publish(&self, request: &OciPublishRequest) -> Result<OciPublishReceipt, String> {
        let spec = self.command_spec(request)?;
        let output = Command::new(&spec.program)
            .args(&spec.args)
            .env_clear()
            .envs(&spec.env)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("failed to start ORAS: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ORAS push failed: {}", stderr.trim()));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|error| format!("ORAS output was not UTF-8: {error}"))?;
        let receipt = Self::parse_receipt(&stdout)?;
        receipt.validate_for_request(request)?;
        Ok(receipt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigstoreIdentityPolicy {
    pub certificate_identity: String,
    pub certificate_oidc_issuer: String,
}

impl SigstoreIdentityPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.certificate_identity.trim().is_empty()
            || self.certificate_oidc_issuer.trim().is_empty()
        {
            return Err(
                "Sigstore verification requires certificate identity and OIDC issuer".to_string(),
            );
        }
        Ok(())
    }
}

/// Cosign verification is deliberately separate from ORAS publication. A
/// registry push does not imply signature trust, and a signature does not prove
/// SLSA provenance. This verifier only upgrades the signature axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CosignCliVerifier {
    pub command: String,
    pub env: BTreeMap<String, String>,
}

impl Default for CosignCliVerifier {
    fn default() -> Self {
        Self {
            command: "cosign".to_string(),
            env: BTreeMap::new(),
        }
    }
}

impl CosignCliVerifier {
    pub fn command_spec(
        &self,
        receipt: &OciPublishReceipt,
        policy: &SigstoreIdentityPolicy,
    ) -> Result<ExternalCommandSpec, String> {
        receipt.validate()?;
        policy.validate()?;
        if self.command.trim().is_empty() {
            return Err("cosign command required".to_string());
        }
        let target = receipt
            .oci_ref
            .strip_prefix("oci://")
            .ok_or_else(|| "cosign subject must use oci://".to_string())?;
        Ok(ExternalCommandSpec {
            program: self.command.clone(),
            args: vec![
                "verify".to_string(),
                target.to_string(),
                "--certificate-identity".to_string(),
                policy.certificate_identity.clone(),
                "--certificate-oidc-issuer".to_string(),
                policy.certificate_oidc_issuer.clone(),
                "--output".to_string(),
                "json".to_string(),
            ],
            env: self.env.clone(),
        })
    }

    pub fn verify_signature(
        &self,
        receipt: &OciPublishReceipt,
        policy: &SigstoreIdentityPolicy,
    ) -> Result<SupplyChainVerificationEvidence, String> {
        let spec = self.command_spec(receipt, policy)?;
        let output = Command::new(&spec.program)
            .args(&spec.args)
            .env_clear()
            .envs(&spec.env)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("failed to start cosign: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("cosign verify failed: {}", stderr.trim()));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|error| format!("cosign output was not UTF-8: {error}"))?;
        if stdout.trim().is_empty() {
            return Err("cosign verify returned no verification evidence".to_string());
        }
        serde_json::from_str::<Value>(&stdout)
            .map_err(|error| format!("cosign verify output was not JSON: {error}"))?;

        let mut digest = Sha256::new();
        digest.update(stdout.as_bytes());
        let proof_digest = format!("sha256:{:x}", digest.finalize());
        Ok(SupplyChainVerificationEvidence {
            subject_digest: receipt.content_digest.clone(),
            verifier_ref: format!(
                "sigstore://cosign?identity={}&issuer={}",
                policy.certificate_identity, policy.certificate_oidc_issuer
            ),
            signature_verified: true,
            provenance_verified: false,
            evidence_refs: vec![format!("cosign-verify-output://{proof_digest}")],
        })
    }

    pub fn provenance_command_spec(
        &self,
        receipt: &OciPublishReceipt,
        policy: &SigstoreIdentityPolicy,
    ) -> Result<ExternalCommandSpec, String> {
        receipt.validate()?;
        policy.validate()?;
        if self.command.trim().is_empty() {
            return Err("cosign command required".to_string());
        }
        let target = receipt
            .oci_ref
            .strip_prefix("oci://")
            .ok_or_else(|| "cosign subject must use oci://".to_string())?;
        Ok(ExternalCommandSpec {
            program: self.command.clone(),
            args: vec![
                "verify-attestation".to_string(),
                target.to_string(),
                "--type".to_string(),
                "slsaprovenance".to_string(),
                "--certificate-identity".to_string(),
                policy.certificate_identity.clone(),
                "--certificate-oidc-issuer".to_string(),
                policy.certificate_oidc_issuer.clone(),
                "--output".to_string(),
                "json".to_string(),
            ],
            env: self.env.clone(),
        })
    }

    pub fn verify_slsa_provenance(
        &self,
        receipt: &OciPublishReceipt,
        policy: &SigstoreIdentityPolicy,
    ) -> Result<SupplyChainVerificationEvidence, String> {
        let spec = self.provenance_command_spec(receipt, policy)?;
        let output = Command::new(&spec.program)
            .args(&spec.args)
            .env_clear()
            .envs(&spec.env)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("failed to start cosign: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "cosign verify-attestation failed: {}",
                stderr.trim()
            ));
        }
        let stdout = String::from_utf8(output.stdout)
            .map_err(|error| format!("cosign attestation output was not UTF-8: {error}"))?;
        if stdout.trim().is_empty() {
            return Err("cosign verify-attestation returned no evidence".to_string());
        }
        serde_json::from_str::<Value>(&stdout)
            .map_err(|error| format!("cosign attestation output was not JSON: {error}"))?;

        let mut digest = Sha256::new();
        digest.update(stdout.as_bytes());
        let proof_digest = format!("sha256:{:x}", digest.finalize());
        Ok(SupplyChainVerificationEvidence {
            subject_digest: receipt.content_digest.clone(),
            verifier_ref: format!(
                "slsa://cosign?identity={}&issuer={}",
                policy.certificate_identity, policy.certificate_oidc_issuer
            ),
            signature_verified: false,
            provenance_verified: true,
            evidence_refs: vec![format!("cosign-attestation-output://{proof_digest}")],
        })
    }
}

fn valid_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn digest(ch: char) -> String {
        format!("sha256:{}", ch.to_string().repeat(64))
    }

    fn request() -> OciPublishRequest {
        OciPublishRequest {
            repository_ref: "oci://registry.example/morn/capabilities/reviewer".to_string(),
            tag: "1.2.3".to_string(),
            artifact_type: "application/vnd.morn.capability.v1+json".to_string(),
            layers: vec![OciLayerInput {
                path: Path::new("dist/reviewer.json").to_path_buf(),
                media_type: "application/vnd.morn.capability.layer.v1+json".to_string(),
            }],
            annotations: BTreeMap::from([(
                "org.opencontainers.image.source".to_string(),
                "https://example.invalid/repo".to_string(),
            )]),
        }
    }

    #[test]
    fn oras_command_is_argument_safe_and_does_not_inherit_environment() {
        let publisher = OrasCliPublisher {
            command: "oras".to_string(),
            env: BTreeMap::from([(
                "DOCKER_CONFIG".to_string(),
                "/run/registry-auth".to_string(),
            )]),
        };
        let spec = publisher.command_spec(&request()).unwrap();
        assert_eq!(spec.program, "oras");
        assert_eq!(spec.env.len(), 1);
        assert_eq!(spec.args[0], "push");
        assert!(spec
            .args
            .windows(2)
            .any(|pair| pair == ["--format", "json"]));
        assert!(spec
            .args
            .contains(&"registry.example/morn/capabilities/reviewer:1.2.3".to_string()));
        assert!(spec.args.contains(
            &"dist/reviewer.json:application/vnd.morn.capability.layer.v1+json".to_string()
        ));
    }

    #[test]
    fn parses_digest_pinned_oras_json_receipt() {
        let digest = digest('a');
        let stdout = serde_json::json!({
            "reference": format!("registry.example/morn/capabilities/reviewer@{digest}"),
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": digest,
            "artifactType": "application/vnd.morn.capability.v1+json"
        })
        .to_string();
        let receipt = OrasCliPublisher::parse_receipt(&stdout).unwrap();
        assert!(receipt.oci_ref.starts_with("oci://registry.example/"));
        assert_eq!(receipt.content_digest, digest('a'));
        assert!(receipt.validate().is_ok());
    }

    #[test]
    fn cosign_verification_is_digest_pinned_and_identity_scoped() {
        let digest = digest('c');
        let receipt = OciPublishReceipt {
            oci_ref: format!("oci://registry.example/morn/cap@{digest}"),
            content_digest: digest,
            manifest_media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            artifact_type: Some("application/vnd.morn.capability.v1+json".to_string()),
        };
        let policy = SigstoreIdentityPolicy {
            certificate_identity:
                "https://github.com/acme/morn/.github/workflows/release.yml@refs/heads/main"
                    .to_string(),
            certificate_oidc_issuer: "https://token.actions.githubusercontent.com".to_string(),
        };
        let spec = CosignCliVerifier::default()
            .command_spec(&receipt, &policy)
            .unwrap();
        assert_eq!(spec.args[0], "verify");
        assert!(spec.args[1].contains("@sha256:"));
        assert!(spec.args.contains(&"--certificate-identity".to_string()));
        assert!(spec.args.contains(&"--certificate-oidc-issuer".to_string()));
        assert!(spec
            .args
            .windows(2)
            .any(|pair| pair == ["--output", "json"]));
    }

    #[test]
    fn slsa_verification_is_digest_pinned_and_identity_scoped() {
        let digest = digest('d');
        let receipt = OciPublishReceipt {
            oci_ref: format!("oci://registry.example/morn/cap@{digest}"),
            content_digest: digest,
            manifest_media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            artifact_type: Some("application/vnd.morn.capability.v1+json".to_string()),
        };
        let policy = SigstoreIdentityPolicy {
            certificate_identity:
                "https://github.com/acme/morn/.github/workflows/release.yml@refs/heads/main"
                    .to_string(),
            certificate_oidc_issuer: "https://token.actions.githubusercontent.com".to_string(),
        };
        let spec = CosignCliVerifier::default()
            .provenance_command_spec(&receipt, &policy)
            .unwrap();
        assert_eq!(spec.args[0], "verify-attestation");
        assert!(spec.args[1].contains("@sha256:"));
        assert!(spec
            .args
            .windows(2)
            .any(|pair| pair == ["--type", "slsaprovenance"]));
        assert!(spec.args.contains(&"--certificate-identity".to_string()));
        assert!(spec.args.contains(&"--certificate-oidc-issuer".to_string()));
    }

    #[test]
    fn publication_receipt_must_match_requested_repository_and_artifact_type() {
        let request = request();
        let digest = digest('a');
        let mut receipt = OciPublishReceipt {
            oci_ref: format!("oci://registry.example/morn/capabilities/reviewer@{digest}"),
            content_digest: digest,
            manifest_media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            artifact_type: Some(request.artifact_type.clone()),
        };
        assert!(receipt.validate_for_request(&request).is_ok());

        receipt.oci_ref = format!(
            "oci://registry.example/other/repository@{}",
            receipt.content_digest
        );
        assert!(receipt.validate_for_request(&request).is_err());

        receipt.oci_ref = format!("{}@{}", request.repository_ref, receipt.content_digest);
        receipt.artifact_type = Some("application/vnd.other+json".to_string());
        assert!(receipt.validate_for_request(&request).is_err());
    }

    #[test]
    fn rejects_mutable_or_mismatched_publication_receipt() {
        let digest_a = digest('a');
        let digest_b = digest('b');
        let mismatched = serde_json::json!({
            "reference": format!("registry.example/morn/capabilities/reviewer@{digest_a}"),
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": digest_b
        })
        .to_string();
        assert!(OrasCliPublisher::parse_receipt(&mismatched).is_err());

        let mut bad = request();
        bad.repository_ref = "oci://registry.example/repo@sha256:deadbeef".to_string();
        assert!(bad.validate().is_err());
    }
}
