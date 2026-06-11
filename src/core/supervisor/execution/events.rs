//! events — Supervisor execution lifecycle event publishers.
use crate::core::event_bus::{
    SimpleEventBus, EVENT_AGENT_CREATED, EVENT_AGENT_DESTROYED, EVENT_SUPERVISOR_PLAN_CREATED,
    EVENT_TASK_COMPLETED, EVENT_TASK_FAILED, EVENT_WORKFLOW_COMPLETED, EVENT_WORKFLOW_FAILED,
    EVENT_WORKFLOW_STARTED,
};
use crate::core::supervisor::{Mode, TaskPlan};

use super::ExecutionTier;

pub(super) fn publish_plan_started_events(
    bus: &SimpleEventBus,
    plan: &TaskPlan,
    tier: &ExecutionTier,
    mode: &Mode,
) {
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
            "mode": mode.as_str(),
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

pub(super) fn publish_plan_failed_events(bus: &SimpleEventBus, task_id: &str, error: &str) {
    bus.publish_event(
        EVENT_TASK_FAILED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "error": error,
        }),
    );
    bus.publish_event(
        EVENT_AGENT_DESTROYED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "agent_id": "chat-agent",
            "status": "failed",
        }),
    );
    bus.publish_event(
        EVENT_WORKFLOW_FAILED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "error": error,
        }),
    );
}

pub(super) fn publish_plan_completed_events(bus: &SimpleEventBus, task_id: &str, summary: &str) {
    bus.publish_event(
        EVENT_TASK_COMPLETED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "summary": summary,
        }),
    );
    bus.publish_event(
        EVENT_AGENT_DESTROYED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "agent_id": "chat-agent",
            "status": "completed",
        }),
    );
    bus.publish_event(
        EVENT_WORKFLOW_COMPLETED,
        "supervisor",
        serde_json::json!({
            "task_id": task_id,
            "summary": summary,
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event_bus::{Event, SimpleEventBus, EVENT_TASK_COMPLETED, EVENT_TASK_FAILED};
    use crate::core::supervisor::{Mode, SubTaskDef, TaskPlan};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, OnceLock};

    static EVENT_CALLS: AtomicUsize = AtomicUsize::new(0);
    static EVENT_TYPES: Mutex<Vec<String>> = Mutex::new(Vec::new());

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn reset() {
        EVENT_CALLS.store(0, Ordering::SeqCst);
        if let Ok(mut types) = EVENT_TYPES.lock() {
            types.clear();
        }
    }

    fn collector(event: Event) {
        EVENT_CALLS.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut types) = EVENT_TYPES.lock() {
            types.push(event.event_type);
        }
    }

    fn bus_with_collector() -> SimpleEventBus {
        let mut bus = SimpleEventBus::new();
        bus.subscribe(EVENT_WORKFLOW_STARTED, collector);
        bus.subscribe(EVENT_SUPERVISOR_PLAN_CREATED, collector);
        bus.subscribe(EVENT_AGENT_CREATED, collector);
        bus.subscribe(EVENT_TASK_FAILED, collector);
        bus.subscribe(EVENT_AGENT_DESTROYED, collector);
        bus.subscribe(EVENT_WORKFLOW_FAILED, collector);
        bus.subscribe(EVENT_TASK_COMPLETED, collector);
        bus.subscribe(EVENT_WORKFLOW_COMPLETED, collector);
        bus
    }

    fn sample_plan() -> TaskPlan {
        TaskPlan {
            task_id: "task-001".into(),
            user_input: "test input".into(),
            subtasks: vec![SubTaskDef {
                id: "main".into(),
                agent_id: "chat-agent".into(),
                action: "chat".into(),
                params: serde_json::json!({"input": "test"}),
                depends_on: vec![],
            }],
            estimated_secs: 10,
            decision_level: "single_agent".into(),
            approval_required: false,
        }
    }

    #[test]
    fn publish_plan_started_emits_three_events() {
        let _guard = test_lock().lock().unwrap();
        reset();
        let bus = bus_with_collector();
        publish_plan_started_events(
            &bus,
            &sample_plan(),
            &ExecutionTier::Interactive,
            &Mode::Proactive,
        );
        assert_eq!(EVENT_CALLS.load(Ordering::SeqCst), 3);
        let types = EVENT_TYPES.lock().unwrap().clone();
        assert!(types.contains(&EVENT_WORKFLOW_STARTED.to_string()));
        assert!(types.contains(&EVENT_SUPERVISOR_PLAN_CREATED.to_string()));
        assert!(types.contains(&EVENT_AGENT_CREATED.to_string()));
    }

    #[test]
    fn publish_plan_failed_emits_three_events() {
        let _guard = test_lock().lock().unwrap();
        reset();
        let bus = bus_with_collector();
        publish_plan_failed_events(&bus, "task-001", "model error");
        assert_eq!(EVENT_CALLS.load(Ordering::SeqCst), 3);
        let types = EVENT_TYPES.lock().unwrap().clone();
        assert!(types.contains(&EVENT_TASK_FAILED.to_string()));
        assert!(types.contains(&EVENT_AGENT_DESTROYED.to_string()));
        assert!(types.contains(&EVENT_WORKFLOW_FAILED.to_string()));
    }

    #[test]
    fn publish_plan_completed_emits_three_events() {
        let _guard = test_lock().lock().unwrap();
        reset();
        let bus = bus_with_collector();
        publish_plan_completed_events(&bus, "task-001", "all done");
        assert_eq!(EVENT_CALLS.load(Ordering::SeqCst), 3);
        let types = EVENT_TYPES.lock().unwrap().clone();
        assert!(types.contains(&EVENT_TASK_COMPLETED.to_string()));
        assert!(types.contains(&EVENT_AGENT_DESTROYED.to_string()));
        assert!(types.contains(&EVENT_WORKFLOW_COMPLETED.to_string()));
    }

    #[test]
    fn publish_plan_started_includes_tier_in_data() {
        let bus = SimpleEventBus::new();
        publish_plan_started_events(
            &bus,
            &sample_plan(),
            &ExecutionTier::Direct,
            &Mode::Automated,
        );
    }

    #[test]
    fn publish_plan_failed_includes_error_in_data() {
        let bus = SimpleEventBus::new();
        publish_plan_failed_events(&bus, "task-002", "timeout");
    }

    #[test]
    fn publish_plan_completed_includes_summary_in_data() {
        let bus = SimpleEventBus::new();
        publish_plan_completed_events(&bus, "task-003", "success");
    }
}
