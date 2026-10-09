//! Typed producers for durable Work readiness evidence.
//!
//! This layer prevents generic callers from asserting readiness booleans. Each
//! positive ConditionEvidence must be derived from a concrete Morn record or
//! resolver result with linkage to the exact Work generation.

use morn_assurance::AdmissionService;
use morn_capability::{CapabilityRecord, WorkcellPlan};
use morn_integration::SourceOfTruthBinding;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use serde::{Deserialize, Serialize};
use morn_runtime::BoundAuthorityDecision;
use morn_work::control::WorkResource;

use crate::ConditionEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CapabilityResolutionDecisionTag;
pub type CapabilityResolutionDecisionId = Id<CapabilityResolutionDecisionTag>;

/// Durable record of which exact capabilities satisfied one Work generation.
/// The planner output is evidence for resolution; it does not grant authority
/// and it is not an ExecutionBinding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityResolutionDecision {
    pub id: CapabilityResolutionDecisionId,
    pub work_ref: String,
    pub work_generation: u64,
    pub source_solution_ref: Option<String>,
    pub plan: WorkcellPlan,
    pub created_at: Timestamp,
}

impl CapabilityResolutionDecision {
    pub fn new(work: &WorkResource, plan: WorkcellPlan) -> Result<Self> {
        if !plan.is_complete() || plan.members.is_empty() {
            return Err(Error::invalid_state(
                "capability resolution decision requires a complete non-empty Workcell plan",
            ));
        }
        Ok(Self {
            id: CapabilityResolutionDecisionId::generate_with("cap-resolution"),
            work_ref: work.id.to_string(),
            work_generation: work.generation,
            source_solution_ref: work.spec.source_solution_ref.clone(),
            plan,
            created_at: Timestamp::now(),
        })
    }

    pub fn matches_work(&self, work: &WorkResource) -> bool {
        self.work_ref == work.id.to_string()
            && self.work_generation == work.generation
            && self.source_solution_ref == work.spec.source_solution_ref
            && self.plan.is_complete()
            && !self.plan.members.is_empty()
    }
}

pub fn capability_resolution_evidence(
    work: &WorkResource,
    plan: &WorkcellPlan,
) -> Result<ConditionEvidence> {
    if !plan.is_complete() {
        return Err(Error::invalid_state(format!(
            "Workcell plan is incomplete: {:?}",
            plan.uncovered
        )));
    }
    if plan.members.is_empty() {
        return Err(Error::invalid_state(
            "CapabilityResolved requires at least one selected executor capability",
        ));
    }
    let refs = plan
        .members
        .iter()
        .map(|member| member.capability.manifest_id.to_string())
        .collect();
    ConditionEvidence::new(
        work,
        "CapabilityResolved",
        true,
        "controller://capability-resolver",
        refs,
    )
}

pub fn capability_qualification_evidence(
    work: &WorkResource,
    capability: &CapabilityRecord,
    admissions: &AdmissionService,
    now: Timestamp,
) -> Result<ConditionEvidence> {
    workcell_qualification_evidence(work, std::slice::from_ref(capability), admissions, now)
}

/// Qualification readiness is a Workcell property. Every capability selected
/// by the current-generation resolver must retain an exact site/profile
/// qualification + release + admission chain; one qualified member must never
/// make a partially admitted Workcell Ready.
pub fn workcell_qualification_evidence(
    work: &WorkResource,
    capabilities: &[CapabilityRecord],
    admissions: &AdmissionService,
    now: Timestamp,
) -> Result<ConditionEvidence> {
    if capabilities.is_empty() {
        return Err(Error::validation(
            "CapabilityQualified requires at least one resolved capability",
        ));
    }
    let site = work
        .spec
        .site_ref
        .as_deref()
        .ok_or_else(|| Error::validation("qualified governed Work requires site_ref"))?;
    let mut refs = Vec::new();
    for capability in capabilities {
        if !admissions.site_profile_admission_active_at(
            capability,
            site,
            &work.spec.profile_ref,
            now,
        ) {
            return Err(Error::invalid_state(format!(
                "capability {} lacks active qualification/release/admission for site {} profile {}",
                capability.manifest.id, site, work.spec.profile_ref
            )));
        }
        let admissions_for_member = capability
            .admission_refs
            .iter()
            .filter(|reference| {
                reference.site_ref == site && reference.profile_ref == work.spec.profile_ref
            })
            .map(|reference| reference.admission_ref.clone())
            .collect::<Vec<_>>();
        if admissions_for_member.is_empty() {
            return Err(Error::invalid_state(format!(
                "capability {} has no exact site/profile admission reference",
                capability.manifest.id
            )));
        }
        refs.push(capability.manifest.id.to_string());
        refs.extend(admissions_for_member);
    }
    refs.sort();
    refs.dedup();
    ConditionEvidence::new(
        work,
        "CapabilityQualified",
        true,
        "controller://workcell-admission",
        refs,
    )
}

