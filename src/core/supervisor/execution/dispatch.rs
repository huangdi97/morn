//! dispatch — Task execution, lifecycle events, and inline chat dispatch.
use crate::core::event_bus::{
    EVENT_AGENT_CREATED, EVENT_AGENT_DESTROYED, EVENT_SUPERVISOR_PLAN_CREATED,
    EVENT_TASK_COMPLETED, EVENT_TASK_FAILED, EVENT_WORKFLOW_COMPLETED, EVENT_WORKFLOW_FAILED,
    EVENT_WORKFLOW_STARTED,
};
use crate::core::storage::{DecisionRecord, TaskRecord};
use tracing;

use crate::core::supervisor::{
    DecisionLevel, DecisionOverride, Mode, SubTaskDef, SubTaskResult, Supervisor, TaskPlan,
    TaskResult,
};
use super::{classify_execution_time, ExecutionTier};

impl Supervisor {
    /// Executes a task plan with the provided chat function and returns the completed task result.
    pub fn execute_plan(
        &mut self,
        plan: &TaskPlan,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<TaskResult, String> {
        self.turn_count += 1;

        let tier = classify_execution_time(plan.estimated_secs);
        if tier == ExecutionTier::Background {
            tracing::info!(
                "[COO] Background execution: {} (est. {}s)",
                plan.task_id,
                plan.estimated_secs
            );
        }

        self.audit_log.append(
            "supervisor",
            "plan_execute",
            &serde_json::json!({
                "task_id": plan.task_id,
                "decision_level": plan.decision_level,
                "user_input": plan.user_input,
                "mode": self.mode.as_str(),
                "execution_tier": format!("{:?}", tier),
            }).to_string(),
        );

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_WORKFLOW_STARTED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "decision_level": plan.decision_level,
                    "execution_tier": format!("{:?}", tier),
                }),
            );
            bus.publish_event(
                EVENT_SUPERVISOR_PLAN_CREATED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "user_input": plan.user_input,
                    "decision_level": plan.decision_level,
                    "mode": self.mode.as_str(),
                }),
            );
            bus.publish_event(
                EVENT_AGENT_CREATED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "agent_id": "chat-agent",
                }),
            );
        }

        if let Some(ref storage) = self.storage {
            let task_record = TaskRecord {
                id: plan.task_id.clone(),
                user_input: plan.user_input.clone(),
                plan_json: serde_json::to_string(plan).unwrap_or_default(),
                status: "executing".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
            };
            let _ = storage.insert_task(&task_record);

            let decision = DecisionRecord {
                id: format!("dec-{}", uuid::Uuid::new_v4()),
                task_id: plan.task_id.clone(),
                decision_level: plan.decision_level.clone(),
                action: format!("execute with {} subtasks", plan.subtasks.len()),
                context_json: Some(
                    serde_json::json!({
                        "mode": self.mode.as_str(),
                        "estimated_secs": plan.estimated_secs,
                        "decision_point": self.requires_decision_point(plan),
                    })
                    .to_string(),
                ),
                approved: self.mode == Mode::Automated
                    || (self.mode == Mode::Proactive && !self.requires_decision_point(plan)),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = storage.insert_decision(&decision);
        }

        if self.mode == Mode::Safe && self.requires_decision_point(plan) {
            let preview = self.build_context(&plan.user_input);
            tracing::info!("[COO Safe Mode] Plan requires approval:");
            tracing::info!("  Level: {}", plan.decision_level);
            tracing::info!("  Subtasks: {}", plan.subtasks.len());
            tracing::info!("  Estimated: {}s", plan.estimated_secs);
            tracing::info!("  Preview: {}...", &preview[..preview.len().min(200)]);
        }

        let context = self.build_context(&plan.user_input);

        let response = match chat_fn(&context, "You are Morn, a helpful AI assistant.") {
            Ok(response) => response,
            Err(err) => {
                if let Some(ref storage) = self.storage {
                    let _ = storage.update_task_status(&plan.task_id, "failed");
                }
                if let Some(ref bus) = self.event_bus {
                    bus.publish_event(
                        EVENT_TASK_FAILED,
                        "supervisor",
                        serde_json::json!({
                            "task_id": plan.task_id,
                            "error": err,
                        }),
                    );
                    bus.publish_event(
                        EVENT_AGENT_DESTROYED,
                        "supervisor",
                        serde_json::json!({
                            "task_id": plan.task_id,
                            "agent_id": "chat-agent",
                            "status": "failed",
                        }),
                    );
                    bus.publish_event(
                        EVENT_WORKFLOW_FAILED,
                        "supervisor",
                        serde_json::json!({
                            "task_id": plan.task_id,
                            "error": err,
                        }),
                    );
                }
                self.audit_log.append(
                    "supervisor",
                    "plan_failed",
                    &serde_json::json!({
                        "task_id": plan.task_id,
                        "error": err,
                    }).to_string(),
                );
                return Err(err);
            }
        };

        self.record_turn("user", &plan.user_input);
        self.record_turn("assistant", &response);

        let result = TaskResult {
            task_id: plan.task_id.clone(),
            subtask_results: vec![SubTaskResult {
                id: "main".to_string(),
                success: true,
                output: response.clone(),
                error: None,
            }],
            summary: response.clone(),
        };

        if let Some(ref storage) = self.storage {
            let _ = storage.update_task_status(&plan.task_id, "completed");
        }

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_TASK_COMPLETED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "summary": result.summary,
                }),
            );
            bus.publish_event(
                EVENT_AGENT_DESTROYED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "agent_id": "chat-agent",
                    "status": "completed",
                }),
            );
            bus.publish_event(
                EVENT_WORKFLOW_COMPLETED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "summary": result.summary,
                }),
            );
        }

        if let Some(engine) = &self.learning_engine {
            engine.ingest_decision(&plan.user_input, &plan.decision_level, true)?;
        }

        Ok(result)
    }

    /// Builds and executes a single-turn chat plan for the input, returning the response summary.
    pub fn execute_chat(
        &mut self,
        input: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<String, String> {
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let (inline_override, clean_input) = match DecisionOverride::parse_prefixed(input) {
            Some((override_, clean_input)) => (Some(override_), clean_input),
            None => (None, input.to_string()),
        };

        if let Some(override_) = inline_override {
            self.override_decision(override_.level, override_.scope);
        }

        let (level, _reasoning) = match self.take_next_turn_override() {
            Some(override_) => (
                override_.level,
                "Decision override applied before automatic routing".to_string(),
            ),
            None => self.decide_with_rules(&clean_input),
        };

        let plan = TaskPlan {
            task_id: task_id.clone(),
            user_input: clean_input.clone(),
            subtasks: vec![SubTaskDef {
                id: "main".to_string(),
                agent_id: "chat-agent".to_string(),
                action: "chat".to_string(),
                params: serde_json::json!({"input": clean_input}),
                depends_on: vec![],
            }],
            estimated_secs: match level {
                DecisionLevel::L1DirectAnswer => 1,
                DecisionLevel::L2SingleTool => 3,
                DecisionLevel::L3SingleAgent => 10,
                DecisionLevel::L4Team => 30,
                DecisionLevel::L5Workflow => 20,
                DecisionLevel::L6JumpToStudio => 60,
            },
            decision_level: level.as_str().to_string(),
        };

        let result = self.execute_plan(&plan, chat_fn)?;
        Ok(result.summary)
    }

    pub(crate) fn requires_decision_point(&self, plan: &TaskPlan) -> bool {
        let high_level = matches!(
            plan.decision_level.as_str(),
            "team" | "workflow" | "jump_studio"
        );
        let high_risk_action = plan.subtasks.iter().any(|task| {
            let action = task.action.to_lowercase();
            action.contains("delete")
                || action.contains("deploy")
                || action.contains("publish")
                || action.contains("payment")
        });
        high_level || high_risk_action
    }
}