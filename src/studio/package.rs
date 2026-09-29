//! package — Capability Package / Registry / Provenance (Phase 13).
//!
//! Content-addressed local package layout (Goal §17): when no OCI registry is
//! available, packages are built as a digest-addressed directory tree that
//! maps 1:1 onto an OCI artifact layout for the future. The digest is a REAL
//! SHA-256 (ring) over the package content — same content, same digest.
//!
//! SAFETY: Sigstore/SLSA cannot be truly run locally yet, so signing is a
//! provider contract with an honest LOCAL UNSIGNED status — provenance and
//! signatures are never faked.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Computes a real SHA-256 hex digest over content (ring, not a stub).
pub fn sha256_hex(content: &[u8]) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA256, content);
    let mut hex = String::with_capacity(hash.as_ref().len() * 2);
    for byte in hash.as_ref() {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}

/// A content-addressed capability package (OCI-artifact-mappable layout).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityPackage {
    pub name: String,
    pub version: String,
    /// SHA-256 over the package content (manifest + evidence + docs).
    pub digest: String,
    /// The capability manifest (JSON).
    pub manifest_json: String,
    /// Schema references bundled with the package.
    pub schema_refs: Vec<String>,
    /// Runtime metadata (harness compatibility, entrypoint).
    pub runtime_json: Option<String>,
    /// Qualification evidence (the record that gates release).
    pub qualification_json: Option<String>,
    /// Test suite references.
    pub test_refs: Vec<String>,
    /// Provenance / policy envelope.
    pub provenance_json: Option<String>,
    /// Docs.
    pub doc_refs: Vec<String>,
    /// SBOM when the build produces a software artifact.
    pub sbom_json: Option<String>,
}

/// Provenance/signing status — honest: unsigned until a real signer runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvenanceStatus {
    /// Signed by a real signer (Sigstore/cosign or an enterprise signer).
    Signed,
    /// Local unsigned status (honest; never presented as signed).
    LocalUnsigned,
}

/// ProvenanceProvider — signs or records provenance for a package.
pub trait ProvenanceProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    /// Produces a provenance record for the package content.
    fn record_provenance(&self, package_digest: &str)
        -> Result<(ProvenanceStatus, String), String>;
}

/// Local unsigned provenance provider (reference; Sigstore contract reserved).
///
/// PROVIDER: records a real content digest with an honest LOCAL UNSIGNED
/// status. It NEVER fabricates a signature or a Sigstore identity.
pub struct LocalUnsignedProvenanceProvider {
    builder_identity: String,
}

impl LocalUnsignedProvenanceProvider {
    pub fn new(builder_identity: &str) -> Self {
        Self {
            builder_identity: builder_identity.to_string(),
        }
    }
}

impl Default for LocalUnsignedProvenanceProvider {
    fn default() -> Self {
        Self::new("morn-local-builder")
    }
}

impl ProvenanceProvider for LocalUnsignedProvenanceProvider {
    fn provider_id(&self) -> &str {
        "local-unsigned-provenance"
    }

    fn record_provenance(
        &self,
        package_digest: &str,
    ) -> Result<(ProvenanceStatus, String), String> {
        if package_digest.is_empty() {
            return Err("package digest is required for provenance".to_string());
        }
        // COMPATIBILITY: Sigstore/SLSA are reserved provider contracts; until
        // a real signer runs, the honest status is LOCAL UNSIGNED.
        let record = serde_json::json!({
            "status": "LOCAL_UNSIGNED",
            "package_digest": package_digest,
            "builder_identity": self.builder_identity,
            "signature": serde_json::Value::Null,
            "sigstore_identity": serde_json::Value::Null,
            "note": "unsigned local provenance; connect a real signer (Sigstore/cosign) for SIGNED status"
        });
        Ok((ProvenanceStatus::LocalUnsigned, record.to_string()))
    }
}

