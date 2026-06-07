use super::TaskEngine;
use crate::core::event_bus::{
    EVENT_SUPERVISOR_PLAN_EXECUTING, EVENT_TASK_COMPLETED, EVENT_TASK_FAILED,
};
use crate::core::supervisor::{SubTaskDef, SubTaskResult, TaskPlan, TaskResult};

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
