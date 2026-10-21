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

pub fn validate_supply_chain_verifier_build_ref(value: &str) -> Result<(), String> {
    let value = value.trim();
    let Some(sha) = value.strip_prefix("git://huangdi97/morn/") else {
        return Err(
            "supply-chain verifier build must use git://huangdi97/morn/<40-hex-sha>".to_string(),
        );
    };
    if sha.len() != 40
        || !sha.bytes().all(|byte| byte.is_ascii_hexdigit())
        || sha.bytes().any(|byte| byte.is_ascii_uppercase())
    {
        return Err(
            "supply-chain verifier build must use git://huangdi97/morn/<40-lowercase-hex-sha>"
                .to_string(),
        );
    }
    Ok(())
}

pub fn supply_chain_verification_context_ref(kind: &str, value: &str) -> Result<String, String> {
    let kind = kind.trim();
    let value = value.trim();
    if kind.is_empty()
        || value.is_empty()
        || kind.chars().any(char::is_control)
        || value.chars().any(char::is_control)
    {
        return Err(
            "supply-chain verification context requires single-line kind and value".to_string(),
        );
    }
    Ok(format!("morn-supply-chain://{kind}?value={value}"))
}

pub fn supply_chain_subject_ref(oci_ref: &str) -> Result<String, String> {
    let oci_ref = oci_ref.trim();
    if oci_ref.is_empty()
        || oci_ref.chars().any(char::is_control)
        || oci_ref.chars().any(char::is_whitespace)
        || !oci_ref.starts_with("oci://")
    {
        return Err(
            "supply-chain subject must be a non-empty single-line digest-pinned oci:// reference"
                .to_string(),
        );
    }
    let (repository, digest) = oci_ref
        .rsplit_once('@')
        .ok_or_else(|| "supply-chain subject must be pinned to a content digest".to_string())?;
    if repository.trim_start_matches("oci://").is_empty()
        || repository.contains('@')
        || !valid_digest(digest)
    {
        return Err(
            "supply-chain subject must be an exact digest-pinned OCI repository reference"
                .to_string(),
        );
    }
    supply_chain_verification_context_ref("oci-subject", oci_ref)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyChainVerificationEvidence {
    /// Exact published OCI manifest digest that was verified.
    pub subject_digest: String,
    /// Stable verifier implementation / policy identity, never a human-entered
    /// free-form "signed" flag.
    pub verifier_ref: String,
    pub signature_verified: bool,
    pub provenance_verified: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

impl SupplyChainVerificationEvidence {
    pub fn validate_subject_digest(
        &self,
        subject_digest: &str,
        require_signature: bool,
        require_provenance: bool,
    ) -> Result<(), String> {
        if !valid_digest(subject_digest)
            || !valid_digest(&self.subject_digest)
            || !self.subject_digest.eq_ignore_ascii_case(subject_digest)
        {
            return Err("verification evidence must bind the exact artifact digest".to_string());
        }
        if self.verifier_ref.trim().is_empty() || self.evidence_refs.is_empty() {
            return Err(
                "verification evidence requires verifier identity and evidence refs".to_string(),
            );
        }
        if require_signature && !self.signature_verified {
            return Err("artifact signature has not been independently verified".to_string());
        }
        if require_provenance && !self.provenance_verified {
            return Err("artifact provenance has not been independently verified".to_string());
        }
        Ok(())
    }

    pub fn validate_for(
        &self,
        descriptor: &CapabilityArtifactDescriptor,
        require_signature: bool,
        require_provenance: bool,
    ) -> Result<(), String> {
        descriptor.validate(require_signature, require_provenance)?;
        self.validate_for_oci_subject(
            &descriptor.oci_ref,
            &descriptor.content_digest,
            require_signature,
            require_provenance,
        )
    }

    pub fn validate_bound_subject_ref(&self) -> Result<&str, String> {
        let prefix = "morn-supply-chain://oci-subject?value=";
        let subject_refs = self
            .evidence_refs
            .iter()
            .filter_map(|reference| reference.strip_prefix(prefix))
            .collect::<Vec<_>>();
        if subject_refs.len() != 1 {
            return Err(
                "supply-chain verification evidence requires exactly one OCI subject witness"
                    .to_string(),
            );
        }
        let oci_ref = subject_refs[0];
        let expected = supply_chain_subject_ref(oci_ref)?;
        if !self.evidence_refs.iter().any(|reference| reference == &expected) {
            return Err("supply-chain OCI subject witness is not canonical".to_string());
        }
        let digest = oci_ref
            .rsplit_once('@')
            .map(|(_, digest)| digest)
            .ok_or_else(|| "supply-chain OCI subject witness is not digest-pinned".to_string())?;
        if !digest.eq_ignore_ascii_case(&self.subject_digest) {
            return Err(
                "supply-chain OCI subject witness digest does not match verification subject"
                    .to_string(),
            );
        }
        Ok(oci_ref)
    }

    pub fn validate_for_oci_subject(
        &self,
        oci_ref: &str,
        subject_digest: &str,
        require_signature: bool,
        require_provenance: bool,
    ) -> Result<(), String> {
        self.validate_subject_digest(subject_digest, require_signature, require_provenance)?;
        let expected = supply_chain_subject_ref(oci_ref)?;
        let actual = self.validate_bound_subject_ref()?;
        let actual_ref = supply_chain_subject_ref(actual)?;
        if actual_ref != expected {
            return Err(
                "verification evidence must bind the exact OCI repository subject".to_string(),
            );
        }
        Ok(())
    }

    pub fn merge(&self, other: &Self) -> Result<Self, String> {
        if !self
            .subject_digest
            .eq_ignore_ascii_case(&other.subject_digest)
        {
            return Err(
                "cannot merge supply-chain evidence for different artifact digests".to_string(),
            );
        }
        let self_subject = self.validate_bound_subject_ref()?;
        let other_subject = other.validate_bound_subject_ref()?;
        if self_subject != other_subject {
            return Err(
                "cannot merge supply-chain evidence verified against different OCI subjects"
                    .to_string(),
            );
        }
        let mut evidence_refs = self.evidence_refs.clone();
        for reference in &other.evidence_refs {
            if !evidence_refs.contains(reference) {
                evidence_refs.push(reference.clone());
            }
        }
        Ok(Self {
            subject_digest: self.subject_digest.clone(),
            verifier_ref: format!("{} + {}", self.verifier_ref, other.verifier_ref),
            signature_verified: self.signature_verified || other.signature_verified,
            provenance_verified: self.provenance_verified || other.provenance_verified,
            evidence_refs,
        })
    }
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
    fn verifier_build_ref_is_exact_repository_source_sha() {
        assert!(validate_supply_chain_verifier_build_ref(
            "git://huangdi97/morn/0123456789abcdef0123456789abcdef01234567"
        )
        .is_ok());
        for invalid in [
            "git://other/morn/0123456789abcdef0123456789abcdef01234567",
            "git://huangdi97/morn/main",
            "git://huangdi97/morn/ABCDEF0123456789abcdef0123456789abcdef01",
            "git://huangdi97/morn/0123456789abcdef0123456789abcdef0123456",
        ] {
            assert!(
                validate_supply_chain_verifier_build_ref(invalid).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn verification_context_refs_are_explicit_and_single_line() {
        assert_eq!(
            supply_chain_verification_context_ref("release-runner", "runner://release-1").unwrap(),
            "morn-supply-chain://release-runner?value=runner://release-1"
        );
        assert!(supply_chain_verification_context_ref("release-runner", "runner\nother").is_err());
        assert!(supply_chain_verification_context_ref("", "runner").is_err());
    }

    #[test]
    fn verification_axes_merge_only_for_the_same_exact_oci_subject() {
        let subject = format!("oci://registry.example/morn/cap@{}", digest('f'));
        let subject_ref = supply_chain_subject_ref(&subject).unwrap();
        let signature = SupplyChainVerificationEvidence {
            subject_digest: digest('f'),
            verifier_ref: "cosign://signature".to_string(),
            signature_verified: true,
            provenance_verified: false,
            evidence_refs: vec![subject_ref.clone(), "proof://signature".to_string()],
        };
        let provenance = SupplyChainVerificationEvidence {
            subject_digest: digest('f'),
            verifier_ref: "cosign://slsa".to_string(),
            signature_verified: false,
            provenance_verified: true,
            evidence_refs: vec![subject_ref, "proof://provenance".to_string()],
        };
        let merged = signature.merge(&provenance).unwrap();
        assert!(merged.signature_verified);
        assert!(merged.provenance_verified);
        assert_eq!(merged.evidence_refs.len(), 3);

        let mut wrong_digest = provenance.clone();
        wrong_digest.subject_digest = digest('e');
        assert!(signature.merge(&wrong_digest).is_err());

        let mut wrong_repository = provenance;
        wrong_repository.evidence_refs[0] = supply_chain_subject_ref(&format!(
            "oci://registry.example/other/cap@{}",
            digest('f')
        ))
        .unwrap();
        assert!(signature.merge(&wrong_repository).is_err());
    }

    #[test]
    fn strict_supply_chain_requires_verified_evidence_for_the_exact_digest() {
        let descriptor = CapabilityArtifactDescriptor {
            name: "cap".to_string(),
            version: Version::v1(),
            manifest_ref: "cap:1".to_string(),
            oci_ref: format!("oci://registry.example/cap@{}", digest('d')),
            content_digest: digest('d'),
            media_type: "application/vnd.morn.capability.v1+json".to_string(),
            layers: vec![],
            sbom_ref: None,
            slsa_provenance_ref: Some(
                "oci://registry.example/provenance@sha256:fixture".to_string(),
            ),
            signature_ref: Some("sigstore://bundle/ref".to_string()),
        };
        let mut evidence = SupplyChainVerificationEvidence {
            subject_digest: digest('d'),
            verifier_ref: "cosign://policy/github-oidc".to_string(),
            signature_verified: true,
            provenance_verified: true,
            evidence_refs: vec![
                supply_chain_subject_ref(&descriptor.oci_ref).unwrap(),
                "sigstore://bundle/ref".to_string(),
            ],
        };
        assert!(evidence.validate_for(&descriptor, true, true).is_ok());

        evidence.subject_digest = digest('e');
        assert!(evidence.validate_for(&descriptor, true, true).is_err());
        evidence.subject_digest = digest('d');
        evidence.signature_verified = false;
        assert!(evidence.validate_for(&descriptor, true, true).is_err());
        evidence.signature_verified = true;
        evidence.provenance_verified = false;
        assert!(evidence.validate_for(&descriptor, true, true).is_err());
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
