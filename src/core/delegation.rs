use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::registry::Registry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationTask {
    pub action: String,
    pub input: String,
    pub domain: Option<String>,
    pub required_tools: Vec<String>,
    pub context: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    pub delegated_to: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilitySummary {
    pub agent_id: String,
    pub domain: String,
    pub actions: Vec<String>,
    pub trust_score: f64,
}

pub struct DelegationManager {
    registry: Arc<Registry>,
}

impl DelegationManager {
    pub fn new(registry: Arc<Registry>) -> Self {
        DelegationManager { registry }
    }

    pub async fn delegate(
        &self,
        from_agent: &str,
        task: DelegationTask,
    ) -> Result<DelegationResult, String> {
        let best = self.find_best_agent(&task);
        let target = best.ok_or_else(|| {
            format!(
                "No suitable agent found for task action '{}' (requested by agent '{}')",
                task.action, from_agent
            )
        })?;

        let start = std::time::Instant::now();

        let result = DelegationResult {
            delegated_to: target.clone(),
            success: true,
            output: format!(
                "[Delegated] Agent '{}' delegated action '{}' to agent '{}'",
                from_agent, task.action, target
            ),
            error: None,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        };

        Ok(result)
    }

    pub fn find_best_agent(&self, task: &DelegationTask) -> Option<String> {
        let all_capabilities = self.registry.list_all();

        let mut candidates: Vec<AgentCapabilitySummary> = all_capabilities
            .iter()
            .filter(|cap| {
                let domain_match = match &task.domain {
                    Some(d) => cap.domain == *d,
                    None => true,
                };
                let action_match = cap.actions.iter().any(|a| *a == task.action);
                let tool_match = if task.required_tools.is_empty() {
                    false
                } else {
                    task.required_tools.iter().any(|t| cap.actions.contains(t))
                };
                domain_match && (action_match || tool_match)
            })
            .map(|cap| AgentCapabilitySummary {
                agent_id: cap.id.clone(),
                domain: cap.domain.clone(),
                actions: cap.actions.clone(),
                trust_score: cap.trust_score,
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.trust_score
                .partial_cmp(&a.trust_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates.into_iter().next().map(|c| c.agent_id)
    }

    pub fn list_available_delegates(&self, domain: Option<&str>) -> Vec<AgentCapabilitySummary> {
        let all = self.registry.list_all();
        all.iter()
            .filter(|cap| match domain {
                Some(d) => cap.domain == d,
                None => true,
            })
            .map(|cap| AgentCapabilitySummary {
                agent_id: cap.id.clone(),
                domain: cap.domain.clone(),
                actions: cap.actions.clone(),
                trust_score: cap.trust_score,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::registry::{Capability, Registry};

    fn test_registry() -> Arc<Registry> {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let bus = crate::core::event_bus::SimpleEventBus::new();
        let reg = Registry::new(Some(storage), Some(bus));
        Arc::new(reg)
    }

    fn make_cap(reg: &mut Registry) {
        reg.register(Capability {
            id: "test-analyst".into(),
            name: "Test Analyst".into(),
            domain: "analysis".into(),
            actions: vec!["analyze".into(), "report".into()],
            description: "Test agent for analysis".into(),
            trust_score: 85.0,
            total_calls: 100,
            success_calls: 90,
            avg_latency_ms: 500.0,
            visibility: "public".into(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        });
        reg.register(Capability {
            id: "test-researcher".into(),
            name: "Test Researcher".into(),
            domain: "research".into(),
            actions: vec!["search".into(), "analyze".into()],
            description: "Test agent for research".into(),
            trust_score: 90.0,
            total_calls: 200,
            success_calls: 190,
            avg_latency_ms: 300.0,
            visibility: "public".into(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
        });
    }

    #[test]
    fn test_find_best_agent_by_action() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let bus = crate::core::event_bus::SimpleEventBus::new();
        let mut reg = Registry::new(Some(storage), Some(bus));
        make_cap(&mut reg);
        let mgr = DelegationManager::new(Arc::new(reg));

        let task = DelegationTask {
            action: "search".into(),
            input: "test query".into(),
            domain: None,
            required_tools: vec![],
            context: serde_json::json!({}),
        };
        let best = mgr.find_best_agent(&task);
        assert!(best.is_some());
        assert_eq!(best.unwrap(), "test-researcher");
    }

    #[test]
    fn test_find_best_agent_by_domain() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let bus = crate::core::event_bus::SimpleEventBus::new();
        let mut reg = Registry::new(Some(storage), Some(bus));
        make_cap(&mut reg);
        let mgr = DelegationManager::new(Arc::new(reg));

        let task = DelegationTask {
            action: "analyze".into(),
            input: "data".into(),
            domain: Some("analysis".into()),
            required_tools: vec![],
            context: serde_json::json!({}),
        };
        let best = mgr.find_best_agent(&task);
        assert!(best.is_some());
        assert_eq!(best.unwrap(), "test-analyst");
    }

    #[test]
    fn test_list_available_delegates() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let bus = crate::core::event_bus::SimpleEventBus::new();
        let mut reg = Registry::new(Some(storage), Some(bus));
        make_cap(&mut reg);
        let mgr = DelegationManager::new(Arc::new(reg));

        let all = mgr.list_available_delegates(None);
        assert!(all.len() >= 2);

        let analysis_only = mgr.list_available_delegates(Some("analysis"));
        assert_eq!(analysis_only.len(), 1);
        assert_eq!(analysis_only[0].agent_id, "test-analyst");
    }

    #[test]
    fn test_delegate_returns_error_when_no_agent() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let bus = crate::core::event_bus::SimpleEventBus::new();
        let reg = Registry::new(Some(storage), Some(bus));
        let mgr = DelegationManager::new(Arc::new(reg));

        let task = DelegationTask {
            action: "nonexistent".into(),
            input: "test".into(),
            domain: None,
            required_tools: vec![],
            context: serde_json::json!({}),
        };
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(mgr.delegate("sender", task));
        assert!(result.is_err());
    }
}
