//! 注意：此通道需要 \[飞书\] 真实应用注册才能使用
//! 配置方式：在飞书开放平台创建应用，获取 Webhook URL
//! 环境变量：FEISHU_WEBHOOK_URL

use crate::core::error::MornError;
use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct FeishuChannel {
    webhook_url: String,
}

impl FeishuChannel {
    pub fn new(webhook_url: &str) -> Self {
        FeishuChannel {
            webhook_url: webhook_url.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, MornError> {
        let url = std::env::var("FEISHU_WEBHOOK_URL")
            .map_err(|_| "FEISHU_WEBHOOK_URL not set".to_string())?;
        Ok(FeishuChannel::new(&url))
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), MornError> {
        let payload = Self::build_payload(msg);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| MornError::Internal(format!("Failed to create HTTP client: {}", e)))?;
        let resp = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| MornError::Internal(format!("Failed to send Feishu message: {}", e)))?;
        if !resp.status().is_success() {
            return Err(MornError::Internal(format!(
                "Feishu webhook returned non-200 status: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn build_payload(msg: &ChannelMessage) -> serde_json::Value {
        serde_json::json!({
            "msg_type": "text",
            "content": {
                "text": msg.content
            }
        })
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, MornError> {
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

    pub fn handle_event(&mut self, text: &str) -> Result<String, MornError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feishu_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let payload = FeishuChannel::build_payload(&msg);
        assert_eq!(payload["msg_type"], "text");
        assert_eq!(payload["content"]["text"], "Hello");
    }

    #[test]
    fn test_feishu_send_connection_error() {
        let channel = FeishuChannel::new("http://localhost:1/nonexistent");
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
