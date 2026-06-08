//! execution — Supervises execution plans and emits task lifecycle events.
use crate::core::event_bus::{
    EVENT_AGENT_CREATED, EVENT_AGENT_DESTROYED, EVENT_SUPERVISOR_PLAN_CREATED,
    EVENT_TASK_COMPLETED, EVENT_TASK_FAILED, EVENT_WORKFLOW_COMPLETED, EVENT_WORKFLOW_FAILED,
    EVENT_WORKFLOW_STARTED,
};
use crate::core::storage::{DecisionRecord, DecisionRule, TaskRecord};
use tracing;

use super::{
    DecisionLevel, DecisionOverride, Mode, NLAgentDef, SubTaskDef, SubTaskResult, Supervisor,
    TaskPlan, TaskResult,
};

impl Supervisor {
    /// Executes a task plan with the provided chat function and returns the completed task result.
    pub fn execute_plan(
        &mut self,
        plan: &TaskPlan,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<TaskResult, String> {
        self.turn_count += 1;

        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_WORKFLOW_STARTED,
                "supervisor",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "decision_level": plan.decision_level,
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

    fn requires_decision_point(&self, plan: &TaskPlan) -> bool {
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

    /// Converts a natural-language agent request into an agent definition using the provided chat function.
    pub fn create_agent_from_nl(
        &self,
        nl: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    ) -> Result<NLAgentDef, String> {
        let system_prompt = "You are an agent configuration assistant. Analyze the user's natural language description and return a JSON object with the agent definition. Only return valid JSON, no markdown, no explanation.";

        let prompt = format!(
            r#"User wants to create an agent. Analyze this description:
{}
Available personas: assistant, analyst, researcher, writer, coder, translator, reviewer
Available models: deepseek-chat, deepseek-reasoner
Available tools: web_search, read_file, write_file, exec_python, calc, get_time, get_kline, calc_macd, chart
Available knowledge: docs, glossary, data_sources
Available skills: summarization, translation, code_review, grammar_check, format, style, proofread, report_generation, debug, test

Return a JSON object with exactly these fields (all strings or string arrays):
{{
  "name": "short agent name (2-5 words)",
  "persona": "most appropriate persona from the list above",
  "model": "deepseek-chat",
  "tools": ["list", "of", "tool", "names"],
  "knowledge": ["list", "of", "knowledge", "sources"],
  "skills": ["list", "of", "skills"]
}}
Select tools, knowledge, and skills that best match the user's described use case."#,
            nl
        );

        let response = chat_fn(&prompt, system_prompt)?;

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str::<NLAgentDef>(cleaned).map_err(|e| {
            format!(
                "Failed to parse LLM response as AgentDef: {}. Raw: {}",
                e, cleaned
            )
        })
    }

    /// Learns a decision rule from user feedback and returns success when storage updates complete.
    pub fn learn_from_feedback(&mut self, user_input: &str, approved: bool) -> Result<(), String> {
        let user_id = self.user_id.as_deref().unwrap_or("default").to_string();
        let keywords = Self::extract_keywords(user_input);
        if keywords.is_empty() {
            return Ok(());
        }
        let keyword = keywords[0].clone();
        let level = self.decide_level(user_input).as_str().to_string();

        if let Some(ref storage) = self.storage {
            let existing = storage
                .get_decision_rules(&user_id, &keyword)
                .unwrap_or_default();
            if let Some(rule) = existing.first() {
                let change = if approved { -10.0 } else { 15.0 };
                if let Some(rule_id) = rule.id {
                    storage.adjust_rule_threshold(rule_id, change)?;
                }
            } else {
                let rule = DecisionRule {
                    id: None,
                    user_id: user_id.clone(),
                    keyword: keyword.clone(),
                    level,
                    trust_threshold: if approved { 50.0 } else { 75.0 },
                    auto_execute: approved,
                    source: "learned".to_string(),
                    hit_count: 1,
                    last_used_at: Some(chrono::Utc::now().to_rfc3339()),
                    created_at: None,
                };
                storage.upsert_decision_rule(&rule)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event_bus::{
        Event, SimpleEventBus, EVENT_AGENT_CREATED, EVENT_AGENT_DESTROYED, EVENT_TASK_FAILED,
        EVENT_WORKFLOW_COMPLETED, EVENT_WORKFLOW_FAILED, EVENT_WORKFLOW_STARTED,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, OnceLock};

    static WORKFLOW_STARTED_CALLS: AtomicUsize = AtomicUsize::new(0);
    static WORKFLOW_COMPLETED_CALLS: AtomicUsize = AtomicUsize::new(0);
    static WORKFLOW_FAILED_CALLS: AtomicUsize = AtomicUsize::new(0);
    static AGENT_CREATED_CALLS: AtomicUsize = AtomicUsize::new(0);
    static AGENT_DESTROYED_CALLS: AtomicUsize = AtomicUsize::new(0);
    static TASK_FAILED_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn event_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn reset_event_counts() {
        WORKFLOW_STARTED_CALLS.store(0, Ordering::SeqCst);
        WORKFLOW_COMPLETED_CALLS.store(0, Ordering::SeqCst);
        WORKFLOW_FAILED_CALLS.store(0, Ordering::SeqCst);
        AGENT_CREATED_CALLS.store(0, Ordering::SeqCst);
        AGENT_DESTROYED_CALLS.store(0, Ordering::SeqCst);
        TASK_FAILED_CALLS.store(0, Ordering::SeqCst);
    }

    fn workflow_started_handler(_event: Event) {
        WORKFLOW_STARTED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn workflow_completed_handler(_event: Event) {
        WORKFLOW_COMPLETED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn workflow_failed_handler(_event: Event) {
        WORKFLOW_FAILED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn agent_created_handler(_event: Event) {
        AGENT_CREATED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn agent_destroyed_handler(_event: Event) {
        AGENT_DESTROYED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn task_failed_handler(_event: Event) {
        TASK_FAILED_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn lifecycle_bus() -> SimpleEventBus {
        let mut bus = SimpleEventBus::new();
        bus.subscribe(EVENT_WORKFLOW_STARTED, workflow_started_handler);
        bus.subscribe(EVENT_WORKFLOW_COMPLETED, workflow_completed_handler);
        bus.subscribe(EVENT_WORKFLOW_FAILED, workflow_failed_handler);
        bus.subscribe(EVENT_AGENT_CREATED, agent_created_handler);
        bus.subscribe(EVENT_AGENT_DESTROYED, agent_destroyed_handler);
        bus.subscribe(EVENT_TASK_FAILED, task_failed_handler);
        bus
    }

    fn plan(decision_level: &str) -> TaskPlan {
        TaskPlan {
            task_id: "task-test".to_string(),
            user_input: "summarize this".to_string(),
            subtasks: vec![SubTaskDef {
                id: "main".to_string(),
                agent_id: "chat-agent".to_string(),
                action: "chat".to_string(),
                params: serde_json::json!({"input": "summarize this"}),
                depends_on: vec![],
            }],
            estimated_secs: 3,
            decision_level: decision_level.to_string(),
        }
    }

    #[test]
    fn execute_plan_dispatches_chat_task() {
        let mut supervisor = Supervisor::new(None, None);
        let result = supervisor
            .execute_plan(&plan("single_tool"), &|context, system| {
                assert!(context.contains("summarize this"));
                assert!(system.contains("helpful AI assistant"));
                Ok("done".to_string())
            })
            .unwrap();

        assert_eq!(result.task_id, "task-test");
        assert_eq!(result.summary, "done");
        assert_eq!(result.subtask_results.len(), 1);
        assert!(result.subtask_results[0].success);
    }

    #[test]
    fn execute_plan_updates_turn_state() {
        let mut supervisor = Supervisor::new(None, None);

        supervisor
            .execute_plan(&plan("single_agent"), &|_, _| Ok("reply".to_string()))
            .unwrap();

        assert_eq!(supervisor.turn_count(), 1);
        assert_eq!(supervisor.history().len(), 2);
        assert_eq!(supervisor.history()[0].role, "user");
        assert_eq!(supervisor.history()[1].content, "reply");
    }

    #[test]
    fn execute_plan_propagates_chat_error() {
        let mut supervisor = Supervisor::new(None, None);
        let err = supervisor
            .execute_plan(&plan("workflow"), &|_, _| Err("model failed".to_string()))
            .unwrap_err();

        assert_eq!(err, "model failed");
        assert_eq!(supervisor.turn_count(), 1);
        assert!(supervisor.history().is_empty());
    }

    #[test]
    fn execute_plan_safe_mode_still_returns_result_after_approval_preview() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.set_mode(Mode::Safe);

        let result = supervisor
            .execute_plan(&plan("team"), &|_, _| Ok("approved result".to_string()))
            .unwrap();

        assert_eq!(result.summary, "approved result");
        assert_eq!(supervisor.mode(), &Mode::Safe);
    }

    #[test]
    fn execute_chat_builds_single_task_plan() {
        let mut supervisor = Supervisor::new(None, None);
        let reply = supervisor
            .execute_chat("search docs", &|context, _| {
                assert!(context.contains("search docs"));
                Ok("searched".to_string())
            })
            .unwrap();

        assert_eq!(reply, "searched");
        assert_eq!(supervisor.turn_count(), 1);
    }

    #[test]
    fn execute_chat_honors_inline_level_override() {
        let mut supervisor = Supervisor::new(None, None);
        let reply = supervisor
            .execute_chat("L4: hello", &|context, _| {
                assert!(context.contains("hello"));
                assert!(!context.contains("L4:"));
                Ok("team route".to_string())
            })
            .unwrap();

        assert_eq!(reply, "team route");
    }

    #[test]
    fn automated_mode_auto_approves_decision_records() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.set_mode(Mode::Automated);

        assert!(!supervisor.requires_decision_point(&plan("single_tool")));
        assert!(supervisor.requires_decision_point(&plan("workflow")));
    }

    #[test]
    fn execute_plan_feeds_learning_engine_after_success() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let mut supervisor = Supervisor::new(Some(storage.clone()), None);

        supervisor
            .execute_plan(&plan("single_tool"), &|_, _| Ok("learned".to_string()))
            .unwrap();

        let rules = storage.get_decision_rules("default", "summarize").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].level, "single_tool");
    }

    #[test]
    fn execute_plan_publishes_success_lifecycle_events() {
        let _guard = event_test_lock().lock().unwrap();
        reset_event_counts();
        let mut supervisor = Supervisor::new(None, Some(lifecycle_bus()));

        supervisor
            .execute_plan(&plan("single_tool"), &|_, _| Ok("done".to_string()))
            .unwrap();

        assert_eq!(WORKFLOW_STARTED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_CREATED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_DESTROYED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_COMPLETED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_FAILED_CALLS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn execute_plan_publishes_failure_lifecycle_events() {
        let _guard = event_test_lock().lock().unwrap();
        reset_event_counts();
        let mut supervisor = Supervisor::new(None, Some(lifecycle_bus()));

        let err = supervisor
            .execute_plan(&plan("single_tool"), &|_, _| Err("model failed".to_string()))
            .unwrap_err();

        assert_eq!(err, "model failed");
        assert_eq!(WORKFLOW_STARTED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_CREATED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_DESTROYED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(TASK_FAILED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_FAILED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_COMPLETED_CALLS.load(Ordering::SeqCst), 0);
    }
}
