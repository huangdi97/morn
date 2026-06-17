//! 注意：此通道需要 [QQ 机器人] 真实应用注册才能使用
//! 配置方式：在 QQ 开放平台创建机器人，获取 Bot ID 和 Token
//! 环境变量：QQBOT_ID, QQBOT_TOKEN

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use crate::core::error::MornError;

pub struct QqBotChannel {
    bot_id: String,
    token: String,
}

impl QqBotChannel {
    pub fn new(bot_id: &str, token: &str) -> Self {
        QqBotChannel {
            bot_id: bot_id.to_string(),
            token: token.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, MornError> {
        let bot_id = std::env::var("QQBOT_ID").map_err(|_| "QQBOT_ID not set".to_string())?;
        let token = std::env::var("QQBOT_TOKEN").map_err(|_| "QQBOT_TOKEN not set".to_string())?;
        Ok(QqBotChannel::new(&bot_id, &token))
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), MornError> {
        let payload = Self::build_payload(msg);
        let url = format!("https://api.qq.com/v1/robots/{}/messages", self.bot_id);
        let auth_value = format!("Bot {}.{}", self.bot_id, self.token);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| MornError::Internal(format!("Failed to create HTTP client: {}", e)))?;
        let resp = client
            .post(&url)
            .header("Authorization", &auth_value)
            .json(&payload)
            .send()
            .map_err(|e| MornError::Internal(format!("Failed to send QQ Bot message: {}", e)))?;
        if !resp.status().is_success() {
            return Err(MornError::Internal(format!(
                "QQ Bot API returned non-200 status: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn build_payload(msg: &ChannelMessage) -> serde_json::Value {
        let user_id = msg
            .metadata
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        serde_json::json!({
            "user_id": user_id,
            "content": msg.content,
            "msg_type": "text"
        })
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, MornError> {
        Ok(None)
    }
}

pub struct QqBotServer {
    adapter: Option<ChannelAdapter>,
}

impl QqBotServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        QqBotServer { adapter }
    }

    pub fn handle_event(&mut self, text: &str) -> Result<String, MornError> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "qqbot".into(),
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
    fn test_qqbot_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({"user_id": "user_123"}),
        };
        let payload = QqBotChannel::build_payload(&msg);
        assert_eq!(payload["user_id"], "user_123");
        assert_eq!(payload["content"], "Hello");
        assert_eq!(payload["msg_type"], "text");
    }

    #[test]
    fn test_qqbot_send_connection_error() {
        let channel = QqBotChannel::new("test_bot", "test_token");
        let msg = ChannelMessage {
            content: "test".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({"user_id": "user_123"}),
        };
        let result = channel.send(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_qqbot_server_without_adapter_routes_event() {
        let mut server = QqBotServer::new(None);
        let result = server.handle_event("hello");
        assert_eq!(result.as_deref(), Ok("No adapter configured"));
    }
}
