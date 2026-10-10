//! Source-grounded observed outcomes.
//!
//! An outcome is an observation about the world, not a harness self-report.
//! The source kind and source reference remain explicit so acceptance can be
//! evaluated independently.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{OutcomeRecordId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum OutcomeSourceKind {
    ExternalSystem,
    Sensor,
    HumanObservation,
    ValidatedComputation,
}

fn default_work_generation() -> u64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedOutcome {
    pub id: OutcomeRecordId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    /// Desired Work generation whose world state this observation witnesses.
    /// Legacy v11.5 JSON defaults to generation 1 and can never advance a later generation.
    #[serde(default = "default_work_generation")]
    pub work_generation: u64,
    pub objective: String,
    pub source_kind: OutcomeSourceKind,
    pub source_ref: String,
    /// Durable identity of the authoritative SourceOfTruthBinding used to
    /// interpret this observation. Legacy/manual observations may omit it,
    /// but deployment-attested observations should pin it.
    #[serde(default)]
    pub source_binding_ref: Option<String>,
    /// One-shot deployment attestation that supplied the world observation.
    /// This is provenance metadata, not an authorization token.
    #[serde(default)]
    pub source_attestation_ref: Option<String>,
    /// Authoritative fact type evaluated by the source binding.
    #[serde(default)]
    pub fact_type: Option<String>,
    pub observed_facts: Value,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub recorded_at: Timestamp,
}

impl ObservedOutcome {
    pub fn new(
        workspace_id: WorkspaceId,
        work_package_id: WorkPackageId,
        objective: impl Into<String>,
        source_kind: OutcomeSourceKind,
        source_ref: impl Into<String>,
        observed_facts: Value,
    ) -> Self {
        Self {
            id: OutcomeRecordId::generate_with("out"),
            workspace_id,
            work_package_id,
            work_generation: 1,
            objective: objective.into(),
            source_kind,
            source_ref: source_ref.into(),
            source_binding_ref: None,
            source_attestation_ref: None,
            fact_type: None,
            observed_facts,
            evidence_refs: Vec::new(),
            observed_at: Timestamp::now(),
            recorded_at: Timestamp::now(),
        }
    }

    pub fn pin_work_generation(&mut self, generation: u64) -> morn_kernel::error::Result<()> {
        if generation == 0 {
            return Err(morn_kernel::error::Error::validation(
                "observed outcome Work generation must be positive",
            ));
        }
        self.work_generation = generation;
        Ok(())
    }

    pub fn pin_source_provenance(
        &mut self,
        source_binding_ref: impl Into<String>,
        source_attestation_ref: impl Into<String>,
        fact_type: impl Into<String>,
    ) -> morn_kernel::error::Result<()> {
        let source_binding_ref = source_binding_ref.into();
        let source_attestation_ref = source_attestation_ref.into();
        let fact_type = fact_type.into();
        if source_binding_ref.trim().is_empty()
            || source_attestation_ref.trim().is_empty()
            || fact_type.trim().is_empty()
        {
            return Err(morn_kernel::error::Error::validation(
                "observed outcome source provenance requires binding, attestation and fact type",
            ));
        }
        self.source_binding_ref = Some(source_binding_ref);
        self.source_attestation_ref = Some(source_attestation_ref);
        self.fact_type = Some(fact_type);
        Ok(())
    }

    pub fn has_source_provenance(&self) -> bool {
        self.source_binding_ref
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            && self
                .source_attestation_ref
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                .fact_type
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }

    pub fn is_source_grounded(&self) -> bool {
        self.work_generation > 0
            && !self.source_ref.trim().is_empty()
            && !self.evidence_refs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn outcome_generation_is_explicit_and_positive() {
        let mut outcome = ObservedOutcome::new(
            WorkspaceId::generate(),
            WorkPackageId::generate_with("work"),
            "result",
            OutcomeSourceKind::ExternalSystem,
            "system://result",
            json!({"status":"complete"}),
        );
        assert_eq!(outcome.work_generation, 1);
        outcome.pin_work_generation(2).unwrap();
        assert_eq!(outcome.work_generation, 2);
        assert!(outcome.pin_work_generation(0).is_err());
    }

    #[test]
    fn deployment_source_provenance_is_explicit_and_complete() {
        let mut outcome = ObservedOutcome::new(
            WorkspaceId::generate(),
            WorkPackageId::generate_with("work"),
            "result",
            OutcomeSourceKind::ExternalSystem,
            "system://orders/42",
            json!({"status":"complete"}),
        );
        assert!(!outcome.has_source_provenance());
        outcome
            .pin_source_provenance(
                "source-binding://orders",
                "observation-attestation://42",
                "delivery.status",
            )
            .unwrap();
        assert!(outcome.has_source_provenance());
        assert_eq!(outcome.fact_type.as_deref(), Some("delivery.status"));
        assert!(outcome
            .pin_source_provenance("", "attestation", "delivery.status")
            .is_err());
    }

    #[test]
    fn harness_statement_alone_is_not_a_grounded_outcome() {
        let mut outcome = ObservedOutcome::new(
            WorkspaceId::generate(),
            WorkPackageId::generate_with("work"),
            "machine restored",
            OutcomeSourceKind::ExternalSystem,
            "cmms://orders/MO-88273",
            json!({"status":"complete"}),
        );
        assert!(!outcome.is_source_grounded());
        outcome
            .evidence_refs
            .push("cmms://orders/MO-88273/receipt".to_string());
        assert!(outcome.is_source_grounded());
    }
}
