use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentEvent {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub source: String,
    pub data: Value,
    pub timestamp: i64,
}

impl AgentEvent {
    pub fn new(session_id: &str, event_type: &str, source: &str, data: Value) -> Self {
        AgentEvent {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            event_type: event_type.to_string(),
            source: source.to_string(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

pub trait EventConsumer: Send {
    fn on_event(&self, event: &AgentEvent);
}

pub trait EventStorage: Send {
    fn save(&self, event: &AgentEvent) -> Result<(), String>;
    fn replay(&self, session_id: &str) -> Result<Vec<AgentEvent>, String>;
}

pub struct EventBus {
    consumers: Vec<Box<dyn EventConsumer>>,
    storage: Option<Box<dyn EventStorage>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            consumers: Vec::new(),
            storage: None,
        }
    }

    pub fn with_storage(storage: Box<dyn EventStorage>) -> Self {
        EventBus {
            consumers: Vec::new(),
            storage: Some(storage),
        }
    }

    pub fn publish(&self, event: AgentEvent) -> Result<(), String> {
        for consumer in &self.consumers {
            consumer.on_event(&event);
        }
        if let Some(ref storage) = self.storage {
            storage.save(&event)?;
        }
        Ok(())
    }

    pub fn subscribe(&mut self, consumer: Box<dyn EventConsumer>) {
        self.consumers.push(consumer);
    }

    pub fn replay(&self, session_id: &str) -> Result<Vec<AgentEvent>, String> {
        match self.storage {
            Some(ref storage) => storage.replay(session_id),
            None => Err("No storage backend configured for replay".to_string()),
        }
    }

    pub fn publish_event(
        &self,
        session_id: &str,
        event_type: &str,
        source: &str,
        data: Value,
    ) -> Result<(), String> {
        let event = AgentEvent::new(session_id, event_type, source, data);
        self.publish(event)
    }
}

pub struct EventBusBuilder {
    consumers: Vec<Box<dyn EventConsumer>>,
    storage: Option<Box<dyn EventStorage>>,
}

impl Default for EventBusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBusBuilder {
    pub fn new() -> Self {
        EventBusBuilder {
            consumers: Vec::new(),
            storage: None,
        }
    }

    pub fn with_consumer(mut self, consumer: Box<dyn EventConsumer>) -> Self {
        self.consumers.push(consumer);
        self
    }

    pub fn with_storage(mut self, storage: Box<dyn EventStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn build(self) -> EventBus {
        EventBus {
            consumers: self.consumers,
            storage: self.storage,
        }
    }
}

pub struct InMemoryEventStorage {
    events: std::sync::Mutex<Vec<AgentEvent>>,
}

impl Default for InMemoryEventStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryEventStorage {
    pub fn new() -> Self {
        InMemoryEventStorage {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl EventStorage for InMemoryEventStorage {
    fn save(&self, event: &AgentEvent) -> Result<(), String> {
        let mut events = self.events.lock().map_err(|e| e.to_string())?;
        events.push(event.clone());
        Ok(())
    }

    fn replay(&self, session_id: &str) -> Result<Vec<AgentEvent>, String> {
        let events = self.events.lock().map_err(|e| e.to_string())?;
        Ok(events
            .iter()
            .filter(|e| e.session_id == session_id)
            .cloned()
            .collect())
    }
}

pub struct FnEventConsumer<F: Fn(&AgentEvent) + Send + 'static> {
    handler: F,
}

impl<F: Fn(&AgentEvent) + Send + 'static> FnEventConsumer<F> {
    pub fn new(handler: F) -> Self {
        FnEventConsumer { handler }
    }
}

impl<F: Fn(&AgentEvent) + Send + 'static> EventConsumer for FnEventConsumer<F> {
    fn on_event(&self, event: &AgentEvent) {
        (self.handler)(event);
    }
}

pub const EVENT_AGENT_STARTED: &str = "agent.started";
pub const EVENT_AGENT_STEP: &str = "agent.step";
pub const EVENT_AGENT_COMPLETED: &str = "agent.completed";
pub const EVENT_AGENT_FAILED: &str = "agent.failed";
pub const EVENT_APPROVAL_REQUESTED: &str = "approval.requested";
pub const EVENT_APPROVAL_RESPONDED: &str = "approval.responded";
pub const EVENT_TOOL_CALLED: &str = "tool.called";
pub const EVENT_TOOL_RESULT: &str = "tool.result";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_event_publish_subscribe() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();
        let consumer = FnEventConsumer::new(move |_event: &AgentEvent| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut bus = EventBus::new();
        bus.subscribe(Box::new(consumer));

        let event = AgentEvent::new(
            "s1",
            "test.event",
            "test",
            serde_json::json!({"key": "value"}),
        );
        bus.publish(event).unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_replay() {
        let storage = InMemoryEventStorage::new();
        let bus = EventBus::with_storage(Box::new(storage));

        for i in 0..3 {
            let event = AgentEvent::new(
                "session-1",
                &format!("event.{}", i),
                "test",
                serde_json::json!({"idx": i}),
            );
            bus.publish(event).unwrap();
        }

        let events = bus.replay("session-1").unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_event_builder() {
        let storage = InMemoryEventStorage::new();
        let bus = EventBusBuilder::new()
            .with_storage(Box::new(storage))
            .build();

        let event = AgentEvent::new("s1", "test", "src", serde_json::json!({}));
        bus.publish(event).unwrap();
    }
}
