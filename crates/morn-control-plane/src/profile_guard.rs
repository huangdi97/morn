//! Domain-profile semantic guard for external action intent.
//!
//! This is separate from policy/identity authorization: a policy provider may
//! say a principal is allowed to perform an action, while the active Domain
//! Profile can still forbid that class of real-world effect.

use serde::{Deserialize, Serialize};

use morn_capability::{EffectClass, EffectContract};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_profile::DomainProfile;
use morn_runtime::{ActionAttempt, ActionAttemptId, BoundAuthorityDecision, ExecutionBinding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalActionMode {
    Read,
    CandidateOnly,
    SandboxWrite,
    ShadowWrite,
    ProductionWrite,
    PhysicalControl,
}

impl ExternalActionMode {
    pub const fn semantic(self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::CandidateOnly => "CandidateOnly",
            Self::SandboxWrite => "SandboxWrite",
            Self::ShadowWrite => "ShadowWrite",
            Self::ProductionWrite => "ProductionWrite",
            Self::PhysicalControl => "PhysicalControl",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileActionDecision {
    pub profile_ref: String,
    pub mode: ExternalActionMode,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExternalActionPermitTag;
pub type ExternalActionPermitId = Id<ExternalActionPermitTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalActionPermit {
    pub id: ExternalActionPermitId,
    pub profile_ref: String,
    pub mode: ExternalActionMode,
    pub authority_decision_ref: String,
    pub principal: String,
    pub acting_for: Option<String>,
    pub action: String,
    pub resource: String,
    pub site_ref: Option<String>,
    pub scope: Vec<String>,
    pub parameter_envelope: std::collections::BTreeMap<String, String>,
    pub work_ref: String,
    #[serde(default)]
    pub work_generation: u64,
    pub binding_ref: String,
    pub not_before: Option<Timestamp>,
    pub expires_at: Option<Timestamp>,
    pub issued_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsumedExternalActionPermit {
    pub permit_id: ExternalActionPermitId,
    pub work_ref: String,
    pub work_generation: u64,
    pub binding_ref: String,
    pub attempt_id: ActionAttemptId,
    pub business_key: String,
    pub action: String,
    pub resource: String,
    pub mode: ExternalActionMode,
    pub effect_contract: EffectContract,
    pub consumed_at: Timestamp,
}

impl ConsumedExternalActionPermit {
    pub fn from_write_attempt(
        permit: &ExternalActionPermit,
        attempt: &ActionAttempt,
        now: Timestamp,
    ) -> Result<Self> {
        let effect = attempt
            .effect_contract
            .clone()
            .ok_or_else(|| Error::validation("write-like permit consumption requires an effect"))?;
        if !matches!(
            permit.mode,
            ExternalActionMode::SandboxWrite
                | ExternalActionMode::ShadowWrite
                | ExternalActionMode::ProductionWrite
                | ExternalActionMode::PhysicalControl
        ) || effect.class == EffectClass::E0LifecycleReversible
        {
            return Err(Error::validation(
                "only a write-like permit with a non-E0 effect can be consumed as an external action",
            ));
        }
        if attempt.external_action_permit_ref.as_deref() != Some(permit.id.as_str())
            || attempt.binding_id.as_str() != permit.binding_ref
            || attempt.action != permit.action
            || attempt.resource_ref.as_deref() != Some(permit.resource.as_str())
            || attempt.site_ref != permit.site_ref
            || attempt.authority_decision_ref.as_deref()
                != Some(permit.authority_decision_ref.as_str())
            || attempt.business_key.trim().is_empty()
        {
            return Err(Error::validation(
                "external-action permit consumption must exactly match its action attempt",
            ));
        }
        if now < permit.issued_at
            || permit.not_before.is_some_and(|start| now < start)
            || permit.expires_at.is_some_and(|end| now > end)
            || attempt.created_at < permit.issued_at
            || attempt.created_at > now
        {
            return Err(Error::not_authorized(
                "external-action permit cannot be consumed outside its issuance/validity window",
            ));
        }
        Ok(Self {
            permit_id: permit.id.clone(),
            work_ref: permit.work_ref.clone(),
            work_generation: permit.work_generation,
            binding_ref: permit.binding_ref.clone(),
            attempt_id: attempt.id.clone(),
            business_key: attempt.business_key.clone(),
            action: attempt.action.clone(),
            resource: permit.resource.clone(),
            mode: permit.mode,
            effect_contract: effect,
            consumed_at: now,
        })
    }

    pub fn exactly_matches(&self, permit: &ExternalActionPermit, attempt: &ActionAttempt) -> bool {
        self.permit_id == permit.id
            && self.work_ref == permit.work_ref
            && self.work_generation == permit.work_generation
            && self.binding_ref == permit.binding_ref
            && self.attempt_id == attempt.id
            && self.business_key == attempt.business_key
            && self.action == attempt.action
            && self.resource == permit.resource
            && self.mode == permit.mode
            && attempt.effect_contract.as_ref() == Some(&self.effect_contract)
            && attempt.external_action_permit_ref.as_deref() == Some(permit.id.as_str())
    }
}

pub fn evaluate_profile_action(
    profile: &DomainProfile,
    mode: ExternalActionMode,
) -> ProfileActionDecision {
    let semantic = mode.semantic();
    if profile.forbids(semantic) {
        return ProfileActionDecision {
            profile_ref: profile.canonical_ref(),
            mode,
            allowed: false,
            reason: format!("profile explicitly forbids {semantic}"),
        };
    }

    // Consequential production/physical effects are fail-closed. Merely
    // omitting a Forbidden marker is not an opt-in.
    if mode == ExternalActionMode::ProductionWrite && !profile.requires("ProductionWrite") {
        return ProfileActionDecision {
            profile_ref: profile.canonical_ref(),
            mode,
            allowed: false,
            reason: "production write requires an explicit profile guarantee".to_string(),
        };
    }

    // Physical control is fail-closed unless a profile opts in explicitly with
    // a required semantic. The v11.5 profiles intentionally do not.
    if mode == ExternalActionMode::PhysicalControl && !profile.requires("PhysicalControl") {
        return ProfileActionDecision {
            profile_ref: profile.canonical_ref(),
            mode,
            allowed: false,
            reason: "physical control requires an explicit profile guarantee".to_string(),
        };
    }

    ProfileActionDecision {
        profile_ref: profile.canonical_ref(),
        mode,
        allowed: true,
        reason: format!("{semantic} is not forbidden by profile"),
    }
}

pub fn issue_external_action_permit(
    profile: &DomainProfile,
    mode: ExternalActionMode,
    authority: &BoundAuthorityDecision,
    binding: &ExecutionBinding,
) -> Result<ExternalActionPermit> {
    if !authority.decision.allowed {
        return Err(Error::not_authorized(format!(
            "authority provider {} denied external action: {}",
            authority.decision.provider, authority.decision.reason
        )));
    }
    if !authority.request.valid_now(Timestamp::now()) {
        return Err(Error::not_authorized(
            "authority request is revoked or outside its validity window",
        ));
    }

    let write_like = matches!(
        mode,
        ExternalActionMode::SandboxWrite
            | ExternalActionMode::ShadowWrite
            | ExternalActionMode::ProductionWrite
            | ExternalActionMode::PhysicalControl
    );
    if write_like && !binding.autonomy_posture.permits_write_like() {
        return Err(Error::not_authorized(
            "Assist autonomy posture cannot issue write-like external-action permits",
        ));
    }

    let profile_decision = evaluate_profile_action(profile, mode);
    enforce_profile_action(&profile_decision)?;

    if profile_decision.profile_ref != binding.profile_ref {
        return Err(Error::validation(
            "external action profile does not match pinned ExecutionBinding profile",
        ));
    }
    if authority.request.work_ref.as_deref() != Some(binding.work_id.as_str()) {
        return Err(Error::validation(
            "authority request must be bound to the exact Work in ExecutionBinding",
        ));
    }
    if authority.request.work_generation != Some(binding.work_generation) {
        return Err(Error::validation(
            "authority request must be bound to the exact Work generation in ExecutionBinding",
        ));
    }
    if authority.request.site_ref != binding.site_ref {
        return Err(Error::validation(
            "authority request site does not match pinned ExecutionBinding site",
        ));
    }
    if authority.request.action.trim().is_empty() || authority.request.resource.trim().is_empty() {
        return Err(Error::validation(
            "external action permit requires an exact action and resource",
        ));
    }

    Ok(ExternalActionPermit {
        id: ExternalActionPermitId::generate_with("permit"),
        profile_ref: profile_decision.profile_ref,
        mode,
        authority_decision_ref: authority.decision.id.to_string(),
        principal: authority.request.principal.clone(),
        acting_for: authority.request.acting_for.clone(),
        action: authority.request.action.clone(),
        resource: authority.request.resource.clone(),
        site_ref: authority.request.site_ref.clone(),
        scope: authority.request.scope.clone(),
        parameter_envelope: authority.request.parameter_envelope.clone(),
        work_ref: binding.work_id.to_string(),
        work_generation: binding.work_generation,
        binding_ref: binding.id.to_string(),
        not_before: authority.request.not_before,
        expires_at: authority.request.expires_at,
        issued_at: Timestamp::now(),
    })
}

/// Preferred v11.5 permitting path. Consequential actions register a
/// per-permit Work finalizer before the caller can dispatch the attempt.
pub fn issue_external_action_permit_for_work(
    work: &mut morn_work::control::WorkResource,
    profile: &DomainProfile,
    mode: ExternalActionMode,
    authority: &BoundAuthorityDecision,
    binding: &ExecutionBinding,
) -> Result<(ExternalActionPermit, Option<String>)> {
    if !binding.matches_work_generation(work) {
        return Err(Error::conflict(
            "cannot issue external action under a stale Work generation binding",
        ));
    }
    if work.termination_requested_at.is_some() {
        return Err(Error::invalid_state(
            "cannot issue a new external action after Work termination is requested",
        ));
    }

    let permit = issue_external_action_permit(profile, mode, authority, binding)?;
    let consequential = matches!(
        mode,
        ExternalActionMode::SandboxWrite
            | ExternalActionMode::ShadowWrite
            | ExternalActionMode::ProductionWrite
            | ExternalActionMode::PhysicalControl
    );
    let finalizer = if consequential {
        let key = format!("morn.io/external-action-{}", permit.id);
        work.add_finalizer(key.clone())?;
        Some(key)
    } else {
        None
    };
    Ok((permit, finalizer))
}

/// Clear a permit finalizer only after the corresponding attempt has reached a
/// terminal consequence state. Unknown/reconciling/committed-but-unverified
/// effects keep the Work in Terminating when cancellation has been requested.
pub fn resolve_external_action_finalizer(
    work: &mut morn_work::control::WorkResource,
    permit: &ExternalActionPermit,
    attempt: &morn_runtime::ActionAttempt,
) -> Result<bool> {
    use morn_runtime::AttemptState;

    if permit.work_ref != work.id.to_string()
        || attempt.external_action_permit_ref.as_deref() != Some(permit.id.as_str())
    {
        return Err(Error::validation(
            "finalizer resolution must match the exact Work and external-action permit",
        ));
    }
    if !matches!(
        attempt.state,
        AttemptState::Verified | AttemptState::Failed | AttemptState::Cancelled
    ) {
        return Err(Error::invalid_state(format!(
            "cannot resolve external-action finalizer while attempt is {:?}",
            attempt.state
        )));
    }

    let key = format!("morn.io/external-action-{}", permit.id);
    Ok(work.remove_finalizer(&key))
}

pub fn enforce_profile_action(decision: &ProfileActionDecision) -> Result<()> {
    if decision.allowed {
        Ok(())
    } else {
        Err(Error::not_authorized(format!(
            "profile {} denied {:?}: {}",
            decision.profile_ref, decision.mode, decision.reason
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_kernel::policy::{Policy, PolicyRule};
    use morn_runtime::{decide_bound, AuthorityRequest, NativePolicyAuthority};
    use morn_work::control::{WorkResource, WorkSpec};

    fn bound_authority(
        work: &WorkResource,
        binding: &ExecutionBinding,
        action: &str,
        resource: &str,
    ) -> BoundAuthorityDecision {
        let policy = Policy::new(
            work.workspace_id.clone(),
            "fixture",
            vec![PolicyRule::allow(action)],
        );
        let provider = NativePolicyAuthority::new(policy);
        let mut request = AuthorityRequest::new("controller", action, resource);
        request.work_ref = Some(work.id.to_string());
        request.work_generation = Some(work.generation);
        request.site_ref = binding.site_ref.clone();
        decide_bound(&provider, &request).unwrap()
    }

    #[test]
    fn authority_from_previous_work_generation_cannot_issue_new_permit() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "generation one",
            profile.canonical_ref(),
        );
        spec.site_ref = Some("plant-a".to_string());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let first_binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let authority =
            bound_authority(&work, &first_binding, "cmms.sandbox.write", "CMMS-fixture");

        let mut next = work.spec.clone();
        next.goal = "generation two".to_string();
        work.replace_spec(next);
        let next_binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");

        assert!(issue_external_action_permit(
            &profile,
            ExternalActionMode::SandboxWrite,
            &authority,
            &next_binding,
        )
        .is_err());
    }

    #[test]
    fn factory_readonly_rejects_production_write_even_if_other_policy_would_allow() {
        let profile = DomainProfile::factory_readonly_v1();
        let decision = evaluate_profile_action(&profile, ExternalActionMode::ProductionWrite);
        assert!(!decision.allowed);
        assert!(enforce_profile_action(&decision).is_err());
    }

    #[test]
    fn enterprise_profile_does_not_implicitly_enable_production_write() {
        let profile = DomainProfile::enterprise_v1();
        let decision = evaluate_profile_action(&profile, ExternalActionMode::ProductionWrite);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("explicit profile guarantee"));
    }

    #[test]
    fn assist_posture_cannot_smuggle_a_sandbox_write() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "assist-only fixture",
            profile.canonical_ref(),
        );
        spec.site_ref = Some("plant-a".to_string());
        spec.autonomy_posture = morn_work::control::AutonomyPosture::Assist;
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let authority = bound_authority(&work, &binding, "cmms.sandbox.write", "CMMS-fixture");

        assert!(issue_external_action_permit(
            &profile,
            ExternalActionMode::SandboxWrite,
            &authority,
            &binding,
        )
        .is_err());
    }

    #[test]
    fn factory_fixture_can_exercise_sandbox_write_without_claiming_production_write() {
        let profile = DomainProfile::factory_readonly_v1();
        let decision = evaluate_profile_action(&profile, ExternalActionMode::SandboxWrite);
        assert!(decision.allowed);
        enforce_profile_action(&decision).unwrap();
    }

    #[test]
    fn physical_control_fails_closed_without_explicit_profile_opt_in() {
        let profile = DomainProfile::enterprise_v1();
        let decision = evaluate_profile_action(&profile, ExternalActionMode::PhysicalControl);
        assert!(!decision.allowed);
    }

    #[test]
    fn consequential_action_finalizer_survives_unknown_outcome() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "fixture",
            profile.canonical_ref(),
        );
        spec.site_ref = Some("plant-a".to_string());
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        binding.effect_ceiling = Some(morn_capability::EffectClass::E2Compensatable);
        binding.compensation_ref = Some("cmms.cancel".to_string());
        let authority = bound_authority(&work, &binding, "cmms.sandbox.write", "CMMS-fixture");

        let (permit, finalizer) = issue_external_action_permit_for_work(
            &mut work,
            &profile,
            ExternalActionMode::SandboxWrite,
            &authority,
            &binding,
        )
        .unwrap();
        let finalizer = finalizer.unwrap();
        assert!(work.finalizers.contains(&finalizer));

        let mut attempt = crate::begin_external_attempt_for_work_with_effect(
            &work,
            &permit,
            &binding,
            morn_capability::effect::EffectContract::e2("cmms.cancel"),
            "work-key",
            "cmms.sandbox.write",
        )
        .unwrap();
        attempt
            .transition(morn_runtime::AttemptState::Dispatched)
            .unwrap();
        attempt
            .mark_outcome_unknown("timeout after commit")
            .unwrap();

        work.request_termination("operator cancellation").unwrap();
        assert!(resolve_external_action_finalizer(&mut work, &permit, &attempt).is_err());
        assert!(!work.can_finalize_termination());

        attempt
            .transition(morn_runtime::AttemptState::Reconciling)
            .unwrap();
        attempt
            .transition(morn_runtime::AttemptState::Observed)
            .unwrap();
        attempt
            .transition(morn_runtime::AttemptState::Verified)
            .unwrap();
        assert!(resolve_external_action_finalizer(&mut work, &permit, &attempt).unwrap());
        assert!(work.can_finalize_termination());
        work.finalize_termination().unwrap();
        assert_eq!(work.status.phase, morn_work::control::WorkPhase::Cancelled);
    }

    #[test]
    fn permit_rejects_authority_bound_to_other_site_or_work() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "fixture",
            profile.canonical_ref(),
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");

        let policy = Policy::new(
            work.workspace_id.clone(),
            "fixture",
            vec![PolicyRule::allow("cmms.sandbox.write")],
        );
        let provider = NativePolicyAuthority::new(policy);
        let mut wrong = AuthorityRequest::new("controller", "cmms.sandbox.write", "CMMS-fixture");
        wrong.work_ref = Some("other-work".to_string());
        wrong.site_ref = Some("plant-b".to_string());
        let wrong = decide_bound(&provider, &wrong).unwrap();

        assert!(issue_external_action_permit(
            &profile,
            ExternalActionMode::SandboxWrite,
            &wrong,
            &binding,
        )
        .is_err());
    }

    #[test]
    fn external_action_permit_requires_both_authority_and_profile() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "fixture",
            profile.canonical_ref(),
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let authority = bound_authority(&work, &binding, "cmms.sandbox.write", "CMMS-fixture");

        assert!(issue_external_action_permit(
            &profile,
            ExternalActionMode::ProductionWrite,
            &authority,
            &binding,
        )
        .is_err());

        let permit = issue_external_action_permit(
            &profile,
            ExternalActionMode::SandboxWrite,
            &authority,
            &binding,
        )
        .unwrap();
        assert_eq!(permit.work_ref, work.id.to_string());
        assert_eq!(permit.binding_ref, binding.id.to_string());
        assert_eq!(
            permit.authority_decision_ref,
            authority.decision.id.to_string()
        );
        assert_eq!(permit.action, "cmms.sandbox.write");
        assert_eq!(permit.resource, "CMMS-fixture");
    }
}
