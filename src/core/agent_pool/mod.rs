use crate::core::registry::Registry;
use crate::core::trust_scorer::TrustScorer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod selector;
mod stats;

pub use stats::PoolStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInstance {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub status: String,
    pub current_task: Option<String>,
    pub resource_usage: ResourceUsage,
    pub trust_score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub active_sessions: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolConfig {
    pub max_agents: usize,
    pub max_concurrent: usize,
    pub max_memory_mb: f64,
    pub queue_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_agents: 100,
            max_concurrent: 20,
            max_memory_mb: 4096.0,
            queue_size: 500,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_id: String,
    pub input: String,
    pub priority: u8,
    pub status: String,
    pub result: Option<String>,
}

pub struct AgentPool {
    agents: HashMap<String, AgentInstance>,
    tasks: Vec<AgentTask>,
    config: PoolConfig,
    registry: Option<Registry>,
    scorer: Option<Arc<Mutex<TrustScorer>>>,
}

impl AgentPool {
    pub fn new(config: PoolConfig) -> Self {
        AgentPool {
            agents: HashMap::new(),
            tasks: Vec::new(),
            config,
            registry: None,
            scorer: None,
        }
    }

    pub fn with_registry(mut self, registry: Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_scorer(mut self, scorer: Arc<Mutex<TrustScorer>>) -> Self {
        self.scorer = Some(scorer);
        self
    }

    pub fn register_agent(
        &mut self,
        id: &str,
        name: &str,
        agent_type: &str,
    ) -> Result<String, String> {
        if self.agents.len() >= self.config.max_agents {
            return Err(format!(
                "Pool max agents ({}) reached",
                self.config.max_agents
            ));
        }
        if self.agents.contains_key(id) {
            return Err(format!("Agent '{}' already registered in pool", id));
        }
        let instance = AgentInstance {
            id: id.to_string(),
            name: name.to_string(),
            agent_type: agent_type.to_string(),
            status: "idle".into(),
            current_task: None,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_mb: 50.0,
                active_sessions: 0,
            },
            trust_score: 0.7,
        };
        self.agents.insert(id.to_string(), instance);
        Ok(id.to_string())
    }

    pub fn unregister_agent(&mut self, id: &str) -> Result<(), String> {
        self.agents
            .remove(id)
            .ok_or_else(|| format!("Agent '{}' not found", id))
            .map(|_| ())
    }

    pub fn submit_task(&mut self, task: AgentTask) -> Result<String, String> {
        if !self.agents.contains_key(&task.agent_id) {
            return Err(format!("Agent '{}' not found in pool", task.agent_id));
        }
        if self.tasks.len() >= self.config.queue_size {
            return Err("Task queue full".to_string());
        }
        let id = task.id.clone();
        self.tasks.push(task);
        Ok(id)
    }

    pub fn execute_all(&mut self) -> Vec<AgentTask> {
        let mut results = Vec::new();
        let mut remaining = Vec::new();

        for task in self.tasks.drain(..) {
            if results.len() >= self.config.max_concurrent {
                remaining.push(task);
                continue;
            }
            if let Some(agent) = self.agents.get_mut(&task.agent_id) {
                agent.status = "busy".into();
                agent.current_task = Some(task.id.clone());
                agent.resource_usage.active_sessions += 1;

                let result = format!("[{}] executed: {}", task.agent_id, task.input);
                let completed = AgentTask {
                    result: Some(result),
                    status: "completed".into(),
                    ..task
                };

                agent.status = "idle".into();
                agent.current_task = None;
                results.push(completed);
            } else {
                let failed = AgentTask {
                    status: "failed".into(),
                    result: Some("Agent not found".into()),
                    ..task
                };
                results.push(failed);
            }
        }

        self.tasks = remaining;
        results
    }

    pub fn execute_parallel(&mut self, tasks: Vec<AgentTask>) -> Vec<AgentTask> {
        let mut results = Vec::new();
        for task in tasks {
            if let Some(agent) = self.agents.get_mut(&task.agent_id) {
                agent.status = "busy".into();
                agent.current_task = Some(task.id.clone());
                let result = format!("[{}] parallel: {}", task.agent_id, task.input);
                agent.status = "idle".into();
                agent.current_task = None;
                results.push(AgentTask {
                    result: Some(result),
                    status: "completed".into(),
                    ..task
                });
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_agent() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "TestAgent", "worker")
            .unwrap();
        assert_eq!(pool.list_agents().len(), 1);
    }

    #[test]
    fn test_duplicate_agent() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        assert!(pool.register_agent("agent-1", "B", "worker").is_err());
    }

    #[test]
    fn test_unregister_agent() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        pool.unregister_agent("agent-1").unwrap();
        assert_eq!(pool.list_agents().len(), 0);
    }

    #[test]
    fn test_submit_and_execute() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        let id = pool
            .submit_task(AgentTask {
                id: "task-1".into(),
                agent_id: "agent-1".into(),
                input: "hello".into(),
                priority: 1,
                status: "pending".into(),
                result: None,
            })
            .unwrap();
        assert_eq!(id, "task-1");
        let results = pool.execute_all();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "completed");
    }

    #[test]
    fn test_execute_parallel() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        pool.register_agent("agent-2", "B", "worker").unwrap();
        let tasks = vec![
            AgentTask {
                id: "t1".into(),
                agent_id: "agent-1".into(),
                input: "a".into(),
                priority: 1,
                status: "pending".into(),
                result: None,
            },
            AgentTask {
                id: "t2".into(),
                agent_id: "agent-2".into(),
                input: "b".into(),
                priority: 1,
                status: "pending".into(),
                result: None,
            },
        ];
        let results = pool.execute_parallel(tasks);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == "completed"));
    }

    #[test]
    fn test_pool_status() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        pool.register_agent("agent-2", "B", "worker").unwrap();
        let status = pool.pool_status();
        assert_eq!(status.total_agents, 2);
        assert_eq!(status.idle_agents, 2);
    }

    #[test]
    fn test_merge_results() {
        let pool = AgentPool::new(PoolConfig::default());
        let results = vec![
            AgentTask {
                id: "t1".into(),
                agent_id: "a1".into(),
                input: "x".into(),
                priority: 1,
                status: "completed".into(),
                result: Some("ok1".into()),
            },
            AgentTask {
                id: "t2".into(),
                agent_id: "a2".into(),
                input: "y".into(),
                priority: 1,
                status: "failed".into(),
                result: Some("error".into()),
            },
        ];
        let merged = pool.merge_results(&results);
        assert!(merged.contains("1/2 succeeded"));
        assert!(merged.contains("1/2 failed"));
    }

    #[test]
    fn test_list_idle_agents() {
        let mut pool = AgentPool::new(PoolConfig::default());
        pool.register_agent("agent-1", "A", "worker").unwrap();
        pool.register_agent("agent-2", "B", "worker").unwrap();
        assert_eq!(pool.list_idle_agents().len(), 2);
    }

    #[test]
    fn test_max_agents() {
        let mut pool = AgentPool::new(PoolConfig {
            max_agents: 2,
            ..Default::default()
        });
        pool.register_agent("a1", "A", "worker").unwrap();
        pool.register_agent("a2", "B", "worker").unwrap();
        assert!(pool.register_agent("a3", "C", "worker").is_err());
    }

    #[test]
    fn test_submit_nonexistent_agent() {
        let mut pool = AgentPool::new(PoolConfig::default());
        assert!(pool
            .submit_task(AgentTask {
                id: "t1".into(),
                agent_id: "ghost".into(),
                input: "x".into(),
                priority: 1,
                status: "pending".into(),
                result: None,
            })
            .is_err());
    }
}
