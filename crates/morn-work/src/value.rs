//! Value assessment after an observed/accepted outcome.
//!
//! Value metrics are evidence about the work episode. Simulated/reference
//! metrics stay marked as such and cannot be presented as customer business value.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{AcceptanceDecisionId, OutcomeRecordId, ValueAssessmentId, WorkPackageId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ValueEvidenceClass {
    Fixture,
    Simulation,
    Shadow,
    ObservedOperational,
    CustomerValidated,
}

fn default_work_generation() -> u64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueAssessment {
    pub id: ValueAssessmentId,
    pub work_package_id: WorkPackageId,
    /// Exact desired Work generation whose accepted outcome is being valued.
    /// Legacy serialized records default to generation 1 and cannot validate a later generation.
    #[serde(default = "default_work_generation")]
    pub work_generation: u64,
    pub outcome_ref: OutcomeRecordId,
    pub acceptance_ref: Option<AcceptanceDecisionId>,
    pub evidence_class: ValueEvidenceClass,
    pub total_duration_ms: Option<u64>,
    pub human_minutes: Option<f64>,
    pub model_runtime_cost: Option<f64>,
    pub retry_count: u32,
    pub unknown_outcome_count: u32,
    pub baseline_ref: Option<String>,
    pub kpis: Vec<(String, f64)>,
    pub evidence_refs: Vec<String>,
    pub assessed_at: Timestamp,
}

impl ValueAssessment {
    pub fn new(
        work_package_id: WorkPackageId,
        outcome_ref: OutcomeRecordId,
        evidence_class: ValueEvidenceClass,
    ) -> Self {
        Self {
            id: ValueAssessmentId::generate_with("value"),
            work_package_id,
            work_generation: 1,
            outcome_ref,
            acceptance_ref: None,
            evidence_class,
            total_duration_ms: None,
            human_minutes: None,
            model_runtime_cost: None,
            retry_count: 0,
            unknown_outcome_count: 0,
            baseline_ref: None,
            kpis: Vec::new(),
            evidence_refs: Vec::new(),
            assessed_at: Timestamp::now(),
        }
    }

    pub fn pin_work_generation(&mut self, generation: u64) -> morn_kernel::error::Result<()> {
        if generation == 0 {
            return Err(morn_kernel::error::Error::validation(
                "value assessment Work generation must be positive",
            ));
        }
        self.work_generation = generation;
        Ok(())
    }

    pub fn is_customer_value_claim(&self) -> bool {
        self.work_generation > 0
            && self.evidence_class == ValueEvidenceClass::CustomerValidated
            && self.acceptance_ref.is_some()
            && !self.evidence_refs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_assessment_generation_is_explicit_and_positive() {
        let mut assessment = ValueAssessment::new(
            WorkPackageId::generate_with("work"),
            OutcomeRecordId::generate_with("out"),
            ValueEvidenceClass::ObservedOperational,
        );
        assert_eq!(assessment.work_generation, 1);
        assessment.pin_work_generation(3).unwrap();
        assert_eq!(assessment.work_generation, 3);
        assert!(assessment.pin_work_generation(0).is_err());
    }

    #[test]
    fn fixture_metrics_are_not_customer_value_claims() {
        let mut assessment = ValueAssessment::new(
            WorkPackageId::generate_with("work"),
            OutcomeRecordId::generate_with("out"),
            ValueEvidenceClass::Fixture,
        );
        assessment
            .kpis
            .push(("delivery_risk_delta".to_string(), 0.2));
        assert!(!assessment.is_customer_value_claim());
    }
}
