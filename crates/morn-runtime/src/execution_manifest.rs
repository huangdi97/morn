//! Reproducible execution manifest for a concrete Work binding.
//!
//! The manifest records the exact semantic/runtime interpretation used by one
//! execution scope. It is evidence/provenance metadata, not another source of
//! Work truth.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::protocol::{ProtocolSnapshot, MORN_PROTOCOL_V11_5};
use morn_kernel::time::Timestamp;
use morn_work::control::WorkResource;

use crate::binding::ExecutionBinding;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionRuntimeRef {
    pub id: String,
    pub version: String,
    pub digest: Option<String>,
}

impl CompositionRuntimeRef {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            digest: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionManifest {
    pub schema: String,
    pub protocol_version: String,
    pub protocol_invariants: Vec<String>,
    pub work_ref: String,
    pub work_generation: u64,
    pub profile_ref: String,
    pub site_ref: Option<String>,
    pub source_solution_ref: Option<String>,
    pub execution_binding_ref: String,
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub provider_version: String,
    pub provider_digest: Option<String>,
    pub composition_runtime: CompositionRuntimeRef,
    pub runtime_ref: Option<String>,
    pub authority_decision_ref: Option<String>,
    pub created_at: Timestamp,
}

impl ExecutionManifest {
    pub fn from_binding(
        work: &WorkResource,
        binding: &ExecutionBinding,
        composition_runtime: CompositionRuntimeRef,
    ) -> Result<Self> {
        if !binding.matches_work_generation(work) {
            return Err(Error::validation(
                "execution manifest requires a binding pinned to the current Work generation",
            ));
        }
        if binding.profile_ref != work.spec.profile_ref {
            return Err(Error::validation(
                "execution manifest profile must match the Work profile",
            ));
        }
        if binding.site_ref != work.spec.site_ref {
            return Err(Error::validation(
                "execution manifest site must match the Work site",
            ));
        }
        if composition_runtime.id.trim().is_empty() || composition_runtime.version.trim().is_empty()
        {
            return Err(Error::validation(
                "execution manifest requires exact composition runtime identity",
            ));
        }
        if work.spec.protocol_version != MORN_PROTOCOL_V11_5 {
            return Err(Error::invalid_state(format!(
                "reference runtime currently emits only Morn protocol {}; Work pins {}",
                MORN_PROTOCOL_V11_5, work.spec.protocol_version
            )));
        }

        Ok(Self {
            schema: "morn.execution-manifest/v11.5".to_string(),
            protocol_version: work.spec.protocol_version.to_string(),
            protocol_invariants: ProtocolSnapshot::v11_5().invariant_ids(),
            work_ref: work.id.to_string(),
            work_generation: work.generation,
            profile_ref: work.spec.profile_ref.clone(),
            site_ref: work.spec.site_ref.clone(),
            source_solution_ref: work.spec.source_solution_ref.clone(),
            execution_binding_ref: binding.id.to_string(),
            capability_manifest_ref: binding.capability_manifest_ref.clone(),
            provider_ref: binding.provider_ref.clone(),
            provider_version: binding.provider_version.clone(),
            provider_digest: binding.provider_digest.clone(),
            composition_runtime,
            runtime_ref: binding.runtime_ref.clone(),
            authority_decision_ref: binding.authority_decision_ref.clone(),
            created_at: Timestamp::now(),
        })
    }

