use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct WeComChannel {
    webhook_url: String,
}

impl WeComChannel {
    pub fn new(webhook_url: &str) -> Self {
        WeComChannel { webhook_url: webhook_url.to_string() }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(None)
    }
}

pub struct WeComServer {
    adapter: Option<ChannelAdapter>,
}

impl WeComServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        WeComServer { adapter }
    }

    pub fn handle_webhook(&mut self, text: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "wecom".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}