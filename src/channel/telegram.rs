//! telegram — Adapts Telegram messages into the shared channel interface.
use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct TelegramChannel {
    bot_token: String,
    chat_id: String,
    client: reqwest::blocking::Client,
}

impl TelegramChannel {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        TelegramChannel {
            bot_token: bot_token.to_string(),
            chat_id: chat_id.to_string(),
            client,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| "TELEGRAM_BOT_TOKEN not set".to_string())?;
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_else(|_| "0".to_string());
        Ok(Self::new(&token, &chat_id))
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        let parse_mode = msg
            .metadata
            .get("parse_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("Markdown");
        self.send_message(&msg.content, Some(parse_mode))
    }

    pub fn send_message(&self, text: &str, parse_mode: Option<&str>) -> Result<(), String> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let mut body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": text,
        });

        if let Some(mode) = parse_mode {
            body["parse_mode"] = serde_json::json!(mode);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Telegram API request failed: {}", e))?;

        let status = response.status();
        let response_body: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse Telegram response: {}", e))?;

        if status.is_success() {
            if response_body
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                Ok(())
            } else {
                let desc = response_body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                Err(format!("Telegram API error: {}", desc))
            }
        } else {
            let desc = response_body
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            Err(format!(
                "Telegram API error ({}): {}",
                status.as_u16(),
                desc
            ))
        }
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(Some(ChannelMessage {
            content: "[Telegram] simulated incoming message".into(),
            source: "telegram".into(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: serde_json::json!({"type": "text", "chat_id": self.chat_id, "from": "user_123"}),
        }))
    }

    pub fn poll_updates(&mut self, adapter: &mut ChannelAdapter) {
        let client = reqwest::blocking::Client::new();
        let mut offset = 0;

        loop {
            let url = format!("https://api.telegram.org/bot{}/getUpdates", self.bot_token);
            let params = serde_json::json!({
                "offset": offset,
                "timeout": 30,
                "allowed_updates": ["message"],
            });

            match client.post(&url).json(&params).send() {
                Ok(resp) => {
                    if let Ok(body) = resp.json::<serde_json::Value>() {
                        if let Some(updates) = body["result"].as_array() {
                            for update in updates {
                                if let Some(msg) = update["message"].as_object() {
                                    let text = msg
                                        .get("text")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let chat_id =
                                        msg.get("chat").and_then(|c| c["id"].as_i64()).unwrap_or(0);

                                    if !text.is_empty() {
                                        let channel_msg = ChannelMessage {
                                            content: text,
                                            source: "telegram".into(),
                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                            metadata: serde_json::json!({"chat_id": chat_id}),
                                        };
                                        let reply = adapter.handle_message(&channel_msg);
                                        self.send_message(&reply, Some("Markdown")).ok();
                                    }
                                }

                                offset = update["update_id"].as_i64().unwrap_or(0) + 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[Telegram] poll error: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    }

    pub fn set_webhook(&self, url: &str) -> Result<(), String> {
        let api_url = format!("https://api.telegram.org/bot{}/setWebhook", self.bot_token);

        let response = self
            .client
            .post(&api_url)
            .json(&serde_json::json!({"url": url}))
            .send()
            .map_err(|e| format!("Telegram setWebhook request failed: {}", e))?;

        let status = response.status();
        let response_body: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse Telegram response: {}", e))?;

        if status.is_success() {
            let ok = response_body
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if ok {
                Ok(())
            } else {
                let desc = response_body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                Err(format!("Telegram setWebhook error: {}", desc))
            }
        } else {
            let desc = response_body
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            Err(format!(
                "Telegram setWebhook error ({}): {}",
                status.as_u16(),
                desc
            ))
        }
    }

    pub fn handle_update(&self, update_json: &str) -> Result<String, String> {
        let update: serde_json::Value = serde_json::from_str(update_json)
            .map_err(|e| format!("Invalid Telegram update JSON: {}", e))?;

        let message_text = update
            .get("message")
            .and_then(|m| m.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let chat_id = update
            .get("message")
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|i| i.as_i64())
            .unwrap_or(0);

        let from = update
            .get("message")
            .and_then(|m| m.get("from"))
            .and_then(|f| f.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        Ok(serde_json::json!({
            "text": message_text,
            "chat_id": chat_id,
            "from": from,
        })
        .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_new() {
        let channel = TelegramChannel::new("test_token_123", "test_chat_456");
        assert_eq!(channel.bot_token, "test_token_123");
        assert_eq!(channel.chat_id, "test_chat_456");
    }

    #[test]
    fn test_telegram_receive() {
        let channel = TelegramChannel::new("token", "chat");
        let result = channel.receive().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, "telegram");
    }

    #[test]
    fn test_telegram_handle_update() {
        let channel = TelegramChannel::new("token", "chat");
        let update =
            r#"{"message":{"text":"hello","chat":{"id":12345},"from":{"username":"test_user"}}}"#;
        let result = channel.handle_update(update).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("12345"));
        assert!(result.contains("test_user"));
    }

    #[test]
    fn test_telegram_send_message_fails_with_invalid_token() {
        let channel = TelegramChannel::new("invalid_token", "0");
        let result = channel.send_message("test", None);
        assert!(result.is_err());
    }
}
