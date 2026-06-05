use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct FeishuChannel {
    webhook_url: String,
}

impl FeishuChannel {
    pub fn new(webhook_url: &str) -> Self {
        FeishuChannel { webhook_url: webhook_url.to_string() }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(None)
    }
}

pub struct FeishuServer {
    adapter: Option<ChannelAdapter>,
}

impl FeishuServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        FeishuServer { adapter }
    }

    pub fn handle_event(&mut self, text: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "feishu".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}