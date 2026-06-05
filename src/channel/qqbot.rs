//! 注意：此通道需要 [QQ 机器人] 真实应用注册才能使用
//! 配置方式：在 QQ 开放平台创建机器人，获取 Bot ID 和 Token
//! 环境变量：QQBOT_ID, QQBOT_TOKEN

use crate::channel::adapter::ChannelMessage;

#[allow(dead_code)]
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

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        println!(
            "[QQ Bot] Sending message: {} (via QQ guild/group)",
            msg.content
        );
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(Some(ChannelMessage {
            content: "[QQ Bot] simulated incoming message".into(),
            source: "qqbot".into(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: serde_json::json!({"type": "text", "group_id": "qq_group_123"}),
        }))
    }

    pub fn handle_event(&self, event_type: &str, payload: &str) -> Result<String, String> {
        Ok(format!("[QQ Bot] handled '{}': {}", event_type, payload))
    }
}
