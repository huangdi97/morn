//! Execution binding pins a Work generation to an exact capability/provider
//! selection for the lifetime of an attempt.
//!
//! Provider hot-reload may affect future bindings, but must not silently change
//! an attempt that has already begun.

use serde::{Deserialize, Serialize};

use morn_capability::{EffectClass, ResolvedCapability};

use morn_kernel::ids::{Id, RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_work::control::WorkResource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BindingMigrationDecisionTag;
pub type BindingMigrationDecisionId = Id<BindingMigrationDecisionTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BindingMigrationReason {
    ProviderReplacement,
    CapabilityUpgrade,
    ProfileUpgrade,
    RuntimeRecovery,
    ManualRebind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingMigrationRequest {
    pub capability_manifest_ref: String,
    pub provider_ref: String,
    pub provider_version: String,
    pub reason: BindingMigrationReason,
    pub requested_by: String,
    pub evidence_refs: Vec<String>,
    pub new_effect_ceiling: Option<EffectClass>,
    pub new_compensation_ref: Option<String>,
    pub new_idempotency_key_required: Option<bool>,
}

impl BindingMigrationRequest {
    pub fn new(
        capability_manifest_ref: impl Into<String>,
        provider_ref: impl Into<String>,
        provider_version: impl Into<String>,
        reason: BindingMigrationReason,
        requested_by: impl Into<String>,
    ) -> Self {
        Self {
            capability_manifest_ref: capability_manifest_ref.into(),
            provider_ref: provider_ref.into(),
            provider_version: provider_version.into(),
            reason,
            requested_by: requested_by.into(),
            evidence_refs: Vec::new(),
            new_effect_ceiling: None,
            new_compensation_ref: None,
            new_idempotency_key_required: None,
        }
    }

    pub fn with_evidence(mut self, evidence_refs: Vec<String>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    pub fn with_effect_semantics(
        mut self,
        effect_ceiling: EffectClass,
        compensation_ref: Option<String>,
        idempotency_key_required: bool,
    ) -> Self {
        self.new_effect_ceiling = Some(effect_ceiling);
        self.new_compensation_ref = compensation_ref;
        self.new_idempotency_key_required = Some(idempotency_key_required);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingMigrationDecision {
    pub id: BindingMigrationDecisionId,
    pub work_id: WorkPackageId,
    pub from_binding: RuntimeBindingId,
    pub to_binding: RuntimeBindingId,
    pub reason: BindingMigrationReason,
    pub requested_by: String,
    pub evidence_refs: Vec<String>,
    pub from_provider: String,
    pub to_provider: String,
    pub from_profile: String,
    pub to_profile: String,
    #[serde(default)]
    pub from_site_ref: Option<String>,
    #[serde(default)]
    pub to_site_ref: Option<String>,
    pub created_at: Timestamp,
}

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
    /// Maximum real-world effect the selected capability declared and was
    /// resolved under. None is legacy/untyped and must not authorize a write.
    #[serde(default)]
    pub effect_ceiling: Option<EffectClass>,
    #[serde(default)]
    pub compensation_ref: Option<String>,
    #[serde(default)]
    pub idempotency_key_required: bool,
    pub authority_decision_ref: Option<String>,
    pub profile_ref: String,
    #[serde(default)]
    pub site_ref: Option<String>,
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
            effect_ceiling: None,
            compensation_ref: None,
            idempotency_key_required: false,
            authority_decision_ref: None,
            profile_ref: work.spec.profile_ref.clone(),
            site_ref: work.spec.site_ref.clone(),
            migration_from: None,
            created_at: Timestamp::now(),
        }
    }

    pub fn for_resolved_capability(
        work: &WorkResource,
        capability: &ResolvedCapability,
        provider_version: impl Into<String>,
    ) -> Self {
        let mut binding = Self::for_work(
            work,
            capability.manifest_id.to_string(),
            capability.provider_ref.clone(),
            provider_version,
        );
        binding.effect_ceiling = Some(capability.maximum_effect);
        binding.compensation_ref = capability.compensation_ref.clone();
        binding.idempotency_key_required = capability.idempotency_key_required;
        binding
    }

    pub fn matches_work_generation(&self, work: &WorkResource) -> bool {
        self.work_id == work.id && self.work_generation == work.generation
    }

    /// Provider migration creates a new binding. The old binding is not edited.
    pub fn rebind_for_work(
        &self,
        work: &WorkResource,
        request: BindingMigrationRequest,
    ) -> (Self, BindingMigrationDecision) {
        let same_capability = request.capability_manifest_ref == self.capability_manifest_ref;
        let mut replacement = Self::for_work(
            work,
            request.capability_manifest_ref,
            request.provider_ref,
            request.provider_version,
        );
        replacement.provider_digest = None;
        replacement.runtime_ref = None;
        replacement.effect_ceiling = request.new_effect_ceiling.or(if same_capability {
            self.effect_ceiling
        } else {
            None
        });
        replacement.compensation_ref = request.new_compensation_ref.or_else(|| {
            if same_capability {
                self.compensation_ref.clone()
            } else {
                None
            }
        });
        replacement.idempotency_key_required =
            request
                .new_idempotency_key_required
                .unwrap_or(if same_capability {
                    self.idempotency_key_required
                } else {
                    false
                });
        replacement.authority_decision_ref = None;
        replacement.migration_from = Some(self.id.clone());

        let decision = BindingMigrationDecision {
            id: BindingMigrationDecisionId::generate_with("binding-migration"),
            work_id: work.id.clone(),
            from_binding: self.id.clone(),
            to_binding: replacement.id.clone(),
            reason: request.reason,
            requested_by: request.requested_by,
            evidence_refs: request.evidence_refs,
            from_provider: self.provider_ref.clone(),
            to_provider: replacement.provider_ref.clone(),
            from_profile: self.profile_ref.clone(),
            to_profile: replacement.profile_ref.clone(),
            from_site_ref: self.site_ref.clone(),
            to_site_ref: replacement.site_ref.clone(),
            created_at: Timestamp::now(),
        };
        (replacement, decision)
    }

    /// Legacy provider-only convenience retained for compatibility. New v11.5
    /// code should prefer rebind_for_work so migration is explicitly recorded.
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
    fn resolved_binding_pins_effect_semantics() {
        use morn_capability::manifest::CapabilityManifestId;
        use morn_capability::{CapabilityKind, ResolvedCapability};

        let work_id = WorkPackageId::generate_with("wp");
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(work_id, "create order", "factory/v2"),
        );
        let resolved = ResolvedCapability {
            manifest_id: CapabilityManifestId::generate_with("cmanifest"),
            provider_ref: "cmms-provider".to_string(),
            kind: CapabilityKind::Api,
            estimated_cost_micros: Some(1),
            maximum_effect: EffectClass::E2Compensatable,
            compensation_ref: Some("cmms.cancel-order".to_string()),
            idempotency_key_required: true,
            required_execution_class: morn_kernel::ExecutionClass::Container,
            required_execution_guarantees: vec![],
            score: 100,
            rationale: vec![],
        };
        let binding = ExecutionBinding::for_resolved_capability(&work, &resolved, "1");
        assert_eq!(binding.effect_ceiling, Some(EffectClass::E2Compensatable));
        assert_eq!(
            binding.compensation_ref.as_deref(),
            Some("cmms.cancel-order")
        );
        assert!(binding.idempotency_key_required);
    }

    #[test]
    fn explicit_rebind_records_provider_and_profile_migration() {
        let work_id = WorkPackageId::generate_with("wp");
        let mut old_spec = WorkSpec::new(work_id.clone(), "investigate", "factory/v1");
        old_spec.site_ref = Some("plant-a".to_string());
        let old_work = WorkResource::new(WorkspaceId::generate(), old_spec);
        let old = ExecutionBinding::for_work(&old_work, "manifest:a", "dsh", "1");

        let mut next_work = old_work.clone();
        let mut next_spec = next_work.spec.clone();
        next_spec.profile_ref = "factory/v2".to_string();
        next_work.replace_spec(next_spec);

        let (next, decision) = old.rebind_for_work(
            &next_work,
            BindingMigrationRequest::new(
                "manifest:b",
                "pi",
                "2",
                BindingMigrationReason::ProfileUpgrade,
                "operator",
            )
            .with_evidence(vec!["evaluation:profile-v2".to_string()]),
        );

        assert_eq!(next.migration_from, Some(old.id.clone()));
        assert_eq!(next.work_generation, next_work.generation);
        assert_eq!(next.profile_ref, "factory/v2");
        assert_eq!(decision.from_binding, old.id);
        assert_eq!(decision.to_binding, next.id);
        assert_eq!(decision.from_provider, "dsh");
        assert_eq!(decision.to_provider, "pi");
        assert_eq!(decision.from_profile, "factory/v1");
        assert_eq!(decision.to_profile, "factory/v2");
        assert_eq!(decision.from_site_ref.as_deref(), Some("plant-a"));
        assert_eq!(decision.to_site_ref.as_deref(), Some("plant-a"));
    }

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
