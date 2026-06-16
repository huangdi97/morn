//! Group management — defines agent groups, workspaces, projects, and their lifecycle.

use crate::core::error::MornError;
use crate::core::orchestrator::CollaborationMode;

pub mod manager;
pub mod modes;

pub use manager::{GroupExecutor, GroupMetrics};

#[derive(Debug, Clone)]
pub struct AgentGroup {
    pub group_id: String,
    pub agent_ids: Vec<String>,
    pub mode: CollaborationMode,
    pub workspace_id: String,
    pub config: GroupConfig,
}

#[derive(Debug, Clone)]
pub struct GroupConfig {
    pub max_concurrency: usize,
    pub timeout_secs: u64,
    pub retry_on_failure: bool,
    pub approval_required: bool,
}

impl Default for GroupConfig {
    fn default() -> Self {
        GroupConfig {
            max_concurrency: 4,
            timeout_secs: 300,
            retry_on_failure: true,
            approval_required: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub workspace_id: String,
    pub name: String,
    pub groups: Vec<AgentGroup>,
    pub cron_tasks: Vec<CronTask>,
    pub shared_memory_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CronTask {
    pub id: String,
    pub expression: String,
    pub group_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleState {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub goal: String,
    pub workspaces: Vec<Workspace>,
    pub lifecycle_state: LifecycleState,
}

impl AgentGroup {
    pub fn new(
        group_id: &str,
        agent_ids: Vec<String>,
        mode: CollaborationMode,
        workspace_id: &str,
    ) -> Self {
        AgentGroup {
            group_id: group_id.to_string(),
            agent_ids,
            mode,
            workspace_id: workspace_id.to_string(),
            config: GroupConfig::default(),
        }
    }

    pub fn with_config(mut self, config: GroupConfig) -> Self {
        self.config = config;
        self
    }
}

impl Workspace {
    pub fn new(workspace_id: &str, name: &str) -> Self {
        Workspace {
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            groups: Vec::new(),
            cron_tasks: Vec::new(),
            shared_memory_id: None,
        }
    }

    pub fn add_group(&mut self, group: AgentGroup) {
        self.groups.push(group);
    }

    pub fn add_cron_task(&mut self, task: CronTask) {
        self.cron_tasks.push(task);
    }

    pub fn with_shared_memory(mut self, memory_id: &str) -> Self {
        self.shared_memory_id = Some(memory_id.to_string());
        self
    }
}

impl Project {
    pub fn new(project_id: &str, name: &str, goal: &str) -> Self {
        Project {
            project_id: project_id.to_string(),
            name: name.to_string(),
            goal: goal.to_string(),
            workspaces: Vec::new(),
            lifecycle_state: LifecycleState::Draft,
        }
    }

    pub fn add_workspace(&mut self, workspace: Workspace) {
        self.workspaces.push(workspace);
    }

    pub fn activate(&mut self) {
        self.lifecycle_state = LifecycleState::Active;
    }

    pub fn pause(&mut self) {
        self.lifecycle_state = LifecycleState::Paused;
    }

    pub fn complete(&mut self) {
        self.lifecycle_state = LifecycleState::Completed;
    }
}

#[cfg(test)]
mod tests;
