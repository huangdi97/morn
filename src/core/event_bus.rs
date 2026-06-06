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

impl Default for SimpleEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleEventBus {
    pub fn new() -> Self {
        SimpleEventBus {
            subscribers: HashMap::new(),
        }
    }

    pub fn publish(&self, event: Event) {
        if let Some(handlers) = self.subscribers.get(&event.event_type) {
            for handler in handlers {
                handler(event.clone());
            }
        }
    }

    pub fn subscribe(&mut self, event_type: &str, handler: fn(Event)) {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(handler);
    }

    pub fn unsubscribe(&mut self, event_type: &str, handler: fn(Event)) {
        if let Some(handlers) = self.subscribers.get_mut(event_type) {
            handlers.retain(|h| !std::ptr::fn_addr_eq(*h, handler));
        }
    }

    pub fn publish_event(&self, event_type: &str, source: &str, data: Value) {
        let event = Event::new(event_type, source, data);
        self.publish(event);
    }
}

pub const EVENT_SUPERVISOR_PLAN_CREATED: &str = "supervisor.plan.created";
pub const EVENT_SUPERVISOR_PLAN_EXECUTING: &str = "supervisor.plan.executing";
pub const EVENT_TASK_COMPLETED: &str = "supervisor.task.completed";
pub const EVENT_TASK_FAILED: &str = "supervisor.task.failed";
pub const EVENT_CHAT_AGENT_RESPONSE: &str = "chat_agent.response";
pub const EVENT_SYSTEM_READY: &str = "system.ready";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    static EVENTS: std::sync::Mutex<Vec<Event>> = std::sync::Mutex::new(Vec::new());

    fn collect_handler(event: Event) {
        EVENTS.lock().unwrap().push(event);
    }

    fn count_handler(event: Event) {
        EVENTS.lock().unwrap().push(event);
    }

    #[test]
    fn test_publish_subscribe() {
        EVENTS.lock().unwrap().clear();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("test.event", collect_handler);

        let event = Event::new("test.event", "source", json!({"msg": "hello"}));
        bus.publish(event);

        let events = EVENTS.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test.event");
        assert_eq!(events[0].data, json!({"msg": "hello"}));
    }

    #[test]
    fn test_publish_event_convenience() {
        EVENTS.lock().unwrap().clear();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("sys.ready", collect_handler);

        bus.publish_event("sys.ready", "test", json!({"status": "ok"}));

        let events = EVENTS.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, "test");
        assert_eq!(events[0].data, json!({"status": "ok"}));
    }

    #[test]
    fn test_multiple_subscribers() {
        EVENTS.lock().unwrap().clear();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("multi.event", collect_handler);
        bus.subscribe("multi.event", count_handler);

        let event = Event::new("multi.event", "test", json!({"n": 1}));
        bus.publish(event);

        let events = EVENTS.lock().unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_unsubscribe_stops_receiving() {
        EVENTS.lock().unwrap().clear();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("unsub.event", collect_handler);
        bus.subscribe("unsub.event", count_handler);

        bus.unsubscribe("unsub.event", collect_handler);

        let event = Event::new("unsub.event", "test", json!({"n": 1}));
        bus.publish(event);

        let events = EVENTS.lock().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_different_event_types_isolated() {
        EVENTS.lock().unwrap().clear();
        let mut bus = SimpleEventBus::new();
        bus.subscribe("type.a", collect_handler);

        let event_b = Event::new("type.b", "test", json!({}));
        bus.publish(event_b);

        let events = EVENTS.lock().unwrap();
        assert_eq!(events.len(), 0);
    }
}
