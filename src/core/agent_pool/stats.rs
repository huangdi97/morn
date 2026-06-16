//! stats — Calculates utilization and performance statistics for agent pools.
use super::{AgentPool, AgentTask};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolStatus {
    pub total_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub queued_tasks: usize,
    pub max_agents: usize,
    pub memory_used_mb: f64,
}

impl AgentPool {
    pub fn merge_results(&self, results: &[AgentTask]) -> String {
        let total = results.len();
        let success = results.iter().filter(|r| r.status == "completed").count();
        let failed = total - success;
        let mut merged = String::new();
        for r in results {
            if let Some(ref result) = r.result {
                merged.push_str(&format!("[{}] {}\n", r.agent_id, result));
            }
        }
        format!(
            "Pool Results: {}/{} succeeded, {}/{} failed.\n{}",
            success, total, failed, total, merged
        )
    }

    pub fn pool_status(&self) -> PoolStatus {
        let total = self.agents.len();
        let idle = self.agents.values().filter(|a| a.status == "idle").count();
        let busy = total - idle;
        PoolStatus {
            total_agents: total,
            idle_agents: idle,
            busy_agents: busy,
            queued_tasks: self.tasks.len(),
            max_agents: self.config.max_agents,
            memory_used_mb: self
                .agents
                .values()
                .map(|a| a.resource_usage.memory_mb)
                .sum(),
        }
    }
}
