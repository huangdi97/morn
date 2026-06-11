//! executor — Runs task engine steps and emits execution results.
use super::TaskEngine;
use crate::core::event_bus::{
    EVENT_SUPERVISOR_PLAN_EXECUTING, EVENT_TASK_COMPLETED, EVENT_TASK_FAILED,
};
use crate::core::supervisor::{SubTaskDef, SubTaskResult, TaskPlan, TaskResult};
use tracing;

impl TaskEngine {
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
            if let Err(e) = storage.update_task_status(&plan.task_id, "completed") {
                tracing::warn!("Failed to update task status: {}", e);
            }
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
            if let Err(e) = storage.insert_subtask(&crate::core::storage::SubtaskRecord {
                id: subtask.id.clone(),
                task_id: _plan.task_id.clone(),
                agent_id: subtask.agent_id.clone(),
                action: subtask.action.clone(),
                params_json: subtask.params.to_string(),
                status: status.to_string(),
                result_json,
                started_at: Some(now.clone()),
                finished_at: Some(now),
            }) {
                tracing::warn!("Failed to insert subtask: {}", e);
            }

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
            if let Err(e) = storage.update_task_status(&plan.task_id, "completed") {
                tracing::warn!("Failed to update task status: {}", e);
            }
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
    use crate::core::engine::TaskEngine;
    use serde_json::json;

    fn plan_with_subtasks(subtasks: Vec<SubTaskDef>) -> TaskPlan {
        TaskPlan {
            task_id: "task-test".to_string(),
            user_input: "test input".to_string(),
            subtasks,
            estimated_secs: 1,
            decision_level: "single_agent".to_string(),
            approval_required: false,
        }
    }

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
    fn run_plan_executes_empty_plan() {
        let engine = TaskEngine::new(None, None);
        let plan = plan_with_subtasks(vec![]);
        let mut execute_fn = |p: &TaskPlan| {
            Ok(TaskResult {
                task_id: p.task_id.clone(),
                subtask_results: vec![],
                summary: "empty".to_string(),
            })
        };

        let result = engine.run_plan(&plan, &mut execute_fn).unwrap();

        assert_eq!(result.task_id, "task-test");
        assert!(result.subtask_results.is_empty());
        assert_eq!(result.summary, "empty");
    }

    #[test]
    fn run_plan_executes_single_task_plan() {
        let engine = TaskEngine::new(None, None);
        let plan = plan_with_subtasks(vec![subtask("main", vec![])]);
        let mut execute_fn = |p: &TaskPlan| {
            Ok(TaskResult {
                task_id: p.task_id.clone(),
                subtask_results: vec![SubTaskResult {
                    id: "main".to_string(),
                    success: true,
                    output: "done".to_string(),
                    error: None,
                }],
                summary: "done".to_string(),
            })
        };

        let result = engine.run_plan(&plan, &mut execute_fn).unwrap();

        assert_eq!(result.subtask_results.len(), 1);
        assert_eq!(result.summary, "done");
    }

    #[test]
    fn run_plan_propagates_task_failure() {
        let engine = TaskEngine::new(None, None);
        let plan = plan_with_subtasks(vec![subtask("main", vec![])]);
        let mut execute_fn = |_p: &TaskPlan| Err("boom".to_string());

        let err = engine.run_plan(&plan, &mut execute_fn).unwrap_err();

        assert_eq!(err, "boom");
    }

    #[test]
    fn run_dag_plan_marks_failed_subtask() {
        let engine = TaskEngine::new(None, None);
        let plan = plan_with_subtasks(vec![subtask("main", vec![])]);
        let execute = |_subtask: &SubTaskDef| Err("failed".to_string());

        let result = engine.run_dag_plan(&plan, &execute, None, Some(0)).unwrap();

        assert_eq!(result.subtask_results.len(), 1);
        assert!(!result.subtask_results[0].success);
        assert_eq!(result.subtask_results[0].error.as_deref(), Some("failed"));
    }

    #[test]
    fn run_dag_plan_accepts_timeout_argument() {
        let engine = TaskEngine::new(None, None);
        let plan = plan_with_subtasks(vec![subtask("main", vec![])]);
        let execute = |subtask: &SubTaskDef| Ok(format!("{} ok", subtask.id));

        let result = engine
            .run_dag_plan(&plan, &execute, Some(1), Some(0))
            .unwrap();

        assert_eq!(result.summary, "main ok");
        assert!(result.subtask_results[0].success);
    }
}
