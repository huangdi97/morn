use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct DingTalkChannel {
    webhook_url: String,
}

impl DingTalkChannel {
    pub fn new(webhook_url: &str) -> Self {
        DingTalkChannel {
            webhook_url: webhook_url.to_string(),
        }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(None)
    }
}

pub struct DingTalkServer {
    adapter: Option<ChannelAdapter>,
}

impl DingTalkServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        DingTalkServer { adapter }
    }

    pub fn handle_callback(&mut self, text: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "dingtalk".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}
