//! Turn-scoped semantic admission for governed E0 execution.
//!
//! This record is not an Outcome, AcceptanceDecision or executor success.
//! It proves only that one concrete input was admitted for dispatch while the
//! exact Work generation, ExecutionBinding and capability-resolution epoch were
//! current at the durable control-plane boundary.

use serde::{Deserialize, Serialize};

use morn_capability::EffectClass;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{Id, RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_runtime::ExecutionBinding;
use morn_work::control::WorkResource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExecutionDispatchAdmissionTag;
pub type ExecutionDispatchAdmissionId = Id<ExecutionDispatchAdmissionTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionDispatchAdmission {
    pub id: ExecutionDispatchAdmissionId,
    pub workspace_id: WorkspaceId,
    pub work_id: WorkPackageId,
    pub work_generation: u64,
    pub binding_ref: RuntimeBindingId,
    pub resolution_decision_ref: String,
    pub capability_manifest_ref: String,
    pub profile_ref: String,
    pub site_ref: Option<String>,
    pub provider_ref: String,
    pub provider_version: String,
    pub provider_digest: Option<String>,
    pub runtime_ref: Option<String>,
    pub execution_environment_ref: Option<String>,
    /// SHA-256 of the exact user/input payload submitted for this turn. The
    /// control plane stores the digest, never a second copy of prompt content.
    pub input_sha256: String,
    pub admitted_at: Timestamp,
}

impl ExecutionDispatchAdmission {
    pub fn for_e0(
        work: &WorkResource,
        binding: &ExecutionBinding,
        input_sha256: impl Into<String>,
    ) -> Result<Self> {
        if !binding.matches_work_generation(work)
            || binding.profile_ref != work.spec.profile_ref
            || binding.site_ref != work.spec.site_ref
            || binding.autonomy_posture != work.spec.autonomy_posture
        {
            return Err(Error::validation(
                "E0 dispatch admission requires the exact Work generation/profile/site/autonomy binding",
            ));
        }
        if binding.effect_ceiling != Some(EffectClass::E0LifecycleReversible) {
            return Err(Error::not_authorized(
                "E0 dispatch admission requires an E0 lifecycle-reversible binding",
            ));
        }
        if !binding.environment_identity_consistent() {
            return Err(Error::validation(
                "E0 dispatch admission binding has a partial execution-environment identity",
            ));
        }
        let resolution_decision_ref = binding
            .resolution_decision_ref
            .as_deref()
            .filter(|reference| !reference.trim().is_empty())
            .ok_or_else(|| {
                Error::validation(
                    "E0 dispatch admission requires an exact capability-resolution epoch",
                )
            })?
            .to_string();
        if binding.capability_manifest_ref.trim().is_empty()
            || binding.provider_ref.trim().is_empty()
            || binding.provider_version.trim().is_empty()
        {
            return Err(Error::validation(
                "E0 dispatch admission requires non-empty capability/provider identity",
            ));
        }

        let input_sha256 = input_sha256.into();
        let digest = input_sha256.strip_prefix("sha256:").ok_or_else(|| {
            Error::validation("E0 dispatch admission input digest must use sha256:<hex>")
        })?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::validation(
                "E0 dispatch admission input digest must contain exactly 64 hexadecimal SHA-256 characters",
            ));
        }

        Ok(Self {
            id: ExecutionDispatchAdmissionId::generate_with("dispatch-admission"),
            workspace_id: work.workspace_id.clone(),
            work_id: work.id.clone(),
            work_generation: work.generation,
            binding_ref: binding.id.clone(),
            resolution_decision_ref,
            capability_manifest_ref: binding.capability_manifest_ref.clone(),
            profile_ref: binding.profile_ref.clone(),
            site_ref: binding.site_ref.clone(),
            provider_ref: binding.provider_ref.clone(),
            provider_version: binding.provider_version.clone(),
            provider_digest: binding.provider_digest.clone(),
            runtime_ref: binding.runtime_ref.clone(),
            execution_environment_ref: binding.execution_environment_ref.clone(),
            input_sha256,
            admitted_at: Timestamp::now(),
        })
    }

    pub fn validates_against(&self, work: &WorkResource, binding: &ExecutionBinding) -> bool {
        self.workspace_id == work.workspace_id
            && self.work_id == work.id
            && self.work_generation == work.generation
            && self.binding_ref == binding.id
            && self.resolution_decision_ref
                == binding
                    .resolution_decision_ref
                    .as_deref()
                    .unwrap_or_default()
            && self.capability_manifest_ref == binding.capability_manifest_ref
            && self.profile_ref == work.spec.profile_ref
            && self.profile_ref == binding.profile_ref
            && self.site_ref == work.spec.site_ref
            && self.site_ref == binding.site_ref
            && self.provider_ref == binding.provider_ref
            && self.provider_version == binding.provider_version
            && self.provider_digest == binding.provider_digest
            && self.runtime_ref == binding.runtime_ref
            && self.execution_environment_ref == binding.execution_environment_ref
            && self
                .input_sha256
                .strip_prefix("sha256:")
                .is_some_and(|digest| {
                    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            && binding.effect_ceiling == Some(EffectClass::E0LifecycleReversible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::WorkSpec;

    #[test]
    fn e0_dispatch_admission_pins_resolution_provider_and_input_digest() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "draft",
            "morn.lite@1.0.0",
        );
        spec.site_ref = Some("site-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding =
            ExecutionBinding::for_work(&work, "capability:1", "deepseek-harness", "1");
        binding.resolution_decision_ref =
            Some("capability-resolution-decision://epoch-1".to_string());
        binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);

        let admission = ExecutionDispatchAdmission::for_e0(
            &work,
            &binding,
            format!("sha256:{}", "a".repeat(64)),
        )
        .unwrap();
        assert!(admission.validates_against(&work, &binding));
        assert_eq!(
            admission.resolution_decision_ref,
            "capability-resolution-decision://epoch-1"
        );
        assert_eq!(admission.provider_ref, "deepseek-harness");
    }

    #[test]
    fn e0_dispatch_admission_rejects_unpinned_epoch_or_bad_input_digest() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "draft",
                "morn.lite@1.0.0",
            ),
        );
        let mut binding =
            ExecutionBinding::for_work(&work, "capability:1", "deepseek-harness", "1");
        binding.effect_ceiling = Some(EffectClass::E0LifecycleReversible);
        assert!(ExecutionDispatchAdmission::for_e0(
            &work,
            &binding,
            format!("sha256:{}", "a".repeat(64)),
        )
        .is_err());

        binding.resolution_decision_ref =
            Some("capability-resolution-decision://epoch-1".to_string());
        assert!(
            ExecutionDispatchAdmission::for_e0(&work, &binding, "sha256:not-a-digest").is_err()
        );
    }
}
