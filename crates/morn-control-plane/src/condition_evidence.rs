//! Durable evidence for Work readiness conditions.
//!
//! Controller inputs are not business truth by themselves. This module gives
//! strict profiles a durable, generation-scoped witness model so a caller
//! cannot advance Work merely by posting booleans to a control-plane endpoint.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_work::control::WorkResource;

use crate::ControllerInputs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ConditionEvidenceTag;
pub type ConditionEvidenceId = Id<ConditionEvidenceTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionEvidence {
    pub id: ConditionEvidenceId,
    pub work_ref: String,
    pub work_generation: u64,
    pub condition_type: String,
    pub satisfied: bool,
    pub producer_ref: String,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl ConditionEvidence {
    pub fn new(
        work: &WorkResource,
        condition_type: impl Into<String>,
        satisfied: bool,
        producer_ref: impl Into<String>,
        evidence_refs: Vec<String>,
    ) -> Result<Self> {
        let condition_type = condition_type.into();
        let producer_ref = producer_ref.into();
        if condition_type.trim().is_empty() || producer_ref.trim().is_empty() {
            return Err(Error::validation(
                "condition evidence requires condition type and producer",
            ));
        }
        if satisfied && evidence_refs.is_empty() {
            return Err(Error::validation(
                "a satisfied Work condition requires at least one evidence reference",
            ));
        }
        Ok(Self {
            id: ConditionEvidenceId::generate_with("condition-evidence"),
            work_ref: work.id.to_string(),
            work_generation: work.generation,
            condition_type,
            satisfied,
            producer_ref,
            evidence_refs,
            observed_at: Timestamp::now(),
            valid_until: None,
        })
    }

    pub fn active_for(&self, work: &WorkResource, now: Timestamp) -> bool {
        self.work_ref == work.id.to_string()
            && self.work_generation == work.generation
            && self.observed_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
}

fn condition_value(
    work: &WorkResource,
    evidence: &[ConditionEvidence],
    condition_type: &str,
    now: Timestamp,
) -> bool {
    evidence
        .iter()
        .filter(|item| item.condition_type == condition_type && item.active_for(work, now))
        // An equally recent denial (or unsubstantiated positive) wins ties.
        // SQLite row order must never decide whether a Work becomes Ready.
        .max_by_key(|item| {
            (
                item.observed_at.millis(),
                !(item.satisfied && !item.evidence_refs.is_empty()),
            )
        })
        .is_some_and(|item| item.satisfied && !item.evidence_refs.is_empty())
}

pub fn derive_controller_inputs(
    work: &WorkResource,
    evidence: &[ConditionEvidence],
    now: Timestamp,
) -> ControllerInputs {
    ControllerInputs {
        capability_resolved: condition_value(work, evidence, "CapabilityResolved", now),
        capability_qualified: condition_value(work, evidence, "CapabilityQualified", now),
        authority_satisfied: condition_value(work, evidence, "AuthoritySatisfied", now),
        source_of_truth_bound: condition_value(work, evidence, "SourceOfTruthBound", now),
        provenance_ready: condition_value(work, evidence, "ProvenanceReady", now),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::WorkSpec;

    #[test]
    fn stale_generation_evidence_cannot_advance_new_work_generation() {
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.factory.readonly@1.0.0",
            ),
        );
        let evidence = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            true,
            "resolver://fixture",
            vec!["capability://candidate".to_string()],
        )
        .unwrap();
        let mut next = work.spec.clone();
        next.goal = "review changed scope".to_string();
        work.replace_spec(next);

        let inputs = derive_controller_inputs(&work, &[evidence], Timestamp::now());
        assert!(!inputs.capability_resolved);
    }

    #[test]
    fn latest_active_evidence_controls_condition_value() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        let mut positive = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            true,
            "resolver://a",
            vec!["capability://a".to_string()],
        )
        .unwrap();
        positive.observed_at = Timestamp::from_millis(10);
        let mut revoked =
            ConditionEvidence::new(&work, "CapabilityResolved", false, "resolver://a", vec![])
                .unwrap();
        revoked.observed_at = Timestamp::from_millis(20);

        let inputs =
            derive_controller_inputs(&work, &[positive, revoked], Timestamp::from_millis(30));
        assert!(!inputs.capability_resolved);
    }

    #[test]
    fn future_dated_evidence_cannot_make_work_ready_early() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        let mut positive = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            true,
            "resolver://fixture",
            vec!["capability://fixture".to_string()],
        )
        .unwrap();
        positive.observed_at = Timestamp::from_millis(200);
        let input_before =
            derive_controller_inputs(&work, &[positive.clone()], Timestamp::from_millis(199));
        assert!(!input_before.capability_resolved);

        let input_at_observation =
            derive_controller_inputs(&work, &[positive], Timestamp::from_millis(200));
        assert!(input_at_observation.capability_resolved);
    }

    #[test]
    fn equal_timestamp_denial_overrides_positive_regardless_of_row_order() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        let mut positive = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            true,
            "resolver://fixture",
            vec!["capability://fixture".to_string()],
        )
        .unwrap();
        positive.observed_at = Timestamp::from_millis(100);
        let mut denial = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            false,
            "resolver://fixture",
            vec![],
        )
        .unwrap();
        denial.observed_at = Timestamp::from_millis(100);
        let now = Timestamp::from_millis(101);
        assert!(
            !derive_controller_inputs(&work, &[positive.clone(), denial.clone()], now)
                .capability_resolved
        );
        assert!(!derive_controller_inputs(&work, &[denial, positive], now).capability_resolved);
    }

    #[test]
    fn satisfied_condition_without_evidence_is_rejected() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        assert!(
            ConditionEvidence::new(&work, "CapabilityResolved", true, "resolver://a", vec![],)
                .is_err()
        );
    }
}
