//! Provider fabric registry.
//!
//! A Morn Provider is an implementation/runtime endpoint that can satisfy one
//! or more protocol slots (harness, authority, execution environment, solver,
//! connector, workflow, etc.). Provider identity is runtime/composition
//! metadata; it never replaces Capability, Work or ExecutionBinding identity.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderFamily {
    Composition,
    Harness,
    Runtime,
    ExecutionEnvironment,
    Authority,
    Connector,
    Workflow,
    Solver,
    Model,
    Human,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderStatus {
    Registered,
    Healthy,
    Degraded,
    Unavailable,
    Suspended,
}

impl ProviderStatus {
    pub const fn selectable(self) -> bool {
        matches!(self, Self::Healthy)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub family: ProviderFamily,
    pub version: String,
    pub digest: Option<String>,
    pub endpoint_ref: Option<String>,
    pub protocols: BTreeSet<String>,
    pub features: BTreeSet<String>,
    pub status: ProviderStatus,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    #[serde(default)]
    pub health_valid_until: Option<Timestamp>,
}

impl ProviderDescriptor {
    pub fn new(id: impl Into<String>, family: ProviderFamily, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            family,
            version: version.into(),
            digest: None,
            endpoint_ref: None,
            protocols: BTreeSet::new(),
            features: BTreeSet::new(),
            status: ProviderStatus::Registered,
            evidence_refs: Vec::new(),
            observed_at: Timestamp::now(),
            health_valid_until: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::validation("provider id is required"));
        }
        if self.version.trim().is_empty() {
            return Err(Error::validation("provider version is required"));
        }
        if let Some(digest) = &self.digest {
            let Some(hex) = digest.strip_prefix("sha256:") else {
                return Err(Error::validation(
                    "provider digest must use sha256:<64 hex>",
                ));
            };
            if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(Error::validation(
                    "provider digest must use sha256:<64 hex>",
                ));
            }
        }
        Ok(())
    }

    pub fn supports(&self, required_features: &BTreeSet<String>) -> bool {
        required_features.is_subset(&self.features)
    }

    pub fn selectable_at(&self, now: Timestamp) -> bool {
        self.status.selectable()
            && self
                .health_valid_until
                .is_none_or(|valid_until| now <= valid_until)
    }

    /// Exact runtime identity used when creating an ExecutionBinding.
    pub fn binding_ref(&self) -> String {
        match &self.digest {
            Some(digest) => format!("{}@{}#{}", self.id, self.version, digest),
            None => format!("{}@{}", self.id, self.version),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderObservation {
    pub provider_id: String,
    pub previous_status: ProviderStatus,
    pub current_status: ProviderStatus,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
}

#[derive(Debug, Default)]
pub struct ProviderRegistry {
    descriptors: BTreeMap<String, ProviderDescriptor>,
    observations: Vec<ProviderObservation>,
}

impl ProviderRegistry {
    pub fn register(&mut self, descriptor: ProviderDescriptor) -> Result<()> {
        descriptor.validate()?;
        if self.descriptors.contains_key(&descriptor.id) {
            return Err(Error::conflict(format!(
                "provider {} already registered",
                descriptor.id
            )));
        }
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn upsert_projection(&mut self, descriptor: ProviderDescriptor) -> Result<()> {
        descriptor.validate()?;
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn observe_status(
        &mut self,
        provider_id: &str,
        status: ProviderStatus,
        reason: impl Into<String>,
        evidence_refs: Vec<String>,
    ) -> Result<ProviderObservation> {
        self.observe_status_with_ttl(provider_id, status, reason, evidence_refs, None)
    }

    pub fn observe_status_with_ttl(
        &mut self,
        provider_id: &str,
        status: ProviderStatus,
        reason: impl Into<String>,
        evidence_refs: Vec<String>,
        ttl_ms: Option<i64>,
    ) -> Result<ProviderObservation> {
        let descriptor = self
            .descriptors
            .get_mut(provider_id)
            .ok_or_else(|| Error::not_found(format!("provider {provider_id}")))?;
        let previous_status = descriptor.status;
        descriptor.status = status;
        descriptor.observed_at = Timestamp::now();
        descriptor.health_valid_until = ttl_ms
            .filter(|ttl| *ttl > 0)
            .map(|ttl| Timestamp::from_millis(descriptor.observed_at.millis().saturating_add(ttl)));
        descriptor
            .evidence_refs
            .extend(evidence_refs.iter().cloned());

        let observation = ProviderObservation {
            provider_id: provider_id.to_string(),
            previous_status,
            current_status: status,
            reason: reason.into(),
            evidence_refs,
            observed_at: descriptor.observed_at,
        };
        self.observations.push(observation.clone());
        Ok(observation)
    }

    pub fn get(&self, provider_id: &str) -> Option<&ProviderDescriptor> {
        self.descriptors.get(provider_id)
    }

    pub fn list(&self) -> Vec<&ProviderDescriptor> {
        self.descriptors.values().collect()
    }

    pub fn observations(&self) -> &[ProviderObservation] {
        &self.observations
    }

    pub fn eligible(
        &self,
        family: ProviderFamily,
        required_features: &BTreeSet<String>,
    ) -> Vec<&ProviderDescriptor> {
        self.eligible_at(family, required_features, Timestamp::now())
    }

    pub fn eligible_at(
        &self,
        family: ProviderFamily,
        required_features: &BTreeSet<String>,
        now: Timestamp,
    ) -> Vec<&ProviderDescriptor> {
        self.descriptors
            .values()
            .filter(|provider| provider.family == family)
            .filter(|provider| provider.selectable_at(now))
            .filter(|provider| provider.supports(required_features))
            .collect()
    }
}

/// Reference catalog describes only the local/reference implementations that
/// exist in this repository. External providers remain unavailable until their
/// concrete runtime/transport is configured.
pub fn reference_provider_catalog() -> ProviderRegistry {
    let mut registry = ProviderRegistry::default();

    let mut cordis =
        ProviderDescriptor::new("cordis-reference", ProviderFamily::Composition, "4.0.4");
    cordis.protocols.insert("cordis".to_string());
    cordis.features.extend(
        [
            "context",
            "service-slot",
            "fiber-lifecycle",
            "dependency-injection",
        ]
        .into_iter()
        .map(str::to_string),
    );
    cordis.status = ProviderStatus::Healthy;
    cordis
        .evidence_refs
        .push("runtime/cordis-host/package.json".to_string());
    registry.register(cordis).expect("reference provider valid");

    let mut native = ProviderDescriptor::new("morn-native", ProviderFamily::Harness, "reference");
    native.protocols.insert("morn-harness".to_string());
    native.features.extend(
        ["interrupt", "resume", "session-close", "durable-events"]
            .into_iter()
            .map(str::to_string),
    );
    native.status = ProviderStatus::Healthy;
    native
        .evidence_refs
        .push("crates/morn-harness/src/provider.rs".to_string());
    registry.register(native).expect("reference provider valid");

    let mut dsh = ProviderDescriptor::new("deepseek-harness", ProviderFamily::Harness, "external");
    dsh.protocols.insert("json-rpc-stdio".to_string());
    dsh.features.extend(
        ["multi-session", "durable-events"]
            .into_iter()
            .map(str::to_string),
    );
    dsh.status = ProviderStatus::Unavailable;
    dsh.evidence_refs
        .push("crates/morn-harness/src/dsh_sdk.rs".to_string());
    registry.register(dsh).expect("reference provider valid");

    let mut pi = ProviderDescriptor::new("pi", ProviderFamily::Harness, "external");
    pi.protocols.insert("jsonl-rpc-stdio".to_string());
    pi.features.extend(
        ["prompt", "get-state", "abort", "agent-settled"]
            .into_iter()
            .map(str::to_string),
    );
    pi.status = ProviderStatus::Unavailable;
    pi.evidence_refs
        .push("crates/morn-harness/src/pi_rpc.rs".to_string());
    registry.register(pi).expect("reference provider valid");

    let mut fixture_env = ProviderDescriptor::new(
        "fixture-environment",
        ProviderFamily::ExecutionEnvironment,
        "reference",
    );
    fixture_env
        .protocols
        .insert("morn-execution-environment".to_string());
    fixture_env.features.extend(
        ["process", "container", "microvm"]
            .into_iter()
            .map(str::to_string),
    );
    fixture_env.status = ProviderStatus::Healthy;
    fixture_env
        .evidence_refs
        .push("crates/morn-runtime/src/environment.rs".to_string());
    registry
        .register(fixture_env)
        .expect("reference provider valid");

    let mut native_authority =
        ProviderDescriptor::new("morn-native-policy", ProviderFamily::Authority, "reference");
    native_authority
        .protocols
        .insert("morn-authority-provider".to_string());
    native_authority.status = ProviderStatus::Healthy;
    native_authority
        .evidence_refs
        .push("crates/morn-runtime/src/authority.rs".to_string());
    registry
        .register(native_authority)
        .expect("reference provider valid");

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_selects_only_healthy_feature_compatible_provider() {
        let mut registry = ProviderRegistry::default();

        let mut primary = ProviderDescriptor::new("dsh", ProviderFamily::Harness, "1");
        primary.features.insert("durable-events".to_string());
        primary.status = ProviderStatus::Healthy;
        registry.register(primary).unwrap();

        let mut fallback = ProviderDescriptor::new("pi", ProviderFamily::Harness, "1");
        fallback.status = ProviderStatus::Healthy;
        registry.register(fallback).unwrap();

        let required = BTreeSet::from(["durable-events".to_string()]);
        let eligible = registry.eligible(ProviderFamily::Harness, &required);
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].id, "dsh");
    }

    #[test]
    fn stale_health_observation_is_not_selectable() {
        let mut registry = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("dynamic", ProviderFamily::Harness, "1");
        provider.status = ProviderStatus::Healthy;
        registry.register(provider).unwrap();

        let observation = registry
            .observe_status_with_ttl(
                "dynamic",
                ProviderStatus::Healthy,
                "probe passed",
                vec!["probe://dynamic/1".to_string()],
                Some(1_000),
            )
            .unwrap();

        let fresh = registry.eligible_at(
            ProviderFamily::Harness,
            &BTreeSet::new(),
            observation.observed_at,
        );
        assert_eq!(fresh.len(), 1);

        let stale_at = Timestamp::from_millis(observation.observed_at.millis() + 1_001);
        let stale = registry.eligible_at(
            ProviderFamily::Harness,
            &BTreeSet::new(),
            stale_at,
        );
        assert!(stale.is_empty());
    }

    #[test]
    fn health_change_is_projection_plus_append_only_observation() {
        let mut registry = ProviderRegistry::default();
        let mut provider = ProviderDescriptor::new("dsh", ProviderFamily::Harness, "1");
        provider.status = ProviderStatus::Healthy;
        registry.register(provider).unwrap();

        let event = registry
            .observe_status(
                "dsh",
                ProviderStatus::Unavailable,
                "process exited",
                vec!["probe://dsh/1".to_string()],
            )
            .unwrap();

        assert_eq!(
            registry.get("dsh").unwrap().status,
            ProviderStatus::Unavailable
        );
        assert_eq!(event.previous_status, ProviderStatus::Healthy);
        assert_eq!(registry.observations().len(), 1);
    }

    #[test]
    fn reference_catalog_does_not_claim_external_harnesses_are_live() {
        let registry = reference_provider_catalog();
        assert_eq!(
            registry.get("cordis-reference").unwrap().status,
            ProviderStatus::Healthy
        );
        assert_eq!(
            registry.get("deepseek-harness").unwrap().status,
            ProviderStatus::Unavailable
        );
        assert_eq!(
            registry.get("pi").unwrap().status,
            ProviderStatus::Unavailable
        );
    }
}
