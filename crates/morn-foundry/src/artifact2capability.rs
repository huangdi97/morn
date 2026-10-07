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
            return Err(Error::invalid_state(
                "procedure must contain at least one step",
            ));
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


#[derive(Debug, Default)]
pub struct RepositoryManifestCompiler;

impl ArtifactCompiler for RepositoryManifestCompiler {
    fn compiler_name(&self) -> &str {
        "repository-manifest-json"
    }

    fn supports(&self, kind: &ArtifactKind) -> bool {
        *kind == ArtifactKind::Repository
    }

    fn compile(&self, source: &ArtifactSource) -> Result<CandidateCapability> {
        if !self.supports(&source.kind) {
            return Err(Error::invalid_state(
                "compiler does not support artifact kind",
            ));
        }

        let doc: serde_json::Value = serde_json::from_str(&source.content)
            .map_err(|error| Error::external(format!("invalid repository manifest JSON: {error}")))?;
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
                "repository manifest must explicitly declare provides",
            ));
        }
        let entrypoints: Vec<String> = doc
            .get("entrypoints")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if entrypoints.is_empty() {
            return Err(Error::validation(
                "repository manifest must explicitly declare at least one entrypoint",
            ));
        }

        let kind = match doc
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("program")
        {
            "program" => CapabilityKind::Program,
            "service" => CapabilityKind::Service,
            "solver" => CapabilityKind::Solver,
            "model" => CapabilityKind::Model,
            "llm" => CapabilityKind::Llm,
            "agent" => CapabilityKind::Agent,
            "hybrid" => CapabilityKind::Hybrid,
            other => {
                return Err(Error::validation(format!(
                    "unsupported repository capability kind {other}"
                )))
            }
        };

        let maximum_effect = match doc
            .get("maximum_effect")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("E0")
        {
            "E0" => EffectClass::E0LifecycleReversible,
            "E1" => EffectClass::E1Transactional,
            "E2" => EffectClass::E2Compensatable,
            "E3" => EffectClass::E3Irreversible,
            other => {
                return Err(Error::validation(format!(
                    "unsupported repository maximum_effect {other}"
                )))
            }
        };

        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            &source.name,
            &source.source_ref,
            kind,
            maximum_effect,
        );
        manifest.provides = provides;
        manifest.interfaces.push(CapabilityInterface {
            protocol: "repository-entrypoint".to_string(),
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
                discovered_operations: entrypoints,
                warnings: vec![
                    "repository compiler trusts only explicitly declared entrypoints/provides"
                        .to_string(),
                    "compiled candidate is not qualified or site-admitted".to_string(),
                ],
            },
        })
    }
}

/// Paper/research compiler inspired by Paper2Agent, but deliberately requires a
/// reviewed structured extraction manifest. It does not infer executable tools
/// directly from free-form paper text.
#[derive(Debug, Default)]
pub struct ReviewedPaperManifestCompiler;

impl ArtifactCompiler for ReviewedPaperManifestCompiler {
    fn compiler_name(&self) -> &str {
        "reviewed-paper-manifest-json"
    }

    fn supports(&self, kind: &ArtifactKind) -> bool {
        *kind == ArtifactKind::Paper
    }

    fn compile(&self, source: &ArtifactSource) -> Result<CandidateCapability> {
        if !self.supports(&source.kind) {
            return Err(Error::invalid_state(
                "compiler does not support artifact kind",
            ));
        }

        let doc: serde_json::Value = serde_json::from_str(&source.content)
            .map_err(|error| Error::external(format!("invalid paper manifest JSON: {error}")))?;

        let reviewed = doc
            .get("reviewed")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !reviewed {
            return Err(Error::validation(
                "paper capability extraction must be explicitly reviewed",
            ));
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
                "paper manifest must explicitly declare provides",
            ));
        }

        let code_bindings: Vec<String> = doc
            .get("code_bindings")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let evidence_refs: Vec<String> = doc
            .get("evidence_refs")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        if evidence_refs.is_empty() {
            return Err(Error::validation(
                "paper manifest requires explicit evidence references",
            ));
        }

        let executable = doc
            .get("executable")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if executable && code_bindings.is_empty() {
            return Err(Error::validation(
                "executable paper-derived capability requires real code bindings",
            ));
        }

        let kind = if executable {
            CapabilityKind::Program
        } else {
            CapabilityKind::Model
        };
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            &source.name,
            &source.source_ref,
            kind,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provides = provides;
        manifest.interfaces.push(CapabilityInterface {
            protocol: if executable {
                "paper-code-binding".to_string()
            } else {
                "paper-evidence".to_string()
            },
            input_schema_ref: format!("{}#input", source.source_ref),
            output_schema_ref: format!("{}#output", source.source_ref),
        });
        manifest.provenance.source_ref = source.source_ref.clone();
        manifest.provenance.source_digest = source.source_digest.clone();

        let mut warnings = vec![
            "paper-derived capability remains Declared until independent evaluation"
                .to_string(),
        ];
        if !executable {
            warnings.push(
                "non-executable paper capability represents reviewed knowledge/evidence only"
                    .to_string(),
            );
        }

        Ok(CandidateCapability {
            record: CapabilityRecord::new(manifest),
            report: CompilationReport {
                compiler: self.compiler_name().to_string(),
                source_ref: source.source_ref.clone(),
                discovered_operations: if executable {
                    code_bindings
                } else {
                    evidence_refs
                },
                warnings,
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
    fn repository_compiler_requires_explicit_entrypoints() {
        let source = ArtifactSource {
            kind: ArtifactKind::Repository,
            name: "capacity-solver".to_string(),
            source_ref: "repo://solver".to_string(),
            source_digest: Some("sha256:repo".to_string()),
            content: r#"{
                "kind":"solver",
                "provides":["capacity.optimize"],
                "entrypoints":["bin/solve"],
                "maximum_effect":"E0"
            }"#
            .to_string(),
        };
        let candidate = RepositoryManifestCompiler.compile(&source).unwrap();
        assert_eq!(candidate.record.stage, CapabilityStage::Declared);
        assert_eq!(candidate.report.discovered_operations, vec!["bin/solve"]);
    }

    #[test]
    fn executable_paper_capability_requires_review_and_code_binding() {
        let source = ArtifactSource {
            kind: ArtifactKind::Paper,
            name: "paper-method".to_string(),
            source_ref: "doi://example".to_string(),
            source_digest: Some("sha256:paper".to_string()),
            content: r#"{
                "reviewed":true,
                "executable":true,
                "provides":["method.execute"],
                "code_bindings":["repo://paper-code#run"],
                "evidence_refs":["doi://example#method"]
            }"#
            .to_string(),
        };
        let candidate = ReviewedPaperManifestCompiler.compile(&source).unwrap();
        assert_eq!(candidate.record.stage, CapabilityStage::Declared);
        assert_eq!(
            candidate.report.discovered_operations,
            vec!["repo://paper-code#run".to_string()]
        );
    }

    #[test]
    fn unreviewed_paper_does_not_become_a_capability() {
        let source = ArtifactSource {
            kind: ArtifactKind::Paper,
            name: "unreviewed".to_string(),
            source_ref: "doi://unreviewed".to_string(),
            source_digest: None,
            content: r#"{
                "reviewed":false,
                "provides":["x"],
                "evidence_refs":["doi://unreviewed"]
            }"#
            .to_string(),
        };
        assert!(ReviewedPaperManifestCompiler.compile(&source).is_err());
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
