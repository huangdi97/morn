//! Desired/observed work resource used by the v11.5 durable control plane.
//!
//! This deliberately sits beside the existing WorkPackage/Workflow runtime:
//! a Work resource is business state, not an agent session or workflow cursor.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

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
            profile_ref: profile_ref.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkControlStatus {
    pub observed_generation: u64,
    pub phase: WorkPhase,
    pub conditions: Vec<WorkCondition>,
    pub active_binding: Option<RuntimeBindingId>,
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
            last_event_seq: 0,
            updated_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkResource {
    pub id: WorkPackageId,
    pub workspace_id: WorkspaceId,
    pub generation: u64,
    pub spec: WorkSpec,
    pub status: WorkControlStatus,
    pub created_at: Timestamp,
}

impl WorkResource {
    pub fn new(workspace_id: WorkspaceId, spec: WorkSpec) -> Self {
        Self {
            id: spec.work_package_id.clone(),
            workspace_id,
            generation: 1,
            spec,
            status: WorkControlStatus::default(),
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

    pub fn mark_observed(&mut self) {
        self.status.observed_generation = self.generation;
        self.status.updated_at = Timestamp::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
