//! Explicit binding to an authoritative external source for a class of facts.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{Id, WorkPackageId, WorkspaceId};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceObservationAttestation {
    pub attestation_id: String,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub source_binding_id: SourceOfTruthBindingId,
    pub fact_type: String,
    pub objective: String,
    pub source_ref: String,
    pub observed_facts: Value,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl SourceObservationAttestation {
    pub fn validate(&self) -> Result<()> {
        if self.attestation_id.trim().is_empty()
            || self.fact_type.trim().is_empty()
            || self.objective.trim().is_empty()
            || self.source_ref.trim().is_empty()
            || !self.observed_facts.is_object()
            || self.evidence_refs.is_empty()
        {
            return Err(Error::validation(
                "source observation attestation requires id, fact type, objective, source ref, object facts and evidence",
            ));
        }
        if self
            .valid_until
            .is_some_and(|until| until < self.observed_at)
        {
            return Err(Error::validation(
                "source observation attestation validity cannot end before observation",
            ));
        }
        Ok(())
    }

    pub fn active_for(
        &self,
        workspace_id: &WorkspaceId,
        work_package_id: &WorkPackageId,
        source_binding_id: &SourceOfTruthBindingId,
        fact_type: &str,
        now: Timestamp,
    ) -> bool {
        self.validate().is_ok()
            && &self.workspace_id == workspace_id
            && &self.work_package_id == work_package_id
            && &self.source_binding_id == source_binding_id
            && self.fact_type == fact_type
            && self.observed_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
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
    fn source_observation_attestation_is_exact_and_time_bounded() {
        let workspace = WorkspaceId::generate();
        let work = WorkPackageId::generate_with("work");
        let binding = SourceOfTruthBindingId::generate_with("sot");
        let attestation = SourceObservationAttestation {
            attestation_id: "obs-attest-1".to_string(),
            workspace_id: workspace.clone(),
            work_package_id: work.clone(),
            source_binding_id: binding.clone(),
            fact_type: "maintenance.order".to_string(),
            objective: "maintenance order completed".to_string(),
            source_ref: "cmms://plant-a/orders/1".to_string(),
            observed_facts: serde_json::json!({"status":"complete"}),
            evidence_refs: vec!["cmms://plant-a/orders/1/receipt".to_string()],
            observed_at: Timestamp::from_millis(10),
            valid_until: Some(Timestamp::from_millis(20)),
        };
        attestation.validate().unwrap();
        assert!(attestation.active_for(
            &workspace,
            &work,
            &binding,
            "maintenance.order",
            Timestamp::from_millis(15),
        ));
        assert!(!attestation.active_for(
            &workspace,
            &work,
            &binding,
            "machine.temperature",
            Timestamp::from_millis(15),
        ));
        assert!(!attestation.active_for(
            &workspace,
            &work,
            &binding,
            "maintenance.order",
            Timestamp::from_millis(21),
        ));
    }

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
