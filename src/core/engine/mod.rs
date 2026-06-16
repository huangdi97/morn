//! engine — Defines task execution engines and workflow execution primitives.
use crate::core::error::MornError;
use crate::core::event_bus::SimpleEventBus;
use crate::core::storage::Storage;

mod dag;
mod executor;

pub struct TaskEngine {
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl TaskEngine {
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        TaskEngine { storage, event_bus }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::supervisor::{SubTaskDef, SubTaskResult, TaskPlan, TaskResult};
    use serde_json::json;

    #[test]
    fn test_engine_run_plan() {
        let engine = TaskEngine::new(None, None);
        let plan = TaskPlan {
            task_id: "test-1".to_string(),
            user_input: "hello".to_string(),
            subtasks: vec![SubTaskDef {
                id: "main".to_string(),
                agent_id: "agent-1".to_string(),
                action: "chat".to_string(),
                params: json!({"input": "hello"}),
                depends_on: vec![],
            }],
            estimated_secs: 5,
            decision_level: "single_agent".to_string(),
            approval_required: false,
        };

        let mut execute_fn = |_plan: &TaskPlan| -> Result<TaskResult, MornError> {
            Ok(TaskResult {
                task_id: "test-1".to_string(),
                subtask_results: vec![SubTaskResult {
                    id: "main".to_string(),
                    success: true,
                    output: "Hello!".to_string(),
                    error: None,
                }],
                summary: "Hello!".to_string(),
            })
        };

        let result = engine.run_plan(&plan, &mut execute_fn).unwrap();
        assert_eq!(result.summary, "Hello!");
    }

    #[test]
    fn test_topological_sort() {
        let engine = TaskEngine::new(None, None);
        let subtasks = vec![
            SubTaskDef {
                id: "a".into(),
                agent_id: "1".into(),
                action: "x".into(),
                params: json!({}),
                depends_on: vec![],
            },
            SubTaskDef {
                id: "b".into(),
                agent_id: "1".into(),
                action: "y".into(),
                params: json!({}),
                depends_on: vec!["a".into()],
            },
            SubTaskDef {
                id: "c".into(),
                agent_id: "1".into(),
                action: "z".into(),
                params: json!({}),
                depends_on: vec!["a".into()],
            },
        ];
        let levels = engine.compute_topological_order(&subtasks).unwrap();
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[0][0].id, "a");
        assert_eq!(levels[1].len(), 2);
    }

    #[test]
    fn test_circular_dependency() {
        let engine = TaskEngine::new(None, None);
        let subtasks = vec![
            SubTaskDef {
                id: "a".into(),
                agent_id: "1".into(),
                action: "x".into(),
                params: json!({}),
                depends_on: vec!["b".into()],
            },
            SubTaskDef {
                id: "b".into(),
                agent_id: "1".into(),
                action: "y".into(),
                params: json!({}),
                depends_on: vec!["a".into()],
            },
        ];
        assert!(engine.compute_topological_order(&subtasks).is_err());
    }

    #[test]
    fn test_dag_run() {
        let engine = TaskEngine::new(None, None);
        let plan = TaskPlan {
            task_id: "dag-test".into(),
            user_input: "test".into(),
            subtasks: vec![
                SubTaskDef {
                    id: "s1".into(),
                    agent_id: "1".into(),
                    action: "search".into(),
                    params: json!({"q": "ai"}),
                    depends_on: vec![],
                },
                SubTaskDef {
                    id: "s2".into(),
                    agent_id: "1".into(),
                    action: "summarize".into(),
                    params: json!({"text": "results"}),
                    depends_on: vec!["s1".into()],
                },
            ],
            estimated_secs: 10,
            decision_level: "single_agent".into(),
            approval_required: false,
        };

        let execute = |sub: &SubTaskDef| -> Result<String, MornError> {
            match sub.id.as_str() {
                "s1" => Ok("search results".into()),
                "s2" => Ok("summary".into()),
                _ => Err("unknown".into()),
            }
        };

        let result = engine.run_dag_plan(&plan, &execute, None, None).unwrap();
        assert_eq!(result.subtask_results.len(), 2);
        assert!(result.subtask_results[0].success);
        assert!(result.subtask_results[1].success);
    }
}
