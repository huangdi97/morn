//! Domain guarantee profiles.
//!
//! A profile is not a plugin list. It declares guarantees that a composition
//! must satisfy; providers are free to vary as long as conformance stays true.

use serde::{Deserialize, Serialize};

use morn_kernel::version::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RequirementLevel {
    Required,
    Optional,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuaranteeRequirement {
    pub semantic: String,
    pub level: RequirementLevel,
    pub contexts: Vec<String>,
}

impl GuaranteeRequirement {
    pub fn required(semantic: impl Into<String>) -> Self {
        Self {
            semantic: semantic.into(),
            level: RequirementLevel::Required,
            contexts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainProfile {
    pub id: String,
    pub version: Version,
    pub requirements: Vec<GuaranteeRequirement>,
    pub minimum_isolation: String,
    pub source_of_truth_binding_required: bool,
    pub durable_work_state_required: bool,
    pub provenance_required: bool,
}

impl DomainProfile {
    pub fn requires(&self, semantic: &str) -> bool {
        self.requirements.iter().any(|requirement| {
            requirement.semantic == semantic && requirement.level == RequirementLevel::Required
        })
    }

    /// First Factory product profile: brownfield/read-first. It requires
    /// governance and reconciliation semantics but does not grant production
    /// write authority merely because the profile is installed.
    pub fn factory_readonly_v1() -> Self {
        Self {
            id: "morn.factory.readonly".to_string(),
            version: Version::new(1, 0, 0),
            requirements: [
                "DurableWorkState",
                "SourceOfTruthBinding",
                "CapabilityQualification",
                "AuthorityBeforeSideEffect",
                "ReceiptAfterExternalAction",
                "ReconciliationOnUnknown",
                "OutcomeObservation",
                "IndependentAcceptance",
                "Provenance",
            ]
            .into_iter()
            .map(GuaranteeRequirement::required)
            .collect(),
            minimum_isolation: "container".to_string(),
            source_of_truth_binding_required: true,
            durable_work_state_required: true,
            provenance_required: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_profile_requires_reconciliation_but_does_not_name_a_provider() {
        let profile = DomainProfile::factory_readonly_v1();
        assert!(profile.requires("ReconciliationOnUnknown"));
        assert!(profile.requires("AuthorityBeforeSideEffect"));
        assert!(!profile
            .requirements
            .iter()
            .any(|requirement| requirement.semantic.contains("DeepSeek")));
    }
}