    pub fn validates_against(&self, work: &WorkResource, binding: &ExecutionBinding) -> bool {
        self.schema == "morn.execution-manifest/v11.5"
            && work.spec.protocol_version == MORN_PROTOCOL_V11_5
            && binding.matches_work_generation(work)
            && binding.profile_ref == work.spec.profile_ref
            && binding.site_ref == work.spec.site_ref
            && self.protocol_version == work.spec.protocol_version.to_string()
            && self.protocol_invariants == ProtocolSnapshot::v11_5().invariant_ids()
            && self.work_ref == work.id.to_string()
            && self.work_generation == work.generation
            && self.profile_ref == work.spec.profile_ref
            && self.site_ref == work.spec.site_ref
            && self.source_solution_ref == work.spec.source_solution_ref
            && self.runtime_ref == binding.runtime_ref
            && self.authority_decision_ref == binding.authority_decision_ref
            && !self.composition_runtime.id.trim().is_empty()
            && !self.composition_runtime.version.trim().is_empty()
            && self.execution_binding_ref == binding.id.to_string()
            && self.capability_manifest_ref == binding.capability_manifest_ref
            && self.provider_ref == binding.provider_ref
            && self.provider_version == binding.provider_version
            && self.provider_digest == binding.provider_digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::{WorkResource, WorkSpec};

    #[test]
    fn manifest_pins_protocol_work_profile_binding_provider_and_runtime() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "review delivery impact",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        spec.source_solution_ref = Some("solution://factory-review@1.0.0".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding =
            ExecutionBinding::for_work(&work, "cmanifest:1", "deepseek-harness", "0.2.0-rc.2");
        binding.provider_digest = Some(format!("sha256:{}", "a".repeat(64)));
        binding.runtime_ref = Some("runtime://dsh/session-1".to_string());
        binding.authority_decision_ref = Some("authz:1".to_string());

        let manifest = ExecutionManifest::from_binding(
            &work,
            &binding,
            CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
        )
        .unwrap();

        assert_eq!(manifest.protocol_version, "11.5.0");
        assert!(manifest
            .protocol_invariants
            .contains(&"work-truth-independent-of-executor".to_string()));
        assert_eq!(manifest.profile_ref, "morn.factory.readonly@1.0.0");
        assert_eq!(manifest.provider_ref, "deepseek-harness");
        assert_eq!(manifest.composition_runtime.id, "cordis-reference");
        assert!(manifest.validates_against(&work, &binding));
    }

    #[test]
    fn tampering_with_pinned_solution_runtime_or_authority_invalidates_manifest() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "verify maintained asset",
            "morn.factory.readonly@1.0.0",
        );
        spec.source_solution_ref = Some("solution://factory@1.0.0".to_string());
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding = ExecutionBinding::for_work(
            &work,
            "manifest:sha256:abc",
            "provider-a",
            "1.0.0",
        );
        binding.runtime_ref = Some("runtime://session-a".to_string());
        binding.authority_decision_ref = Some("authority://permit-a".to_string());
        let manifest = ExecutionManifest::from_binding(
            &work,
            &binding,
            CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
        )
        .unwrap();
        assert!(manifest.validates_against(&work, &binding));

        let mut forged = manifest.clone();
        forged.source_solution_ref = Some("solution://other".to_string());
        assert!(!forged.validates_against(&work, &binding));

        let mut forged = manifest.clone();
        forged.runtime_ref = Some("runtime://unrelated-session".to_string());
        assert!(!forged.validates_against(&work, &binding));

        let mut forged = manifest.clone();
        forged.authority_decision_ref = Some("authority://other".to_string());
        assert!(!forged.validates_against(&work, &binding));

        let mut forged = manifest.clone();
        forged.composition_runtime.version.clear();
        assert!(!forged.validates_against(&work, &binding));

        let mut forged = manifest.clone();
        forged.schema = "morn.execution-manifest/v0".to_string();
        assert!(!forged.validates_against(&work, &binding));

        let mut wrong_binding = binding.clone();
        wrong_binding.profile_ref = "morn.lite@1.0.0".to_string();
        assert!(!manifest.validates_against(&work, &wrong_binding));
    }

    #[test]
    fn unsupported_protocol_requires_runtime_migration() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "investigate",
            "morn.lite@1.0.0",
        );
        spec.protocol_version = morn_kernel::version::Version::new(11, 6, 0);
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "cmanifest:1", "pi", "1");
        assert!(ExecutionManifest::from_binding(
            &work,
            &binding,
            CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
        )
        .is_err());
    }

    #[test]
    fn stale_binding_cannot_generate_current_execution_manifest() {
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "investigate",
            "morn.lite@1.0.0",
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "cmanifest:1", "pi", "1");

        let mut next = work.spec.clone();
        next.goal = "investigate and verify".to_string();
        work.replace_spec(next);

        assert!(ExecutionManifest::from_binding(
            &work,
            &binding,
            CompositionRuntimeRef::new("cordis-reference", "4.0.4"),
        )
        .is_err());
    }
}
