use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::core::supervisor::Supervisor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub content: String,
    pub source: String,
    pub timestamp: i64,
    pub metadata: Value,
}

impl ChannelMessage {
    pub fn new(content: &str, source: &str) -> Self {
        ChannelMessage {
            content: content.to_string(),
            source: source.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: serde_json::json!({}),
        }
    }
}

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

pub struct ChannelAdapter {
    supervisor: Option<Supervisor>,
    chat_fn: Option<ChatFn>,
}

impl ChannelAdapter {
    pub fn new(supervisor: Option<Supervisor>) -> Self {
        ChannelAdapter { supervisor, chat_fn: None }
    }

    pub fn with_chat_fn(mut self, chat_fn: ChatFn) -> Self {
        self.chat_fn = Some(chat_fn);
        self
    }

    pub fn set_chat_fn(&mut self, chat_fn: ChatFn) {
        self.chat_fn = Some(chat_fn);
    }

    pub fn handle_message(&mut self, msg: &ChannelMessage) -> String {
        match (self.supervisor.as_mut(), self.chat_fn.as_ref()) {
            (Some(ref mut supervisor), Some(ref chat_fn)) => {
                match supervisor.execute_chat(&msg.content, chat_fn.as_ref()) {
                    Ok(response) => response,
                    Err(e) => format!("Error: {}", e),
                }
            }
            (Some(_), None) => "Chat function not configured".to_string(),
            (None, _) => "Supervisor not initialized. Please set MORN_API_KEY.".to_string(),
        }
    }

    pub fn format_response(text: &str, _source: &str) -> String {
        text.to_string()
    }
}