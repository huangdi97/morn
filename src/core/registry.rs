use std::collections::HashMap;

use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE, EVENT_SYSTEM_READY};
use crate::core::storage::Storage;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub actions: Vec<String>,
    pub description: String,
    pub trust_score: f64,
    pub total_calls: u64,
    pub success_calls: u64,
    pub avg_latency_ms: f64,
    pub visibility: String,
    pub owner_id: Option<String>,
    pub team_id: Option<String>,
    pub daily_quota: u64,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Registry {
    capabilities: HashMap<String, Capability>,
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl Registry {
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        let mut registry = Registry {
            capabilities: HashMap::new(),
            storage,
            event_bus,
        };

        registry.register_defaults();

        if let Some(ref bus) = registry.event_bus {
            bus.publish_event(
                EVENT_SYSTEM_READY,
                "registry",
                serde_json::json!({"status": "ready"}),
            );
        }

        registry
    }

    fn register_defaults(&mut self) {
        let default_cap = Capability {
            id: "chat-agent".to_string(),
            name: "Chat Agent".to_string(),
            domain: "general".to_string(),
            actions: vec![
                "chat".to_string(),
                "analyze".to_string(),
                "report".to_string(),
            ],
            description: "General purpose chat agent powered by LLM".to_string(),
            trust_score: 70.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "public".to_string(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        };
        self.capabilities
            .insert(default_cap.id.clone(), default_cap);
    }

    pub fn register(&mut self, capability: Capability) {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_CHAT_AGENT_RESPONSE,
                "registry",
                serde_json::json!({"action": "register", "capability_id": capability.id}),
            );
        }
        self.capabilities.insert(capability.id.clone(), capability);
    }

    pub fn unregister(&mut self, id: &str) -> Option<Capability> {
        self.capabilities.remove(id)
    }

    pub fn find_by_domain(&self, domain: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.domain == domain)
            .collect()
    }

    pub fn find_by_action(&self, action: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.actions.iter().any(|a| a == action))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    pub fn list_available(&self, user_id: Option<&str>, user_teams: &[String]) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| match c.visibility.as_str() {
                "public" => true,
                "private" => {
                    if let Some(uid) = user_id {
                        c.owner_id.as_deref() == Some(uid)
                    } else {
                        false
                    }
                }
                "team" => {
                    if let Some(ref tid) = c.team_id {
                        user_teams.iter().any(|ut| ut == tid)
                    } else {
                        false
                    }
                }
                _ => true,
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Capability> {
        self.capabilities.get_mut(id)
    }

    pub fn update_trust_score(&mut self, id: &str, success: bool, latency_ms: f64) {
        if let Some(cap) = self.capabilities.get_mut(id) {
            cap.total_calls += 1;
            if success {
                cap.success_calls += 1;
            }

            let execution_success = if cap.total_calls > 0 {
                cap.success_calls as f64 / cap.total_calls as f64
            } else {
                0.0
            };

            let latency_score = if latency_ms > 0.0 {
                (1000.0 / latency_ms).min(1.0)
            } else {
                0.0
            };

            cap.avg_latency_ms = if cap.total_calls > 1 {
                (cap.avg_latency_ms * (cap.total_calls as f64 - 1.0) + latency_ms)
                    / cap.total_calls as f64
            } else {
                latency_ms
            };

            cap.trust_score =
                70.0 * 0.3 + execution_success * 30.0 + latency_score * 20.0 + 50.0 * 0.2;
        }
    }
}
