//! Domain guarantee profiles.
//!
//! A profile is not a plugin list. It declares guarantees that a composition
//! must satisfy; providers are free to vary as long as conformance stays true.

pub mod compatibility;
pub mod conformance;
pub mod registry;

use serde::{Deserialize, Serialize};

use morn_kernel::version::Version;
use morn_kernel::ExecutionGuarantee;

pub use compatibility::{compare_profiles, plan_profile_migration, ProfileCompatibility, ProfileMigrationPlan};
pub use conformance::{evaluate_profile, ConformanceEvidence, ConformanceReport};
pub use registry::{reference_profile_registry, ProfileRegistry};

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

    pub fn optional(semantic: impl Into<String>) -> Self {
        Self {
            semantic: semantic.into(),
            level: RequirementLevel::Optional,
            contexts: Vec::new(),
        }
    }

    pub fn forbidden(semantic: impl Into<String>) -> Self {
        Self {
            semantic: semantic.into(),
            level: RequirementLevel::Forbidden,
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
    #[serde(default)]
    pub required_execution_guarantees: Vec<ExecutionGuarantee>,
    pub source_of_truth_binding_required: bool,
    pub durable_work_state_required: bool,
    pub provenance_required: bool,
}

impl DomainProfile {
    pub fn validate(&self) -> morn_kernel::error::Result<()> {
        use std::collections::BTreeMap;
        use morn_kernel::error::Error;
        use morn_kernel::ExecutionClass;

        if self.id.trim().is_empty() || self.id.contains('@') {
            return Err(Error::validation(
                "DomainProfile id must be non-empty and must not contain @",
            ));
        }
        if ExecutionClass::parse(&self.minimum_isolation).is_none() {
            return Err(Error::validation(format!(
                "unknown minimum isolation class {}",
                self.minimum_isolation
            )));
        }

        let mut semantics = BTreeMap::new();
        for requirement in &self.requirements {
            if requirement.semantic.trim().is_empty() {
                return Err(Error::validation(
                    "Profile requirement semantic must be non-empty",
                ));
            }
            if let Some(existing) = semantics.insert(
                requirement.semantic.clone(),
                requirement.level,
            ) {
                if existing != requirement.level {
                    return Err(Error::validation(format!(
                        "Profile semantic {} has conflicting requirement levels",
                        requirement.semantic
                    )));
                }
                return Err(Error::validation(format!(
                    "Profile semantic {} is duplicated",
                    requirement.semantic
                )));
            }
        }

        let mut execution_guarantees = self.required_execution_guarantees.clone();
        execution_guarantees.sort();
        execution_guarantees.dedup();
        if execution_guarantees.len() != self.required_execution_guarantees.len() {
            return Err(Error::validation(
                "Profile execution guarantee vector contains duplicates",
            ));
        }

        for (semantic, flag) in [
            ("DurableWorkState", self.durable_work_state_required),
            ("SourceOfTruthBinding", self.source_of_truth_binding_required),
            ("Provenance", self.provenance_required),
        ] {
            if self.requires(semantic) != flag {
                return Err(Error::validation(format!(
                    "Profile {semantic} semantic and explicit requirement flag disagree"
                )));
            }
        }

        Ok(())
    }

    pub fn canonical_ref(&self) -> String {
        format!(
            "{}@{}.{}.{}",
            self.id, self.version.major, self.version.minor, self.version.patch
        )
    }

    pub fn from_ref(profile_ref: &str) -> Option<Self> {
        [
            Self::lite_v1(),
            Self::enterprise_v1(),
            Self::factory_readonly_v1(),
            Self::research_v1(),
        ]
        .into_iter()
        .find(|profile| profile.canonical_ref() == profile_ref)
    }

    /// Work readiness gates are a subset of Profile guarantees. Post-execution
    /// guarantees (receipt, reconciliation, outcome and acceptance) must not
    /// block a Work from becoming Ready before an ExecutionBinding exists.
    pub fn pre_execution_work_conditions(&self) -> Vec<String> {
        let mut conditions = vec!["CapabilityResolved".to_string()];
        if self.requires("CapabilityQualification") {
            conditions.push("CapabilityQualified".to_string());
        }
        if self.requires("AuthorityBeforeSideEffect") {
            conditions.push("AuthoritySatisfied".to_string());
        }
        if self.requires("SourceOfTruthBinding") {
            conditions.push("SourceOfTruthBound".to_string());
        }
        if self.requires("Provenance") {
            conditions.push("ProvenanceReady".to_string());
        }
        conditions.sort();
        conditions.dedup();
        conditions
    }

    pub fn requires(&self, semantic: &str) -> bool {
        self.requirements.iter().any(|requirement| {
            requirement.semantic == semantic && requirement.level == RequirementLevel::Required
        })
    }

    pub fn forbids(&self, semantic: &str) -> bool {
        self.requirements.iter().any(|requirement| {
            requirement.semantic == semantic && requirement.level == RequirementLevel::Forbidden
        })
    }

    pub fn lite_v1() -> Self {
        Self {
            id: "morn.lite".to_string(),
            version: Version::new(1, 0, 0),
            requirements: [
                GuaranteeRequirement::required("WorkTruthIndependentOfHarness"),
                GuaranteeRequirement::required("ExplicitAcceptanceWhenDeclared"),
                GuaranteeRequirement::optional("DurableWorkState"),
                GuaranteeRequirement::optional("Provenance"),
            ]
            .to_vec(),
            minimum_isolation: "process".to_string(),
            required_execution_guarantees: Vec::new(),
            source_of_truth_binding_required: false,
            durable_work_state_required: false,
            provenance_required: false,
        }
    }

    pub fn enterprise_v1() -> Self {
        Self {
            id: "morn.enterprise".to_string(),
            version: Version::new(1, 0, 0),
            requirements: [
                "DurableWorkState",
                "CapabilityQualification",
                "AuthorityBeforeSideEffect",
                "ReceiptAfterExternalAction",
                "ReconciliationOnUnknown",
                "IndependentAcceptance",
                "ProfileVersionPinned",
                "Provenance",
            ]
            .into_iter()
            .map(GuaranteeRequirement::required)
            .collect(),
            minimum_isolation: "container".to_string(),
            required_execution_guarantees: vec![ExecutionGuarantee::SecretIndirection],
            source_of_truth_binding_required: false,
            durable_work_state_required: true,
            provenance_required: true,
        }
    }

    pub fn research_v1() -> Self {
        Self {
            id: "morn.research".to_string(),
            version: Version::new(1, 0, 0),
            requirements: [
                "DurableWorkState",
                "DatasetVersionPinned",
                "CodeVersionPinned",
                "EnvironmentDigest",
                "ReproducibilityEvidence",
                "IndependentAcceptance",
                "Provenance",
            ]
            .into_iter()
            .map(GuaranteeRequirement::required)
            .collect(),
            minimum_isolation: "container".to_string(),
            required_execution_guarantees: vec![
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
            ],
            source_of_truth_binding_required: false,
            durable_work_state_required: true,
            provenance_required: true,
        }
    }

    /// First Factory product profile: brownfield/read-first. It requires
    /// governance and reconciliation semantics but does not grant production
    /// write authority merely because the profile is installed.
    pub fn factory_readonly_v1() -> Self {
        Self {
            id: "morn.factory.readonly".to_string(),
            version: Version::new(1, 0, 0),
            requirements: {
                let mut requirements: Vec<GuaranteeRequirement> = [
                    "DurableWorkState",
                    "SourceOfTruthBinding",
                    "CapabilityQualification",
                    "AuthorityBeforeSideEffect",
                    "ReceiptAfterExternalAction",
                    "ReconciliationOnUnknown",
                    "OutcomeObservation",
                    "AcceptedOutcomeSemantics",
                    "IndependentAcceptance",
                    "ProfileVersionPinned",
                    "Provenance",
                ]
                .into_iter()
                .map(GuaranteeRequirement::required)
                .collect();
                requirements.push(GuaranteeRequirement::forbidden("ProductionWrite"));
                requirements
            },
            minimum_isolation: "container".to_string(),
            required_execution_guarantees: vec![
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
            ],
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
    fn profile_validation_rejects_conflicting_or_inconsistent_contract() {
        let mut profile = DomainProfile::factory_readonly_v1();
        assert!(profile.validate().is_ok());

        profile.requirements.push(GuaranteeRequirement::optional("Provenance"));
        assert!(profile.validate().is_err());

        let mut inconsistent = DomainProfile::enterprise_v1();
        inconsistent.provenance_required = false;
        assert!(inconsistent.validate().is_err());
    }

    #[test]
    fn profile_family_is_machine_readable() {
        let profiles = [
            DomainProfile::lite_v1(),
            DomainProfile::enterprise_v1(),
            DomainProfile::factory_readonly_v1(),
            DomainProfile::research_v1(),
        ];
        assert_eq!(profiles.len(), 4);
        assert!(profiles
            .iter()
            .all(|profile| profile.version == Version::new(1, 0, 0)));
        assert!(DomainProfile::factory_readonly_v1().forbids("ProductionWrite"));
        assert!(profiles.iter().all(|profile| profile.validate().is_ok()));
    }

    #[test]
    fn pre_execution_conditions_do_not_require_post_execution_receipts() {
        let profile = DomainProfile::factory_readonly_v1();
        let conditions = profile.pre_execution_work_conditions();
        assert!(conditions.contains(&"CapabilityResolved".to_string()));
        assert!(conditions.contains(&"CapabilityQualified".to_string()));
        assert!(conditions.contains(&"SourceOfTruthBound".to_string()));
        assert!(!conditions.contains(&"ReceiptAfterExternalAction".to_string()));
        assert!(!conditions.contains(&"IndependentAcceptance".to_string()));
        assert_eq!(
            DomainProfile::from_ref(&profile.canonical_ref()),
            Some(profile)
        );
    }

    #[test]
    fn factory_profile_requires_network_and_secret_guarantees() {
        let profile = DomainProfile::factory_readonly_v1();
        assert!(profile
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::NetworkEgressPolicy));
        assert!(profile
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::SecretIndirection));
    }

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
