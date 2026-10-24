//! Deployment-attested customer value evidence.
//!
//! A RealSite evidence class proves that a Work/Outcome was observed at a real
//! site. It does not prove arbitrary KPI numbers submitted later by a browser.
//! CustomerValueAttestation binds the exact accepted Work generation, Outcome,
//! Acceptance decision, baseline and KPI values to deployment-owned evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    AcceptanceDecisionId, OutcomeRecordId, ValueAssessmentId, WorkPackageId, WorkspaceId,
};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomerValueMetric {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumedCustomerValueAttestation {
    pub attestation_id: String,
    pub workspace_id: WorkspaceId,
    pub work_id: WorkPackageId,
    pub work_generation: u64,
    pub outcome_id: OutcomeRecordId,
    pub acceptance_id: AcceptanceDecisionId,
    pub assessment_id: ValueAssessmentId,
    pub issuer: String,
    pub consumed_at: Timestamp,
}

impl ConsumedCustomerValueAttestation {
    pub fn from_attestation(
        attestation: &CustomerValueAttestation,
        assessment_id: ValueAssessmentId,
        consumed_at: Timestamp,
    ) -> Result<Self> {
        if !attestation.active_for(
            &attestation.workspace_id,
            (&attestation.work_package_id, attestation.work_generation),
            &attestation.outcome_id,
            &attestation.acceptance_id,
            consumed_at,
        ) {
            return Err(Error::not_authorized(
                "customer value attestation is not active at consumption time",
            ));
        }
        Ok(Self {
            attestation_id: attestation.attestation_id.clone(),
            workspace_id: attestation.workspace_id.clone(),
            work_id: attestation.work_package_id.clone(),
            work_generation: attestation.work_generation,
            outcome_id: attestation.outcome_id.clone(),
            acceptance_id: attestation.acceptance_id.clone(),
            assessment_id,
            issuer: attestation.issuer.clone(),
            consumed_at,
        })
    }

    pub fn exactly_matches(
        &self,
        attestation: &CustomerValueAttestation,
        assessment: &morn_work::value::ValueAssessment,
    ) -> bool {
        self.attestation_id == attestation.attestation_id
            && self.workspace_id == attestation.workspace_id
            && self.work_id == assessment.work_package_id
            && self.work_generation == assessment.work_generation
            && self.outcome_id == assessment.outcome_ref
            && assessment.acceptance_ref.as_ref() == Some(&self.acceptance_id)
            && self.acceptance_id == attestation.acceptance_id
            && self.assessment_id == assessment.id
            && self.issuer == attestation.issuer
            && self.consumed_at == assessment.assessed_at
            && assessment.customer_value_attestation_ref.as_deref()
                == Some(attestation.attestation_id.as_str())
            && attestation.active_for(
                &self.workspace_id,
                (&self.work_id, self.work_generation),
                &self.outcome_id,
                &self.acceptance_id,
                self.consumed_at,
            )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomerValueAttestation {
    pub attestation_id: String,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub work_generation: u64,
    pub outcome_id: OutcomeRecordId,
    pub acceptance_id: AcceptanceDecisionId,
    pub baseline_ref: Option<String>,
    pub kpis: Vec<CustomerValueMetric>,
    pub evidence_refs: Vec<String>,
    pub issuer: String,
    pub observed_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl CustomerValueAttestation {
    pub fn validate(&self) -> Result<()> {
        if self.attestation_id.trim().is_empty()
            || self.issuer.trim().is_empty()
            || self.work_generation == 0
        {
            return Err(Error::validation(
                "customer value attestation requires id, issuer and positive Work generation",
            ));
        }
        if self.evidence_refs.is_empty()
            || self
                .evidence_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "customer value attestation requires non-empty deployment evidence references",
            ));
        }
        if self
            .baseline_ref
            .as_deref()
            .is_some_and(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "customer value attestation baseline reference must be non-empty when present",
            ));
        }
        if self.kpis.is_empty() {
            return Err(Error::validation(
                "customer value attestation requires at least one attested KPI",
            ));
        }
        let mut names = BTreeSet::new();
        for metric in &self.kpis {
            if metric.name.trim().is_empty() || !metric.value.is_finite() {
                return Err(Error::validation(
                    "customer value KPI names must be non-empty and values finite",
                ));
            }
            if !names.insert(metric.name.as_str()) {
                return Err(Error::validation(
                    "customer value KPI names must be unique within one attestation",
                ));
            }
        }
        if self
            .valid_until
            .is_some_and(|until| until < self.observed_at)
        {
            return Err(Error::validation(
                "customer value attestation validity cannot end before observation",
            ));
        }
        Ok(())
    }

    pub fn active_for(
        &self,
        workspace_id: &WorkspaceId,
        work_identity: (&WorkPackageId, u64),
        outcome_id: &OutcomeRecordId,
        acceptance_id: &AcceptanceDecisionId,
        now: Timestamp,
    ) -> bool {
        self.validate().is_ok()
            && &self.workspace_id == workspace_id
            && &self.work_package_id == work_identity.0
            && self.work_generation == work_identity.1
            && &self.outcome_id == outcome_id
            && &self.acceptance_id == acceptance_id
            && self.observed_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> CustomerValueAttestation {
        CustomerValueAttestation {
            attestation_id: "customer-value-1".to_string(),
            workspace_id: WorkspaceId::generate(),
            work_package_id: WorkPackageId::generate_with("work"),
            work_generation: 2,
            outcome_id: OutcomeRecordId::generate_with("outcome"),
            acceptance_id: AcceptanceDecisionId::generate_with("acceptance"),
            baseline_ref: Some("baseline://customer-approved".to_string()),
            kpis: vec![CustomerValueMetric {
                name: "human_minutes_saved".to_string(),
                value: 12.5,
            }],
            evidence_refs: vec!["customer://signed/value-review-1".to_string()],
            issuer: "customer-value-reviewer".to_string(),
            observed_at: Timestamp::from_millis(10),
            valid_until: Some(Timestamp::from_millis(20)),
        }
    }

    #[test]
    fn customer_value_attestation_is_exact_and_time_bounded() {
        let attestation = fixture();
        attestation.validate().unwrap();
        assert!(attestation.active_for(
            &attestation.workspace_id,
            (&attestation.work_package_id, 2),
            &attestation.outcome_id,
            &attestation.acceptance_id,
            Timestamp::from_millis(15),
        ));
        assert!(!attestation.active_for(
            &attestation.workspace_id,
            (&attestation.work_package_id, 1),
            &attestation.outcome_id,
            &attestation.acceptance_id,
            Timestamp::from_millis(15),
        ));
        assert!(!attestation.active_for(
            &attestation.workspace_id,
            (&attestation.work_package_id, 2),
            &attestation.outcome_id,
            &attestation.acceptance_id,
            Timestamp::from_millis(21),
        ));
    }

    #[test]
    fn consumed_customer_value_attestation_binds_exact_assessment_and_time() {
        let attestation = fixture();
        let mut assessment = morn_work::value::ValueAssessment::new(
            attestation.work_package_id.clone(),
            attestation.outcome_id.clone(),
            morn_work::value::ValueEvidenceClass::CustomerValidated,
        );
        assessment
            .pin_work_generation(attestation.work_generation)
            .unwrap();
        assessment.acceptance_ref = Some(attestation.acceptance_id.clone());
        assessment.customer_value_attestation_ref = Some(attestation.attestation_id.clone());
        assessment.assessed_at = Timestamp::from_millis(15);

        let consumed = ConsumedCustomerValueAttestation::from_attestation(
            &attestation,
            assessment.id.clone(),
            assessment.assessed_at,
        )
        .unwrap();
        assert!(consumed.exactly_matches(&attestation, &assessment));

        let mut other = assessment.clone();
        other.id = ValueAssessmentId::generate_with("value");
        assert!(!consumed.exactly_matches(&attestation, &other));

        let mut shifted = consumed.clone();
        shifted.consumed_at = Timestamp::from_millis(14);
        assert!(!shifted.exactly_matches(&attestation, &assessment));
    }

    #[test]
    fn customer_value_attestation_rejects_ambiguous_or_unmeasured_claims() {
        let mut attestation = fixture();
        attestation.kpis.clear();
        assert!(attestation.validate().is_err());

        let mut attestation = fixture();
        attestation.kpis.push(CustomerValueMetric {
            name: "human_minutes_saved".to_string(),
            value: 99.0,
        });
        assert!(attestation.validate().is_err());

        let mut attestation = fixture();
        attestation.kpis[0].value = f64::NAN;
        assert!(attestation.validate().is_err());
    }
}
