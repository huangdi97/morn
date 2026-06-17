//! Group execution manager — tracks registered groups, execution metrics, and cost estimates.

use crate::core::error::MornError;
use crate::core::orchestrator::group::modes;
use crate::core::orchestrator::group::AgentGroup;
use crate::core::orchestrator::CollaborationMode;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct GroupMetrics {
    pub group_id: String,
    pub execution_count: u64,
    pub total_cost_estimate: f64,
    pub last_executed_at: Option<SystemTime>,
    pub avg_duration_secs: f64,
}

impl GroupMetrics {
    pub fn new(group_id: &str) -> Self {
        GroupMetrics {
            group_id: group_id.to_string(),
            execution_count: 0,
            total_cost_estimate: 0.0,
            last_executed_at: None,
            avg_duration_secs: 0.0,
        }
    }
}

pub struct GroupExecutor {
    groups: HashMap<String, AgentGroup>,
    metrics: HashMap<String, GroupMetrics>,
}

impl GroupExecutor {
    pub fn new() -> Self {
        GroupExecutor {
            groups: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn register_group(&mut self, group: AgentGroup) {
        let gid = group.group_id.clone();
        self.metrics
            .entry(gid.clone())
            .or_insert_with(|| GroupMetrics::new(&gid));
        self.groups.insert(gid, group);
    }

    pub fn get_group(&self, group_id: &str) -> Option<&AgentGroup> {
        self.groups.get(group_id)
    }

    /// Returns a mutable reference to the metrics for a group, creating them if needed.
    pub fn get_metrics_mut(&mut self, group_id: &str) -> Option<&mut GroupMetrics> {
        self.metrics.get_mut(group_id)
    }

    /// Returns metrics for a group by id.
    pub fn group_metrics(&self, group_id: &str) -> Option<&GroupMetrics> {
        self.metrics.get(group_id)
    }

    /// Returns metrics for all registered groups.
    pub fn all_metrics(&self) -> Vec<&GroupMetrics> {
        self.metrics.values().collect()
    }

    /// Estimates the per-execution cost for a group based on its mode and agent count.
    pub fn group_cost_estimate(&self, group_id: &str) -> Result<f64, MornError> {
        let group = self
            .groups
            .get(group_id)
            .ok_or_else(|| format!("group '{}' not found", group_id))?;
        let base = modes::mode_base_cost(&group.mode);
        Ok(base * group.agent_ids.len() as f64)
    }

    /// Returns a supervisor-friendly summary string for a group.
    pub fn group_summary(&self, group_id: &str) -> Result<String, MornError> {
        let group = self
            .groups
            .get(group_id)
            .ok_or_else(|| format!("group '{}' not found", group_id))?;
        let metrics = self.metrics.get(group_id);
        let cost = self.group_cost_estimate(group_id)?;
        Ok(format!(
            "Group[{}] mode={:?} agents={} cost_per_run=¥{:.3} execs={} desc={}",
            group.group_id,
            group.mode,
            group.agent_ids.len(),
            cost,
            metrics.map(|m| m.execution_count).unwrap_or(0),
            modes::mode_description(&group.mode)
        ))
    }

    pub fn execute_group(
        &mut self,
        group_id: &str,
        _input: &str,
    ) -> Result<Vec<String>, MornError> {
        let cost_estimate = self.group_cost_estimate(group_id).ok();
        let agent_ids: Vec<String>;
        let mode: CollaborationMode;
        {
            let group = self
                .groups
                .get(group_id)
                .ok_or_else(|| format!("group '{}' not found", group_id))?;
            agent_ids = group.agent_ids.clone();
            mode = group.mode.clone();
        }

        let result = Ok(modes::handle_mode(&mode, &agent_ids, _input));

        if result.is_ok() {
            if let Some(m) = self.metrics.get_mut(group_id) {
                m.execution_count += 1;
                if let Some(cost) = cost_estimate {
                    m.total_cost_estimate += cost;
                }
                m.last_executed_at = Some(SystemTime::now());
            }
        }

        result
    }

    pub fn group_status(&self, group_id: &str) -> Result<String, MornError> {
        let group = self
            .groups
            .get(group_id)
            .ok_or_else(|| format!("group '{}' not found", group_id))?;
        Ok(format!(
            "group '{}' with {} agents in {:?} mode",
            group.group_id,
            group.agent_ids.len(),
            group.mode
        ))
    }

    pub fn groups(&self) -> Vec<&AgentGroup> {
        self.groups.values().collect()
    }
}

impl Default for GroupExecutor {
    fn default() -> Self {
        Self::new()
    }
}
