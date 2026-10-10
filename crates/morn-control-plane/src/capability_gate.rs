//! Strict runtime eligibility gate for governed capabilities.
//!
//! Site admission and provider liveness answer different questions:
//! - AdmissionService proves the released capability is currently qualified for
//!   this site/profile.
//! - ProviderGate proves the concrete runtime/provider is currently selectable.
//!
//! Governed Enterprise/Factory work must satisfy both before the semantic
//! CapabilityResolver is allowed to plan a new binding.

use serde::{Deserialize, Serialize};

use morn_assurance::AdmissionService;
use morn_capability::CapabilityRecord;
use morn_kernel::time::Timestamp;
use morn_runtime::ProviderRegistry;

use crate::{ProviderGate, ProviderGatePolicy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEligibilityBlock {
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEligibilityReport {
    pub selectable: Vec<CapabilityRecord>,
    pub blocked: Vec<CapabilityEligibilityBlock>,
}

impl CapabilityEligibilityReport {
    pub fn all_selectable(&self) -> bool {
        self.blocked.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct CapabilityEligibilityGate;

impl CapabilityEligibilityGate {
    pub fn strict(
        &self,
        candidates: &[CapabilityRecord],
        admissions: &AdmissionService,
        providers: &ProviderRegistry,
        site_ref: &str,
        profile_ref: &str,
        now: Timestamp,
    ) -> CapabilityEligibilityReport {
        let mut admission_passed = Vec::new();
        let mut blocked = Vec::new();

        for candidate in candidates {
            if admissions.site_profile_admission_active_at(candidate, site_ref, profile_ref, now) {
                admission_passed.push(candidate.clone());
            } else {
                blocked.push(CapabilityEligibilityBlock {
                    capability_manifest_ref: candidate.manifest.id.to_string(),
                    provider_ref: candidate.manifest.provider_ref.clone(),
                    reason: format!(
                        "no active qualification/release/site admission for {site_ref} under {profile_ref}"
                    ),
                });
            }
        }

        let provider_report = ProviderGate.evaluate(
            &admission_passed,
            providers,
            ProviderGatePolicy::strict(),
            now,
        );
        blocked.extend(provider_report.blocked.into_iter().map(|item| {
            CapabilityEligibilityBlock {
                capability_manifest_ref: item.capability_manifest_ref,
                provider_ref: item.provider_ref,
                reason: item.reason,
            }
        }));

        CapabilityEligibilityReport {
            selectable: provider_report.selectable,
            blocked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_assurance::{
        QualificationEvidence, StrictQualificationRequest, VerifiedReleaseRequest,
    };
    use morn_capability::{CapabilityKind, CapabilityManifest, CapabilityRecord, EffectClass};
    use morn_kernel::ids::CapabilityId;
    use morn_package::SupplyChainVerificationEvidence;
    use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};
    use morn_runtime::{ProviderDescriptor, ProviderFamily, ProviderStatus};
    use std::collections::BTreeSet;

    fn conformance(profile: &DomainProfile) -> morn_profile::ConformanceReport {
        let mut semantics = BTreeSet::new();
        semantics.extend(
            profile
                .requirements
                .iter()
                .filter(|item| item.level == RequirementLevel::Required)
                .map(|item| item.semantic.clone()),
        );
        evaluate_profile(
            profile,
            &ConformanceEvidence {
                satisfied_semantics: semantics,
                isolation: profile.minimum_isolation.clone(),
                execution_guarantees: profile
                    .required_execution_guarantees
                    .iter()
                    .copied()
                    .collect(),
                durable_work_state: profile.durable_work_state_required,
                source_of_truth_bound: profile.source_of_truth_binding_required,
                provenance_ready: profile.provenance_required,
                ..Default::default()
            },
        )
    }

    fn admitted_fixture(provider_ref: &str) -> (CapabilityRecord, AdmissionService, String) {
        let mut capability = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "fixture",
            provider_ref,
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));
        capability.manifest.provenance.source_ref = "repo://fixture".to_string();

        let profile = DomainProfile::factory_readonly_v1();
        let report = conformance(&profile);
        let mut service = AdmissionService::default();
        service
            .observe(
                &mut capability,
                vec!["eval:observation".to_string()],
                "evaluator",
            )
            .unwrap();
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:1".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["works".to_string()],
                        evaluator_identity: Some("evaluator".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let release_digest = format!("sha256:{}", "a".repeat(64));
        service
            .record_verified_release(
                &mut capability,
                &qualification,
                VerifiedReleaseRequest::new(
                    format!("oci://fixture/cap@{release_digest}"),
                    release_digest.clone(),
                    "sigstore://fixture",
                    "slsa://fixture",
                    SupplyChainVerificationEvidence {
                        subject_digest: release_digest,
                        verifier_ref: "fixture://supply-chain".to_string(),
                        signature_verified: true,
                        provenance_verified: true,
                        evidence_refs: vec!["fixture://supply-chain/proof".to_string()],
                    },
                ),
            )
            .unwrap();
        service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .unwrap();

        (capability, service, report.profile_ref)
    }

    #[test]
    fn admitted_capability_still_blocks_when_provider_is_unavailable() {
        let (capability, admissions, profile_ref) = admitted_fixture("runtime-a");
        let mut providers = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("runtime-a", ProviderFamily::Runtime, "1");
        provider.status = ProviderStatus::Unavailable;
        providers.register(provider).unwrap();

        let report = CapabilityEligibilityGate.strict(
            &[capability],
            &admissions,
            &providers,
            "plant-a",
            &profile_ref,
            Timestamp::now(),
        );
        assert!(report.selectable.is_empty());
        assert_eq!(report.blocked.len(), 1);
    }

    #[test]
    fn healthy_provider_cannot_bypass_missing_site_admission() {
        let (mut capability, admissions, profile_ref) = admitted_fixture("runtime-a");
        capability.admission_refs.clear();
        capability.admitted_sites.clear();

        let mut providers = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("runtime-a", ProviderFamily::Runtime, "1");
        provider.status = ProviderStatus::Healthy;
        providers.register(provider).unwrap();

        let report = CapabilityEligibilityGate.strict(
            &[capability],
            &admissions,
            &providers,
            "plant-a",
            &profile_ref,
            Timestamp::now(),
        );
        assert!(report.selectable.is_empty());
        assert_eq!(report.blocked.len(), 1);
    }
}
