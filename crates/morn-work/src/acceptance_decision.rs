//! Independent acceptance decisions.
//!
//! Acceptance consumes observed outcomes/evidence. It is not the same record
//! as execution success or an outcome observation.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    AcceptanceDecisionId, AcceptanceSpecId, OutcomeRecordId, PrincipalId, WorkPackageId,
};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AcceptanceDisposition {
    Accept,
    Reject,
    Conditional,
    RequestMoreEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceDecision {
    pub id: AcceptanceDecisionId,
    pub work_package_id: WorkPackageId,
    pub acceptance_spec_id: AcceptanceSpecId,
    pub disposition: AcceptanceDisposition,
    pub outcome_refs: Vec<OutcomeRecordId>,
    pub evidence_refs: Vec<String>,
    pub decided_by: PrincipalId,
    pub acting_role: String,
    pub reason: String,
    pub conditions: Vec<String>,
    pub decided_at: Timestamp,
}

impl AcceptanceDecision {
    pub fn new(
        work_package_id: WorkPackageId,
        acceptance_spec_id: AcceptanceSpecId,
        disposition: AcceptanceDisposition,
        decided_by: PrincipalId,
        acting_role: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: AcceptanceDecisionId::generate_with("adec"),
            work_package_id,
            acceptance_spec_id,
            disposition,
            outcome_refs: Vec::new(),
            evidence_refs: Vec::new(),
            decided_by,
            acting_role: acting_role.into(),
            reason: reason.into(),
            conditions: Vec::new(),
            decided_at: Timestamp::now(),
        }
    }

    pub fn is_final_acceptance(&self) -> bool {
        self.disposition == AcceptanceDisposition::Accept && !self.outcome_refs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceptance_requires_an_outcome_reference_to_be_final() {
        let mut decision = AcceptanceDecision::new(
            WorkPackageId::generate_with("work"),
            AcceptanceSpecId::generate_with("acc"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("principal"),
            "production-owner",
            "criteria satisfied",
        );
        assert!(!decision.is_final_acceptance());
        decision
            .outcome_refs
            .push(OutcomeRecordId::generate_with("out"));
        assert!(decision.is_final_acceptance());
    }
}
