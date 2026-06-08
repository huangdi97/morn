//! adapter — Defines shared channel message types and adapter behavior.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::core::supervisor::{Mode, Supervisor};

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
        ChannelAdapter {
            supervisor,
            chat_fn: None,
        }
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

    pub fn set_supervisor_mode(&mut self, mode: Mode) -> Result<(), String> {
        match self.supervisor.as_mut() {
            Some(supervisor) => {
                supervisor.set_mode(mode);
                Ok(())
            }
            None => Err("Supervisor not initialized. Please set MORN_API_KEY.".to_string()),
        }
    }

    pub fn supervisor_mode(&self) -> Option<&Mode> {
        self.supervisor
            .as_ref()
            .map(|supervisor| supervisor.get_mode())
    }

    pub fn format_response(text: &str, _source: &str) -> String {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_new_sets_core_fields() {
        let msg = ChannelMessage::new("hello", "cli");
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.source, "cli");
        assert!(msg.timestamp > 0);
        assert!(msg.metadata.is_object());
    }

    #[test]
    fn adapter_without_supervisor_reports_lifecycle_state() {
        let mut adapter = ChannelAdapter::new(None);
        let response = adapter.handle_message(&ChannelMessage::new("ping", "test"));
        assert!(response.contains("Supervisor not initialized"));
    }

    #[test]
    fn supervisor_mode_requires_supervisor() {
        let mut adapter = ChannelAdapter::new(None);
        let result = adapter.set_supervisor_mode(Mode::Safe);
        assert!(result.is_err());
        assert!(adapter.supervisor_mode().is_none());
    }

    #[test]
    fn format_response_preserves_message_text() {
        assert_eq!(ChannelAdapter::format_response("ok", "cli"), "ok");
    }
}
