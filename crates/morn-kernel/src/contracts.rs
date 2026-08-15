//! Stable Semantic Kernel v1: a frozen, versioned registry of the canonical
//! contracts Morn Core owns. Providers/domains cannot redefine them.

use serde::{Deserialize, Serialize};

use crate::version::Version;

pub const CONTRACT_NAMES: [&str; 22] = [
    "Identity",
    "Workspace",
    "OperationalObject",
    "Event",
    "Action",
    "StateDiff",
    "Artifact",
    "Decision",
    "Outcome",
    "WorkPackage",
    "WorkContract",
    "AcceptanceSpec",
    "OutcomeContract",
    "Actor",
    "Role",
    "Capability",
    "Policy",
    "Permission",
    "Approval",
    "EffectClass",
    "Provenance",
    "LifecycleVersion",
];

pub const SEMANTIC_CONTRACT_V1: Version = Version::new(1, 0, 0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVersion {
    pub name: String,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractSnapshot {
    pub kernel_version: Version,
    pub contracts: Vec<ContractVersion>,
}

impl ContractSnapshot {
    pub fn v1() -> Self {
        Self {
            kernel_version: SEMANTIC_CONTRACT_V1,
            contracts: CONTRACT_NAMES
                .iter()
                .map(|n| ContractVersion {
                    name: n.to_string(),
                    version: SEMANTIC_CONTRACT_V1,
                })
                .collect(),
        }
    }

    pub fn contract(&self, name: &str) -> Option<Version> {
        self.contracts
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.version)
    }

    pub fn is_complete(&self) -> bool {
        self.contracts.len() == CONTRACT_NAMES.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compatibility {
    Compatible,
    RequiresReevaluation,
    Incompatible,
}

pub struct ContractCompatibility;

impl ContractCompatibility {
    pub fn check(base: &ContractSnapshot, candidate: &ContractSnapshot) -> Compatibility {
        if base.kernel_version.major != candidate.kernel_version.major {
            return Compatibility::Incompatible;
        }
        if base.kernel_version.minor != candidate.kernel_version.minor {
            return Compatibility::RequiresReevaluation;
        }
        Compatibility::Compatible
    }

    pub fn provider_claim_allowed(
        snapshot: &ContractSnapshot,
        name: &str,
        claimed: Version,
    ) -> bool {
        match snapshot.contract(name) {
            Some(canonical) => claimed.major == canonical.major && claimed.minor == canonical.minor,
            None => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeprecationPolicy {
    pub contract: String,
    pub deprecated_in: Version,
    pub removed_in: Option<Version>,
    pub replacement: Option<String>,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_snapshot_is_complete_and_stable() {
        let snapshot = ContractSnapshot::v1();
        assert!(snapshot.is_complete());
        assert_eq!(snapshot.kernel_version, SEMANTIC_CONTRACT_V1);
        assert_eq!(snapshot.contract("Identity"), Some(Version::new(1, 0, 0)));
        assert_eq!(snapshot.contract("Artifact"), Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn compatibility_matrix() {
        let v1 = ContractSnapshot::v1();
        let mut patch = v1.clone();
        patch.kernel_version = Version::new(1, 0, 1);
        let mut minor = v1.clone();
        minor.kernel_version = Version::new(1, 1, 0);
        let mut major = v1.clone();
        major.kernel_version = Version::new(2, 0, 0);
        assert_eq!(
            ContractCompatibility::check(&v1, &patch),
            Compatibility::Compatible
        );
        assert_eq!(
            ContractCompatibility::check(&v1, &minor),
            Compatibility::RequiresReevaluation
        );
        assert_eq!(
            ContractCompatibility::check(&v1, &major),
            Compatibility::Incompatible
        );
    }

    #[test]
    fn providers_cannot_redefine_canonical_contracts() {
        let snapshot = ContractSnapshot::v1();
        assert!(!ContractCompatibility::provider_claim_allowed(
            &snapshot,
            "Identity",
            Version::new(2, 0, 0)
        ));
        assert!(ContractCompatibility::provider_claim_allowed(
            &snapshot,
            "Identity",
            Version::new(1, 0, 2)
        ));
    }
}
