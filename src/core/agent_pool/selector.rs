//! selector — Selects suitable pooled agents for incoming tasks.
use super::{AgentInstance, AgentPool};

impl AgentPool {
    pub fn get_agent(&self, id: &str) -> Option<&AgentInstance> {
        self.agents.get(id)
    }

    pub fn list_agents(&self) -> Vec<&AgentInstance> {
        self.agents.values().collect()
    }

    pub fn list_agents_by_type(&self, agent_type: &str) -> Vec<&AgentInstance> {
        self.agents
            .values()
            .filter(|a| a.agent_type == agent_type)
            .collect()
    }

    pub fn list_idle_agents(&self) -> Vec<&AgentInstance> {
        self.agents
            .values()
            .filter(|a| a.status == "idle")
            .collect()
    }
}
