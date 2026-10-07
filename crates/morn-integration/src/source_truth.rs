//! Explicit binding to an authoritative external source for a class of facts.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SourceOfTruthBindingTag;
pub type SourceOfTruthBindingId = Id<SourceOfTruthBindingTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TruthAuthorityKind {
    SystemOfRecord,
    Sensor,
    HumanAuthority,
    ValidatedComputation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConflictPolicy {
    ExternalWins,
    HumanReview,
    ReconcileBeforeUse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceOfTruthBinding {
    pub id: SourceOfTruthBindingId,
    pub site_ref: Option<String>,
    pub source_ref: String,
    pub authority_kind: TruthAuthorityKind,
    pub authoritative_fact_types: Vec<String>,
    pub key_mapping_ref: String,
    pub query_capability_ref: String,
    pub freshness_sla_ms: Option<u64>,
    pub conflict_policy: ConflictPolicy,
    pub version_ref: String,
    pub created_at: Timestamp,
}

impl SourceOfTruthBinding {
    pub fn validate(&self) -> Result<()> {
        if self.source_ref.trim().is_empty()
            || self.authoritative_fact_types.is_empty()
            || self.key_mapping_ref.trim().is_empty()
            || self.query_capability_ref.trim().is_empty()
            || self.version_ref.trim().is_empty()
        {
            return Err(Error::validation(
                "source-of-truth binding requires source, fact types, key mapping, query capability and version",
            ));
        }
        Ok(())
    }

    pub fn authoritative_for(&self, fact_type: &str) -> bool {
        self.authoritative_fact_types
            .iter()
            .any(|item| item == fact_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_of_truth_is_explicit_and_fact_scoped() {
        let binding = SourceOfTruthBinding {
            id: SourceOfTruthBindingId::generate_with("sot"),
            site_ref: Some("plant-a".to_string()),
            source_ref: "cmms://plant-a".to_string(),
            authority_kind: TruthAuthorityKind::SystemOfRecord,
            authoritative_fact_types: vec!["maintenance.order".to_string()],
            key_mapping_ref: "mapping://cmms-order-key@1".to_string(),
            query_capability_ref: "capability://cmms.read-order@1".to_string(),
            freshness_sla_ms: Some(30_000),
            conflict_policy: ConflictPolicy::ReconcileBeforeUse,
            version_ref: "binding:v1".to_string(),
            created_at: Timestamp::now(),
        };
        binding.validate().unwrap();
        assert!(binding.authoritative_for("maintenance.order"));
        assert!(!binding.authoritative_for("machine.temperature"));
    }
}
