//! Domain-profile semantic guard for external action intent.
//!
//! This is separate from policy/identity authorization: a policy provider may
//! say a principal is allowed to perform an action, while the active Domain
//! Profile can still forbid that class of real-world effect.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use morn_runtime::AuthorityDecisionRecord;
use morn_profile::DomainProfile;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalActionPermit {
    pub profile_ref: String,
    pub mode: ExternalActionMode,
    pub authority_decision_ref: String,
    pub work_ref: String,
    pub binding_ref: String,
    pub issued_at: Timestamp,
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
    authority: &AuthorityDecisionRecord,
    work_ref: impl Into<String>,
    binding_ref: impl Into<String>,
) -> Result<ExternalActionPermit> {
    if !authority.allowed {
        return Err(Error::not_authorized(format!(
            "authority provider {} denied external action: {}",
            authority.provider, authority.reason
        )));
    }

    let profile_decision = evaluate_profile_action(profile, mode);
    enforce_profile_action(&profile_decision)?;

    let work_ref = work_ref.into();
    let binding_ref = binding_ref.into();
    if work_ref.trim().is_empty() || binding_ref.trim().is_empty() {
        return Err(Error::validation(
            "external action permit requires Work and ExecutionBinding references",
        ));
    }

    Ok(ExternalActionPermit {
        profile_ref: profile_decision.profile_ref,
        mode,
        authority_decision_ref: authority.id.to_string(),
        work_ref,
        binding_ref,
        issued_at: Timestamp::now(),
    })
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
    use morn_runtime::AuthorityDecisionId;

    fn allowed_authority() -> AuthorityDecisionRecord {
        AuthorityDecisionRecord {
            id: AuthorityDecisionId::generate_with("authz"),
            provider: "fixture-authority".to_string(),
            allowed: true,
            reason: "allowed".to_string(),
            evidence_refs: vec!["policy://fixture".to_string()],
            decided_at: Timestamp::now(),
        }
    }

    #[test]
    fn factory_readonly_rejects_production_write_even_if_other_policy_would_allow() {
        let profile = DomainProfile::factory_readonly_v1();
        let decision = evaluate_profile_action(&profile, ExternalActionMode::ProductionWrite);
        assert!(!decision.allowed);
        assert!(enforce_profile_action(&decision).is_err());
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
    fn external_action_permit_requires_both_authority_and_profile() {
        let profile = DomainProfile::factory_readonly_v1();
        let authority = allowed_authority();

        assert!(issue_external_action_permit(
            &profile,
            ExternalActionMode::ProductionWrite,
            &authority,
            "work-1",
            "binding-1",
        )
        .is_err());

        let permit = issue_external_action_permit(
            &profile,
            ExternalActionMode::SandboxWrite,
            &authority,
            "work-1",
            "binding-1",
        )
        .unwrap();
        assert_eq!(permit.work_ref, "work-1");
        assert_eq!(permit.binding_ref, "binding-1");
        assert_eq!(permit.authority_decision_ref, authority.id.to_string());
    }
}
