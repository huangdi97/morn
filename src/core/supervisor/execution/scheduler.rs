//! scheduler — Schedules TaskPlan execution via WorkflowEngine.
use crate::core::supervisor::TaskPlan;

pub struct Scheduler;

impl Scheduler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Scheduler
    }

    #[allow(dead_code)]
    pub fn schedule(&self, _workflow_id: &str, plan: &TaskPlan) -> Result<Vec<String>, String> {
        // 骨架：返回 plan 中的 subtask ids
        let ids: Vec<String> = plan.subtasks.iter().map(|s| s.id.clone()).collect();
        if ids.is_empty() {
            return Err("no subtasks to schedule".to_string());
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::supervisor::{SubTaskDef, TaskPlan};

    fn make_plan(subtask_ids: Vec<&str>) -> TaskPlan {
        TaskPlan {
            task_id: "test-task".into(),
            user_input: "test".into(),
            subtasks: subtask_ids.iter().map(|id| SubTaskDef {
                id: id.to_string(),
                agent_id: "agent".into(),
                action: "chat".into(),
                params: serde_json::json!({}),
                depends_on: vec![],
            }).collect(),
            estimated_secs: 10,
            decision_level: "single_agent".into(),
            approval_required: false,
        }
    }

    #[test]
    fn scheduler_new_creates_instance() {
        let s = Scheduler::new();
        let ids = s.schedule("wf1", &make_plan(vec!["s1", "s2"])).unwrap();
        assert_eq!(ids, vec!["s1", "s2"]);
    }

    #[test]
    fn scheduler_returns_subtask_ids_in_order() {
        let s = Scheduler::new();
        let ids = s.schedule("wf1", &make_plan(vec!["a", "b", "c"])).unwrap();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn scheduler_fails_on_empty_subtasks() {
        let s = Scheduler::new();
        let result = s.schedule("wf1", &make_plan(vec![]));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "no subtasks to schedule");
    }
}
