//! Capability package supply-chain metadata.
//!
//! Distribution uses OCI-style content addressing; signatures and build
//! provenance are references to external Sigstore/SLSA artifacts rather than
//! proprietary Morn formats.

use serde::{Deserialize, Serialize};

use morn_kernel::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLayer {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    pub annotations: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityArtifactDescriptor {
    pub name: String,
    pub version: Version,
    pub manifest_ref: String,
    pub oci_ref: String,
    pub content_digest: String,
    pub media_type: String,
    pub layers: Vec<ArtifactLayer>,
    pub sbom_ref: Option<String>,
    pub slsa_provenance_ref: Option<String>,
    pub signature_ref: Option<String>,
}

impl CapabilityArtifactDescriptor {
    pub fn validate(
        &self,
        require_signature: bool,
        require_provenance: bool,
    ) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("capability artifact name required".to_string());
        }
        if !self.oci_ref.starts_with("oci://") {
            return Err("oci_ref must use oci:// scheme".to_string());
        }
        if !valid_digest(&self.content_digest) {
            return Err("content_digest must be sha256:<64 hex>".to_string());
        }
        let embedded_digest = self
            .oci_ref
            .rsplit_once('@')
            .map(|(_, digest)| digest)
            .ok_or_else(|| "oci_ref must be pinned to a content digest".to_string())?;
        if !valid_digest(embedded_digest)
            || !embedded_digest.eq_ignore_ascii_case(&self.content_digest)
        {
            return Err("OCI reference digest must match content_digest".to_string());
        }
        if self.manifest_ref.trim().is_empty() {
            return Err("manifest_ref required".to_string());
        }
        if require_signature && self.signature_ref.as_deref().unwrap_or("").is_empty() {
            return Err("signature reference required by profile".to_string());
        }
        if require_provenance && self.slsa_provenance_ref.as_deref().unwrap_or("").is_empty() {
            return Err("SLSA provenance reference required by profile".to_string());
        }
        for layer in &self.layers {
            if !valid_digest(&layer.digest) {
                return Err(format!("invalid layer digest {}", layer.digest));
            }
        }
        Ok(())
    }
}

fn valid_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(ch: char) -> String {
        format!("sha256:{}", ch.to_string().repeat(64))
    }

    #[test]
    fn capability_package_requires_content_addressing_and_optional_attestations() {
        let descriptor = CapabilityArtifactDescriptor {
            name: "equipment-investigator".to_string(),
            version: Version::v1(),
            manifest_ref: "capability:equipment-investigator@1".to_string(),
            oci_ref: format!(
                "oci://registry.example/morn/equipment-investigator@{}",
                digest('a')
            ),
            content_digest: digest('a'),
            media_type: "application/vnd.morn.capability.v1+json".to_string(),
            layers: vec![ArtifactLayer {
                media_type: "application/vnd.morn.capability.layer.v1".to_string(),
                digest: digest('b'),
                size: 42,
                annotations: vec![],
            }],
            sbom_ref: Some("oci://registry.example/sbom@sha256:fixture".to_string()),
            slsa_provenance_ref: Some(
                "oci://registry.example/provenance@sha256:fixture".to_string(),
            ),
            signature_ref: Some("sigstore://rekor/fixture".to_string()),
        };
        assert!(descriptor.validate(true, true).is_ok());
    }

    #[test]
    fn package_rejects_digest_mismatch_or_mutable_tag() {
        let mut descriptor = CapabilityArtifactDescriptor {
            name: "cap".to_string(),
            version: Version::v1(),
            manifest_ref: "cap:1".to_string(),
            oci_ref: format!("oci://registry.example/cap@{}", digest('a')),
            content_digest: digest('b'),
            media_type: "application/vnd.morn.capability.v1+json".to_string(),
            layers: vec![],
            sbom_ref: None,
            slsa_provenance_ref: None,
            signature_ref: None,
        };
        assert!(descriptor.validate(false, false).is_err());

        descriptor.oci_ref = "oci://registry.example/cap:latest".to_string();
        descriptor.content_digest = digest('a');
        assert!(descriptor.validate(false, false).is_err());

        descriptor.oci_ref = format!("oci://registry.example/cap@{}", digest('a'));
        assert!(descriptor.validate(false, false).is_ok());
    }

    #[test]
    fn unsigned_package_can_be_rejected_by_strict_profile() {
        let descriptor = CapabilityArtifactDescriptor {
            name: "cap".to_string(),
            version: Version::v1(),
            manifest_ref: "cap:1".to_string(),
            oci_ref: format!("oci://registry.example/cap@{}", digest('c')),
            content_digest: digest('c'),
            media_type: "application/vnd.morn.capability.v1+json".to_string(),
            layers: vec![],
            sbom_ref: None,
            slsa_provenance_ref: None,
            signature_ref: None,
        };
        assert!(descriptor.validate(false, false).is_ok());
        assert!(descriptor.validate(true, true).is_err());
    }
}
