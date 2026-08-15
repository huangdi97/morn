//! Domain SDK v1: lets a Domain Pack declare its ontology, objects, relations,
//! actions, events, artifacts, outcomes, roles, work templates, policies,
//! capabilities, connector requirements, evaluations and UI extensions —
//! without modifying Core.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DomainDefinitionTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DomainPackTag;

pub type DomainDefinitionId = Id<DomainDefinitionTag>;
pub type DomainPackId = Id<DomainPackTag>;

/// A declaration a domain pack can make.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainDeclaration {
    pub kind: String, // object_type | action | artifact | outcome | role | work_template | policy | capability | connector_requirement | evaluation | ui_extension
    pub name: String,
    pub payload: serde_json::Value,
}

/// A domain pack definition (registered by a pack via the public SDK).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainDefinition {
    pub id: DomainDefinitionId,
    pub domain: String,
    pub version: String,
    pub sdk_version: String,
    pub dependencies: Vec<String>,
    pub declarations: Vec<DomainDeclaration>,
}

impl DomainDefinition {
    pub fn new(domain: &str, version: &str, sdk_version: &str) -> Self {
        Self {
            id: DomainDefinitionId::generate_with("dom"),
            domain: domain.to_string(),
            version: version.to_string(),
            sdk_version: sdk_version.to_string(),
            dependencies: Vec::new(),
            declarations: Vec::new(),
        }
    }

    pub fn declare(mut self, kind: &str, name: &str, payload: serde_json::Value) -> Self {
        self.declarations.push(DomainDeclaration {
            kind: kind.to_string(),
            name: name.to_string(),
            payload,
        });
        self
    }

    /// Validate: domain/version/sdk required; declarations have known kinds.
    pub fn validate(&self) -> Result<(), String> {
        if self.domain.trim().is_empty()
            || self.version.trim().is_empty()
            || self.sdk_version.trim().is_empty()
        {
            return Err("domain/version/sdk_version required".to_string());
        }
        const KINDS: [&str; 13] = [
            "object_type",
            "relation",
            "action",
            "event",
            "artifact",
            "outcome",
            "role",
            "work_template",
            "policy",
            "capability",
            "connector_requirement",
            "evaluation",
            "ui_extension",
        ];
        for d in &self.declarations {
            if !KINDS.contains(&d.kind.as_str()) {
                return Err(format!("unknown declaration kind {}", d.kind));
            }
        }
        Ok(())
    }
}

/// Runtime domain registry (enabled/disabled packs).
#[derive(Debug, Default)]
pub struct DomainRegistry {
    pub definitions: Vec<DomainDefinition>,
    pub enabled: Vec<String>,
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install(&mut self, def: DomainDefinition) -> Result<(), String> {
        def.validate()?;
        self.definitions.push(def);
        Ok(())
    }

    pub fn enable(&mut self, domain: &str) -> bool {
        if self.definitions.iter().any(|d| d.domain == domain)
            && !self.enabled.contains(&domain.to_string())
        {
            self.enabled.push(domain.to_string());
            true
        } else {
            false
        }
    }

    pub fn disable(&mut self, domain: &str) {
        self.enabled.retain(|d| d != domain);
    }

    pub fn is_enabled(&self, domain: &str) -> bool {
        self.enabled.iter().any(|d| d == domain)
    }

    pub fn declarations_for(&self, domain: &str, kind: &str) -> Vec<&DomainDeclaration> {
        self.definitions
            .iter()
            .filter(|d| d.domain == domain && self.is_enabled(domain))
            .flat_map(|d| d.declarations.iter())
            .filter(|x| x.kind == kind)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn biolab_def() -> DomainDefinition {
        DomainDefinition::new("biolab", "1.0.0", "1.0.0")
            .declare(
                "object_type",
                "Dataset",
                serde_json::json!({"states": ["registered", "locked"]}),
            )
            .declare("work_template", "dataset_to_claim", serde_json::json!({}))
            .declare("ui_extension", "biolab-panel", serde_json::json!({}))
    }

    #[test]
    fn domain_sdk_install_enable_declare() {
        let mut reg = DomainRegistry::new();
        reg.install(biolab_def()).unwrap();
        assert!(reg.enable("biolab"));
        assert!(reg.is_enabled("biolab"));
        assert_eq!(reg.declarations_for("biolab", "object_type").len(), 1);
        assert_eq!(reg.declarations_for("biolab", "ui_extension").len(), 1);
        reg.disable("biolab");
        assert!(!reg.is_enabled("biolab"));
        assert!(reg.declarations_for("biolab", "object_type").is_empty());
    }

    #[test]
    fn connector_requirement_and_evaluation_kinds_valid() {
        let mut reg = DomainRegistry::new();
        let def = DomainDefinition::new("d", "1", "1")
            .declare(
                "connector_requirement",
                "lims",
                serde_json::json!({"auth": "oauth"}),
            )
            .declare(
                "evaluation",
                "eval-pack",
                serde_json::json!({"scenarios": []}),
            );
        reg.install(def).unwrap();
        assert_eq!(reg.definitions[0].declarations.len(), 2);
    }

    #[test]
    fn invalid_declaration_rejected() {
        let mut reg = DomainRegistry::new();
        let bad = DomainDefinition::new("x", "1", "1").declare(
            "unknown_kind",
            "y",
            serde_json::json!({}),
        );
        assert!(reg.install(bad).is_err());
    }
}
