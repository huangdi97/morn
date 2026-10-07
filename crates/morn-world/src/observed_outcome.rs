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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedOutcome {
    pub id: OutcomeRecordId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub objective: String,
    pub source_kind: OutcomeSourceKind,
    pub source_ref: String,
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
            objective: objective.into(),
            source_kind,
            source_ref: source_ref.into(),
            observed_facts,
            evidence_refs: Vec::new(),
            observed_at: Timestamp::now(),
            recorded_at: Timestamp::now(),
        }
    }

    pub fn is_source_grounded(&self) -> bool {
        !self.source_ref.trim().is_empty() && !self.evidence_refs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
