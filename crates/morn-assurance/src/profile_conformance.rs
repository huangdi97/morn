//! Durable profile-conformance attestation.
//!
//! A ConformanceReport is a computed result; a SiteAdmission must be able to
//! point at who evaluated it and what concrete evidence was used. This wrapper
//! keeps raw caller booleans from becoming a production/site-admission claim.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_profile::{evaluate_profile, ConformanceEvidence, ConformanceReport, DomainProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ProfileConformanceAttestationTag;
pub type ProfileConformanceAttestationId = Id<ProfileConformanceAttestationTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileConformanceAttestation {
    pub id: ProfileConformanceAttestationId,
    pub site_ref: String,
    pub profile_ref: String,
    pub report: ConformanceReport,
    pub evidence_refs: Vec<String>,
    pub evaluator_identity: String,
    pub evaluated_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl ProfileConformanceAttestation {
    pub fn evaluate(
        profile: &DomainProfile,
        site_ref: impl Into<String>,
        evidence: &ConformanceEvidence,
        evidence_refs: Vec<String>,
        evaluator_identity: impl Into<String>,
        valid_until: Option<Timestamp>,
    ) -> Result<Self> {
        let site_ref = site_ref.into();
        let evaluator_identity = evaluator_identity.into();
        if site_ref.trim().is_empty() || evaluator_identity.trim().is_empty() {
            return Err(Error::validation(
                "profile conformance attestation requires site and evaluator identity",
            ));
        }
        if evidence_refs.is_empty() {
            return Err(Error::validation(
                "profile conformance attestation requires concrete evidence references",
            ));
        }
        if valid_until.is_some_and(|end| end < Timestamp::now()) {
            return Err(Error::validation(
                "profile conformance attestation validity has already expired",
            ));
        }

        Ok(Self {
            id: ProfileConformanceAttestationId::generate_with("profile-conformance"),
            site_ref,
            profile_ref: profile.canonical_ref(),
            report: evaluate_profile(profile, evidence),
            evidence_refs,
            evaluator_identity,
            evaluated_at: Timestamp::now(),
            valid_until,
        })
    }

    pub fn passing_for(&self, site_ref: &str, profile_ref: &str, now: Timestamp) -> bool {
        self.report.passed
            && self.site_ref == site_ref
            && self.profile_ref == profile_ref
            && self.report.profile_ref == profile_ref
            && self.valid_until.is_none_or(|end| now <= end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ExecutionGuarantee;
    use morn_profile::RequirementLevel;
    use std::collections::BTreeSet;

    fn passing_evidence(profile: &DomainProfile) -> ConformanceEvidence {
        ConformanceEvidence {
            satisfied_semantics: profile
                .requirements
                .iter()
                .filter(|item| item.level == RequirementLevel::Required)
                .map(|item| item.semantic.clone())
                .collect::<BTreeSet<_>>(),
            isolation: "container".to_string(),
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .collect(),
            durable_work_state: true,
            source_of_truth_bound: true,
            provenance_ready: true,
            ..Default::default()
        }
    }

    #[test]
    fn passing_report_without_evidence_refs_cannot_be_attested() {
        let profile = DomainProfile::factory_readonly_v1();
        assert!(ProfileConformanceAttestation::evaluate(
            &profile,
            "plant-a",
            &passing_evidence(&profile),
            vec![],
            "evaluator",
            None,
        )
        .is_err());
    }

    #[test]
    fn attestation_is_site_profile_and_time_scoped() {
        let profile = DomainProfile::factory_readonly_v1();
        let now = Timestamp::now();
        let attestation = ProfileConformanceAttestation::evaluate(
            &profile,
            "plant-a",
            &passing_evidence(&profile),
            vec![
                "runtime-attestation://plant-a".to_string(),
                "source-binding://cmms".to_string(),
            ],
            "conformance-suite",
            Some(Timestamp::from_millis(now.millis() + 60_000)),
        )
        .unwrap();
        assert!(attestation.passing_for(
            "plant-a",
            &profile.canonical_ref(),
            Timestamp::from_millis(now.millis() + 1),
        ));
        assert!(!attestation.passing_for(
            "plant-b",
            &profile.canonical_ref(),
            Timestamp::from_millis(now.millis() + 1),
        ));
    }

    #[test]
    fn execution_guarantees_are_real_evidence_inputs_not_profile_names() {
        let profile = DomainProfile::enterprise_v1();
        let mut evidence = passing_evidence(&profile);
        evidence
            .execution_guarantees
            .remove(&ExecutionGuarantee::SecretIndirection);
        let attestation = ProfileConformanceAttestation::evaluate(
            &profile,
            "site-a",
            &evidence,
            vec!["evaluation://site-a".to_string()],
            "conformance-suite",
            None,
        )
        .unwrap();
        assert!(!attestation.report.passed);
    }
}
