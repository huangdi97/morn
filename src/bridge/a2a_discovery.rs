use crate::bridge::a2a::{A2AMessage, A2AProtocol, AgentCard};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[allow(dead_code)]
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

    pub fn discover_peers(&self) -> Result<Vec<AgentCard>, String> {
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
                            let mut agents = self.remote_agents.lock().map_err(|e| e.to_string())?;
                            agents.insert(id, agent.clone());
                        }
                        discovered.push(agent);
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
        Ok(discovered)
    }

    pub fn send_heartbeat(&self, endpoint: &str) -> Result<bool, String> {
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
    ) -> Result<A2AMessage, String> {
        let msg = A2AMessage::TaskAssign {
            task_id: task_id.to_string(),
            input: input.to_string(),
            max_tokens,
        };
        A2AProtocol::send(endpoint, &msg)
    }

    pub fn poll_task_status(
        &self,
        endpoint: &str,
        task_id: &str,
    ) -> Result<A2AMessage, String> {
        let msg = A2AMessage::TaskStatus {
            task_id: task_id.to_string(),
            status: "polling".into(),
            progress: 0.0,
        };
        A2AProtocol::send(endpoint, &msg)
    }

    pub fn get_remote_agents(&self) -> Result<Vec<AgentCard>, String> {
        let agents = self.remote_agents.lock().map_err(|e| e.to_string())?;
        Ok(agents.values().cloned().collect())
    }
}

pub fn start_discovery_service(discovery: Arc<Mutex<A2ADiscovery>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(30));
            if let Ok(d) = discovery.lock() {
                let _ = d.discover_peers();
            }
        }
    });
}
