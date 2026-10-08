//! Versioned DomainProfile compatibility and explicit migration planning.
//!
//! Profiles are guarantee contracts. They may evolve, but a running Work must
//! never be silently reinterpreted under a different profile release.

use serde::{Deserialize, Serialize};


use crate::DomainProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ProfileCompatibility {
    Compatible,
    RequiresReevaluation,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileMigrationPlan {
    pub from_ref: String,
    pub to_ref: String,
    pub compatibility: ProfileCompatibility,
    pub reasons: Vec<String>,
    pub requires_new_work_generation: bool,
    pub requires_new_execution_bindings: bool,
    pub requires_site_readmission: bool,
}

fn same_guarantee_contract(left: &DomainProfile, right: &DomainProfile) -> bool {
    left.requirements == right.requirements
        && left.minimum_isolation == right.minimum_isolation
        && left.required_execution_guarantees == right.required_execution_guarantees
        && left.source_of_truth_binding_required == right.source_of_truth_binding_required
        && left.durable_work_state_required == right.durable_work_state_required
        && left.provenance_required == right.provenance_required
}

pub fn compare_profiles(
    base: &DomainProfile,
    candidate: &DomainProfile,
) -> ProfileCompatibility {
    if base.id != candidate.id {
        return ProfileCompatibility::Incompatible;
    }
    if candidate.version < base.version {
        return ProfileCompatibility::Incompatible;
    }

    // Same published version with different content is a silent mutation.
    if base.version == candidate.version {
        return if same_guarantee_contract(base, candidate) {
            ProfileCompatibility::Compatible
        } else {
            ProfileCompatibility::Incompatible
        };
    }

    if base.version.major != candidate.version.major {
        return ProfileCompatibility::Incompatible;
    }

    // Patch releases are not allowed to change guarantee semantics.
    if base.version.minor == candidate.version.minor
        && base.version.patch != candidate.version.patch
    {
        return if same_guarantee_contract(base, candidate) {
            ProfileCompatibility::Compatible
        } else {
            ProfileCompatibility::Incompatible
        };
    }

    // A minor release may legitimately strengthen/reshape guarantees, but a
    // Work/site admission created under the previous profile must be
    // reevaluated explicitly.
    ProfileCompatibility::RequiresReevaluation
}

pub fn plan_profile_migration(
    base: &DomainProfile,
    candidate: &DomainProfile,
) -> ProfileMigrationPlan {
    let compatibility = compare_profiles(base, candidate);
    let mut reasons = Vec::new();

    if base.id != candidate.id {
        reasons.push("profile identity changed".to_string());
    }
    if base.version.major != candidate.version.major {
        reasons.push("profile major version changed".to_string());
    }
    if base.requirements != candidate.requirements {
        reasons.push("semantic guarantee requirements changed".to_string());
    }
    if base.minimum_isolation != candidate.minimum_isolation {
        reasons.push("minimum execution class changed".to_string());
    }
    if base.required_execution_guarantees != candidate.required_execution_guarantees {
        reasons.push("execution guarantee vector changed".to_string());
    }
    if base.source_of_truth_binding_required != candidate.source_of_truth_binding_required {
        reasons.push("source-of-truth requirement changed".to_string());
    }
    if base.durable_work_state_required != candidate.durable_work_state_required {
        reasons.push("durable Work requirement changed".to_string());
    }
    if base.provenance_required != candidate.provenance_required {
        reasons.push("provenance requirement changed".to_string());
    }
    if reasons.is_empty() && base.version != candidate.version {
        reasons.push("version changed with equivalent guarantee contract".to_string());
    }

    ProfileMigrationPlan {
        from_ref: base.canonical_ref(),
        to_ref: candidate.canonical_ref(),
        compatibility,
        reasons,
        requires_new_work_generation: base.version != candidate.version,
        requires_new_execution_bindings: base.version != candidate.version,
        requires_site_readmission: matches!(
            compatibility,
            ProfileCompatibility::RequiresReevaluation | ProfileCompatibility::Incompatible
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GuaranteeRequirement;
    use morn_kernel::version::Version;
    use morn_kernel::ExecutionGuarantee;

    #[test]
    fn identical_published_profile_is_compatible() {
        let base = DomainProfile::factory_readonly_v1();
        assert_eq!(
            compare_profiles(&base, &base),
            ProfileCompatibility::Compatible
        );
    }

    #[test]
    fn same_version_with_changed_semantics_is_incompatible_silent_mutation() {
        let base = DomainProfile::factory_readonly_v1();
        let mut mutated = base.clone();
        mutated
            .requirements
            .push(GuaranteeRequirement::required("NewSafetyGuarantee"));
        assert_eq!(
            compare_profiles(&base, &mutated),
            ProfileCompatibility::Incompatible
        );
    }

    #[test]
    fn patch_must_not_change_guarantee_contract() {
        let base = DomainProfile::factory_readonly_v1();
        let mut patch = base.clone();
        patch.version = Version::new(1, 0, 1);
        assert_eq!(
            compare_profiles(&base, &patch),
            ProfileCompatibility::Compatible
        );

        patch
            .required_execution_guarantees
            .push(ExecutionGuarantee::RuntimeAttestation);
        assert_eq!(
            compare_profiles(&base, &patch),
            ProfileCompatibility::Incompatible
        );
    }

    #[test]
    fn minor_change_requires_explicit_reevaluation_and_readmission() {
        let base = DomainProfile::factory_readonly_v1();
        let mut next = base.clone();
        next.version = Version::new(1, 1, 0);
        next.required_execution_guarantees
            .push(ExecutionGuarantee::RuntimeAttestation);

        let plan = plan_profile_migration(&base, &next);
        assert_eq!(
            plan.compatibility,
            ProfileCompatibility::RequiresReevaluation
        );
        assert!(plan.requires_new_work_generation);
        assert!(plan.requires_new_execution_bindings);
        assert!(plan.requires_site_readmission);
    }

    #[test]
    fn downgrade_is_incompatible_by_default() {
        let mut base = DomainProfile::factory_readonly_v1();
        base.version = Version::new(1, 1, 0);
        let older = DomainProfile::factory_readonly_v1();
        assert_eq!(
            compare_profiles(&base, &older),
            ProfileCompatibility::Incompatible
        );
    }

    #[test]
    fn cross_profile_or_major_change_is_incompatible() {
        let factory = DomainProfile::factory_readonly_v1();
        let enterprise = DomainProfile::enterprise_v1();
        assert_eq!(
            compare_profiles(&factory, &enterprise),
            ProfileCompatibility::Incompatible
        );

        let mut major = factory.clone();
        major.version = Version::new(2, 0, 0);
        assert_eq!(
            compare_profiles(&factory, &major),
            ProfileCompatibility::Incompatible
        );
    }
}
