//! execution — Supervises execution plans and emits task lifecycle events.
use crate::core::error::MornError;
mod dispatch;
pub mod dual_llm;
pub mod events;
mod intent;
pub mod planner;
pub mod scheduler;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionTier {
    Direct,      // <3s: no confirmation needed
    Interactive, // 3-30s: show plan -> confirm -> progress
    Background,  // >30s: background -> notify on completion
}

pub fn classify_execution_level(decision_level: &str) -> Option<ExecutionTier> {
    match decision_level {
        "direct_answer" => Some(ExecutionTier::Direct),
        "single_agent" => Some(ExecutionTier::Interactive),
        "team" => Some(ExecutionTier::Background),
        _ => None,
    }
}

pub fn classify_execution_time(estimated_secs: u64) -> ExecutionTier {
    if estimated_secs < 3 {
        ExecutionTier::Direct
    } else if estimated_secs <= 30 {
        ExecutionTier::Interactive
    } else {
        ExecutionTier::Background
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

    use crate::core::supervisor::{Mode, SubTaskDef, Supervisor, TaskPlan};

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
            approval_required: false,
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
            .execute_plan(&plan("workflow"), &|_, _| Err(MornError::Internal("model failed".to_string())))
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
            .execute_plan(
                &plan("single_tool"),
                &|_, _| Err(MornError::Internal("model failed".to_string()))
            )
            .unwrap_err();

        assert_eq!(err, "model failed");
        assert_eq!(WORKFLOW_STARTED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_CREATED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(AGENT_DESTROYED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(TASK_FAILED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_FAILED_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(WORKFLOW_COMPLETED_CALLS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn classify_direct_tier_under_3s() {
        assert_eq!(classify_execution_time(0), ExecutionTier::Direct);
        assert_eq!(classify_execution_time(1), ExecutionTier::Direct);
        assert_eq!(classify_execution_time(2), ExecutionTier::Direct);
    }

    #[test]
    fn classify_interactive_tier_3_to_30s() {
        assert_eq!(classify_execution_time(3), ExecutionTier::Interactive);
        assert_eq!(classify_execution_time(15), ExecutionTier::Interactive);
        assert_eq!(classify_execution_time(30), ExecutionTier::Interactive);
    }

    #[test]
    fn classify_background_tier_over_30s() {
        assert_eq!(classify_execution_time(31), ExecutionTier::Background);
        assert_eq!(classify_execution_time(300), ExecutionTier::Background);
    }

    #[test]
    fn classify_level_overrides_known_execution_strategies() {
        assert_eq!(
            classify_execution_level("direct_answer"),
            Some(ExecutionTier::Direct)
        );
        assert_eq!(
            classify_execution_level("single_agent"),
            Some(ExecutionTier::Interactive)
        );
        assert_eq!(
            classify_execution_level("team"),
            Some(ExecutionTier::Background)
        );
        assert_eq!(classify_execution_level("workflow"), None);
    }
}