pub fn authority_condition_evidence(
    work: &WorkResource,
    authority: &BoundAuthorityDecision,
    now: Timestamp,
) -> Result<ConditionEvidence> {
    if !authority.decision.allowed || !authority.request.valid_now(now) {
        return Err(Error::not_authorized(
            "AuthoritySatisfied requires a currently valid allow decision",
        ));
    }
    if authority.request.work_ref.as_deref() != Some(work.id.as_str()) {
        return Err(Error::validation(
            "authority decision is not bound to this Work",
        ));
    }
    if authority.request.site_ref != work.spec.site_ref {
        return Err(Error::validation(
            "authority decision site does not match Work site",
        ));
    }
    let mut refs = vec![authority.decision.id.to_string()];
    refs.extend(authority.decision.evidence_refs.clone());
    ConditionEvidence::new(
        work,
        "AuthoritySatisfied",
        true,
        format!("authority://{}", authority.decision.provider),
        refs,
    )
}

pub fn source_of_truth_condition_evidence(
    work: &WorkResource,
    binding: &SourceOfTruthBinding,
) -> Result<ConditionEvidence> {
    binding.validate()?;
    if binding.site_ref != work.spec.site_ref {
        return Err(Error::validation(
            "source-of-truth binding site does not match Work site",
        ));
    }
    ConditionEvidence::new(
        work,
        "SourceOfTruthBound",
        true,
        "controller://source-of-truth",
        vec![binding.id.to_string(), binding.source_ref.clone()],
    )
}

