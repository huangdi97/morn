//! Execution binding pins a Work generation to an exact capability/provider
//! selection for the lifetime of an attempt.
//!
//! Provider hot-reload may affect future bindings, but must not silently change
//! an attempt that has already begun.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_work::control::WorkResource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBinding {
    pub id: RuntimeBindingId,
    pub work_id: WorkPackageId,
    pub work_generation: u64,
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub provider_version: String,
    pub provider_digest: Option<String>,
    pub runtime_ref: Option<String>,
    pub authority_decision_ref: Option<String>,
    pub profile_ref: String,
    pub migration_from: Option<RuntimeBindingId>,
    pub created_at: Timestamp,
}

impl ExecutionBinding {
    pub fn for_work(
        work: &WorkResource,
        capability_manifest_ref: impl Into<String>,
        provider_ref: impl Into<String>,
        provider_version: impl Into<String>,
    ) -> Self {
        Self {
            id: RuntimeBindingId::generate_with("binding"),
            work_id: work.id.clone(),
            work_generation: work.generation,
            capability_manifest_ref: capability_manifest_ref.into(),
            provider_ref: provider_ref.into(),
            provider_version: provider_version.into(),
            provider_digest: None,
            runtime_ref: None,
            authority_decision_ref: None,
            profile_ref: work.spec.profile_ref.clone(),
            migration_from: None,
            created_at: Timestamp::now(),
        }
    }

    pub fn matches_work_generation(&self, work: &WorkResource) -> bool {
        self.work_id == work.id && self.work_generation == work.generation
    }

    /// Provider migration creates a new binding. The old binding is not edited.
    pub fn migrate_to_provider(
        &self,
        provider_ref: impl Into<String>,
        provider_version: impl Into<String>,
    ) -> Self {
        let mut replacement = self.clone();
        replacement.id = RuntimeBindingId::generate_with("binding");
        replacement.provider_ref = provider_ref.into();
        replacement.provider_version = provider_version.into();
        replacement.provider_digest = None;
        replacement.migration_from = Some(self.id.clone());
        replacement.created_at = Timestamp::now();
        replacement
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::{WorkResource, WorkSpec};

    #[test]
    fn binding_is_pinned_to_work_generation_and_provider() {
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "investigate", "factory/v1");
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        let old = ExecutionBinding::for_work(&work, "manifest@sha256:a", "dsh", "1");
        assert!(old.matches_work_generation(&work));

        let mut next_spec = work.spec.clone();
        next_spec.goal = "investigate and reschedule".into();
        work.replace_spec(next_spec);
        assert!(!old.matches_work_generation(&work));

        let replacement = old.migrate_to_provider("pi", "2");
        assert_eq!(old.provider_ref, "dsh");
        assert_eq!(replacement.provider_ref, "pi");
        assert_eq!(replacement.migration_from, Some(old.id));
    }
}
