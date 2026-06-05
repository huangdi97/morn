use crate::channel::adapter::ChannelMessage;

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