/// Pre-execution provenance gate. This intentionally validates capability/source
/// lineage rather than ExecutionManifest, because the latter is created only
/// after an ExecutionBinding exists. Binding/runtime provenance is checked by
/// the separate ExecutionManifest path after Work becomes Ready.
pub fn provenance_condition_evidence(
    work: &WorkResource,
    capabilities: &[CapabilityRecord],
) -> Result<ConditionEvidence> {
    if capabilities.is_empty() {
        return Err(Error::validation(
            "ProvenanceReady requires at least one selected capability",
        ));
    }
    let mut refs = Vec::new();
    for capability in capabilities {
        let source_ref = capability.manifest.provenance.source_ref.trim();
        let source_digest = capability
            .manifest
            .provenance
            .source_digest
            .as_deref()
            .unwrap_or("")
            .trim();
        if source_ref.is_empty() || source_digest.is_empty() {
            return Err(Error::invalid_state(format!(
                "capability {} lacks source reference/digest provenance",
                capability.manifest.id
            )));
        }
        refs.push(capability.manifest.id.to_string());
        refs.push(capability.manifest.provenance.source_ref.clone());
        refs.push(source_digest.to_string());
    }
    if let Some(solution) = &work.spec.source_solution_ref {
        refs.push(solution.clone());
    }
    ConditionEvidence::new(
        work,
        "ProvenanceReady",
        true,
        "controller://provenance",
        refs,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::{
        CapabilityKind, CapabilityManifest, EffectClass, ResolvedCapability, WorkcellMember,
    };
    use morn_kernel::ids::{CapabilityId, WorkPackageId, WorkspaceId};
    use morn_work::control::WorkSpec;

    #[test]
    fn workcell_qualification_fails_if_any_selected_member_lacks_exact_admission() {
        use morn_assurance::{
            CapabilityDistributionRelease, CapabilityDistributionReleaseId,
            CapabilityDistributionReleaseStatus, QualificationEvidence, QualificationRecord,
            QualificationRecordId, QualificationStatus, SiteAdmission, SiteAdmissionId,
            SiteAdmissionStatus,
        };
        use morn_kernel::version::Version;

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut first = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "first",
            "provider-a",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));
        let second = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "second",
            "provider-b",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));

        let qualification = QualificationRecord {
            id: QualificationRecordId::generate_with("qual"),
            manifest_id: first.manifest.id.clone(),
            qualification_version: Version::v1(),
            candidate_ref: "candidate://first".to_string(),
            decision_ref: "decision://first".to_string(),
            evidence_refs: vec!["test://first".to_string()],
            qualification_evidence: QualificationEvidence {
                test_suite_refs: vec!["test://suite".to_string()],
                environment_digest: Some("sha256:fixture".to_string()),
                expected_properties: vec!["safe".to_string()],
                evaluator_identity: Some("evaluator".to_string()),
                ..Default::default()
            },
            context_of_use: vec![],
            status: QualificationStatus::Qualified,
            valid_until: None,
            created_at: Timestamp::now(),
        };
        let release = CapabilityDistributionRelease {
            id: CapabilityDistributionReleaseId::generate_with("release"),
            manifest_id: first.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            package_ref: "package://first".to_string(),
            content_digest: format!("sha256:{}", "a".repeat(64)),
            signature_ref: None,
            provenance_ref: Some("build://first".to_string()),
            status: CapabilityDistributionReleaseStatus::Released,
            created_at: Timestamp::now(),
        };
        let admission = SiteAdmission {
            id: SiteAdmissionId::generate_with("admit"),
            manifest_id: first.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            release_id: release.id.clone(),
            site_ref: "plant-a".to_string(),
            profile_ref: work.spec.profile_ref.clone(),
            conformance_ref: "conformance://first".to_string(),
            approved_by: "owner".to_string(),
            status: SiteAdmissionStatus::Admitted,
            created_at: Timestamp::now(),
        };
        first.admission_refs.push(morn_capability::CapabilityAdmissionRef {
            admission_ref: admission.id.to_string(),
            site_ref: "plant-a".to_string(),
            profile_ref: work.spec.profile_ref.clone(),
        });
        let mut service = AdmissionService::default();
        service.qualifications.push(qualification);
        service.releases.push(release);
        service.admissions.push(admission);

        assert!(workcell_qualification_evidence(
            &work,
            &[first.clone(), second],
            &service,
            Timestamp::now()
        )
        .is_err());
        let evidence = workcell_qualification_evidence(
            &work,
            &[first],
            &service,
            Timestamp::now(),
        )
        .unwrap();
        assert_eq!(evidence.condition_type, "CapabilityQualified");
    }

    #[test]
    fn resolution_decision_is_generation_and_solution_scoped() {
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "resolve",
                "morn.lite@1.0.0",
            ),
        );
        work.spec.source_solution_ref = Some("solution://one@1.0.0".to_string());
        let manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "rule",
            "provider",
            CapabilityKind::Rule,
            EffectClass::E0LifecycleReversible,
        );
        let plan = WorkcellPlan {
            members: vec![WorkcellMember {
                capability: ResolvedCapability {
                    manifest_id: manifest.id,
                    provider_ref: "provider".to_string(),
                    kind: CapabilityKind::Rule,
                    estimated_cost_micros: None,
                    maximum_effect: EffectClass::E0LifecycleReversible,
                    compensation_ref: None,
                    idempotency_key_required: false,
                    required_execution_class: morn_kernel::ExecutionClass::NoIsolation,
                    required_execution_guarantees: vec![],
                    score: 1,
                    rationale: vec![],
                },
                covers: vec!["x".to_string()],
            }],
            uncovered: vec![],
            total_estimated_cost_micros: 0,
            rationale: vec![],
        };
        let decision = CapabilityResolutionDecision::new(&work, plan).unwrap();
        assert!(decision.matches_work(&work));
        let mut next = work.spec.clone();
        next.goal = "changed".to_string();
        work.replace_spec(next);
        assert!(!decision.matches_work(&work));
    }

    #[test]
    fn provenance_readiness_does_not_depend_on_future_execution_binding() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.factory.readonly@1.0.0",
            ),
        );
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "reader",
            "reader-provider",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provenance.source_ref = "repo://reader".to_string();
        manifest.provenance.source_digest = Some("sha256:fixture".to_string());
        let capability = CapabilityRecord::new(manifest);
        let evidence = provenance_condition_evidence(&work, &[capability]).unwrap();
        assert_eq!(evidence.condition_type, "ProvenanceReady");
        assert!(evidence
            .evidence_refs
            .iter()
            .any(|item| item == "repo://reader"));
    }

    #[test]
    fn incomplete_workcell_cannot_produce_capability_resolved() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        let plan = WorkcellPlan {
            members: vec![],
            uncovered: vec!["capacity.optimize".to_string()],
            total_estimated_cost_micros: 0,
            rationale: vec![],
        };
        assert!(capability_resolution_evidence(&work, &plan).is_err());
    }

    #[test]
    fn complete_non_agent_workcell_can_produce_resolution_evidence() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review",
                "morn.lite@1.0.0",
            ),
        );
        let manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "rule",
            "rule-provider",
            CapabilityKind::Rule,
            EffectClass::E0LifecycleReversible,
        );
        let resolved = ResolvedCapability {
            manifest_id: manifest.id,
            provider_ref: "rule-provider".to_string(),
            kind: CapabilityKind::Rule,
            estimated_cost_micros: Some(1),
            maximum_effect: EffectClass::E0LifecycleReversible,
            compensation_ref: None,
            idempotency_key_required: false,
            required_execution_class: morn_capability::IsolationLevel::Process,
            required_execution_guarantees: vec![],
            score: 100,
            rationale: vec!["fixture".to_string()],
        };
        let plan = WorkcellPlan {
            members: vec![WorkcellMember {
                capability: resolved,
                covers: vec!["alarm.classify".to_string()],
            }],
            uncovered: vec![],
            total_estimated_cost_micros: 1,
            rationale: vec![],
        };
        let evidence = capability_resolution_evidence(&work, &plan).unwrap();
        assert_eq!(evidence.condition_type, "CapabilityResolved");
        assert!(!evidence.evidence_refs.is_empty());
    }
}
