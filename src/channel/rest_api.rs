use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct RestApiServer {
    adapter: Option<ChannelAdapter>,
    turn_count: u64,
}

impl RestApiServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        RestApiServer { adapter, turn_count: 0 }
    }

    pub fn chat(&mut self, text: &str) -> Result<String, String> {
        self.turn_count += 1;
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "rest_api".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }

    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "version": "0.1.0",
            "turn_count": self.turn_count,
            "components": ["chat-agent"],
        })
    }

    pub fn clear(&mut self) -> Result<(), String> {
        self.turn_count = 0;
        Ok(())
    }
}