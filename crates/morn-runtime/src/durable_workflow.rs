//! Durable workflow runtime boundary.
//!
//! Workflow engines (the legacy Morn DurableRuntime, Temporal/Dapr-class
//! providers, etc.) are execution providers. Their run/task state is runtime
//! evidence and must never become canonical Morn Work/Outcome/Acceptance truth.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{Id, RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_work::control::WorkResource;
use morn_work::workflow::{RunStatus, WorkflowRun};

use crate::ExecutionBinding;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DurableWorkflowBindingTag;
pub type DurableWorkflowBindingId = Id<DurableWorkflowBindingTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableWorkflowBinding {
    pub id: DurableWorkflowBindingId,
    pub work_id: WorkPackageId,
    pub work_generation: u64,
    pub execution_binding_id: RuntimeBindingId,
    pub provider_ref: String,
    pub workflow_run_ref: String,
    pub workflow_definition_ref: Option<String>,
    pub created_at: Timestamp,
}

impl DurableWorkflowBinding {
    pub fn new(
        work: &WorkResource,
        execution_binding: &ExecutionBinding,
        provider_ref: impl Into<String>,
        workflow_run_ref: impl Into<String>,
    ) -> Result<Self> {
        if execution_binding.work_id != work.id
            || execution_binding.work_generation != work.generation
        {
            return Err(Error::validation(
                "durable workflow binding must target the same Work generation as ExecutionBinding",
            ));
        }
        let provider_ref = provider_ref.into();
        let workflow_run_ref = workflow_run_ref.into();
        if provider_ref.trim().is_empty() || workflow_run_ref.trim().is_empty() {
            return Err(Error::validation(
                "durable workflow binding requires provider and workflow run references",
            ));
        }
        Ok(Self {
            id: DurableWorkflowBindingId::generate_with("workflow-binding"),
            work_id: work.id.clone(),
            work_generation: work.generation,
            execution_binding_id: execution_binding.id.clone(),
            provider_ref,
            workflow_run_ref,
            workflow_definition_ref: None,
            created_at: Timestamp::now(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DurableWorkflowState {
    Draft,
    Ready,
    Running,
    Waiting,
    Paused,
    RetryScheduled,
    Compensating,
    Blocked,
    Escalated,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

impl From<RunStatus> for DurableWorkflowState {
    fn from(value: RunStatus) -> Self {
        match value {
            RunStatus::Draft => Self::Draft,
            RunStatus::Ready => Self::Ready,
            RunStatus::Running => Self::Running,
            RunStatus::WaitingSignal | RunStatus::WaitingApproval => Self::Waiting,
            RunStatus::Paused => Self::Paused,
            RunStatus::RetryScheduled => Self::RetryScheduled,
            RunStatus::Compensating => Self::Compensating,
            RunStatus::Blocked => Self::Blocked,
            RunStatus::Escalated => Self::Escalated,
            RunStatus::Completed => Self::Completed,
            RunStatus::Failed => Self::Failed,
            RunStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableWorkflowEvidence {
    pub binding_id: DurableWorkflowBindingId,
    pub execution_binding_id: RuntimeBindingId,
    pub provider_ref: String,
    pub workflow_run_ref: String,
    pub state: DurableWorkflowState,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
}

impl DurableWorkflowEvidence {
    pub fn from_legacy_run(binding: &DurableWorkflowBinding, run: &WorkflowRun) -> Result<Self> {
        if binding.workflow_run_ref != run.id.to_string() {
            return Err(Error::validation(
                "workflow evidence run does not match durable workflow binding",
            ));
        }
        Ok(Self {
            binding_id: binding.id.clone(),
            execution_binding_id: binding.execution_binding_id.clone(),
            provider_ref: binding.provider_ref.clone(),
            workflow_run_ref: run.id.to_string(),
            state: run.status.into(),
            current_step: run.current_step.clone(),
            completed_steps: run.completed_steps.clone(),
            evidence_refs: vec![format!("workflow-run://{}", run.id)],
            observed_at: run.updated_at,
        })
    }

    pub fn is_executor_terminal(&self) -> bool {
        matches!(
            self.state,
            DurableWorkflowState::Completed
                | DurableWorkflowState::Failed
                | DurableWorkflowState::Cancelled
        )
    }

    /// Workflow completion is executor/runtime evidence only. Source-grounded
    /// Outcome + independent Acceptance are required to close Morn Work.
    pub const fn proves_morn_acceptance(&self) -> bool {
        false
    }
}

pub trait DurableWorkflowProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn inspect(&self, binding: &DurableWorkflowBinding) -> Result<DurableWorkflowEvidence>;
    fn signal(
        &mut self,
        binding: &DurableWorkflowBinding,
        signal_type: &str,
        payload_ref: &str,
    ) -> Result<()>;
    fn cancel(&mut self, binding: &DurableWorkflowBinding, reason: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkflowDefinitionId, WorkspaceId};
    use morn_kernel::version::Version;
    use morn_work::control::WorkSpec;

    #[test]
    fn completed_workflow_run_is_not_accepted_work() {
        let work_id = WorkPackageId::generate_with("work");
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(work_id, "review outage", "morn.factory.readonly@1.0.0"),
        );
        let execution = ExecutionBinding::for_work(&work, "cap:a", "workflow:a", "1");
        let mut run = WorkflowRun::new(
            WorkflowDefinitionId::generate_with("wf"),
            Version::v1(),
            work.workspace_id.clone(),
        );
        run.status = RunStatus::Completed;

        let binding = DurableWorkflowBinding::new(
            &work,
            &execution,
            "legacy-morn-durable-runtime",
            run.id.to_string(),
        )
        .unwrap();
        let evidence = DurableWorkflowEvidence::from_legacy_run(&binding, &run).unwrap();

        assert!(evidence.is_executor_terminal());
        assert!(!evidence.proves_morn_acceptance());
        assert_eq!(evidence.execution_binding_id, execution.id);
    }

    #[test]
    fn workflow_binding_cannot_cross_work_generation() {
        let work_id = WorkPackageId::generate_with("work");
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(work_id, "goal", "morn.lite@1.0.0"),
        );
        let old = ExecutionBinding::for_work(&work, "cap:a", "workflow:a", "1");
        let mut new_spec = work.spec.clone();
        new_spec.goal = "changed".to_string();
        work.replace_spec(new_spec);

        assert!(DurableWorkflowBinding::new(
            &work,
            &old,
            "workflow-provider",
            "run-1"
        )
        .is_err());
    }
}
