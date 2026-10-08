//! Versioned DomainProfile registry.
//!
//! Profiles are published guarantee contracts, not hard-coded provider lists.
//! The registry allows multiple explicit profile versions while preventing a
//! same-version payload from being silently overwritten.

use std::collections::BTreeMap;

use morn_kernel::error::{Error, Result};

use crate::DomainProfile;

#[derive(Debug, Clone, Default)]
pub struct ProfileRegistry {
    profiles: BTreeMap<String, DomainProfile>,
}

impl ProfileRegistry {
    pub fn register(&mut self, profile: DomainProfile) -> Result<()> {
        profile.validate()?;
        let key = profile.canonical_ref();
        if self.profiles.contains_key(&key) {
            return Err(Error::conflict(format!(
                "DomainProfile {key} is already published; publish a new version instead"
            )));
        }
        self.profiles.insert(key, profile);
        Ok(())
    }

    pub fn get(&self, profile_ref: &str) -> Option<&DomainProfile> {
        self.profiles.get(profile_ref)
    }

    pub fn list(&self) -> Vec<&DomainProfile> {
        self.profiles.values().collect()
    }

    pub fn family(&self, profile_id: &str) -> Vec<&DomainProfile> {
        self.profiles
            .values()
            .filter(|profile| profile.id == profile_id)
            .collect()
    }
}

pub fn reference_profile_registry() -> ProfileRegistry {
    let mut registry = ProfileRegistry::default();
    for profile in [
        DomainProfile::lite_v1(),
        DomainProfile::enterprise_v1(),
        DomainProfile::factory_readonly_v1(),
        DomainProfile::research_v1(),
    ] {
        registry
            .register(profile)
            .expect("built-in reference profile must be valid");
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::version::Version;

    #[test]
    fn registry_can_hold_explicit_profile_versions_without_mutating_old_release() {
        let mut registry = ProfileRegistry::default();
        let v1 = DomainProfile::factory_readonly_v1();
        let mut v11 = v1.clone();
        v11.version = Version::new(1, 1, 0);
        v11.requirements
            .push(crate::GuaranteeRequirement::required("RuntimeAttestation"));

        registry.register(v1.clone()).unwrap();
        registry.register(v11.clone()).unwrap();

        assert_eq!(registry.family("morn.factory.readonly").len(), 2);
        assert_eq!(
            registry.get("morn.factory.readonly@1.0.0"),
            Some(&v1)
        );
        assert_eq!(
            registry.get("morn.factory.readonly@1.1.0"),
            Some(&v11)
        );
    }

    #[test]
    fn same_published_ref_cannot_be_overwritten() {
        let mut registry = ProfileRegistry::default();
        let profile = DomainProfile::factory_readonly_v1();
        registry.register(profile.clone()).unwrap();
        assert!(registry.register(profile).is_err());
    }

    #[test]
    fn reference_registry_contains_all_reference_profiles() {
        let registry = reference_profile_registry();
        assert!(registry.get("morn.lite@1.0.0").is_some());
        assert!(registry.get("morn.enterprise@1.0.0").is_some());
        assert!(registry.get("morn.factory.readonly@1.0.0").is_some());
        assert!(registry.get("morn.research@1.0.0").is_some());
    }
}
