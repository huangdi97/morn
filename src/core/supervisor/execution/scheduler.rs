//! scheduler — Schedules TaskPlan execution via WorkflowEngine.
use crate::core::supervisor::TaskPlan;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Scheduler
    }

    pub fn schedule(&self, _workflow_id: &str, plan: &TaskPlan) -> Result<Vec<String>, String> {
        // 骨架：返回 plan 中的 subtask ids
        let ids: Vec<String> = plan.subtasks.iter().map(|s| s.id.clone()).collect();
        if ids.is_empty() {
            return Err("no subtasks to schedule".to_string());
        }
        Ok(ids)
    }
}
