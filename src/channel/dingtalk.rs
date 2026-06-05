//! 注意：此通道需要 [钉钉] 真实应用注册才能使用
//! 配置方式：在钉钉开放平台创建应用，获取 Webhook URL
//! 环境变量：DINGTALK_WEBHOOK_URL

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

    pub fn from_env() -> Result<Self, String> {
        let url = std::env::var("DINGTALK_WEBHOOK_URL")
            .map_err(|_| "DINGTALK_WEBHOOK_URL not set".to_string())?;
        Ok(DingTalkChannel::new(&url))
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        let payload = Self::build_payload(msg);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| format!("Failed to send DingTalk message: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!(
                "DingTalk webhook returned non-200 status: {}",
                resp.status()
            ));
        }
        Ok(())
    }

    fn build_payload(msg: &ChannelMessage) -> serde_json::Value {
        serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": msg.content
            }
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dingtalk_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let payload = DingTalkChannel::build_payload(&msg);
        assert_eq!(payload["msgtype"], "text");
        assert_eq!(payload["text"]["content"], "Hello");
    }

    #[test]
    fn test_dingtalk_send_connection_error() {
        let channel = DingTalkChannel::new("http://localhost:1/nonexistent");
        let msg = ChannelMessage {
            content: "test".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let result = channel.send(&msg);
        assert!(result.is_err());
    }
}
