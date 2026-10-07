//! Artifact-to-capability compilation.
//!
//! A compiler can turn an existing asset into a candidate capability manifest,
//! but compilation never means the capability is qualified, released or admitted.

use serde::{Deserialize, Serialize};

use morn_capability::effect::EffectClass;
use morn_capability::manifest::{CapabilityInterface, CapabilityManifest, CapabilityRecord};
use morn_capability::registry::CapabilityKind;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::CapabilityId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    OpenApi,
    Repository,
    Procedure,
    Paper,
    Model,
    Workflow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSource {
    pub kind: ArtifactKind,
    pub name: String,
    pub source_ref: String,
    pub source_digest: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationReport {
    pub compiler: String,
    pub source_ref: String,
    pub discovered_operations: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateCapability {
    pub record: CapabilityRecord,
    pub report: CompilationReport,
}

pub trait ArtifactCompiler: Send + Sync {
    fn compiler_name(&self) -> &str;
    fn supports(&self, kind: &ArtifactKind) -> bool;
    fn compile(&self, source: &ArtifactSource) -> Result<CandidateCapability>;
}

/// Minimal OpenAPI JSON compiler. It discovers method/path operations and emits
/// an API capability candidate. The candidate remains Declared until a separate
/// qualification/admission pipeline promotes it.
#[derive(Debug, Default)]
pub struct OpenApiJsonCompiler;

impl ArtifactCompiler for OpenApiJsonCompiler {
    fn compiler_name(&self) -> &str {
        "openapi-json"
    }

    fn supports(&self, kind: &ArtifactKind) -> bool {
        *kind == ArtifactKind::OpenApi
    }

    fn compile(&self, source: &ArtifactSource) -> Result<CandidateCapability> {
        if !self.supports(&source.kind) {
            return Err(Error::invalid_state("compiler does not support artifact kind"));
        }

        let doc: serde_json::Value = serde_json::from_str(&source.content)
            .map_err(|error| Error::external(format!("invalid OpenAPI JSON: {error}")))?;
        let paths = doc
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| Error::invalid_state("OpenAPI document has no paths object"))?;

        let mut operations = Vec::new();
        for (path, item) in paths {
            let Some(methods) = item.as_object() else {
                continue;
            };
            for method in ["get", "post", "put", "patch", "delete"] {
                if methods.contains_key(method) {
                    operations.push(format!("{method}:{path}"));
                }
            }
        }
        operations.sort();

        if operations.is_empty() {
            return Err(Error::invalid_state(
                "OpenAPI document contains no supported operations",
            ));
        }

        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            &source.name,
            &source.source_ref,
            CapabilityKind::Api,
            EffectClass::E2Compensatable,
        );
        manifest.provides = operations.clone();
        manifest.interfaces.push(CapabilityInterface {
            protocol: "openapi".to_string(),
            input_schema_ref: format!("{}#request", source.source_ref),
            output_schema_ref: format!("{}#response", source.source_ref),
        });
        manifest.provenance.source_ref = source.source_ref.clone();
        manifest.provenance.source_digest = source.source_digest.clone();

        Ok(CandidateCapability {
            record: CapabilityRecord::new(manifest),
            report: CompilationReport {
                compiler: self.compiler_name().to_string(),
                source_ref: source.source_ref.clone(),
                discovered_operations: operations,
                warnings: vec![
                    "compiled candidate is not qualified or site-admitted".to_string(),
                ],
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::manifest::CapabilityStage;

    #[test]
    fn openapi_compiler_creates_declared_candidate_only() {
        let source = ArtifactSource {
            kind: ArtifactKind::OpenApi,
            name: "cmms-api".to_string(),
            source_ref: "repo://cmms/openapi.json".to_string(),
            source_digest: Some("sha256:abc".to_string()),
            content: r#"{
                "openapi":"3.1.0",
                "paths":{
                    "/orders":{"get":{},"post":{}},
                    "/orders/{id}":{"get":{}}
                }
            }"#
            .to_string(),
        };

        let candidate = OpenApiJsonCompiler.compile(&source).unwrap();
        assert_eq!(candidate.record.stage, CapabilityStage::Declared);
        assert_eq!(candidate.report.discovered_operations.len(), 3);
        assert!(candidate
            .record
            .manifest
            .provides
            .iter()
            .any(|operation| operation == "post:/orders"));
    }

    #[test]
    fn invalid_openapi_does_not_fabricate_a_capability() {
        let source = ArtifactSource {
            kind: ArtifactKind::OpenApi,
            name: "broken".to_string(),
            source_ref: "repo://broken".to_string(),
            source_digest: None,
            content: "{}".to_string(),
        };
        assert!(OpenApiJsonCompiler.compile(&source).is_err());
    }
}
