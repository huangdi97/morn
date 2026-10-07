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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct QualificationRecordTag;
pub type QualificationRecordId = Id<QualificationRecordTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SiteAdmissionTag;
pub type SiteAdmissionId = Id<SiteAdmissionTag>;

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
pub struct QualificationRecord {
    pub id: QualificationRecordId,
    pub manifest_id: CapabilityManifestId,
    pub qualification_version: Version,
    pub release_ref: String,
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
    pub site_ref: String,
    pub profile_ref: String,
    pub conformance_ref: String,
    pub approved_by: String,
    pub status: SiteAdmissionStatus,
    pub created_at: Timestamp,
}

#[derive(Debug, Default)]
pub struct AdmissionService {
    pub qualifications: Vec<QualificationRecord>,
    pub admissions: Vec<SiteAdmission>,
}

impl AdmissionService {
    pub fn qualify(
        &mut self,
        capability: &mut CapabilityRecord,
        release_ref: impl Into<String>,
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
        let release_ref = release_ref.into();
        let decision_ref = decision_ref.into();
        if release_ref.trim().is_empty() || decision_ref.trim().is_empty() {
            return Err(Error::validation(
                "qualification requires explicit release and decision references",
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
            release_ref,
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
        Ok(record)
    }

    pub fn qualify_with_evidence(
        &mut self,
        capability: &mut CapabilityRecord,
        release_ref: impl Into<String>,
        decision_ref: impl Into<String>,
        evidence_refs: Vec<String>,
        qualification_evidence: QualificationEvidence,
        context_of_use: Vec<String>,
        valid_until: Option<Timestamp>,
    ) -> Result<QualificationRecord> {
        if !matches!(
            capability.stage,
            CapabilityStage::Declared | CapabilityStage::Observed | CapabilityStage::Qualified
        ) {
            return Err(Error::invalid_state(
                "only declared/observed/qualified capabilities can be qualified",
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
        let release_ref = release_ref.into();
        let decision_ref = decision_ref.into();
        if release_ref.trim().is_empty() || decision_ref.trim().is_empty() {
            return Err(Error::validation(
                "qualification requires explicit release and decision references",
            ));
        }

        let record = QualificationRecord {
            id: QualificationRecordId::generate_with("qual"),
            manifest_id: capability.manifest.id.clone(),
            qualification_version: Version::v1(),
            release_ref,
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
        Ok(admission)
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
        admission.status = SiteAdmissionStatus::Suspended;
        capability
            .admitted_sites
            .retain(|site| site != &admission.site_ref);
        capability.stage = CapabilityStage::Suspended;
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
                durable_work_state: true,
                source_of_truth_bound: true,
                provenance_ready: true,
                ..Default::default()
            },
        )
    }

    #[test]
    fn compile_is_not_qualification_and_qualification_is_not_admission() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        assert_eq!(capability.stage, CapabilityStage::Declared);

        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                "release:1",
                "certification-decision:1",
                vec!["eval:1".to_string()],
                QualificationEvidence {
                    test_suite_refs: vec!["suite:factory".to_string()],
                    environment_digest: Some("sha256:env".to_string()),
                    expected_properties: vec!["safe-reconcile".to_string()],
                    evaluator_identity: Some("evaluator:independent".to_string()),
                    ..Default::default()
                },
                vec!["factory-readonly".to_string()],
                None,
            )
            .unwrap();
        assert_eq!(capability.stage, CapabilityStage::Qualified);
        assert!(capability.admitted_sites.is_empty());

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
    fn admission_is_scoped_to_profile_as_well_as_site() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        let qualification = service
            .qualify_with_evidence(
                &mut capability,
                "release:1",
                "decision:1",
                vec!["eval:1".to_string()],
                QualificationEvidence {
                    test_suite_refs: vec!["suite:factory".to_string()],
                    environment_digest: Some("sha256:env".to_string()),
                    expected_properties: vec!["safe-reconcile".to_string()],
                    evaluator_identity: Some("evaluator:independent".to_string()),
                    ..Default::default()
                },
                vec!["factory-readonly".to_string()],
                None,
            )
            .unwrap();
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
    fn legacy_qualification_cannot_be_site_admitted() {
        let mut capability = candidate();
        let mut service = AdmissionService::default();
        let qualification = service
            .qualify(
                &mut capability,
                "release:legacy",
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
                "release:1",
                "decision:1",
                vec!["eval:1".to_string()],
                evidence,
                vec!["factory-readonly".to_string()],
                None,
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
        let qualification = service
            .qualify(
                &mut capability,
                "release:1",
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
