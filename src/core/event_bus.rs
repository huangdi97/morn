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
