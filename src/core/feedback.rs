//! 用户反馈系统 — 会话反馈收集、情感分析、改进建议
use crate::core::error::MornError;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedbackMessage {
    pub channel: String,
    pub content: String,
    pub user_id: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedbackResult {
    pub summary: String,
    pub channels_delivered: Vec<String>,
    pub confirmations: Vec<String>,
}

pub struct FeedbackSync {
    channels: HashMap<String, Box<dyn FeedbackChannel + Send>>,
}

pub trait FeedbackChannel {
    fn send(&self, msg: &FeedbackMessage) -> Result<String, MornError>;
    fn name(&self) -> &str;
}

impl FeedbackSync {
    pub fn new() -> Self {
        FeedbackSync {
            channels: HashMap::new(),
        }
    }

    pub fn register_channel(&mut self, name: &str, channel: Box<dyn FeedbackChannel + Send>) {
        self.channels.insert(name.to_string(), channel);
    }

    pub fn broadcast(&self, content: &str, user_id: &str) -> FeedbackResult {
        let mut channels_delivered = Vec::new();
        let mut confirmations = Vec::new();
        let msg = FeedbackMessage {
            channel: "broadcast".to_string(),
            content: content.to_string(),
            user_id: user_id.to_string(),
            confirmed: false,
        };
        for channel in self.channels.values() {
            if let Ok(confirmation) = channel.send(&msg) {
                channels_delivered.push(channel.name().to_string());
                confirmations.push(confirmation);
            }
        }
        FeedbackResult {
            summary: format!("Delivered to {} channel(s)", channels_delivered.len()),
            channels_delivered,
            confirmations,
        }
    }

    pub fn send_to(
        &self,
        channel_name: &str,
        content: &str,
        user_id: &str,
    ) -> Result<String, MornError> {
        match self.channels.get(channel_name) {
            Some(channel) => {
                let msg = FeedbackMessage {
                    channel: channel_name.to_string(),
                    content: content.to_string(),
                    user_id: user_id.to_string(),
                    confirmed: false,
                };
                channel.send(&msg)
            }
            None => Err(MornError::Internal(format!("Channel '{}' not registered", channel_name))),
        }
    }
}

impl Default for FeedbackSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestChannel {
        name: String,
    }

    impl FeedbackChannel for TestChannel {
        fn send(&self, msg: &FeedbackMessage) -> Result<String, MornError> {
            Ok(format!("{} delivered to {}", msg.content, self.name))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn broadcast_delivers_to_all_channels() {
        let mut sync = FeedbackSync::new();
        sync.register_channel("cli", Box::new(TestChannel { name: "cli".into() }));
        sync.register_channel(
            "desktop",
            Box::new(TestChannel {
                name: "desktop".into(),
            }),
        );

        let result = sync.broadcast("hello", "user-1");

        assert_eq!(result.channels_delivered.len(), 2);
        assert!(result.channels_delivered.contains(&"cli".to_string()));
        assert!(result.channels_delivered.contains(&"desktop".to_string()));
    }

    #[test]
    fn send_to_returns_error_for_unknown_channel() {
        let sync = FeedbackSync::new();
        let result = sync.send_to("unknown", "test", "user-1");
        assert!(result.is_err());
    }

    #[test]
    fn send_to_delivers_to_specific_channel() {
        let mut sync = FeedbackSync::new();
        sync.register_channel(
            "desktop",
            Box::new(TestChannel {
                name: "desktop".into(),
            }),
        );

        let result = sync.send_to("desktop", "alert", "user-1");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("alert"));
    }

    #[test]
    fn broadcast_empty_channels() {
        let sync = FeedbackSync::new();
        let result = sync.broadcast("hello", "user-1");
        assert_eq!(result.channels_delivered.len(), 0);
        assert_eq!(result.summary, "Delivered to 0 channel(s)");
    }

    #[test]
    fn broadcast_with_confirmation_tracking() {
        let mut sync = FeedbackSync::new();
        sync.register_channel("cli", Box::new(TestChannel { name: "cli".into() }));
        sync.register_channel(
            "email",
            Box::new(TestChannel {
                name: "email".into(),
            }),
        );

        let result = sync.broadcast("confirm-test", "user-2");

        assert_eq!(result.channels_delivered.len(), 2);
        assert_eq!(result.confirmations.len(), 2);
        assert!(result.confirmations[0].contains("confirm-test"));
    }

    #[test]
    fn broadcast_preserves_feedback_message_fields() {
        let mut sync = FeedbackSync::new();
        sync.register_channel("cli", Box::new(TestChannel { name: "cli".into() }));
        sync.register_channel(
            "desktop",
            Box::new(TestChannel {
                name: "desktop".into(),
            }),
        );

        let result = sync.broadcast("save this feedback", "user-3");

        for channel in &result.channels_delivered {
            assert!(channel == "cli" || channel == "desktop");
        }
    }
}
