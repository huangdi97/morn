//! dispatch — Task execution, lifecycle events, and inline chat dispatch.
use crate::bridge::chat_agent::ChatAgent;
use crate::core::storage::{DecisionRecord, TaskRecord};
use tracing;

use super::events::*;
use super::{classify_execution_level, classify_execution_time, ExecutionTier};
use crate::core::supervisor::{
    DecisionLevel, DecisionOverride, Mode, SubTaskDef, SubTaskResult, Supervisor, TaskPlan,
    TaskResult,
};

impl Supervisor {
    /// Executes a task plan with the provided chat function and returns the completed task result.
    pub fn execute_plan(
        &mut self,
        plan: &TaskPlan,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<TaskResult, String> {
        self.turn_count += 1;

        let tier = classify_execution_level(&plan.decision_level)
            .unwrap_or_else(|| classify_execution_time(plan.estimated_secs));
        if tier == ExecutionTier::Background {
            tracing::info!(
                "[COO] Background execution: {} (est. {}s)",
                plan.task_id,
                plan.estimated_secs
            );
        }
        if tier == ExecutionTier::Interactive {
            tracing::info!(
                "[COO] Interactive execution requires plan confirmation: {} (level {})",
                plan.task_id,
                plan.decision_level
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
            })
            .to_string(),
        );

        if let Some(ref bus) = self.event_bus {
            publish_plan_started_events(bus, plan, &tier, &self.mode);
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
            if let Err(e) = storage.insert_task(&task_record) {
                tracing::warn!("Failed to insert task: {}", e);
            }

            let decision = DecisionRecord {
                id: format!("dec-{}", uuid::Uuid::new_v4()),
                task_id: plan.task_id.clone(),
                decision_level: plan.decision_level.clone(),
                action: format!("execute with {} subtasks", plan.subtasks.len()),
                context_json: Some(
                    serde_json::json!({
                        "mode": self.mode.as_str(),
                        "estimated_secs": plan.estimated_secs,
                        "execution_tier": format!("{:?}", tier),
                        "decision_point": self.requires_decision_point(plan),
                    })
                    .to_string(),
                ),
                approved: self.mode == Mode::Automated
                    || (self.mode == Mode::Proactive && !self.requires_decision_point(plan)),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            if let Err(e) = storage.insert_decision(&decision) {
                tracing::warn!("Failed to insert decision: {}", e);
            }
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
                    if let Err(e) = storage.update_task_status(&plan.task_id, "failed") {
                        tracing::warn!("Failed to update task status: {}", e);
                    }
                }
                if let Some(ref bus) = self.event_bus {
                    publish_plan_failed_events(bus, &plan.task_id, &err);
                }
                self.audit_log.append(
                    "supervisor",
                    "plan_failed",
                    &serde_json::json!({
                        "task_id": plan.task_id,
                        "error": err,
                    })
                    .to_string(),
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
            if let Err(e) = storage.update_task_status(&plan.task_id, "completed") {
                tracing::warn!("Failed to update task status: {}", e);
            }
        }

        if let Some(ref bus) = self.event_bus {
            publish_plan_completed_events(bus, &plan.task_id, &result.summary);
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

        let routed_model = self.model_router.route(&clean_input)?;
        let routed_agent = match ChatAgent::from_route(&routed_model) {
            Ok(agent) => Some(agent),
            Err(err) => {
                tracing::debug!(
                    "[COO] Routed model {} via {} is not directly callable by ChatAgent: {}",
                    routed_model.name,
                    routed_model.provider,
                    err
                );
                None
            }
        };

        let (level, _reasoning) = match self.take_next_turn_override() {
            Some(override_) => (
                override_.level,
                "Decision override applied before automatic routing".to_string(),
            ),
            None => self.decide_with_rules(&clean_input),
        };

        let mut plan = TaskPlan {
            task_id: task_id.clone(),
            user_input: clean_input.clone(),
            subtasks: vec![SubTaskDef {
                id: "main".to_string(),
                agent_id: "chat-agent".to_string(),
                action: "chat".to_string(),
                params: serde_json::json!({
                    "input": clean_input,
                    "provider": routed_model.provider.clone(),
                    "model": routed_model.name.clone(),
                }),
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
            approval_required: false,
        };
        let routed_chat_fn = |context: &str, system: &str| -> Result<String, String> {
            if let Some(agent) = &routed_agent {
                agent.chat(context, system)
            } else {
                chat_fn(context, system)
            }
        };

        self.apply_dual_llm_check(&mut plan, &routed_chat_fn);

        let result = self.execute_plan(&plan, &routed_chat_fn)?;
        Ok(result.summary)
    }

    pub(crate) fn requires_decision_point(&self, plan: &TaskPlan) -> bool {
        if plan.approval_required {
            return true;
        }

        let high_level = matches!(
            plan.decision_level.as_str(),
            "team" | "workflow" | "jump_studio"
        );
        let interactive_level = matches!(plan.decision_level.as_str(), "single_agent");
        let high_risk_action = plan.subtasks.iter().any(|task| {
            let action = task.action.to_lowercase();
            action.contains("delete")
                || action.contains("deploy")
                || action.contains("publish")
                || action.contains("payment")
        });
        high_level || interactive_level || high_risk_action
    }
}

#[cfg(test)]
mod tests {
    use crate::core::supervisor::{SubTaskDef, Supervisor, TaskPlan};

    fn make_supervisor() -> Supervisor {
        Supervisor::new(None, None)
    }

    #[test]
    fn requires_decision_point_when_approval_required() {
        let sup = make_supervisor();
        let plan = TaskPlan {
            task_id: "t1".into(),
            user_input: "test".into(),
            subtasks: vec![],
            estimated_secs: 1,
            decision_level: "direct".into(),
            approval_required: true,
        };
        assert!(sup.requires_decision_point(&plan));
    }

    #[test]
    fn requires_decision_point_false_when_no_approval_needed() {
        let sup = make_supervisor();
        let plan = TaskPlan {
            task_id: "t2".into(),
            user_input: "hello".into(),
            subtasks: vec![SubTaskDef {
                id: "main".into(),
                agent_id: "agent".into(),
                action: "chat".into(),
                params: serde_json::json!({}),
                depends_on: vec![],
            }],
            estimated_secs: 1,
            decision_level: "direct".into(),
            approval_required: false,
        };
        assert!(!sup.requires_decision_point(&plan));
    }

    #[test]
    fn requires_decision_point_for_high_risk_action() {
        let sup = make_supervisor();
        let plan = TaskPlan {
            task_id: "t3".into(),
            user_input: "deploy now".into(),
            subtasks: vec![SubTaskDef {
                id: "main".into(),
                agent_id: "deployer".into(),
                action: "deploy".into(),
                params: serde_json::json!({}),
                depends_on: vec![],
            }],
            estimated_secs: 10,
            decision_level: "single_agent".into(),
            approval_required: false,
        };
        assert!(sup.requires_decision_point(&plan));
    }
}
