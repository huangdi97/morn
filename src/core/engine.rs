use crate::core::event_bus::{
    SimpleEventBus, EVENT_SUPERVISOR_PLAN_EXECUTING, EVENT_TASK_COMPLETED, EVENT_TASK_FAILED,
};
use crate::core::storage::Storage;
use crate::core::supervisor::{SubTaskDef, SubTaskResult, TaskPlan, TaskResult};
use std::collections::{HashMap, VecDeque};

pub struct TaskEngine {
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl TaskEngine {
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        TaskEngine { storage, event_bus }
    }

    pub fn run_plan(
        &self,
        plan: &TaskPlan,
        execute_fn: &mut dyn FnMut(&TaskPlan) -> Result<TaskResult, String>,
    ) -> Result<TaskResult, String> {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_SUPERVISOR_PLAN_EXECUTING,
                "engine",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "subtask_count": plan.subtasks.len(),
                }),
            );
        }

        let result = execute_fn(plan)?;

        if let Some(ref storage) = self.storage {
            let _ = storage.update_task_status(&plan.task_id, "completed");
        }

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_TASK_COMPLETED,
                "engine",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "summary": result.summary,
                }),
            );
        }

        Ok(result)
    }

    pub fn record_execution(
        &self,
        _plan: &TaskPlan,
        subtask: &SubTaskDef,
        result: &Result<String, String>,
    ) {
        if let Some(ref storage) = self.storage {
            let now = chrono::Utc::now().to_rfc3339();
            let status = match result {
                Ok(_) => "completed",
                Err(_) => "failed",
            };
            let result_json = match result {
                Ok(output) => Some(output.clone()),
                Err(e) => Some(e.clone()),
            };
            let _ = storage.insert_subtask(&crate::core::storage::SubtaskRecord {
                id: subtask.id.clone(),
                task_id: _plan.task_id.clone(),
                agent_id: subtask.agent_id.clone(),
                action: subtask.action.clone(),
                params_json: subtask.params.to_string(),
                status: status.to_string(),
                result_json,
                started_at: Some(now.clone()),
                finished_at: Some(now),
            });

            if result.is_err() {
                if let Some(ref bus) = self.event_bus {
                    bus.publish_event(
                        EVENT_TASK_FAILED,
                        "engine",
                        serde_json::json!({
                            "task_id": _plan.task_id,
                            "subtask_id": subtask.id,
                        }),
                    );
                }
            }
        }
    }

    pub fn compute_topological_order(
        &self,
        subtasks: &[SubTaskDef],
    ) -> Result<Vec<Vec<SubTaskDef>>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut subtask_map: HashMap<String, &SubTaskDef> = HashMap::new();

        for s in subtasks {
            in_degree.insert(s.id.clone(), 0);
            adj.insert(s.id.clone(), Vec::new());
            subtask_map.insert(s.id.clone(), s);
        }

        for s in subtasks {
            for dep in &s.depends_on {
                if let Some(children) = adj.get_mut(dep) {
                    children.push(s.id.clone());
                }
                *in_degree.entry(s.id.clone()).or_insert(0) += 1;
            }
        }

        let mut levels: Vec<Vec<SubTaskDef>> = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        for (id, deg) in in_degree.iter() {
            if *deg == 0 {
                queue.push_back(id.clone());
            }
        }

        let mut visited = 0;
        while !queue.is_empty() {
            let mut level = Vec::new();
            for _ in 0..queue.len() {
                if let Some(node) = queue.pop_front() {
                    if let Some(sub) = subtask_map.get(&node) {
                        level.push((*sub).clone());
                    }
                    visited += 1;
                    if let Some(children) = adj.get(&node) {
                        for child in children {
                            if let Some(deg) = in_degree.get_mut(child) {
                                *deg -= 1;
                                if *deg == 0 {
                                    queue.push_back(child.clone());
                                }
                            }
                        }
                    }
                }
            }
            if !level.is_empty() {
                levels.push(level);
            }
        }

        if visited != subtasks.len() {
            return Err("Circular dependency detected in subtask DAG".into());
        }

        Ok(levels)
    }

    pub fn run_dag_plan(
        &self,
        plan: &TaskPlan,
        execute_fn: &dyn Fn(&SubTaskDef) -> Result<String, String>,
        _timeout_secs: Option<u64>,
        max_retries: Option<u32>,
    ) -> Result<TaskResult, String> {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_SUPERVISOR_PLAN_EXECUTING,
                "engine",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "subtask_count": plan.subtasks.len(),
                    "mode": "dag",
                }),
            );
        }

        let levels = self.compute_topological_order(&plan.subtasks)?;
        let mut subtask_results = Vec::new();

        for level in &levels {
            let mut handles = Vec::new();
            for subtask in level {
                let max_retry = max_retries.unwrap_or(1);
                let mut last_error = String::new();
                let mut success = false;
                let mut output = String::new();

                for attempt in 0..=max_retry {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    let result = execute_fn(subtask);
                    match result {
                        Ok(resp) => {
                            success = true;
                            output = resp;
                            break;
                        }
                        Err(e) => {
                            last_error = e;
                        }
                    }
                }

                if success {
                    self.record_execution(plan, subtask, &Ok(output.clone()));
                    handles.push(SubTaskResult {
                        id: subtask.id.clone(),
                        success: true,
                        output: output.clone(),
                        error: None,
                    });
                } else {
                    self.record_execution(plan, subtask, &Err(last_error.clone()));
                    handles.push(SubTaskResult {
                        id: subtask.id.clone(),
                        success: false,
                        output: String::new(),
                        error: Some(last_error.clone()),
                    });
                    if let Some(ref bus) = self.event_bus {
                        bus.publish_event(
                            EVENT_TASK_FAILED,
                            "engine",
                            serde_json::json!({
                                "task_id": plan.task_id,
                                "subtask_id": subtask.id,
                                "error": last_error,
                            }),
                        );
                    }
                }
            }
            subtask_results.extend(handles);
        }

        if let Some(ref storage) = self.storage {
            let _ = storage.update_task_status(&plan.task_id, "completed");
        }

        let summary = subtask_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.output.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let result = TaskResult {
            task_id: plan.task_id.clone(),
            subtask_results,
            summary,
        };

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_TASK_COMPLETED,
                "engine",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "summary": result.summary,
                }),
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::supervisor::SubTaskDef;
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
        };

        let mut execute_fn = |_plan: &TaskPlan| -> Result<TaskResult, String> {
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
        };

        let execute = |sub: &SubTaskDef| -> Result<String, String> {
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
