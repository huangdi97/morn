use crate::channel::adapter::ChannelMessage;

pub struct TelegramChannel {
    bot_token: String,
    chat_id: String,
}

impl TelegramChannel {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        TelegramChannel {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        println!(
            "[Telegram] Sending message: {} (to chat {})",
            msg.content, self.chat_id
        );
        Ok(())
    }

    pub fn send_message(&self, text: &str) -> Result<(), String> {
        println!("[Telegram] Sending text: {}", text);
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(Some(ChannelMessage {
            content: "[Telegram] simulated incoming message".into(),
            source: "telegram".into(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: serde_json::json!({"type": "text", "chat_id": self.chat_id, "from": "user_123"}),
        }))
    }

    pub fn set_webhook(&self, url: &str) -> Result<(), String> {
        println!("[Telegram] webhook set to: {}", url);
        Ok(())
    }

    pub fn handle_update(&self, update_json: &str) -> Result<String, String> {
        Ok(format!("[Telegram] processed update: {}", update_json))
    }
}
