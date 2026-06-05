use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct WebhookReceiver {
    adapter: Option<ChannelAdapter>,
}

impl WebhookReceiver {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        WebhookReceiver { adapter }
    }

    pub fn handle_event(&mut self, event_type: &str, payload: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: format!("[{}] {}", event_type, payload),
                source: "webhook".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({"event_type": event_type}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}