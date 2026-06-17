//! A2A 消息路由器 — Agent 注册发现、消息转发、广播中继
use crate::core::error::MornError;
use std::collections::HashMap;

use super::protocol::{A2AEnvelope, AgentCapability, RoutingMode};

/// The A2A router that handles message dispatch between agents.
#[derive(Debug)]
pub struct A2ARouter {
    agents: HashMap<String, AgentCapability>,
    message_log: Vec<A2AEnvelope>,
}

impl A2ARouter {
    pub fn new() -> Self {
        A2ARouter {
            agents: HashMap::new(),
            message_log: Vec::new(),
        }
    }

    pub fn register_agent(&mut self, capability: AgentCapability) {
        self.agents.insert(capability.agent_id.clone(), capability);
    }

    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.agents.remove(agent_id);
    }

    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentCapability> {
        self.agents.get(agent_id)
    }

    pub fn list_agents(&self) -> Vec<&AgentCapability> {
        self.agents.values().collect()
    }

    /// Route an A2A envelope to the appropriate recipient(s).
    pub fn route(&mut self, envelope: A2AEnvelope) -> Result<Vec<A2AEnvelope>, MornError> {
        let delivered = match envelope.routing_mode {
            RoutingMode::Direct => self.route_direct(&envelope)?,
            RoutingMode::Relay => self.route_relay(&envelope)?,
            RoutingMode::Broadcast => self.route_broadcast(&envelope)?,
        };

        self.message_log.push(envelope);
        Ok(delivered)
    }

    fn route_direct(&self, envelope: &A2AEnvelope) -> Result<Vec<A2AEnvelope>, MornError> {
        let recipient_id = envelope
            .recipient_id
            .as_deref()
            .ok_or("Direct routing requires a recipient_id")?;

        if !self.agents.contains_key(recipient_id) {
            return Err(MornError::Internal(format!(
                "Recipient agent '{}' not found",
                recipient_id
            )));
        }

        Ok(vec![envelope.clone()])
    }

    fn route_relay(&self, envelope: &A2AEnvelope) -> Result<Vec<A2AEnvelope>, MornError> {
        let recipient_id = envelope
            .recipient_id
            .as_deref()
            .ok_or("Relay routing requires a recipient_id")?;

        for hop in &envelope.relay_path {
            if !self.agents.contains_key(hop) {
                return Err(MornError::Internal(format!(
                    "Relay hop agent '{}' not found",
                    hop
                )));
            }
        }

        if !self.agents.contains_key(recipient_id) {
            return Err(MornError::Internal(format!(
                "Recipient agent '{}' not found",
                recipient_id
            )));
        }

        Ok(vec![envelope.clone()])
    }

    fn route_broadcast(&self, envelope: &A2AEnvelope) -> Result<Vec<A2AEnvelope>, MornError> {
        let sender_id = &envelope.sender_id;
        let recipients: Vec<A2AEnvelope> = self
            .agents
            .keys()
            .filter(|id| *id != sender_id)
            .map(|id| A2AEnvelope {
                recipient_id: Some(id.clone()),
                ..envelope.clone()
            })
            .collect();

        Ok(recipients)
    }

    /// Get message log (for replay/debug).
    pub fn message_log(&self) -> &[A2AEnvelope] {
        &self.message_log
    }

    /// Get message count.
    pub fn message_count(&self) -> usize {
        self.message_log.len()
    }

    /// Agent count.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for A2ARouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::a2a::protocol::{
        A2AMessage, AgentCapability, ContentType, TransportProtocol,
    };

    fn setup_router() -> A2ARouter {
        let mut router = A2ARouter::new();
        router.register_agent(AgentCapability {
            agent_id: "agent_alice".into(),
            supported_content_types: vec![ContentType::Text, ContentType::Task],
            transport: vec![TransportProtocol::WebSocket],
            endpoint: "ws://alice.local:9001".into(),
            is_available: true,
        });
        router.register_agent(AgentCapability {
            agent_id: "agent_bob".into(),
            supported_content_types: vec![
                ContentType::Text,
                ContentType::Json,
                ContentType::Result,
            ],
            transport: vec![TransportProtocol::HttpSse],
            endpoint: "http://bob.local:9002/sse".into(),
            is_available: true,
        });
        router.register_agent(AgentCapability {
            agent_id: "agent_charlie".into(),
            supported_content_types: vec![
                ContentType::Text,
                ContentType::Task,
                ContentType::Result,
            ],
            transport: vec![TransportProtocol::WebSocket, TransportProtocol::HttpSse],
            endpoint: "ws://charlie.local:9003".into(),
            is_available: true,
        });
        router
    }

    #[test]
    fn test_direct_routing() {
        let mut router = setup_router();
        let msg = A2AMessage::text("Hello Bob!");
        let envelope = A2AEnvelope::direct(msg, "agent_alice", "agent_bob");

        let result = router.route(envelope).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].recipient_id, Some("agent_bob".to_string()));
    }

    #[test]
    fn test_direct_routing_fails_for_unknown_recipient() {
        let mut router = setup_router();
        let msg = A2AMessage::text("Hello unknown!");
        let envelope = A2AEnvelope::direct(msg, "agent_alice", "agent_unknown");

        let result = router.route(envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_broadcast_routing() {
        let mut router = setup_router();
        let msg = A2AMessage::text("Hello everyone!");
        let envelope = A2AEnvelope::broadcast(msg, "agent_alice");

        let result = router.route(envelope).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .all(|e| e.recipient_id != Some("agent_alice".to_string())));
    }

    #[test]
    fn test_relay_routing() {
        let mut router = setup_router();
        let msg = A2AMessage::text("Relay this to Bob");
        let envelope = A2AEnvelope::relay(
            msg,
            "agent_alice",
            "agent_bob",
            vec!["agent_charlie".to_string()],
        );

        let result = router.route(envelope).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].recipient_id, Some("agent_bob".to_string()));
    }

    #[test]
    fn test_relay_routing_fails_with_unknown_hop() {
        let mut router = setup_router();
        let msg = A2AMessage::text("Relay via unknown");
        let envelope = A2AEnvelope::relay(
            msg,
            "agent_alice",
            "agent_bob",
            vec!["agent_unknown".to_string()],
        );

        let result = router.route(envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_registration() {
        let mut router = setup_router();
        assert_eq!(router.agent_count(), 3);

        router.register_agent(AgentCapability {
            agent_id: "agent_dave".into(),
            supported_content_types: vec![ContentType::Text],
            transport: vec![TransportProtocol::WebSocket],
            endpoint: "ws://dave.local:9004".into(),
            is_available: true,
        });
        assert_eq!(router.agent_count(), 4);

        router.unregister_agent("agent_alice");
        assert_eq!(router.agent_count(), 3);
    }

    #[test]
    fn test_message_logging() {
        let mut router = setup_router();
        let msg1 = A2AMessage::text("First");
        let msg2 = A2AMessage::text("Second");

        router
            .route(A2AEnvelope::direct(msg1, "agent_alice", "agent_bob"))
            .unwrap();
        router
            .route(A2AEnvelope::broadcast(msg2, "agent_charlie"))
            .unwrap();

        assert_eq!(router.message_count(), 2);
    }

    #[test]
    fn test_direct_routing_missing_recipient_id() {
        let mut router = A2ARouter::new();
        router.register_agent(AgentCapability {
            agent_id: "agent_alice".into(),
            supported_content_types: vec![ContentType::Text],
            transport: vec![TransportProtocol::WebSocket],
            endpoint: "ws://alice.local:9001".into(),
            is_available: true,
        });
        let msg = A2AMessage::text("hello");
        let envelope = A2AEnvelope {
            message: msg,
            sender_id: "alice".into(),
            recipient_id: None,
            routing_mode: RoutingMode::Direct,
            relay_path: vec![],
            correlation_id: None,
            reply_to: None,
        };
        let result = router.route(envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_agents() {
        let router = setup_router();
        let agents = router.list_agents();
        assert_eq!(agents.len(), 3);
    }

    #[test]
    fn test_get_agent_returns_none_for_unknown() {
        let router = setup_router();
        assert!(router.get_agent("agent_unknown").is_none());
    }

    #[test]
    fn test_get_known_agent() {
        let router = setup_router();
        let agent = router.get_agent("agent_alice");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().endpoint, "ws://alice.local:9001");
    }
}
