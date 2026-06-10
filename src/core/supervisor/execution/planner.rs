use crate::core::engine::TaskEngine;
use crate::core::supervisor::{SubTaskDef, TaskPlan};

pub struct Planner;

impl Planner {
    pub fn plan(intent: &str, subtasks: Vec<SubTaskDef>) -> Result<TaskPlan, String> {
        let engine = TaskEngine::new(None, None);
        let levels = engine.compute_topological_order(&subtasks)?;

        let mut flat = Vec::new();
        for level in &levels {
            flat.extend(level.iter().cloned());
        }

        let estimated_secs = flat.len() as u64;

        Ok(TaskPlan {
            task_id: format!("plan_{}", chrono::Utc::now().timestamp()),
            user_input: intent.to_string(),
            subtasks: flat,
            estimated_secs,
            decision_level: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn subtask(id: &str, depends_on: Vec<&str>) -> SubTaskDef {
        SubTaskDef {
            id: id.to_string(),
            agent_id: "agent-test".to_string(),
            action: "run".to_string(),
            params: json!({}),
            depends_on: depends_on.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn planner_generates_plan_with_no_deps() {
        let tasks = vec![subtask("a", vec![])];
        let plan = Planner::plan("test", tasks).unwrap();
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.user_input, "test");
    }

    #[test]
    fn planner_detects_cycle() {
        let tasks = vec![subtask("a", vec!["b"]), subtask("b", vec!["a"])];
        let result = Planner::plan("cycle", tasks);
        assert!(result.is_err());
    }

    #[test]
    fn planner_orders_by_dependency() {
        let tasks = vec![
            subtask("a", vec![]),
            subtask("b", vec!["a"]),
            subtask("c", vec!["b"]),
        ];
        let plan = Planner::plan("ordered", tasks).unwrap();
        assert_eq!(plan.subtasks.len(), 3);
        let ids: Vec<&str> = plan.subtasks.iter().map(|s| s.id.as_str()).collect();
        let pos_a = ids.iter().position(|&id| id == "a").unwrap();
        let pos_b = ids.iter().position(|&id| id == "b").unwrap();
        let pos_c = ids.iter().position(|&id| id == "c").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }
}
