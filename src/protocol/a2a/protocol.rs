//! A2A 协议核心 — 消息信封、序列化、路由寻址
use serde::{Deserialize, Serialize};

/// Supported content types for A2A messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Text,
    Json,
    Binary,
    Task,
    Result,
    Error,
}

/// The core message payload sent between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub id: String,
    pub content_type: ContentType,
    pub payload: serde_json::Value,
    pub created_at: i64,
    pub ttl_seconds: Option<u64>,
}

impl A2AMessage {
    pub fn new(content_type: ContentType, payload: serde_json::Value) -> Self {
        A2AMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content_type,
            payload,
            created_at: chrono::Utc::now().timestamp(),
            ttl_seconds: None,
        }
    }

    pub fn text(text: &str) -> Self {
        Self::new(ContentType::Text, serde_json::json!({"text": text}))
    }

    pub fn task(task_type: &str, params: serde_json::Value) -> Self {
        Self::new(
            ContentType::Task,
            serde_json::json!({"type": task_type, "params": params}),
        )
    }

    pub fn result(task_id: &str, data: serde_json::Value) -> Self {
        Self::new(
            ContentType::Result,
            serde_json::json!({"task_id": task_id, "data": data}),
        )
    }
}

/// Routing mode for A2A message delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingMode {
    Direct,
    Relay,
    Broadcast,
}

/// The envelope that wraps an A2A message with routing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AEnvelope {
    pub message: A2AMessage,
    pub sender_id: String,
    pub recipient_id: Option<String>,
    pub routing_mode: RoutingMode,
    pub relay_path: Vec<String>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
}

impl A2AEnvelope {
    pub fn direct(message: A2AMessage, sender_id: &str, recipient_id: &str) -> Self {
        A2AEnvelope {
            message,
            sender_id: sender_id.to_string(),
            recipient_id: Some(recipient_id.to_string()),
            routing_mode: RoutingMode::Direct,
            relay_path: Vec::new(),
            correlation_id: None,
            reply_to: None,
        }
    }

    pub fn relay(
        message: A2AMessage,
        sender_id: &str,
        recipient_id: &str,
        relay_path: Vec<String>,
    ) -> Self {
        A2AEnvelope {
            message,
            sender_id: sender_id.to_string(),
            recipient_id: Some(recipient_id.to_string()),
            routing_mode: RoutingMode::Relay,
            relay_path,
            correlation_id: None,
            reply_to: None,
        }
    }

    pub fn broadcast(message: A2AMessage, sender_id: &str) -> Self {
        A2AEnvelope {
            message,
            sender_id: sender_id.to_string(),
            recipient_id: None,
            routing_mode: RoutingMode::Broadcast,
            relay_path: Vec::new(),
            correlation_id: None,
            reply_to: None,
        }
    }

    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_reply_to(mut self, reply_to: &str) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }
}

/// Transport protocol for A2A communication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransportProtocol {
    WebSocket,
    HttpSse,
}

/// Agent capability descriptor for routing decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub agent_id: String,
    pub supported_content_types: Vec<ContentType>,
    pub transport: Vec<TransportProtocol>,
    pub endpoint: String,
    pub is_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let text_msg = A2AMessage::text("hello");
        assert_eq!(text_msg.content_type, ContentType::Text);

        let task_msg = A2AMessage::task("analyze", serde_json::json!({"data": [1, 2, 3]}));
        assert_eq!(task_msg.content_type, ContentType::Task);

        let result_msg = A2AMessage::result("task-1", serde_json::json!({"status": "done"}));
        assert_eq!(result_msg.content_type, ContentType::Result);
    }

    #[test]
    fn test_envelope_with_correlation() {
        let msg = A2AMessage::text("reply pls");
        let envelope = A2AEnvelope::direct(msg, "agent_alice", "agent_bob")
            .with_correlation("corr-123")
            .with_reply_to("agent_alice");

        assert_eq!(envelope.correlation_id, Some("corr-123".to_string()));
        assert_eq!(envelope.reply_to, Some("agent_alice".to_string()));
    }
}
