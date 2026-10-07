//! Domain-profile semantic guard for external action intent.
//!
//! This is separate from policy/identity authorization: a policy provider may
//! say a principal is allowed to perform an action, while the active Domain
//! Profile can still forbid that class of real-world effect.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
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
}
