//! Desired/observed work resource used by the v11.5 durable control plane.
//!
//! This deliberately sits beside the existing WorkPackage/Workflow runtime:
//! a Work resource is business state, not an agent session or workflow cursor.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

fn default_persisted_resource_version() -> u64 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConditionStatus {
    Unknown,
    False,
    True,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkCondition {
    pub condition_type: String,
    pub status: ConditionStatus,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
}

impl WorkCondition {
    pub fn new(condition_type: impl Into<String>, status: ConditionStatus) -> Self {
        Self {
            condition_type: condition_type.into(),
            status,
            reason: String::new(),
            evidence_refs: Vec::new(),
            observed_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkPhase {
    Proposed,
    Resolving,
    Ready,
    Running,
    Waiting,
    Reconciling,
    Terminating,
    Blocked,
    Delivered,
    Accepted,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkSpec {
    pub work_package_id: WorkPackageId,
    pub goal: String,
    pub constraints: Vec<String>,
    pub required_conditions: Vec<String>,
    pub acceptance_ref: Option<String>,
    #[serde(default)]
    pub source_solution_ref: Option<String>,
    #[serde(default)]
    pub site_ref: Option<String>,
    pub profile_ref: String,
}

impl WorkSpec {
    pub fn new(
        work_package_id: WorkPackageId,
        goal: impl Into<String>,
        profile_ref: impl Into<String>,
    ) -> Self {
        Self {
            work_package_id,
            goal: goal.into(),
            constraints: Vec::new(),
            required_conditions: Vec::new(),
            acceptance_ref: None,
            source_solution_ref: None,
            site_ref: None,
            profile_ref: profile_ref.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkControlStatus {
    pub observed_generation: u64,
    pub phase: WorkPhase,
    pub conditions: Vec<WorkCondition>,
    /// Legacy/primary binding projection retained for compatibility.
    pub active_binding: Option<RuntimeBindingId>,
    /// All currently relevant bindings for a Workcell-style Work. A real Work
    /// may simultaneously bind an agent, solver, connector and human role.
    #[serde(default)]
    pub active_bindings: Vec<RuntimeBindingId>,
    pub last_event_seq: u64,
    pub updated_at: Timestamp,
}

impl Default for WorkControlStatus {
    fn default() -> Self {
        Self {
            observed_generation: 0,
            phase: WorkPhase::Proposed,
            conditions: Vec::new(),
            active_binding: None,
            active_bindings: Vec::new(),
            last_event_seq: 0,
            updated_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkResource {
    pub id: WorkPackageId,
    pub workspace_id: WorkspaceId,
    /// Persistence CAS revision. Newly constructed resources start at 0 and
    /// become 1 on first durable write. Older serialized v11.5 rows without
    /// this field deserialize as revision 1.
    #[serde(default = "default_persisted_resource_version")]
    pub resource_version: u64,
    pub generation: u64,
    pub spec: WorkSpec,
    pub status: WorkControlStatus,
    /// Graceful termination is monotonic. Once requested, it cannot be silently
    /// unset; finalizers must be cleared by the responsible controllers.
    #[serde(default)]
    pub termination_requested_at: Option<Timestamp>,
    #[serde(default)]
    pub termination_reason: Option<String>,
    #[serde(default)]
    pub finalizers: Vec<String>,
    pub created_at: Timestamp,
}

impl WorkResource {
    pub fn new(workspace_id: WorkspaceId, spec: WorkSpec) -> Self {
        Self {
            id: spec.work_package_id.clone(),
            workspace_id,
            resource_version: 0,
            generation: 1,
            spec,
            status: WorkControlStatus::default(),
            termination_requested_at: None,
            termination_reason: None,
            finalizers: Vec::new(),
            created_at: Timestamp::now(),
        }
    }

    /// Replacing desired state creates a new generation. Existing execution
    /// bindings stay pinned to the generation they were created against.
    pub fn replace_spec(&mut self, spec: WorkSpec) {
        self.spec = spec;
        self.generation += 1;
        self.status.updated_at = Timestamp::now();
    }

    pub fn set_condition(&mut self, condition: WorkCondition) {
        if let Some(existing) = self
            .status
            .conditions
            .iter_mut()
            .find(|item| item.condition_type == condition.condition_type)
        {
            *existing = condition;
        } else {
            self.status.conditions.push(condition);
        }
        self.status.updated_at = Timestamp::now();
    }

    pub fn condition_is_true(&self, name: &str) -> bool {
        self.status.conditions.iter().any(|condition| {
            condition.condition_type == name && condition.status == ConditionStatus::True
        })
    }

    pub fn required_conditions_satisfied(&self) -> bool {
        self.spec
            .required_conditions
            .iter()
            .all(|required| self.condition_is_true(required))
    }

    pub fn add_finalizer(
        &mut self,
        finalizer: impl Into<String>,
    ) -> morn_kernel::error::Result<()> {
        let finalizer = finalizer.into();
        if finalizer.trim().is_empty() || !finalizer.contains('/') {
            return Err(morn_kernel::error::Error::validation(
                "Work finalizer must be a qualified non-empty key",
            ));
        }
        if self.termination_requested_at.is_some() {
            return Err(morn_kernel::error::Error::invalid_state(
                "cannot add a new Work finalizer after termination is requested",
            ));
        }
        if !self.finalizers.iter().any(|existing| existing == &finalizer) {
            self.finalizers.push(finalizer);
            self.finalizers.sort();
        }
        self.status.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn request_termination(
        &mut self,
        reason: impl Into<String>,
    ) -> morn_kernel::error::Result<()> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(morn_kernel::error::Error::validation(
                "Work termination requires a reason",
            ));
        }
        if self.termination_requested_at.is_none() {
            self.termination_requested_at = Some(Timestamp::now());
            self.termination_reason = Some(reason);
        }
        self.status.phase = WorkPhase::Terminating;
        self.status.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn remove_finalizer(&mut self, finalizer: &str) -> bool {
        let before = self.finalizers.len();
        self.finalizers.retain(|existing| existing != finalizer);
        let removed = self.finalizers.len() != before;
        if removed {
            self.status.updated_at = Timestamp::now();
        }
        removed
    }

    pub fn can_finalize_termination(&self) -> bool {
        self.termination_requested_at.is_some() && self.finalizers.is_empty()
    }

    pub fn finalize_termination(&mut self) -> morn_kernel::error::Result<()> {
        if self.termination_requested_at.is_none() {
            return Err(morn_kernel::error::Error::invalid_state(
                "termination has not been requested",
            ));
        }
        if !self.finalizers.is_empty() {
            return Err(morn_kernel::error::Error::conflict(format!(
                "Work cannot terminate while finalizers remain: {:?}",
                self.finalizers
            )));
        }
        self.status.phase = WorkPhase::Cancelled;
        self.status.updated_at = Timestamp::now();
        Ok(())
    }

    pub fn record_active_binding(&mut self, binding_id: RuntimeBindingId) {
        if self.status.active_binding.is_none() {
            self.status.active_binding = Some(binding_id.clone());
        }
        if !self
            .status
            .active_bindings
            .iter()
            .any(|existing| existing == &binding_id)
        {
            self.status.active_bindings.push(binding_id);
        }
        self.status.updated_at = Timestamp::now();
    }

    pub fn remove_active_binding(&mut self, binding_id: &RuntimeBindingId) {
        self.status.active_bindings.retain(|existing| existing != binding_id);
        if self.status.active_binding.as_ref() == Some(binding_id) {
            self.status.active_binding = self.status.active_bindings.first().cloned();
        }
        self.status.updated_at = Timestamp::now();
    }

    pub fn mark_observed(&mut self) {
        self.status.observed_generation = self.generation;
        self.status.updated_at = Timestamp::now();
    }

    pub fn mark_persisted_revision(&mut self, revision: u64) {
        self.resource_version = revision;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn termination_is_monotonic_and_waits_for_finalizers() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "goal", "morn.enterprise@1.0.0");
        let mut resource = WorkResource::new(ws, spec);
        resource
            .add_finalizer("morn.io/external-effects")
            .unwrap();
        resource.request_termination("operator cancelled").unwrap();
        assert_eq!(resource.status.phase, WorkPhase::Terminating);
        assert!(resource.finalize_termination().is_err());
        assert!(resource
            .add_finalizer("morn.io/new-finalizer")
            .is_err());
        assert!(resource.remove_finalizer("morn.io/external-effects"));
        resource.finalize_termination().unwrap();
        assert_eq!(resource.status.phase, WorkPhase::Cancelled);
        assert!(resource.termination_requested_at.is_some());
    }

    #[test]
    fn work_can_track_multiple_active_bindings_without_parallel_truth() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "goal", "morn.lite@1.0.0");
        let mut resource = WorkResource::new(ws, spec);
        let a = RuntimeBindingId::generate_with("binding");
        let b = RuntimeBindingId::generate_with("binding");
        resource.record_active_binding(a.clone());
        resource.record_active_binding(b.clone());
        resource.record_active_binding(a.clone());
        assert_eq!(resource.status.active_bindings.len(), 2);
        assert_eq!(resource.status.active_binding, Some(a.clone()));
        resource.remove_active_binding(&a);
        assert_eq!(resource.status.active_binding, Some(b));
        assert_eq!(resource.status.active_bindings.len(), 1);
    }

    #[test]
    fn new_resource_starts_unpersisted_and_revision_is_store_managed() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let spec = WorkSpec::new(work_id, "goal", "morn.lite@1.0.0");
        let mut resource = WorkResource::new(ws, spec);
        assert_eq!(resource.resource_version, 0);
        resource.mark_persisted_revision(1);
        assert_eq!(resource.resource_version, 1);
    }

    #[test]
    fn site_scope_survives_generation_change() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let mut spec = WorkSpec::new(work_id, "investigate", "morn.factory.readonly@1.0.0");
        spec.site_ref = Some("plant-a".to_string());
        let mut resource = WorkResource::new(ws, spec.clone());

        spec.goal = "investigate and verify".to_string();
        resource.replace_spec(spec);
        assert_eq!(resource.spec.site_ref.as_deref(), Some("plant-a"));
    }

    #[test]
    fn source_solution_reference_survives_generation_change() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let mut spec = WorkSpec::new(work_id, "investigate", "morn.lite@1.0.0");
        spec.source_solution_ref = Some("solution://review@1.0.0".to_string());
        let mut resource = WorkResource::new(ws, spec.clone());
        assert_eq!(
            resource.spec.source_solution_ref.as_deref(),
            Some("solution://review@1.0.0")
        );

        spec.goal = "investigate and verify".to_string();
        resource.replace_spec(spec);
        assert_eq!(resource.generation, 2);
        assert_eq!(
            resource.spec.source_solution_ref.as_deref(),
            Some("solution://review@1.0.0")
        );
    }

    #[test]
    fn desired_and_observed_generation_are_distinct() {
        let ws = WorkspaceId::generate();
        let work_id = WorkPackageId::generate_with("wp");
        let mut spec = WorkSpec::new(work_id.clone(), "restore machine", "factory/v1");
        spec.required_conditions = vec!["AuthoritySatisfied".into(), "CapabilityResolved".into()];
        let mut resource = WorkResource::new(ws, spec.clone());

        resource.set_condition(WorkCondition::new(
            "AuthoritySatisfied",
            ConditionStatus::True,
        ));
        assert!(!resource.required_conditions_satisfied());
        resource.set_condition(WorkCondition::new(
            "CapabilityResolved",
            ConditionStatus::True,
        ));
        assert!(resource.required_conditions_satisfied());

        resource.mark_observed();
        assert_eq!(resource.status.observed_generation, 1);

        spec.goal = "restore machine and account for orders".into();
        resource.replace_spec(spec);
        assert_eq!(resource.generation, 2);
        assert_eq!(resource.status.observed_generation, 1);
    }
}
