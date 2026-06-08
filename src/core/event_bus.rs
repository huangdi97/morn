//! event_bus — Publishes and subscribes to internal application events.
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub source: String,
    pub data: Value,
    pub timestamp: i64,
}

impl Event {
    /// Creates an event from a type, source, and data payload with the current timestamp.
    pub fn new(event_type: &str, source: &str, data: Value) -> Self {
        Event {
            event_type: event_type.to_string(),
            source: source.to_string(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Clone)]
pub struct SimpleEventBus {
    subscribers: HashMap<String, Vec<fn(Event)>>,
}

impl SimpleEventBus {
    /// Creates an empty event bus with no subscribers.
    pub fn new() -> Self {
        SimpleEventBus {
            subscribers: HashMap::new(),
        }
    }

    /// Publishes an event to all handlers subscribed to its event type.
    pub fn publish(&self, event: Event) {
        if let Some(handlers) = self.subscribers.get(&event.event_type) {
            for handler in handlers {
                handler(event.clone());
            }
        }
    }

    /// Subscribes a handler function to an event type.
    pub fn subscribe(&mut self, event_type: &str, handler: fn(Event)) {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(handler);
    }

    /// Removes a handler function from an event type subscription list.
    pub fn unsubscribe(&mut self, event_type: &str, handler: fn(Event)) {
        if let Some(handlers) = self.subscribers.get_mut(event_type) {
            handlers.retain(|h| !std::ptr::fn_addr_eq(*h, handler));
        }
    }

    /// Builds and publishes an event from type, source, and data fields.
    pub fn publish_event(&self, event_type: &str, source: &str, data: Value) {
        let event = Event::new(event_type, source, data);
        self.publish(event);
    }
}

impl Default for SimpleEventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub const EVENT_SUPERVISOR_PLAN_CREATED: &str = "supervisor.plan.created";
pub const EVENT_SUPERVISOR_PLAN_EXECUTING: &str = "supervisor.plan.executing";
pub const EVENT_TASK_COMPLETED: &str = "supervisor.task.completed";
pub const EVENT_TASK_FAILED: &str = "supervisor.task.failed";
pub const EVENT_CHAT_AGENT_RESPONSE: &str = "chat_agent.response";
pub const EVENT_SYSTEM_READY: &str = "system.ready";
pub const EVENT_SYSTEM_SHUTDOWN: &str = "system.shutdown";
pub const EVENT_AGENT_CREATED: &str = "agent.created";
pub const EVENT_AGENT_DESTROYED: &str = "agent.destroyed";
pub const EVENT_WORKFLOW_STARTED: &str = "workflow.started";
pub const EVENT_WORKFLOW_COMPLETED: &str = "workflow.completed";
pub const EVENT_WORKFLOW_FAILED: &str = "workflow.failed";
pub const EVENT_CHANNEL_CONNECTED: &str = "channel.connected";
pub const EVENT_CHANNEL_DISCONNECTED: &str = "channel.disconnected";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static HANDLER_ONE_CALLS: AtomicUsize = AtomicUsize::new(0);
    static HANDLER_TWO_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn handler_one(event: Event) {
        assert_eq!(event.source, "test");
        HANDLER_ONE_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn handler_two(_event: Event) {
        HANDLER_TWO_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn reset_calls() {
        HANDLER_ONE_CALLS.store(0, Ordering::SeqCst);
        HANDLER_TWO_CALLS.store(0, Ordering::SeqCst);
    }

    #[test]
    fn subscribes_handler_to_event_type() {
        reset_calls();
        let mut bus = SimpleEventBus::new();

        bus.subscribe("test.event", handler_one);
        bus.publish_event("test.event", "test", serde_json::json!({"ok": true}));

        assert_eq!(HANDLER_ONE_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn publishes_only_matching_event_type() {
        reset_calls();
        let mut bus = SimpleEventBus::new();

        bus.subscribe("test.event", handler_one);
        bus.publish_event("other.event", "test", serde_json::json!({}));

        assert_eq!(HANDLER_ONE_CALLS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn unsubscribes_handler_from_event_type() {
        reset_calls();
        let mut bus = SimpleEventBus::new();

        bus.subscribe("test.event", handler_one);
        bus.unsubscribe("test.event", handler_one);
        bus.publish_event("test.event", "test", serde_json::json!({}));

        assert_eq!(HANDLER_ONE_CALLS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn publishes_to_multiple_subscribers() {
        reset_calls();
        let mut bus = SimpleEventBus::new();

        bus.subscribe("test.event", handler_one);
        bus.subscribe("test.event", handler_two);
        bus.publish_event("test.event", "test", serde_json::json!({"value": 1}));

        assert_eq!(HANDLER_ONE_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(HANDLER_TWO_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn lifecycle_event_names_are_stable() {
        assert_eq!(EVENT_SYSTEM_SHUTDOWN, "system.shutdown");
        assert_eq!(EVENT_AGENT_CREATED, "agent.created");
        assert_eq!(EVENT_AGENT_DESTROYED, "agent.destroyed");
        assert_eq!(EVENT_WORKFLOW_STARTED, "workflow.started");
        assert_eq!(EVENT_WORKFLOW_COMPLETED, "workflow.completed");
        assert_eq!(EVENT_WORKFLOW_FAILED, "workflow.failed");
        assert_eq!(EVENT_CHANNEL_CONNECTED, "channel.connected");
        assert_eq!(EVENT_CHANNEL_DISCONNECTED, "channel.disconnected");
    }
}
