//! Qualification and site-admission bridge for capability supply-chain semantics.
//!
//! This module does not replace the existing CertificationService. Instead it
//! normalizes a certification/release decision into a capability qualification
//! record and then binds that qualified capability to one site/profile only
//! after profile conformance passes.

use serde::{Deserialize, Serialize};

use morn_capability::manifest::{
    CapabilityAdmissionRef, CapabilityManifestId, CapabilityRecord, CapabilityStage,
};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;
use morn_profile::ConformanceReport;

use crate::profile_conformance::ProfileConformanceAttestation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapabilityLifecycleEventTag;
pub type CapabilityLifecycleEventId = Id<CapabilityLifecycleEventTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityLifecycleEvent {
    pub id: CapabilityLifecycleEventId,
    pub manifest_id: CapabilityManifestId,
    pub entity_ref: String,
    pub event_type: String,
    pub reason: String,
    pub actor_ref: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapabilityObservationTag;
pub type CapabilityObservationId = Id<CapabilityObservationTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityObservation {
    pub id: CapabilityObservationId,
    pub manifest_id: CapabilityManifestId,
    pub evidence_refs: Vec<String>,
    pub evaluator_identity: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct QualificationRecordTag;
pub type QualificationRecordId = Id<QualificationRecordTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SiteAdmissionTag;
pub type SiteAdmissionId = Id<SiteAdmissionTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapabilityDistributionReleaseTag;
pub type CapabilityDistributionReleaseId = Id<CapabilityDistributionReleaseTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CapabilityDistributionReleaseStatus {
    Released,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDistributionRelease {
    pub id: CapabilityDistributionReleaseId,
    pub manifest_id: CapabilityManifestId,
    pub qualification_id: QualificationRecordId,
    pub package_ref: String,
    pub content_digest: String,
    pub signature_ref: Option<String>,
    pub provenance_ref: Option<String>,
    pub status: CapabilityDistributionReleaseStatus,
    pub created_at: Timestamp,
}

impl CapabilityDistributionRelease {
    pub fn active(&self) -> bool {
        self.status == CapabilityDistributionReleaseStatus::Released
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum QualificationStatus {
    Qualified,
    Rejected,
    Suspended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QualificationEvidence {
    pub test_suite_refs: Vec<String>,
    pub environment_digest: Option<String>,
    pub input_scope: Vec<String>,
    pub expected_properties: Vec<String>,
    pub known_failure_modes: Vec<String>,
    pub cost_evidence_ref: Option<String>,
    pub latency_evidence_ref: Option<String>,
    pub evaluator_identity: Option<String>,
}

impl QualificationEvidence {
    pub fn is_strict_enough_for_site_admission(&self) -> bool {
        !self.test_suite_refs.is_empty()
            && self
                .environment_digest
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && !self.expected_properties.is_empty()
            && self
                .evaluator_identity
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictQualificationRequest {
    pub candidate_ref: String,
    pub decision_ref: String,
    pub evidence_refs: Vec<String>,
    pub qualification_evidence: QualificationEvidence,
    pub context_of_use: Vec<String>,
    pub valid_until: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualificationRecord {
    pub id: QualificationRecordId,
    pub manifest_id: CapabilityManifestId,
    pub qualification_version: Version,
    pub candidate_ref: String,
    pub decision_ref: String,
    pub evidence_refs: Vec<String>,
    pub qualification_evidence: QualificationEvidence,
    pub context_of_use: Vec<String>,
    pub status: QualificationStatus,
    pub valid_until: Option<Timestamp>,
    pub created_at: Timestamp,
}

impl QualificationRecord {
    pub fn is_active_at(&self, now: Timestamp) -> bool {
        self.status == QualificationStatus::Qualified
            && self.valid_until.is_none_or(|end| now <= end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SiteAdmissionStatus {
    Admitted,
    Suspended,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteAdmission {
    pub id: SiteAdmissionId,
    pub manifest_id: CapabilityManifestId,
    pub qualification_id: QualificationRecordId,
    pub release_id: CapabilityDistributionReleaseId,
    pub site_ref: String,
    pub profile_ref: String,
    pub conformance_ref: String,
    pub approved_by: String,
    pub status: SiteAdmissionStatus,
    pub created_at: Timestamp,
}

#[derive(Debug, Default)]
pub struct AdmissionService {
    pub events: Vec<CapabilityLifecycleEvent>,
    pub observations: Vec<CapabilityObservation>,
    pub qualifications: Vec<QualificationRecord>,
    pub releases: Vec<CapabilityDistributionRelease>,
    pub admissions: Vec<SiteAdmission>,
}

impl AdmissionService {
    fn record_event(
        &mut self,
        manifest_id: CapabilityManifestId,
        entity_ref: impl Into<String>,
        event_type: impl Into<String>,
        reason: impl Into<String>,
        actor_ref: impl Into<String>,
    ) {
        self.events.push(CapabilityLifecycleEvent {
            id: CapabilityLifecycleEventId::generate_with("cap-event"),
            manifest_id,
            entity_ref: entity_ref.into(),
            event_type: event_type.into(),
            reason: reason.into(),
            actor_ref: actor_ref.into(),
            created_at: Timestamp::now(),
        });
    }

    pub fn observe(
        &mut self,
        capability: &mut CapabilityRecord,
        evidence_refs: Vec<String>,
        evaluator_identity: impl Into<String>,
    ) -> Result<CapabilityObservation> {
        if !matches!(
            capability.stage,
            CapabilityStage::Declared | CapabilityStage::Observed
        ) {
            return Err(Error::invalid_state(
                "only declared/observed capabilities can record evaluation observation",
            ));
        }
        if evidence_refs.is_empty() {
            return Err(Error::validation(
                "capability observation requires evaluation evidence",
            ));
        }
        let evaluator_identity = evaluator_identity.into();
        if evaluator_identity.trim().is_empty() {
            return Err(Error::validation(
                "capability observation requires evaluator identity",
            ));
        }

        let observation = CapabilityObservation {
            id: CapabilityObservationId::generate_with("cap-observation"),
            manifest_id: capability.manifest.id.clone(),
            evidence_refs,
            evaluator_identity,
            created_at: Timestamp::now(),
        };
        capability.stage = CapabilityStage::Observed;
        self.observations.push(observation.clone());
        self.record_event(
            capability.manifest.id.clone(),
            observation.id.to_string(),
            "Observed",
            "evaluation observation recorded",
            observation.evaluator_identity.clone(),
        );
        Ok(observation)
    }

    /// Legacy qualification path retained for compatibility. It may normalize
    /// older certification evidence but remains insufficient for site admission.
    pub fn qualify(
        &mut self,
        capability: &mut CapabilityRecord,
        candidate_ref: impl Into<String>,
        decision_ref: impl Into<String>,
        evidence_refs: Vec<String>,
        context_of_use: Vec<String>,
    ) -> Result<QualificationRecord> {
        if !matches!(
            capability.stage,
            CapabilityStage::Declared | CapabilityStage::Observed | CapabilityStage::Qualified
        ) {
            return Err(Error::invalid_state(
                "only declared/observed/qualified capabilities can be qualified",
            ));
        }
        let candidate_ref = candidate_ref.into();
        let decision_ref = decision_ref.into();
        if candidate_ref.trim().is_empty() || decision_ref.trim().is_empty() {
            return Err(Error::validation(
                "qualification requires explicit candidate/build and decision references",
            ));
        }
        if evidence_refs.is_empty() {
            return Err(Error::validation(
                "qualification requires non-empty evaluation evidence",
            ));
        }

        let record = QualificationRecord {
            id: QualificationRecordId::generate_with("qual"),
            manifest_id: capability.manifest.id.clone(),
            qualification_version: Version::v1(),
            candidate_ref,
            decision_ref,
            qualification_evidence: QualificationEvidence::default(),
            evidence_refs,
            context_of_use,
            status: QualificationStatus::Qualified,
            valid_until: None,
            created_at: Timestamp::now(),
        };
        capability.stage = CapabilityStage::Qualified;
        capability.qualification_refs.push(record.id.to_string());
        self.qualifications.push(record.clone());
        self.record_event(
            capability.manifest.id.clone(),
            record.id.to_string(),
            "Qualified",
            "qualification decision recorded",
            record
                .qualification_evidence
                .evaluator_identity
                .clone()
                .unwrap_or_else(|| "legacy-qualification".to_string()),
        );
        Ok(record)
    }

    pub fn qualify_with_evidence(
        &mut self,
        capability: &mut CapabilityRecord,
        request: StrictQualificationRequest,
    ) -> Result<QualificationRecord> {
        let StrictQualificationRequest {
            candidate_ref,
            decision_ref,
            evidence_refs,
            qualification_evidence,
            context_of_use,
            valid_until,
        } = request;
        if !matches!(
            capability.stage,
            CapabilityStage::Observed | CapabilityStage::Qualified
        ) {
            return Err(Error::invalid_state(
                "strict qualification requires an observed/evaluated capability",
            ));
        }
        if evidence_refs.is_empty() || !qualification_evidence.is_strict_enough_for_site_admission()
        {
            return Err(Error::validation(
                "strict qualification requires evidence, tests, environment digest, expected properties and evaluator identity",
            ));
        }
        if valid_until.is_some_and(|end| end < Timestamp::now()) {
            return Err(Error::validation(
                "qualification validity has already expired",
            ));
        }
        if candidate_ref.trim().is_empty() || decision_ref.trim().is_empty() {
            return Err(Error::validation(
                "qualification requires explicit candidate/build and decision references",
            ));
        }

        let record = QualificationRecord {
            id: QualificationRecordId::generate_with("qual"),
            manifest_id: capability.manifest.id.clone(),
            qualification_version: Version::v1(),
            candidate_ref,
            decision_ref,
            evidence_refs,
            qualification_evidence,
            context_of_use,
            status: QualificationStatus::Qualified,
            valid_until,
            created_at: Timestamp::now(),
        };
        capability.stage = CapabilityStage::Qualified;
        capability.qualification_refs.push(record.id.to_string());
        self.qualifications.push(record.clone());
        Ok(record)
    }

    pub fn record_release(
        &mut self,
        capability: &mut CapabilityRecord,
        qualification: &QualificationRecord,
        package_ref: impl Into<String>,
        content_digest: impl Into<String>,
        signature_ref: Option<String>,
        provenance_ref: Option<String>,
    ) -> Result<CapabilityDistributionRelease> {
        if !qualification.is_active_at(Timestamp::now()) {
            return Err(Error::invalid_state(
                "cannot release from inactive qualification",
            ));
        }
        if qualification.manifest_id != capability.manifest.id {
            return Err(Error::validation(
                "release qualification does not match capability manifest",
            ));
        }
        let package_ref = package_ref.into();
        let content_digest = content_digest.into();
        if package_ref.trim().is_empty() {
            return Err(Error::validation("release requires package reference"));
        }
        let digest = content_digest
            .strip_prefix("sha256:")
            .ok_or_else(|| Error::validation("release content digest must use sha256:<64 hex>"))?;
        if digest.len() != 64 || !digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(Error::validation(
                "release content digest must use sha256:<64 hex>",
            ));
        }
        if provenance_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "release requires build/provenance evidence reference",
            ));
        }

        let release = CapabilityDistributionRelease {
            id: CapabilityDistributionReleaseId::generate_with("release"),
            manifest_id: capability.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            package_ref,
            content_digest,
            signature_ref,
            provenance_ref,
            status: CapabilityDistributionReleaseStatus::Released,
            created_at: Timestamp::now(),
        };
        capability.release_refs.push(release.id.to_string());
        self.releases.push(release.clone());
        self.record_event(
            capability.manifest.id.clone(),
            release.id.to_string(),
            "Released",
            "content-addressed capability release recorded",
            qualification
                .qualification_evidence
                .evaluator_identity
                .clone()
                .unwrap_or_else(|| "release-service".to_string()),
        );
        Ok(release)
    }

    pub fn admit_with_attestation(
        &mut self,
        capability: &mut CapabilityRecord,
        qualification: &QualificationRecord,
        attestation: &ProfileConformanceAttestation,
        approved_by: impl Into<String>,
    ) -> Result<SiteAdmission> {
        let now = Timestamp::now();
        if !attestation.passing_for(&attestation.site_ref, &attestation.profile_ref, now) {
            return Err(Error::validation(
                "site admission requires a currently valid passing Profile conformance attestation",
            ));
        }

        let approved_by = approved_by.into();
        let mut admission = self.admit(
            capability,
            qualification,
            attestation.site_ref.clone(),
            attestation.profile_ref.clone(),
            &attestation.report,
            approved_by,
        )?;
        admission.conformance_ref = attestation.id.to_string();
        let stored = self
            .admissions
            .iter_mut()
            .find(|item| item.id == admission.id)
            .ok_or_else(|| Error::internal("new SiteAdmission was not recorded"))?;
        stored.conformance_ref = attestation.id.to_string();
        Ok(admission)
    }

    pub fn admit(
        &mut self,
        capability: &mut CapabilityRecord,
        qualification: &QualificationRecord,
        site_ref: impl Into<String>,
        profile_ref: impl Into<String>,
        conformance: &ConformanceReport,
        approved_by: impl Into<String>,
    ) -> Result<SiteAdmission> {
        if !qualification.is_active_at(Timestamp::now()) {
            return Err(Error::invalid_state(
                "qualification is suspended, rejected or expired",
            ));
        }
        if !qualification
            .qualification_evidence
            .is_strict_enough_for_site_admission()
        {
            return Err(Error::validation(
                "site admission requires strict qualification evidence; legacy/minimal qualification is insufficient",
            ));
        }
        if qualification.manifest_id != capability.manifest.id {
            return Err(Error::validation(
                "qualification does not belong to capability manifest",
            ));
        }
        let release = self
            .releases
            .iter()
            .rev()
            .find(|release| {
                release.manifest_id == capability.manifest.id
                    && release.qualification_id == qualification.id
                    && release.active()
            })
            .ok_or_else(|| {
                Error::invalid_state(
                    "site admission requires an active content-addressed release after qualification",
                )
            })?;
        if capability.stage != CapabilityStage::Qualified
            && capability.stage != CapabilityStage::Admitted
        {
            return Err(Error::invalid_state(
                "site admission requires a qualified capability",
            ));
        }
        if !conformance.passed {
            return Err(Error::validation(format!(
                "profile conformance failed: missing={:?} violations={:?}",
                conformance.missing, conformance.violations
            )));
        }

        let site_ref = site_ref.into();
        if site_ref.trim().is_empty() {
            return Err(Error::validation(
                "site admission requires a site reference",
            ));
        }
        let profile_ref = profile_ref.into();
        if profile_ref != conformance.profile_ref {
            return Err(Error::validation(
                "site admission profile does not match conformance report",
            ));
        }

        let admission = SiteAdmission {
            id: SiteAdmissionId::generate_with("admission"),
            manifest_id: capability.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            release_id: release.id.clone(),
            site_ref: site_ref.clone(),
            profile_ref,
            conformance_ref: format!("conformance:{}", qualification.id),
            approved_by: approved_by.into(),
            status: SiteAdmissionStatus::Admitted,
            created_at: Timestamp::now(),
        };

        capability.stage = CapabilityStage::Admitted;
        if !capability
            .admitted_sites
            .iter()
            .any(|site| site == &site_ref)
        {
            capability.admitted_sites.push(site_ref.clone());
        }
        capability.admission_refs.retain(|existing| {
            !(existing.site_ref == site_ref && existing.profile_ref == admission.profile_ref)
        });
        capability.admission_refs.push(CapabilityAdmissionRef {
            admission_ref: admission.id.to_string(),
            site_ref: site_ref.clone(),
            profile_ref: admission.profile_ref.clone(),
        });
        self.admissions.push(admission.clone());
        self.record_event(
            capability.manifest.id.clone(),
            admission.id.to_string(),
            "SiteAdmitted",
            format!(
                "admitted to {} under {}",
                admission.site_ref, admission.profile_ref
            ),
            admission.approved_by.clone(),
        );
        Ok(admission)
    }

    /// Re-evaluate a concrete site/profile admission at selection time.
    ///
    /// CapabilityRecord carries projection refs for efficient discovery, but a
    /// strict runtime must still prove that the exact admission, qualification
    /// and release are active *now*. This prevents an expired qualification or
    /// revoked release from remaining selectable because a cached projection
    /// still says "Admitted".
    pub fn site_profile_admission_active_at(
        &self,
        capability: &CapabilityRecord,
        site_ref: &str,
        profile_ref: &str,
        now: Timestamp,
    ) -> bool {
        capability.admission_refs.iter().any(|reference| {
            if reference.site_ref != site_ref || reference.profile_ref != profile_ref {
                return false;
            }
            let Some(admission) = self
                .admissions
                .iter()
                .find(|item| item.id.to_string() == reference.admission_ref)
            else {
                return false;
            };
            if admission.status != SiteAdmissionStatus::Admitted
                || admission.manifest_id != capability.manifest.id
                || admission.site_ref != site_ref
                || admission.profile_ref != profile_ref
            {
                return false;
            }
            let Some(qualification) = self
                .qualifications
                .iter()
                .find(|item| item.id == admission.qualification_id)
            else {
                return false;
            };
            if !qualification.is_active_at(now)
                || qualification.manifest_id != capability.manifest.id
            {
                return false;
            }
            self.releases.iter().any(|release| {
                release.id == admission.release_id
                    && release.manifest_id == capability.manifest.id
                    && release.qualification_id == qualification.id
                    && release.active()
            })
        })
    }

    pub fn revoke_release(
        &mut self,
        capability: &mut CapabilityRecord,
        release_id: &CapabilityDistributionReleaseId,
    ) -> Result<()> {
        let release = self
            .releases
            .iter_mut()
            .find(|item| item.id == *release_id)
            .ok_or_else(|| Error::not_found(format!("capability release {release_id}")))?;
        if release.manifest_id != capability.manifest.id {
            return Err(Error::validation(
                "release does not belong to supplied capability manifest",
            ));
        }
        release.status = CapabilityDistributionReleaseStatus::Revoked;
        capability
            .release_refs
            .retain(|reference| reference != &release.id.to_string());

        let affected: Vec<String> = self
            .admissions
            .iter_mut()
            .filter(|admission| admission.release_id == release.id)
            .map(|admission| {
                admission.status = SiteAdmissionStatus::Suspended;
                admission.id.to_string()
            })
            .collect();
        capability.admission_refs.retain(|reference| {
            !affected
                .iter()
                .any(|admission_id| admission_id == &reference.admission_ref)
        });
        capability.admitted_sites = capability
            .admission_refs
            .iter()
            .map(|reference| reference.site_ref.clone())
            .collect();
        capability.admitted_sites.sort();
        capability.admitted_sites.dedup();
        capability.stage = if capability.admission_refs.is_empty() {
            CapabilityStage::Suspended
        } else {
            CapabilityStage::Admitted
        };
        self.record_event(
            capability.manifest.id.clone(),
            release_id.to_string(),
            "ReleaseRevoked",
            "release revoked; dependent site admissions suspended",
            "system",
        );
        Ok(())
    }

    pub fn suspend(
        &mut self,
        capability: &mut CapabilityRecord,
        admission_id: &SiteAdmissionId,
    ) -> Result<()> {
        let admission = self
            .admissions
            .iter_mut()
            .find(|item| item.id == *admission_id)
            .ok_or_else(|| Error::not_found(format!("site admission {admission_id}")))?;
        if admission.manifest_id != capability.manifest.id {
            return Err(Error::validation(
                "admission does not belong to supplied capability manifest",
            ));
        }
        admission.status = SiteAdmissionStatus::Suspended;
        capability
            .admission_refs
            .retain(|reference| reference.admission_ref != admission.id.to_string());
        capability.admitted_sites = capability
            .admission_refs
            .iter()
            .map(|reference| reference.site_ref.clone())
            .collect();
        capability.admitted_sites.sort();
        capability.admitted_sites.dedup();
        capability.stage = if capability.admission_refs.is_empty() {
            CapabilityStage::Suspended
        } else {
            CapabilityStage::Admitted
        };
        self.record_event(
            capability.manifest.id.clone(),
            admission_id.to_string(),
            "SiteAdmissionSuspended",
            "site admission suspended",
            "system",
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::effect::EffectClass;
    use morn_capability::manifest::CapabilityManifest;
    use morn_capability::registry::CapabilityKind;
    use morn_kernel::ids::CapabilityId;
    use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};
    use std::collections::BTreeSet;

    fn candidate() -> CapabilityRecord {
        CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "equipment-investigator",
            "provider://fixture",
            CapabilityKind::Llm,
            EffectClass::E0LifecycleReversible,
        ))
    }

    fn observe_candidate(service: &mut AdmissionService, capability: &mut CapabilityRecord) {
        service
            .observe(
                capability,
                vec!["evaluation:observation".to_string()],
                "evaluator:observation",
            )
            .unwrap();
    }

    fn record_fixture_release(
        service: &mut AdmissionService,
        capability: &mut CapabilityRecord,
        qualification: &QualificationRecord,
    ) -> CapabilityDistributionRelease {
        service
            .record_release(
                capability,
                qualification,
                format!("oci://fixture/morn/capability@sha256:{}", "a".repeat(64)),
                format!("sha256:{}", "a".repeat(64)),
                Some("sigstore://fixture/signature".to_string()),
                Some("slsa://fixture/provenance".to_string()),
            )
            .unwrap()
    }

    fn passing_conformance(profile: &DomainProfile) -> ConformanceReport {
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
            },
        )
    }

    #[test]
    fn strict_qualification_rejects_unobserved_candidate() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        let result = service.qualify_with_evidence(
            &mut capability,
            StrictQualificationRequest {
                candidate_ref: "candidate:1".to_string(),
                decision_ref: "decision:1".to_string(),
                evidence_refs: vec!["eval:1".to_string()],
                qualification_evidence: QualificationEvidence {
                    test_suite_refs: vec!["suite:factory".to_string()],
                    environment_digest: Some("sha256:env".to_string()),
                    expected_properties: vec!["safe-reconcile".to_string()],
                    evaluator_identity: Some("evaluator:independent".to_string()),
                    ..Default::default()
                },
                context_of_use: vec!["factory-readonly".to_string()],
                valid_until: None,
            },
        );
        assert!(result.is_err());
        assert_eq!(capability.stage, CapabilityStage::Declared);
    }

    #[test]
    fn compile_is_not_qualification_and_qualification_is_not_admission() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        assert_eq!(capability.stage, CapabilityStage::Declared);
        observe_candidate(&mut service, &mut capability);

        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "certification-decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        assert_eq!(capability.stage, CapabilityStage::Qualified);
        assert!(capability.admitted_sites.is_empty());

        let _release = record_fixture_release(&mut service, &mut capability, &qualification);
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        let admission = service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .unwrap();
        assert_eq!(admission.status, SiteAdmissionStatus::Admitted);
        assert_eq!(capability.stage, CapabilityStage::Admitted);
        assert_eq!(capability.admitted_sites, vec!["plant-a".to_string()]);
    }

    #[test]
    fn expired_qualification_invalidates_existing_admission_at_selection_time() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let base = Timestamp::now();
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:expiring".to_string(),
                    decision_ref: "decision:expiring".to_string(),
                    evidence_refs: vec!["eval:expiring".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: Some(Timestamp::from_millis(base.millis() + 60_000)),
                },
            )
            .unwrap();
        let _release = record_fixture_release(&mut service, &mut capability, &qualification);
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
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

        assert!(service.site_profile_admission_active_at(
            &capability,
            "plant-a",
            &report.profile_ref,
            base,
        ));
        assert!(!service.site_profile_admission_active_at(
            &capability,
            "plant-a",
            &report.profile_ref,
            Timestamp::from_millis(base.millis() + 60_001),
        ));
    }

    #[test]
    fn admission_is_scoped_to_profile_as_well_as_site() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let _release = record_fixture_release(&mut service, &mut capability, &qualification);
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        let admission = service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .unwrap();
        assert!(capability.admission_refs.iter().any(|reference| {
            reference.admission_ref == admission.id.to_string()
                && reference.site_ref == "plant-a"
                && reference.profile_ref == report.profile_ref
        }));
    }

    #[test]
    fn qualification_without_release_cannot_be_site_admitted() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        assert!(service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .is_err());
    }

    #[test]
    fn revoked_release_suspends_dependent_site_admission() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let release = record_fixture_release(&mut service, &mut capability, &qualification);
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        let admission = service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .unwrap();
        service
            .revoke_release(&mut capability, &release.id)
            .unwrap();

        assert_eq!(
            service
                .admissions
                .iter()
                .find(|item| item.id == admission.id)
                .unwrap()
                .status,
            SiteAdmissionStatus::Suspended
        );
        assert!(capability.admission_refs.is_empty());
        assert!(capability.admitted_sites.is_empty());
        assert_eq!(capability.stage, CapabilityStage::Suspended);
    }

    #[test]
    fn cross_manifest_revocation_or_suspension_cannot_mutate_other_capability() {
        let mut original = candidate();
        let mut unrelated = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut original);

        let qualification = service
            .qualify_with_evidence(
                &mut original,
                StrictQualificationRequest {
                    candidate_ref: "candidate:owner".to_string(),
                    decision_ref: "decision:owner".to_string(),
                    evidence_refs: vec!["eval:owner".to_string()],
                    qualification_evidence: QualificationEvidence {
                        test_suite_refs: vec!["suite:factory".to_string()],
                        environment_digest: Some("sha256:env".to_string()),
                        expected_properties: vec!["safe-reconcile".to_string()],
                        evaluator_identity: Some("evaluator:independent".to_string()),
                        ..Default::default()
                    },
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let release = record_fixture_release(&mut service, &mut original, &qualification);
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        let admission = service
            .admit(
                &mut original,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .unwrap();

        assert!(service.revoke_release(&mut unrelated, &release.id).is_err());
        assert!(service.suspend(&mut unrelated, &admission.id).is_err());
        assert!(service
            .releases
            .iter()
            .find(|record| record.id == release.id)
            .unwrap()
            .active());
        assert_eq!(
            service
                .admissions
                .iter()
                .find(|record| record.id == admission.id)
                .unwrap()
                .status,
            SiteAdmissionStatus::Admitted
        );
        assert_eq!(original.stage, CapabilityStage::Admitted);
        assert_eq!(unrelated.stage, CapabilityStage::Declared);
        assert!(unrelated.admission_refs.is_empty());
    }

    #[test]
    fn legacy_qualification_cannot_be_site_admitted() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let qualification = service
            .qualify(
                &mut capability,
                "candidate:legacy",
                "decision:legacy",
                vec!["eval:legacy".to_string()],
                vec!["factory-readonly".to_string()],
            )
            .unwrap();
        assert!(!qualification
            .qualification_evidence
            .is_strict_enough_for_site_admission());
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        assert!(service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .is_err());
    }

    #[test]
    fn expired_qualification_blocks_site_admission() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let evidence = QualificationEvidence {
            test_suite_refs: vec!["suite:factory".to_string()],
            environment_digest: Some("sha256:environment".to_string()),
            expected_properties: vec!["no-blind-retry".to_string()],
            evaluator_identity: Some("evaluator:independent".to_string()),
            ..Default::default()
        };
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                StrictQualificationRequest {
                    candidate_ref: "candidate:1".to_string(),
                    decision_ref: "decision:1".to_string(),
                    evidence_refs: vec!["eval:1".to_string()],
                    qualification_evidence: evidence,
                    context_of_use: vec!["factory-readonly".to_string()],
                    valid_until: None,
                },
            )
            .unwrap();
        let mut expired = qualification.clone();
        expired.valid_until = Some(Timestamp::from_millis(0));
        let profile = DomainProfile::factory_readonly_v1();
        let report = passing_conformance(&profile);
        assert!(service
            .admit(
                &mut capability,
                &expired,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .is_err());
    }

    #[test]
    fn failed_profile_conformance_blocks_site_admission() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        observe_candidate(&mut service, &mut capability);
        let qualification = service
            .qualify(
                &mut capability,
                "candidate:1",
                "decision:1",
                vec!["eval:1".to_string()],
                vec![],
            )
            .unwrap();
        let profile = DomainProfile::factory_readonly_v1();
        let report = evaluate_profile(&profile, &ConformanceEvidence::default());
        assert!(!report.passed);
        assert!(service
            .admit(
                &mut capability,
                &qualification,
                "plant-a",
                report.profile_ref.clone(),
                &report,
                "site-owner",
            )
            .is_err());
    }
}