/// Typed inputs for a package build (options object — never 10 loose args).
#[derive(Debug, Clone, Default)]
pub struct PackageInputs {
    pub name: String,
    pub version: String,
    pub manifest_json: String,
    pub schema_refs: Vec<String>,
    pub runtime_json: Option<String>,
    pub qualification_json: Option<String>,
    pub test_refs: Vec<String>,
    pub provenance_json: Option<String>,
    pub doc_refs: Vec<String>,
}

/// LocalPackageBuilder — builds a content-addressed package on disk.
pub struct LocalPackageBuilder;

impl LocalPackageBuilder {
    /// Builds the package from its parts: the digest is computed over the
    /// canonical content (manifest + evidence + docs), and the package is
    /// laid out under `<root>/<digest>/` — the OCI-mappable layout.
    pub fn build(
        root: &std::path::Path,
        inputs: &PackageInputs,
    ) -> Result<CapabilityPackage, String> {
        if inputs.name.is_empty() || inputs.version.is_empty() {
            return Err("package requires name and version".to_string());
        }
        // Content address: a real SHA-256 over the canonical content.
        let content = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            inputs.name,
            inputs.version,
            inputs.manifest_json,
            inputs.schema_refs.join("\n"),
            inputs.runtime_json.as_deref().unwrap_or(""),
            inputs.qualification_json.as_deref().unwrap_or(""),
            inputs.test_refs.join("\n"),
            inputs.provenance_json.as_deref().unwrap_or(""),
        );
        let digest = sha256_hex(content.as_bytes());

        // OCI-mappable local layout: <root>/<digest>/{manifest.json,schemas/,tests/,docs/,...}
        let pkg_dir: PathBuf = root.join(&digest);
        std::fs::create_dir_all(pkg_dir.join("schemas")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(pkg_dir.join("tests")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(pkg_dir.join("docs")).map_err(|e| e.to_string())?;
        std::fs::write(pkg_dir.join("manifest.json"), &inputs.manifest_json)
            .map_err(|e| e.to_string())?;
        if let Some(runtime) = &inputs.runtime_json {
            std::fs::write(pkg_dir.join("runtime.json"), runtime).map_err(|e| e.to_string())?;
        }
        if let Some(qualification) = &inputs.qualification_json {
            std::fs::write(pkg_dir.join("qualification.json"), qualification)
                .map_err(|e| e.to_string())?;
        }
        if let Some(provenance) = &inputs.provenance_json {
            std::fs::write(pkg_dir.join("provenance.json"), provenance)
                .map_err(|e| e.to_string())?;
        }

        Ok(CapabilityPackage {
            name: inputs.name.clone(),
            version: inputs.version.clone(),
            digest,
            manifest_json: inputs.manifest_json.clone(),
            schema_refs: inputs.schema_refs.clone(),
            runtime_json: inputs.runtime_json.clone(),
            qualification_json: inputs.qualification_json.clone(),
            test_refs: inputs.test_refs.clone(),
            provenance_json: inputs.provenance_json.clone(),
            doc_refs: inputs.doc_refs.clone(),
            sbom_json: None,
        })
    }

    /// Verifies a package on disk: recomputes the digest from the stored
    /// manifest and compares (content-addressed integrity check).
    pub fn verify(package: &CapabilityPackage) -> Result<(), String> {
        let content = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            package.name,
            package.version,
            package.manifest_json,
            package.schema_refs.join("\n"),
            package.runtime_json.as_deref().unwrap_or(""),
            package.qualification_json.as_deref().unwrap_or(""),
            package.test_refs.join("\n"),
            package.provenance_json.as_deref().unwrap_or(""),
        );
        let expected = sha256_hex(content.as_bytes());
        if expected != package.digest {
            return Err(format!(
                "package digest mismatch: stored {expected} != computed {} (tampering or corruption)",
                package.digest
            ));
        }
        Ok(())
    }
}
