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
