//! A2A agent discovery registry — registration, heartbeat, pruning, and local agent tracking.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::protocol::{AgentCapability, ContentType, TransportProtocol};

const DEFAULT_TTL_SECONDS: u64 = 90;

/// Stored registration for a discovered A2A agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub capability: AgentCapability,
    pub registered_at: i64,
    pub last_seen: i64,
    pub ttl_seconds: u64,
    pub metadata: serde_json::Value,
}

impl AgentRegistration {
    pub fn new(capability: AgentCapability) -> Self {
        Self::with_ttl(capability, DEFAULT_TTL_SECONDS)
    }

    pub fn with_ttl(capability: AgentCapability, ttl_seconds: u64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            capability,
            registered_at: now,
            last_seen: now,
            ttl_seconds,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn is_expired_at(&self, timestamp: i64) -> bool {
        timestamp.saturating_sub(self.last_seen) > self.ttl_seconds as i64
    }
}

/// Query parameters for local A2A discovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub content_type: Option<ContentType>,
    pub transport: Option<TransportProtocol>,
    pub available_only: bool,
}

impl DiscoveryQuery {
    pub fn available() -> Self {
        Self {
            available_only: true,
            ..Self::default()
        }
    }

    pub fn with_content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn with_transport(mut self, transport: TransportProtocol) -> Self {
        self.transport = Some(transport);
        self
    }
}

/// In-memory A2A discovery registry with registration, heartbeat, and filtering.
#[derive(Debug, Clone)]
pub struct A2ADiscoveryRegistry {
    local_agent_id: Option<String>,
    agents: HashMap<String, AgentRegistration>,
    heartbeat_interval: Duration,
}

impl A2ADiscoveryRegistry {
    pub fn new() -> Self {
        Self {
            local_agent_id: None,
            agents: HashMap::new(),
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    pub fn with_local_agent(local_agent: AgentCapability) -> Self {
        let mut registry = Self::new();
        registry.register_local(local_agent);
        registry
    }

    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    pub fn register_local(&mut self, capability: AgentCapability) {
        self.local_agent_id = Some(capability.agent_id.clone());
        self.register_agent(capability);
    }

    pub fn register_agent(&mut self, capability: AgentCapability) {
        self.register_agent_with_ttl(capability, DEFAULT_TTL_SECONDS);
    }

    pub fn register_agent_with_ttl(&mut self, capability: AgentCapability, ttl_seconds: u64) {
        let registration = AgentRegistration::with_ttl(capability, ttl_seconds);
        self.agents
            .insert(registration.capability.agent_id.clone(), registration);
    }

    pub fn register(&mut self, registration: AgentRegistration) {
        self.agents
            .insert(registration.capability.agent_id.clone(), registration);
    }

    pub fn unregister_agent(&mut self, agent_id: &str) -> Option<AgentRegistration> {
        if self.local_agent_id.as_deref() == Some(agent_id) {
            self.local_agent_id = None;
        }
        self.agents.remove(agent_id)
    }

    pub fn heartbeat(&mut self, agent_id: &str) -> Result<(), String> {
        let registration = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("A2A discovery agent '{}' not registered", agent_id))?;

        registration.last_seen = chrono::Utc::now().timestamp();
        registration.capability.is_available = true;
        Ok(())
    }

    pub fn mark_unavailable(&mut self, agent_id: &str) -> Result<(), String> {
        let registration = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("A2A discovery agent '{}' not registered", agent_id))?;
        registration.capability.is_available = false;
        Ok(())
    }

    pub fn discover(&mut self, query: DiscoveryQuery) -> Vec<AgentCapability> {
        self.prune_expired();
        self.agents
            .values()
            .filter(|registration| Self::matches_query(registration, &query))
            .map(|registration| registration.capability.clone())
            .collect()
    }

    pub fn discover_available(&mut self) -> Vec<AgentCapability> {
        self.discover(DiscoveryQuery::available())
    }

    pub fn get_agent(&mut self, agent_id: &str) -> Option<AgentCapability> {
        self.prune_expired();
        self.agents
            .get(agent_id)
            .map(|registration| registration.capability.clone())
    }

    pub fn registrations(&mut self) -> Vec<AgentRegistration> {
        self.prune_expired();
        self.agents.values().cloned().collect()
    }

    pub fn local_agent(&mut self) -> Option<AgentCapability> {
        let agent_id = self.local_agent_id.clone()?;
        self.get_agent(&agent_id)
    }

    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn prune_expired(&mut self) -> usize {
        let before = self.agents.len();
        let now = chrono::Utc::now().timestamp();
        self.agents
            .retain(|_, registration| !registration.is_expired_at(now));
        before - self.agents.len()
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    fn matches_query(registration: &AgentRegistration, query: &DiscoveryQuery) -> bool {
        if query.available_only && !registration.capability.is_available {
            return false;
        }

        if let Some(content_type) = &query.content_type {
            if !registration
                .capability
                .supported_content_types
                .contains(content_type)
            {
                return false;
            }
        }

        if let Some(transport) = &query.transport {
            if !registration.capability.transport.contains(transport) {
                return false;
            }
        }

        true
    }
}

impl Default for A2ADiscoveryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(agent_id: &str, is_available: bool) -> AgentCapability {
        AgentCapability {
            agent_id: agent_id.to_string(),
            supported_content_types: vec![ContentType::Text, ContentType::Task],
            transport: vec![TransportProtocol::WebSocket],
            endpoint: format!("ws://{}.local:9000", agent_id),
            is_available,
        }
    }

    #[test]
    fn registers_and_discovers_available_agents() {
        let mut registry = A2ADiscoveryRegistry::new();
        registry.register_agent(capability("agent-a", true));
        registry.register_agent(capability("agent-b", false));

        let agents = registry.discover_available();

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_id, "agent-a");
    }

    #[test]
    fn heartbeat_marks_agent_available() {
        let mut registry = A2ADiscoveryRegistry::new();
        registry.register_agent(capability("agent-a", false));

        registry.heartbeat("agent-a").unwrap();

        assert!(registry.get_agent("agent-a").unwrap().is_available);
    }

    #[test]
    fn discovery_filters_by_content_and_transport() {
        let mut registry = A2ADiscoveryRegistry::new();
        registry.register_agent(capability("agent-a", true));
        registry.register_agent(AgentCapability {
            agent_id: "agent-b".to_string(),
            supported_content_types: vec![ContentType::Json],
            transport: vec![TransportProtocol::HttpSse],
            endpoint: "http://agent-b.local/sse".to_string(),
            is_available: true,
        });

        let agents = registry.discover(
            DiscoveryQuery::available()
                .with_content_type(ContentType::Task)
                .with_transport(TransportProtocol::WebSocket),
        );

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_id, "agent-a");
    }

    #[test]
    fn prunes_expired_agents() {
        let mut registry = A2ADiscoveryRegistry::new();
        let mut registration = AgentRegistration::with_ttl(capability("agent-a", true), 1);
        registration.last_seen = chrono::Utc::now().timestamp() - 5;
        registry.register(registration);

        assert_eq!(registry.prune_expired(), 1);
        assert_eq!(registry.agent_count(), 0);
    }
}
