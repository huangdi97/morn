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
            return Err(Error::invalid_state(
                "compiler does not support artifact kind",
            ));
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
                warnings: vec!["compiled candidate is not qualified or site-admitted".to_string()],
            },
        })
    }
}


/// Structured SOP/procedure compiler. It accepts an explicit JSON procedure
/// document and produces a candidate capability without inferring hidden steps.
/// The compiler intentionally refuses an empty or malformed procedure.
#[derive(Debug, Default)]
pub struct ProcedureJsonCompiler;

impl ArtifactCompiler for ProcedureJsonCompiler {
    fn compiler_name(&self) -> &str {
        "procedure-json"
    }

    fn supports(&self, kind: &ArtifactKind) -> bool {
        *kind == ArtifactKind::Procedure
    }

    fn compile(&self, source: &ArtifactSource) -> Result<CandidateCapability> {
        if !self.supports(&source.kind) {
            return Err(Error::invalid_state(
                "compiler does not support artifact kind",
            ));
        }

        let doc: serde_json::Value = serde_json::from_str(&source.content)
            .map_err(|error| Error::external(format!("invalid procedure JSON: {error}")))?;
        let steps = doc
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| Error::invalid_state("procedure document has no steps array"))?;
        if steps.is_empty() {
            return Err(Error::invalid_state("procedure must contain at least one step"));
        }

        let mut discovered = Vec::new();
        let mut maximum_effect = EffectClass::E0LifecycleReversible;
        for step in steps {
            let id = step
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| Error::validation("every procedure step requires an id"))?;
            let capability = step
                .get("capability")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("human-or-unresolved");
            let effect = step
                .get("effect")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("E0");
            maximum_effect = match effect {
                "E0" => maximum_effect,
                "E1" if matches!(maximum_effect, EffectClass::E0LifecycleReversible) => {
                    EffectClass::E1Transactional
                }
                "E2" if !matches!(maximum_effect, EffectClass::E3Irreversible) => {
                    EffectClass::E2Compensatable
                }
                "E3" => EffectClass::E3Irreversible,
                "E0" | "E1" | "E2" => maximum_effect,
                other => {
                    return Err(Error::validation(format!(
                        "unsupported procedure effect class {other}"
                    )))
                }
            };
            discovered.push(format!("{id}:{capability}:{effect}"));
        }

        let provides: Vec<String> = doc
            .get("provides")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if provides.is_empty() {
            return Err(Error::validation(
                "procedure must explicitly declare at least one provided capability",
            ));
        }

        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            &source.name,
            &source.source_ref,
            CapabilityKind::Hybrid,
            maximum_effect,
        );
        manifest.provides = provides;
        manifest.interfaces.push(CapabilityInterface {
            protocol: "morn-procedure-json".to_string(),
            input_schema_ref: format!("{}#input", source.source_ref),
            output_schema_ref: format!("{}#output", source.source_ref),
        });
        manifest.provenance.source_ref = source.source_ref.clone();
        manifest.provenance.source_digest = source.source_digest.clone();

        Ok(CandidateCapability {
            record: CapabilityRecord::new(manifest),
            report: CompilationReport {
                compiler: self.compiler_name().to_string(),
                source_ref: source.source_ref.clone(),
                discovered_operations: discovered,
                warnings: vec![
                    "procedure steps are declared source facts; unresolved executors remain unresolved"
                        .to_string(),
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
    fn procedure_compiler_preserves_explicit_steps_and_stays_declared() {
        let source = ArtifactSource {
            kind: ArtifactKind::Procedure,
            name: "outage-review-sop".to_string(),
            source_ref: "sop://factory/outage-review".to_string(),
            source_digest: Some("sha256:def".to_string()),
            content: r#"{
                "provides":["factory.outage.review"],
                "steps":[
                    {"id":"observe","capability":"historian.read","effect":"E0"},
                    {"id":"review","capability":"human.approve","effect":"E0"}
                ]
            }"#
            .to_string(),
        };

        let candidate = ProcedureJsonCompiler.compile(&source).unwrap();
        assert_eq!(candidate.record.stage, CapabilityStage::Declared);
        assert_eq!(
            candidate.record.manifest.provides,
            vec!["factory.outage.review".to_string()]
        );
        assert_eq!(candidate.report.discovered_operations.len(), 2);
    }

    #[test]
    fn procedure_compiler_rejects_implicit_empty_work() {
        let source = ArtifactSource {
            kind: ArtifactKind::Procedure,
            name: "empty".to_string(),
            source_ref: "sop://empty".to_string(),
            source_digest: None,
            content: r#"{"provides":["x"],"steps":[]}"#.to_string(),
        };
        assert!(ProcedureJsonCompiler.compile(&source).is_err());
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
