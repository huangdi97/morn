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
use morn_runtime::BoundAuthorityDecision;
use morn_work::control::WorkResource;
use serde::{Deserialize, Serialize};

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
    decision: &CapabilityResolutionDecision,
) -> Result<ConditionEvidence> {
    if !decision.matches_work(work) {
        return Err(Error::validation(
            "capability resolution decision is not bound to this Work generation/solution",
        ));
    }
    let mut refs = decision
        .plan
        .members
        .iter()
        .map(|member| member.capability.manifest_id.to_string())
        .collect::<Vec<_>>();
    refs.push(decision.id.to_string());
    refs.sort();
    refs.dedup();
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
    decision: &CapabilityResolutionDecision,
    capability: &CapabilityRecord,
    admissions: &AdmissionService,
    now: Timestamp,
) -> Result<ConditionEvidence> {
    workcell_qualification_evidence(
        work,
        decision,
        std::slice::from_ref(capability),
        admissions,
        now,
    )
}

/// Qualification readiness is a Workcell property. Every capability selected
/// by the current-generation resolver must retain an exact site/profile
/// qualification + release + admission chain; one qualified member must never
/// make a partially admitted Workcell Ready.
pub fn workcell_qualification_evidence(
    work: &WorkResource,
    decision: &CapabilityResolutionDecision,
    capabilities: &[CapabilityRecord],
    admissions: &AdmissionService,
    now: Timestamp,
) -> Result<ConditionEvidence> {
    if !decision.matches_work(work) {
        return Err(Error::validation(
            "CapabilityQualified requires the current Work generation/solution resolution decision",
        ));
    }
    if capabilities.is_empty() {
        return Err(Error::validation(
            "CapabilityQualified requires at least one resolved capability",
        ));
    }
    let mut resolved_ids = decision
        .plan
        .members
        .iter()
        .map(|member| member.capability.manifest_id.to_string())
        .collect::<Vec<_>>();
    resolved_ids.sort();
    resolved_ids.dedup();
    let mut supplied_ids = capabilities
        .iter()
        .map(|capability| capability.manifest.id.to_string())
        .collect::<Vec<_>>();
    supplied_ids.sort();
    supplied_ids.dedup();
    if supplied_ids != resolved_ids || supplied_ids.len() != capabilities.len() {
        return Err(Error::validation(
            "CapabilityQualified capability set must exactly match the current resolution decision",
        ));
    }
    let site = work
        .spec
        .site_ref
        .as_deref()
        .ok_or_else(|| Error::validation("qualified governed Work requires site_ref"))?;
    let mut refs = Vec::new();
    let mut workcell_valid_until: Option<Timestamp> = None;
    for capability in capabilities {
        let active_admissions = admissions.active_site_profile_admissions_at(
            capability,
            site,
            &work.spec.profile_ref,
            now,
        );
        let Some((selected_admission, selected_qualification)) = active_admissions
            .into_iter()
            .filter_map(|admission| {
                admissions
                    .qualifications
                    .iter()
                    .find(|qualification| qualification.id == admission.qualification_id)
                    .map(|qualification| (admission, qualification))
            })
            .max_by_key(|(admission, qualification)| {
                (
                    qualification.valid_until.is_none(),
                    qualification
                        .valid_until
                        .map(|until| until.millis())
                        .unwrap_or(i64::MAX),
                    admission.id.to_string(),
                )
            })
        else {
            return Err(Error::invalid_state(format!(
                "capability {} lacks active qualification/release/admission for site {} profile {}",
                capability.manifest.id, site, work.spec.profile_ref
            )));
        };

        refs.push(capability.manifest.id.to_string());
        refs.push(selected_admission.id.to_string());
        workcell_valid_until = match (workcell_valid_until, selected_qualification.valid_until) {
            (Some(current), Some(candidate)) => Some(if current <= candidate {
                current
            } else {
                candidate
            }),
            (None, Some(candidate)) => Some(candidate),
            (Some(current), None) => Some(current),
            (None, None) => None,
        };
    }
    refs.push(decision.id.to_string());
    refs.sort();
    refs.dedup();
    let mut evidence = ConditionEvidence::new(
        work,
        "CapabilityQualified",
        true,
        "controller://workcell-admission",
        refs,
    )?;
    evidence.valid_until = workcell_valid_until;
    Ok(evidence)
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
    if authority.request.work_generation != Some(work.generation) {
        return Err(Error::validation(
            "authority decision is not bound to this Work generation",
        ));
    }
    if authority.decision.provider.trim().is_empty() {
        return Err(Error::validation(
            "authority decision requires a non-empty provider identity",
        ));
    }
    if authority.decision.decided_at > now
        || authority
            .request
            .not_before
            .is_some_and(|start| authority.decision.decided_at < start)
        || authority
            .request
            .expires_at
            .is_some_and(|end| authority.decision.decided_at > end)
    {
        return Err(Error::validation(
            "authority decision timestamp is outside the request validity window",
        ));
    }
    if authority.request.site_ref != work.spec.site_ref {
        return Err(Error::validation(
            "authority decision site does not match Work site",
        ));
    }
    let mut refs = vec![authority.decision.id.to_string()];
    refs.extend(authority.decision.evidence_refs.clone());
    let mut evidence = ConditionEvidence::new(
        work,
        "AuthoritySatisfied",
        true,
        format!("authority://{}", authority.decision.provider),
        refs,
    )?;
    evidence.valid_until = authority.request.expires_at;
    Ok(evidence)
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
    decision: &CapabilityResolutionDecision,
    capabilities: &[CapabilityRecord],
) -> Result<ConditionEvidence> {
    if !decision.matches_work(work) {
        return Err(Error::validation(
            "ProvenanceReady requires the current Work generation/solution resolution decision",
        ));
    }
    if capabilities.is_empty() {
        return Err(Error::validation(
            "ProvenanceReady requires at least one selected capability",
        ));
    }
    let mut resolved_ids = decision
        .plan
        .members
        .iter()
        .map(|member| member.capability.manifest_id.to_string())
        .collect::<Vec<_>>();
    resolved_ids.sort();
    resolved_ids.dedup();
    let mut supplied_ids = capabilities
        .iter()
        .map(|capability| capability.manifest.id.to_string())
        .collect::<Vec<_>>();
    supplied_ids.sort();
    supplied_ids.dedup();
    if supplied_ids != resolved_ids || supplied_ids.len() != capabilities.len() {
        return Err(Error::validation(
            "ProvenanceReady capability set must exactly match the current resolution decision",
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
    refs.push(decision.id.to_string());
    if let Some(solution) = &work.spec.source_solution_ref {
        refs.push(solution.clone());
    }
    refs.sort();
    refs.dedup();
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
    use crate::condition_evidence::derive_controller_inputs;
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
        use morn_package::{supply_chain_subject_ref, SupplyChainVerificationEvidence};

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

        let qualification_expires_at = Timestamp::from_millis(200);
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
            valid_until: Some(qualification_expires_at),
            created_at: Timestamp::from_millis(1),
        };
        let digest = format!("sha256:{}", "a".repeat(64));
        let package_ref = format!("oci://fixture/morn/first@{digest}");
        let release = CapabilityDistributionRelease {
            id: CapabilityDistributionReleaseId::generate_with("release"),
            manifest_id: first.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            package_ref: package_ref.clone(),
            content_digest: digest.clone(),
            signature_ref: Some("sigstore://fixture/signature".to_string()),
            provenance_ref: Some("slsa://fixture/provenance".to_string()),
            supply_chain_verification: Some(SupplyChainVerificationEvidence {
                subject_digest: digest,
                verifier_ref: "fixture://supply-chain-verifier".to_string(),
                signature_verified: true,
                provenance_verified: true,
                evidence_refs: vec![
                    supply_chain_subject_ref(&package_ref).unwrap(),
                    "fixture://supply-chain-proof".to_string(),
                ],
            }),
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
            created_at: Timestamp::from_millis(1),
        };
        let stale_admission = SiteAdmission {
            id: SiteAdmissionId::generate_with("admit-stale"),
            status: SiteAdmissionStatus::Suspended,
            ..admission.clone()
        };
        first
            .admission_refs
            .push(morn_capability::CapabilityAdmissionRef {
                admission_ref: stale_admission.id.to_string(),
                site_ref: "plant-a".to_string(),
                profile_ref: work.spec.profile_ref.clone(),
            });
        first
            .admission_refs
            .push(morn_capability::CapabilityAdmissionRef {
                admission_ref: admission.id.to_string(),
                site_ref: "plant-a".to_string(),
                profile_ref: work.spec.profile_ref.clone(),
            });
        let mut service = AdmissionService::default();
        service.qualifications.push(qualification);
        service.releases.push(release);
        service.admissions.push(stale_admission.clone());
        service.admissions.push(admission.clone());

        let observed_at = Timestamp::from_millis(100);
        let single_plan = WorkcellPlan {
            members: vec![WorkcellMember {
                capability: ResolvedCapability {
                    manifest_id: first.manifest.id.clone(),
                    provider_ref: first.manifest.provider_ref.clone(),
                    kind: first.manifest.kind,
                    estimated_cost_micros: None,
                    maximum_effect: first.manifest.authority.maximum_effect,
                    compensation_ref: first.manifest.compensation_ref.clone(),
                    idempotency_key_required: first.manifest.idempotency_key_required,
                    required_execution_class: first.manifest.execution.minimum_isolation,
                    required_execution_guarantees: first
                        .manifest
                        .execution
                        .required_guarantees
                        .clone(),
                    score: 1,
                    rationale: vec![],
                },
                covers: vec!["x".to_string()],
            }],
            uncovered: vec![],
            total_estimated_cost_micros: 0,
            rationale: vec![],
        };
        let decision = CapabilityResolutionDecision::new(&work, single_plan).unwrap();
        assert!(workcell_qualification_evidence(
            &work,
            &decision,
            &[first.clone(), second],
            &service,
            observed_at
        )
        .is_err());
        let mut evidence =
            workcell_qualification_evidence(&work, &decision, &[first], &service, observed_at)
                .unwrap();
        assert_eq!(evidence.condition_type, "CapabilityQualified");
        assert_eq!(evidence.valid_until, Some(qualification_expires_at));
        assert!(evidence.evidence_refs.contains(&admission.id.to_string()));
        assert!(!evidence
            .evidence_refs
            .contains(&stale_admission.id.to_string()));

        evidence.observed_at = observed_at;
        assert!(
            derive_controller_inputs(&work, &[evidence.clone()], Timestamp::from_millis(150))
                .capability_qualified
        );
        assert!(
            !derive_controller_inputs(&work, &[evidence], Timestamp::from_millis(201))
                .capability_qualified
        );
    }

    #[test]
    fn qualification_cannot_use_capabilities_outside_current_resolution_decision() {
        use morn_assurance::{
            CapabilityDistributionRelease, CapabilityDistributionReleaseId,
            CapabilityDistributionReleaseStatus, QualificationEvidence, QualificationRecord,
            QualificationRecordId, QualificationStatus, SiteAdmission, SiteAdmissionId,
            SiteAdmissionStatus,
        };
        use morn_kernel::version::Version;
        use morn_package::{supply_chain_subject_ref, SupplyChainVerificationEvidence};

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);

        let resolved_manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "resolved",
            "provider-a",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        );
        let resolved_plan = WorkcellPlan {
            members: vec![WorkcellMember {
                capability: ResolvedCapability {
                    manifest_id: resolved_manifest.id,
                    provider_ref: "provider-a".to_string(),
                    kind: CapabilityKind::Program,
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
        let decision = CapabilityResolutionDecision::new(&work, resolved_plan).unwrap();

        let mut foreign = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "foreign",
            "provider-b",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));
        let qualification = QualificationRecord {
            id: QualificationRecordId::generate_with("qual"),
            manifest_id: foreign.manifest.id.clone(),
            qualification_version: Version::v1(),
            candidate_ref: "candidate://foreign".to_string(),
            decision_ref: "decision://foreign".to_string(),
            evidence_refs: vec!["test://foreign".to_string()],
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
            created_at: Timestamp::from_millis(1),
        };
        let digest = format!("sha256:{}", "a".repeat(64));
        let package_ref = format!("oci://fixture/morn/foreign@{digest}");
        let release = CapabilityDistributionRelease {
            id: CapabilityDistributionReleaseId::generate_with("release"),
            manifest_id: foreign.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            package_ref: package_ref.clone(),
            content_digest: digest.clone(),
            signature_ref: Some("sigstore://fixture/signature".to_string()),
            provenance_ref: Some("slsa://fixture/provenance".to_string()),
            supply_chain_verification: Some(SupplyChainVerificationEvidence {
                subject_digest: digest,
                verifier_ref: "fixture://supply-chain-verifier".to_string(),
                signature_verified: true,
                provenance_verified: true,
                evidence_refs: vec![supply_chain_subject_ref(&package_ref).unwrap()],
            }),
            status: CapabilityDistributionReleaseStatus::Released,
            created_at: Timestamp::from_millis(1),
        };
        let admission = SiteAdmission {
            id: SiteAdmissionId::generate_with("admit"),
            manifest_id: foreign.manifest.id.clone(),
            qualification_id: qualification.id.clone(),
            release_id: release.id.clone(),
            site_ref: "plant-a".to_string(),
            profile_ref: work.spec.profile_ref.clone(),
            conformance_ref: "conformance://foreign".to_string(),
            approved_by: "owner".to_string(),
            status: SiteAdmissionStatus::Admitted,
            created_at: Timestamp::from_millis(1),
        };
        foreign.admission_refs.push(morn_capability::CapabilityAdmissionRef {
            admission_ref: admission.id.to_string(),
            site_ref: "plant-a".to_string(),
            profile_ref: work.spec.profile_ref.clone(),
        });
        let mut service = AdmissionService::default();
        service.qualifications.push(qualification);
        service.releases.push(release);
        service.admissions.push(admission);

        let error = workcell_qualification_evidence(
            &work,
            &decision,
            &[foreign],
            &service,
            Timestamp::from_millis(10),
        )
        .unwrap_err();
        assert!(format!("{error}").contains("exactly match"));
    }

    #[test]
    fn authority_from_prior_work_generation_cannot_be_rebound_to_current_readiness() {
        use morn_runtime::{AuthorityDecisionId, AuthorityDecisionRecord, AuthorityRequest};

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "authorized review",
            "morn.lite@1.0.0",
        );
        spec.site_ref = Some("site-a".to_string());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);

        let mut request = AuthorityRequest::new("principal-a", "read", "resource-a");
        request.work_ref = Some(work.id.to_string());
        request.work_generation = Some(work.generation);
        request.site_ref = work.spec.site_ref.clone();
        let authority = BoundAuthorityDecision {
            request,
            decision: AuthorityDecisionRecord {
                id: AuthorityDecisionId::generate_with("authz"),
                provider: "fixture-authority".to_string(),
                allowed: true,
                reason: "fixture allow".to_string(),
                evidence_refs: vec!["policy://fixture".to_string()],
                decided_at: Timestamp::from_millis(100),
            },
        };

        let mut next = work.spec.clone();
        next.goal = "changed scope".to_string();
        work.replace_spec(next);

        let error =
            authority_condition_evidence(&work, &authority, Timestamp::from_millis(101))
                .unwrap_err();
        assert!(format!("{error}").contains("Work generation"));
    }

    #[test]
    fn authority_decision_must_be_decided_inside_request_window() {
        use morn_runtime::{AuthorityDecisionId, AuthorityDecisionRecord, AuthorityRequest};

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "authorized review",
            "morn.lite@1.0.0",
        );
        spec.site_ref = Some("site-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);

        let mut request = AuthorityRequest::new("principal-a", "read", "resource-a");
        request.work_ref = Some(work.id.to_string());
        request.work_generation = Some(work.generation);
        request.site_ref = work.spec.site_ref.clone();
        request.not_before = Some(Timestamp::from_millis(100));
        request.expires_at = Some(Timestamp::from_millis(200));

        for decided_at in [Timestamp::from_millis(99), Timestamp::from_millis(151)] {
            let authority = BoundAuthorityDecision {
                request: request.clone(),
                decision: AuthorityDecisionRecord {
                    id: AuthorityDecisionId::generate_with("authz"),
                    provider: "fixture-authority".to_string(),
                    allowed: true,
                    reason: "fixture allow".to_string(),
                    evidence_refs: vec!["policy://fixture".to_string()],
                    decided_at,
                },
            };
            let now = Timestamp::from_millis(150);
            let error = authority_condition_evidence(&work, &authority, now).unwrap_err();
            assert!(format!("{error}").contains("timestamp"));
        }

        let valid = BoundAuthorityDecision {
            request,
            decision: AuthorityDecisionRecord {
                id: AuthorityDecisionId::generate_with("authz"),
                provider: "fixture-authority".to_string(),
                allowed: true,
                reason: "fixture allow".to_string(),
                evidence_refs: vec!["policy://fixture".to_string()],
                decided_at: Timestamp::from_millis(120),
            },
        };
        assert!(authority_condition_evidence(&work, &valid, Timestamp::from_millis(150)).is_ok());
    }

    #[test]
    fn authority_readiness_expires_with_authority_request() {
        use morn_runtime::{AuthorityDecisionId, AuthorityDecisionRecord, AuthorityRequest};

        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "authorized review",
            "morn.lite@1.0.0",
        );
        spec.site_ref = Some("site-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let expires_at = Timestamp::from_millis(200);
        let mut request = AuthorityRequest::new("principal-a", "read", "resource-a");
        request.work_ref = Some(work.id.to_string());
        request.work_generation = Some(work.generation);
        request.site_ref = work.spec.site_ref.clone();
        request.expires_at = Some(expires_at);
        let authority = BoundAuthorityDecision {
            request,
            decision: AuthorityDecisionRecord {
                id: AuthorityDecisionId::generate_with("authz"),
                provider: "fixture-authority".to_string(),
                allowed: true,
                reason: "fixture allow".to_string(),
                evidence_refs: vec!["policy://fixture".to_string()],
                decided_at: Timestamp::from_millis(100),
            },
        };

        let mut evidence =
            authority_condition_evidence(&work, &authority, Timestamp::from_millis(100)).unwrap();
        assert_eq!(evidence.valid_until, Some(expires_at));
        evidence.observed_at = Timestamp::from_millis(100);
        assert!(
            derive_controller_inputs(&work, &[evidence.clone()], Timestamp::from_millis(150))
                .authority_satisfied
        );
        assert!(
            !derive_controller_inputs(&work, &[evidence], Timestamp::from_millis(201))
                .authority_satisfied
        );
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
    fn prior_generation_resolution_decision_cannot_be_rebound_to_current_work() {
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

        let mut next = work.spec.clone();
        next.goal = "changed generation".to_string();
        work.replace_spec(next);

        let error = capability_resolution_evidence(&work, &decision).unwrap_err();
        assert!(format!("{error}").contains("generation/solution"));
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
        let plan = WorkcellPlan {
            members: vec![WorkcellMember {
                capability: ResolvedCapability {
                    manifest_id: capability.manifest.id.clone(),
                    provider_ref: capability.manifest.provider_ref.clone(),
                    kind: capability.manifest.kind,
                    estimated_cost_micros: None,
                    maximum_effect: capability.manifest.authority.maximum_effect,
                    compensation_ref: capability.manifest.compensation_ref.clone(),
                    idempotency_key_required: capability.manifest.idempotency_key_required,
                    required_execution_class: capability.manifest.execution.minimum_isolation,
                    required_execution_guarantees: capability
                        .manifest
                        .execution
                        .required_guarantees
                        .clone(),
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
        let evidence =
            provenance_condition_evidence(&work, &decision, &[capability]).unwrap();
        assert_eq!(evidence.condition_type, "ProvenanceReady");
        assert!(evidence.evidence_refs.contains(&decision.id.to_string()));
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
        assert!(CapabilityResolutionDecision::new(&work, plan).is_err());
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
        let decision = CapabilityResolutionDecision::new(&work, plan).unwrap();
        let evidence = capability_resolution_evidence(&work, &decision).unwrap();
        assert_eq!(evidence.condition_type, "CapabilityResolved");
        assert!(evidence.evidence_refs.contains(&decision.id.to_string()));
    }
}
