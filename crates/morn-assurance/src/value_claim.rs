//! Business-value claim validation.
//!
//! ValueEvidenceClass (fixture/simulation/shadow/operational/customer-validated)
//! and assurance EvidenceClass (design/fixture/CI/runtime/site/write) are
//! orthogonal axes. A label on ValueAssessment cannot manufacture missing
//! real-site evidence.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_work::value::{ValueAssessment, ValueEvidenceClass};

use crate::{EvidenceClass, EvidenceLedger};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomerValueClaimValidation {
    pub subject: String,
    pub valid: bool,
    pub reasons: Vec<String>,
}

pub fn validate_customer_value_claim(
    assessment: &ValueAssessment,
    ledger: &EvidenceLedger,
    claim_subject: impl Into<String>,
) -> Result<CustomerValueClaimValidation> {
    let subject = claim_subject.into();
    if subject.trim().is_empty() {
        return Err(Error::validation("customer-value claim subject is required"));
    }

    let mut reasons = Vec::new();
    if assessment.evidence_class != ValueEvidenceClass::CustomerValidated {
        reasons.push("ValueAssessment is not CustomerValidated".to_string());
    }
    if assessment.acceptance_ref.is_none() {
        reasons.push("customer-value claim requires independent AcceptanceDecision".to_string());
    }
    if assessment.evidence_refs.is_empty() {
        reasons.push("customer-value claim requires explicit value evidence references".to_string());
    }
    if !ledger.satisfies(&subject, EvidenceClass::RealSite) {
        reasons.push("customer-value claim requires explicit RealSite evidence class".to_string());
    }

    Ok(CustomerValueClaimValidation {
        subject,
        valid: reasons.is_empty(),
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{AcceptanceDecisionId, OutcomeRecordId, WorkPackageId};

    fn assessment() -> ValueAssessment {
        let mut value = ValueAssessment::new(
            WorkPackageId::generate_with("work"),
            OutcomeRecordId::generate_with("out"),
            ValueEvidenceClass::CustomerValidated,
        );
        value.acceptance_ref = Some(AcceptanceDecisionId::generate_with("accept"));
        value.evidence_refs = vec!["customer://accepted-kpi-report".to_string()];
        value
    }

    #[test]
    fn customer_validated_label_cannot_replace_real_site_evidence() {
        let ledger = EvidenceLedger::default();
        let result = validate_customer_value_claim(&assessment(), &ledger, "value:work-1").unwrap();
        assert!(!result.valid);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("RealSite")));
    }

    #[test]
    fn real_site_evidence_plus_acceptance_can_validate_customer_value_claim() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                crate::EvidenceClaim::proven(
                    "value:work-1",
                    EvidenceClass::RealSite,
                    vec!["customer://site-pilot".to_string()],
                    "customer-owner",
                    "customer site pilot with accepted KPI evidence",
                )
                .unwrap(),
            )
            .unwrap();

        let result = validate_customer_value_claim(&assessment(), &ledger, "value:work-1").unwrap();
        assert!(result.valid, "{:?}", result.reasons);
    }

    #[test]
    fn production_write_is_not_required_for_read_only_customer_value() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                crate::EvidenceClaim::proven(
                    "value:work-1",
                    EvidenceClass::RealSite,
                    vec!["customer://read-only-pilot".to_string()],
                    "customer-owner",
                    "read-only pilot measured accepted outcome",
                )
                .unwrap(),
            )
            .unwrap();

        let result = validate_customer_value_claim(&assessment(), &ledger, "value:work-1").unwrap();
        assert!(result.valid);
        assert!(!ledger.satisfies("value:work-1", EvidenceClass::ProductionWrite));
    }
}
