//! Runtime validity guard for an already-created ExecutionBinding.
//!
//! Capability admission/provider health are re-checked before a *new* external
//! effect. Revocation never erases the old binding/attempt, and it must not
//! prevent reconciliation of a side effect that may already have happened.

use serde::{Deserialize, Serialize};

use morn_assurance::AdmissionService;
use morn_capability::CapabilityRecord;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_runtime::{ExecutionBinding, ProviderRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BindingValidityDecisionTag;
pub type BindingValidityDecisionId = Id<BindingValidityDecisionTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingValidityDecision {
    pub id: BindingValidityDecisionId,
    pub binding_ref: String,
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub site_ref: Option<String>,
    pub profile_ref: String,
    pub admission_active: bool,
    pub provider_selectable: bool,
    pub provider_identity_matches: bool,
    pub allow_new_effect: bool,
    pub allow_reconciliation: bool,
    pub reasons: Vec<String>,
    pub checked_at: Timestamp,
}

impl BindingValidityDecision {
    pub fn enforce_new_effect(&self) -> Result<()> {
        if self.allow_new_effect {
            Ok(())
        } else {
            Err(Error::invalid_state(format!(
                "binding {} is not valid for a new external effect: {}",
                self.binding_ref,
                self.reasons.join("; ")
            )))
        }
    }

    pub fn enforce_reconciliation(&self) -> Result<()> {
        if self.allow_reconciliation {
            Ok(())
        } else {
            Err(Error::invalid_state(format!(
                "binding {} cannot be reconciled: {}",
                self.binding_ref,
                self.reasons.join("; ")
            )))
        }
    }
}

#[derive(Debug, Default)]
pub struct BindingOperationalGate;

impl BindingOperationalGate {
    /// Strict Enterprise/Factory check. A site-scoped binding must still have an
    /// active exact site/profile admission and an exact selectable provider
    /// identity before a *new* effect can be issued.
    pub fn evaluate(
        &self,
        binding: &ExecutionBinding,
        capability: &CapabilityRecord,
        admissions: &AdmissionService,
        providers: &ProviderRegistry,
        now: Timestamp,
    ) -> BindingValidityDecision {
        let mut reasons = Vec::new();

        let capability_matches =
            binding.capability_manifest_ref == capability.manifest.id.to_string();
        if !capability_matches {
            reasons.push("binding capability manifest no longer matches candidate".to_string());
        }

        let admission_active = match binding.site_ref.as_deref() {
            Some(site) if capability_matches => admissions.site_profile_admission_active_at(
                capability,
                site,
                &binding.profile_ref,
                now,
            ),
            Some(_) => false,
            None => {
                reasons.push(
                    "strict governed binding requires an explicit site for admission re-check"
                        .to_string(),
                );
                false
            }
        };
        if !admission_active {
            reasons.push(
                "exact site/profile qualification-release-admission is not active".to_string(),
            );
        }

        let provider = providers.get(&binding.provider_ref);
        let provider_selectable = provider.is_some_and(|provider| provider.selectable_at(now));
        if !provider_selectable {
            reasons.push("bound provider is unavailable, stale or unregistered".to_string());
        }

        let provider_identity_matches = provider.is_some_and(|provider| {
            provider.version == binding.provider_version
                && binding
                    .provider_digest
                    .as_ref()
                    .is_none_or(|digest| provider.digest.as_ref() == Some(digest))
        });
        if !provider_identity_matches {
            reasons.push(
                "current provider projection does not match pinned version/digest".to_string(),
            );
        }

        // Reconciliation is consequence accounting for an already-started
        // attempt. It remains allowed even after capability/provider
        // revocation; otherwise revocation could strand an unknown real-world
        // effect forever. The reconciler itself may use a separate eligible
        // source-of-truth provider.
        let allow_reconciliation = capability_matches;
        let allow_new_effect = capability_matches
            && admission_active
            && provider_selectable
            && provider_identity_matches;

        BindingValidityDecision {
            id: BindingValidityDecisionId::generate_with("binding-validity"),
            binding_ref: binding.id.to_string(),
            capability_manifest_ref: binding.capability_manifest_ref.clone(),
            provider_ref: binding.provider_ref.clone(),
            site_ref: binding.site_ref.clone(),
            profile_ref: binding.profile_ref.clone(),
            admission_active,
            provider_selectable,
            provider_identity_matches,
            allow_new_effect,
            allow_reconciliation,
            reasons,
            checked_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::{CapabilityKind, CapabilityManifest, EffectClass};
    use morn_kernel::ids::{CapabilityId, WorkPackageId, WorkspaceId};
    use morn_runtime::{ProviderDescriptor, ProviderFamily, ProviderStatus};
    use morn_work::control::{WorkResource, WorkSpec};

    fn fixture() -> (WorkResource, CapabilityRecord, ExecutionBinding) {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "governed work",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let capability = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "fixture",
            "provider-a",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));
        let binding = ExecutionBinding::for_work(
            &work,
            capability.manifest.id.to_string(),
            "provider-a",
            "1",
        );
        (work, capability, binding)
    }

    #[test]
    fn missing_admission_blocks_new_effect_but_not_consequence_accounting() {
        let (_work, capability, binding) = fixture();
        let admissions = AdmissionService::default();
        let mut providers = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("provider-a", ProviderFamily::Runtime, "1");
        provider.status = ProviderStatus::Healthy;
        providers.register(provider).unwrap();

        let decision = BindingOperationalGate.evaluate(
            &binding,
            &capability,
            &admissions,
            &providers,
            Timestamp::now(),
        );
        assert!(!decision.allow_new_effect);
        assert!(decision.allow_reconciliation);
        assert!(decision.enforce_new_effect().is_err());
        decision.enforce_reconciliation().unwrap();
    }

    #[test]
    fn provider_upgrade_requires_explicit_rebind_before_new_effect() {
        let (_work, capability, binding) = fixture();
        let admissions = AdmissionService::default();
        let mut providers = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("provider-a", ProviderFamily::Runtime, "2");
        provider.status = ProviderStatus::Healthy;
        providers.register(provider).unwrap();

        let decision = BindingOperationalGate.evaluate(
            &binding,
            &capability,
            &admissions,
            &providers,
            Timestamp::now(),
        );
        assert!(!decision.provider_identity_matches);
        assert!(!decision.allow_new_effect);
        assert!(decision.allow_reconciliation);
    }

    #[test]
    fn wrong_capability_blocks_both_new_effect_and_reconciliation() {
        let (_work, _capability, binding) = fixture();
        let other = CapabilityRecord::new(CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "other",
            "provider-a",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        ));
        let decision = BindingOperationalGate.evaluate(
            &binding,
            &other,
            &AdmissionService::default(),
            &ProviderRegistry::default(),
            Timestamp::now(),
        );
        assert!(!decision.allow_new_effect);
        assert!(!decision.allow_reconciliation);
    }
}
