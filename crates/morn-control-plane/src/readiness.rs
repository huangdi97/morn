//! Typed producers for durable Work readiness evidence.
//!
//! This layer prevents generic callers from asserting readiness booleans. Each
//! positive ConditionEvidence must be derived from a concrete Morn record or
//! resolver result with linkage to the exact Work generation.

use morn_assurance::AdmissionService;
use morn_capability::{CapabilityRecord, WorkcellPlan};
use morn_integration::SourceOfTruthBinding;
use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use morn_runtime::{
    BoundAuthorityDecision, ExecutionBinding, ExecutionManifest,
};
use morn_work::control::WorkResource;

use crate::ConditionEvidence;

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
    let site = work
        .spec
        .site_ref
        .as_deref()
        .ok_or_else(|| Error::validation("qualified governed Work requires site_ref"))?;
    if !admissions.site_profile_admission_active_at(
        capability,
        site,
        &work.spec.profile_ref,
        now,
    ) {
        return Err(Error::invalid_state(
            "capability lacks active qualification/release/site/profile admission",
        ));
    }
    let refs = capability
        .admission_refs
        .iter()
        .filter(|reference| {
            reference.site_ref == site && reference.profile_ref == work.spec.profile_ref
        })
        .map(|reference| reference.admission_ref.clone())
        .collect::<Vec<_>>();
    ConditionEvidence::new(
        work,
        "CapabilityQualified",
        true,
        "controller://capability-admission",
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

pub fn provenance_condition_evidence(
    work: &WorkResource,
    binding: &ExecutionBinding,
    manifest: &ExecutionManifest,
) -> Result<ConditionEvidence> {
    if !manifest.validates_against(work, binding) {
        return Err(Error::validation(
            "execution manifest does not validate against Work/ExecutionBinding",
        ));
    }
    ConditionEvidence::new(
        work,
        "ProvenanceReady",
        true,
        "controller://execution-provenance",
        vec![
            manifest.execution_binding_ref.clone(),
            manifest.capability_manifest_ref.clone(),
            manifest.provider_ref.clone(),
        ],
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
