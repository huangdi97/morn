//! Workflow definitions, steps and durable workflow runs.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{WorkflowDefinitionId, WorkflowRunId, WorkflowStepId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// What kind of work a workflow step performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkflowStepKind {
    Auto,
    ManualWait,
    ExternalWait,
    ApprovalWait,
    SignalWait,
}

/// A single step in a workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: WorkflowStepId,
    pub name: String,
    pub kind: WorkflowStepKind,
    pub depends_on: Vec<WorkflowStepId>,
    pub timeout_secs: Option<u64>,
    pub retryable: bool,
    pub max_attempts: u32,
    pub compensation_ref: Option<String>,
}

impl WorkflowStep {
    pub fn new(name: impl Into<String>, kind: WorkflowStepKind) -> Self {
        Self {
            id: WorkflowStepId::generate_with("wstep"),
            name: name.into(),
            kind,
            depends_on: Vec::new(),
            timeout_secs: None,
            retryable: true,
            max_attempts: 1,
            compensation_ref: None,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn with_retry(mut self, max_attempts: u32) -> Self {
        self.retryable = true;
        self.max_attempts = max_attempts;
        self
    }
}

/// A versioned workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: WorkflowDefinitionId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub version: Version,
    pub steps: Vec<WorkflowStep>,
    pub created_at: Timestamp,
}

impl WorkflowDefinition {
    pub fn new(workspace_id: WorkspaceId, name: impl Into<String>) -> Self {
        Self {
            id: WorkflowDefinitionId::generate_with("wfdef"),
            workspace_id,
            name: name.into(),
            version: Version::v1(),
            steps: Vec::new(),
            created_at: Timestamp::now(),
        }
    }

    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn step(&self, id: &WorkflowStepId) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id == *id)
    }
}

/// Status of a durable workflow run (state machine per GOAL2_DURABLE_RUNTIME_SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RunStatus {
    Draft,
    Ready,
    Running,
    WaitingSignal,
    WaitingApproval,
    Paused,
    RetryScheduled,
    Compensating,
    Blocked,
    Escalated,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn can_transition_to(self, next: RunStatus) -> bool {
        use RunStatus::*;
        matches!(
            (self, next),
            (Draft, Ready)
                | (Ready, Running)
                | (Ready, Cancelled)
                | (Running, WaitingSignal)
                | (Running, WaitingApproval)
                | (Running, Paused)
                | (Running, RetryScheduled)
                | (Running, Compensating)
                | (Running, Blocked)
                | (Running, Escalated)
                | (Running, Completed)
                | (Running, Failed)
                | (Running, Cancelled)
                | (WaitingSignal, Running)
                | (WaitingSignal, Paused)
                | (WaitingSignal, Blocked)
                | (WaitingSignal, Cancelled)
                | (WaitingApproval, Running)
                | (WaitingApproval, Paused)
                | (WaitingApproval, Blocked)
                | (WaitingApproval, Cancelled)
                | (Paused, Running)
                | (Paused, Cancelled)
                | (RetryScheduled, Running)
                | (RetryScheduled, Failed)
                | (Compensating, Completed)
                | (Compensating, Failed)
                | (Escalated, Running)
                | (Escalated, Blocked)
                | (Escalated, Cancelled)
                | (Blocked, Escalated)
                | (Blocked, Running)
                | (Blocked, Cancelled)
        )
    }
}

/// A running instance of a workflow definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: WorkflowRunId,
    pub definition_id: WorkflowDefinitionId,
    pub definition_version: Version,
    pub workspace_id: WorkspaceId,
    pub status: RunStatus,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
    pub attempts: u32,
    pub budget_consumed: f64,
    pub budget_limit: f64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl WorkflowRun {
    pub fn new(
        definition_id: WorkflowDefinitionId,
        definition_version: Version,
        workspace_id: WorkspaceId,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: WorkflowRunId::generate_with("wrun"),
            definition_id,
            definition_version,
            workspace_id,
            status: RunStatus::Draft,
            current_step: None,
            completed_steps: Vec::new(),
            pending_steps: Vec::new(),
            attempts: 0,
            budget_consumed: 0.0,
            budget_limit: 1.0e15,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, next: RunStatus) -> morn_kernel::error::Result<()> {
        if !self.status.can_transition_to(next) {
            return Err(morn_kernel::error::Error::invalid_state(format!(
                "run {} cannot transition from {:?} to {:?}",
                self.id, self.status, next
            )));
        }
        self.status = next;
        self.updated_at = Timestamp::now();
        Ok(())
    }
}
