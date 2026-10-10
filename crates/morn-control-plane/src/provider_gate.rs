//! Provider-health gate between semantic capability resolution and runtime selection.
//!
//! CapabilityResolver deliberately stays provider-runtime agnostic. The control
//! plane adds this gate before resolution when a Profile requires live provider
//! evidence. A provider being registered in a manifest is not enough: a strict
//! composition requires a current selectable ProviderRegistry projection.

use serde::{Deserialize, Serialize};

use morn_capability::CapabilityRecord;
use morn_kernel::time::Timestamp;
use morn_runtime::ProviderRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderGatePolicy {
    /// When true, a capability whose provider_ref is absent from the runtime
    /// registry is blocked. Factory/Enterprise compositions should use true.
    pub require_registered_provider: bool,
}

impl ProviderGatePolicy {
    pub const fn strict() -> Self {
        Self {
            require_registered_provider: true,
        }
    }

    pub const fn permissive() -> Self {
        Self {
            require_registered_provider: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderGateBlock {
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderGateReport {
    pub selectable: Vec<CapabilityRecord>,
    pub blocked: Vec<ProviderGateBlock>,
}

impl ProviderGateReport {
    pub fn all_selectable(&self) -> bool {
        self.blocked.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct ProviderGate;

impl ProviderGate {
    pub fn evaluate(
        &self,
        candidates: &[CapabilityRecord],
        registry: &ProviderRegistry,
        policy: ProviderGatePolicy,
        now: Timestamp,
    ) -> ProviderGateReport {
        let mut selectable = Vec::new();
        let mut blocked = Vec::new();

        for candidate in candidates {
            let provider_ref = candidate.manifest.provider_ref.clone();
            match registry.get(&provider_ref) {
                Some(provider) if provider.selectable_at(now) => {
                    selectable.push(candidate.clone());
                }
                Some(provider) => blocked.push(ProviderGateBlock {
                    capability_manifest_ref: candidate.manifest.id.to_string(),
                    provider_ref,
                    reason: format!(
                        "provider status {:?} is not selectable at {}",
                        provider.status, now
                    ),
                }),
                None if policy.require_registered_provider => {
                    blocked.push(ProviderGateBlock {
                        capability_manifest_ref: candidate.manifest.id.to_string(),
                        provider_ref,
                        reason: "provider is not registered in strict composition".to_string(),
                    });
                }
                None => selectable.push(candidate.clone()),
            }
        }

        ProviderGateReport {
            selectable,
            blocked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::{CapabilityKind, CapabilityManifest, EffectClass};
    use morn_kernel::ids::CapabilityId;
    use morn_runtime::{ProviderDescriptor, ProviderFamily, ProviderStatus};

    fn capability(provider: &str) -> CapabilityRecord {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "test-capability",
            provider,
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provenance.source_ref = "repo://test-capability".to_string();
        CapabilityRecord::new(manifest)
    }

    #[test]
    fn strict_gate_blocks_unregistered_provider() {
        let registry = ProviderRegistry::default();
        let report = ProviderGate.evaluate(
            &[capability("unknown-provider")],
            &registry,
            ProviderGatePolicy::strict(),
            Timestamp::now(),
        );
        assert!(report.selectable.is_empty());
        assert_eq!(report.blocked.len(), 1);
    }

    #[test]
    fn provider_must_be_healthy_and_fresh() {
        let now = Timestamp::now();
        let mut registry = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("runtime-a", ProviderFamily::Runtime, "1.0.0");
        provider.status = ProviderStatus::Healthy;
        provider.health_valid_until = Some(Timestamp::from_millis(now.millis() + 500));
        registry.register(provider).unwrap();

        let fresh = ProviderGate.evaluate(
            &[capability("runtime-a")],
            &registry,
            ProviderGatePolicy::strict(),
            now,
        );
        assert_eq!(fresh.selectable.len(), 1);

        let stale = ProviderGate.evaluate(
            &[capability("runtime-a")],
            &registry,
            ProviderGatePolicy::strict(),
            Timestamp::from_millis(now.millis() + 501),
        );
        assert!(stale.selectable.is_empty());
        assert_eq!(stale.blocked.len(), 1);
    }

    #[test]
    fn permissive_gate_supports_lite_unmanaged_local_capabilities() {
        let registry = ProviderRegistry::default();
        let report = ProviderGate.evaluate(
            &[capability("local-script")],
            &registry,
            ProviderGatePolicy::permissive(),
            Timestamp::now(),
        );
        assert_eq!(report.selectable.len(), 1);
        assert!(report.blocked.is_empty());
    }
}
