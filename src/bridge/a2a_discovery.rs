//! a2a_discovery — Discovers peer agents and exchanges A2A agent cards.
use crate::core::error::MornError;
use crate::bridge::a2a::{A2AMessage, A2AProtocol, AgentCard};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing;

#[allow(dead_code)] /* 预留：A2A peer discovery 后台循环 */
pub struct A2ADiscovery {
    local_agent: AgentCard,
    remote_agents: Arc<Mutex<HashMap<String, AgentCard>>>,
    peer_endpoints: Vec<String>,
    running: bool,
}

impl A2ADiscovery {
    pub fn new(local_agent: AgentCard) -> Self {
        A2ADiscovery {
            local_agent,
            remote_agents: Arc::new(Mutex::new(HashMap::new())),
            peer_endpoints: Vec::new(),
            running: false,
        }
    }

    pub fn add_peer(&mut self, endpoint: &str) {
        self.peer_endpoints.push(endpoint.to_string());
    }

    pub fn discover_peers(&self) -> Result<Vec<AgentCard>, MornError> {
        let mut discovered = Vec::new();
        for endpoint in &self.peer_endpoints {
            let discovery_msg = A2AMessage::AgentDiscovery {
                query: "capabilities".into(),
            };
            match A2AProtocol::send(endpoint, &discovery_msg) {
                Ok(A2AMessage::AgentList { agents }) => {
                    for agent in agents {
                        let id = agent.id.clone();
                        {
                            let mut agents =
                                self.remote_agents.lock().map_err(|e| MornError::Internal(e.to_string()))?;
                            agents.insert(id, agent.clone());
                        }
                        discovered.push(agent);
                    }
                }
                Ok(other) => tracing::warn!("[Discovery] unexpected response from {}: {:?}", endpoint, other),
                Err(e) => tracing::warn!("[Discovery] peer unreachable at {}: {}", endpoint, e),
            }
        }
        Ok(discovered)
    }

    pub fn send_heartbeat(&self, endpoint: &str) -> Result<bool, MornError> {
        match A2AProtocol::send(endpoint, &A2AMessage::Heartbeat) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn assign_remote_task(
        &self,
        endpoint: &str,
        task_id: &str,
        input: &str,
        max_tokens: u32,
    ) -> Result<A2AMessage, MornError> {
        let msg = A2AMessage::TaskAssign {
            task_id: task_id.to_string(),
            input: input.to_string(),
            max_tokens,
        };
        A2AProtocol::send(endpoint, &msg)
    }

    pub fn poll_task_status(&self, endpoint: &str, task_id: &str) -> Result<A2AMessage, MornError> {
        let msg = A2AMessage::TaskStatus {
            task_id: task_id.to_string(),
            status: "polling".into(),
            progress: 0.0,
        };
        A2AProtocol::send(endpoint, &msg)
    }

    pub fn get_remote_agents(&self) -> Result<Vec<AgentCard>, MornError> {
        let agents = self.remote_agents.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        Ok(agents.values().cloned().collect())
    }
}

pub fn start_discovery_service(discovery: Arc<Mutex<A2ADiscovery>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(30));
        if let Ok(d) = discovery.lock() {
            if let Err(e) = d.discover_peers() {
                tracing::warn!("discover_peers failed: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::a2a::{A2AProtocol, AgentCard};

    fn make_agent_card(id: &str) -> AgentCard {
        AgentCard {
            id: id.into(),
            name: format!("Agent {}", id),
            description: "test agent".into(),
            version: "1.0".into(),
            capabilities: vec!["test".into()],
            endpoint: "http://localhost:9999".into(),
            public_key: "pk".into(),
        }
    }

    #[test]
    fn test_new_discovery() {
        let card = make_agent_card("local-1");
        let d = A2ADiscovery::new(card.clone());
        assert_eq!(d.local_agent.id, "local-1");
        assert!(d.peer_endpoints.is_empty());
    }

    #[test]
    fn test_add_peer() {
        let card = make_agent_card("local-1");
        let mut d = A2ADiscovery::new(card);
        d.add_peer("http://peer1:8080");
        d.add_peer("http://peer2:8080");
        assert_eq!(d.peer_endpoints.len(), 2);
    }

    #[test]
    fn test_initial_no_remote_agents() {
        let card = make_agent_card("local-1");
        let d = A2ADiscovery::new(card);
        let agents = d.get_remote_agents().unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_task_assign() {
        let msg = A2AMessage::TaskAssign {
            task_id: "task-42".into(),
            input: "analyze data".into(),
            max_tokens: 2048,
        };
        let data = A2AProtocol::serialize(&msg).unwrap();
        let back = A2AProtocol::deserialize(&data).unwrap();
        match back {
            A2AMessage::TaskAssign {
                task_id,
                input,
                max_tokens,
            } => {
                assert_eq!(task_id, "task-42");
                assert_eq!(input, "analyze data");
                assert_eq!(max_tokens, 2048);
            }
            _ => panic!("Expected TaskAssign"),
        }
    }

    #[test]
    fn test_discover_empty_peers() {
        let card = make_agent_card("local-1");
        let d = A2ADiscovery::new(card);
        let result = d.discover_peers();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
