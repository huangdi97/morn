//! Replaceable node-local composition runtime contract.
//!
//! Cordis is the reference implementation, not a semantic dependency. This
//! contract captures only the Morn-facing composition responsibilities:
//! service-slot binding, lifecycle and inspection. It intentionally has no
//! Work/Authority/Outcome APIs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionProviderRef {
    pub id: String,
    pub version: String,
    pub digest: Option<String>,
    pub endpoint_ref: Option<String>,
}

impl CompositionProviderRef {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.version.trim().is_empty() {
            return Err(Error::validation(
                "composition provider requires non-empty id and version",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionSlotBinding {
    pub slot: String,
    pub provider: CompositionProviderRef,
    pub mounted_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionRuntimeSnapshot {
    pub runtime_id: String,
    pub runtime_version: String,
    pub slots: Vec<CompositionSlotBinding>,
    pub observed_at: Timestamp,
}

impl CompositionRuntimeSnapshot {
    pub fn contains_business_truth(&self) -> bool {
        false
    }
}

pub trait CompositionRuntimeProvider: Send + Sync {
    fn runtime_id(&self) -> &str;
    fn runtime_version(&self) -> &str;
    fn mount(&mut self, slot: &str, provider: CompositionProviderRef) -> Result<()>;
    fn unmount(&mut self, slot: &str) -> Result<bool>;
    fn get(&self, slot: &str) -> Option<&CompositionProviderRef>;
    fn snapshot(&self) -> CompositionRuntimeSnapshot;
}

#[derive(Debug)]
pub struct FixtureCompositionRuntime {
    runtime_id: String,
    runtime_version: String,
    slots: BTreeMap<String, CompositionSlotBinding>,
}

impl FixtureCompositionRuntime {
    pub fn new(runtime_id: impl Into<String>, runtime_version: impl Into<String>) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            runtime_version: runtime_version.into(),
            slots: BTreeMap::new(),
        }
    }
}

impl CompositionRuntimeProvider for FixtureCompositionRuntime {
    fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    fn runtime_version(&self) -> &str {
        &self.runtime_version
    }

    fn mount(&mut self, slot: &str, provider: CompositionProviderRef) -> Result<()> {
        if slot.trim().is_empty() {
            return Err(Error::validation("composition slot is required"));
        }
        provider.validate()?;
        self.slots.insert(
            slot.to_string(),
            CompositionSlotBinding {
                slot: slot.to_string(),
                provider,
                mounted_at: Timestamp::now(),
            },
        );
        Ok(())
    }

    fn unmount(&mut self, slot: &str) -> Result<bool> {
        Ok(self.slots.remove(slot).is_some())
    }

    fn get(&self, slot: &str) -> Option<&CompositionProviderRef> {
        self.slots.get(slot).map(|binding| &binding.provider)
    }

    fn snapshot(&self) -> CompositionRuntimeSnapshot {
        CompositionRuntimeSnapshot {
            runtime_id: self.runtime_id.clone(),
            runtime_version: self.runtime_version.clone(),
            slots: self.slots.values().cloned().collect(),
            observed_at: Timestamp::now(),
        }
    }
}

pub fn run_composition_contract(
    runtime: &mut dyn CompositionRuntimeProvider,
) -> Result<CompositionRuntimeSnapshot> {
    let first = CompositionProviderRef {
        id: "provider-a".to_string(),
        version: "1".to_string(),
        digest: None,
        endpoint_ref: None,
    };
    runtime.mount("harness", first)?;
    if runtime.get("harness").map(|provider| provider.id.as_str()) != Some("provider-a") {
        return Err(Error::invalid_state(
            "composition runtime failed to expose mounted provider",
        ));
    }

    let replacement = CompositionProviderRef {
        id: "provider-b".to_string(),
        version: "2".to_string(),
        digest: None,
        endpoint_ref: None,
    };
    runtime.mount("harness", replacement)?;
    if runtime.get("harness").map(|provider| provider.id.as_str()) != Some("provider-b") {
        return Err(Error::invalid_state(
            "composition runtime failed to replace service-slot provider",
        ));
    }

    let snapshot = runtime.snapshot();
    if snapshot.contains_business_truth() {
        return Err(Error::invalid_state(
            "composition snapshot must not contain canonical Work truth",
        ));
    }
    if !runtime.unmount("harness")? || runtime.get("harness").is_some() {
        return Err(Error::invalid_state(
            "composition runtime failed lifecycle cleanup",
        ));
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_contract_is_runtime_neutral_and_business_truth_free() {
        let mut runtime = FixtureCompositionRuntime::new("fixture-composition", "1");
        let snapshot = run_composition_contract(&mut runtime).unwrap();
        assert_eq!(snapshot.runtime_id, "fixture-composition");
        assert!(!snapshot.contains_business_truth());
    }

    #[test]
    fn service_slot_replacement_is_composition_not_execution_binding_migration() {
        let mut runtime = FixtureCompositionRuntime::new("runtime", "1");
        runtime
            .mount(
                "authority",
                CompositionProviderRef {
                    id: "opa-a".to_string(),
                    version: "1".to_string(),
                    digest: None,
                    endpoint_ref: None,
                },
            )
            .unwrap();
        runtime
            .mount(
                "authority",
                CompositionProviderRef {
                    id: "cedar-b".to_string(),
                    version: "1".to_string(),
                    digest: None,
                    endpoint_ref: None,
                },
            )
            .unwrap();

        assert_eq!(runtime.get("authority").unwrap().id, "cedar-b");
        // No Work id / ExecutionBinding id exists on this contract by design.
        let json = serde_json::to_value(runtime.snapshot()).unwrap();
        assert!(json.get("work_id").is_none());
        assert!(json.get("execution_binding_id").is_none());
    }
}
