//! Collaboration mode helpers — cost factors and execution-prefix mappings.

use crate::core::orchestrator::CollaborationMode;

/// Returns the base cost multiplier for a collaboration mode.
pub(crate) fn mode_base_cost(mode: &CollaborationMode) -> f64 {
    match mode {
        CollaborationMode::Chain => 0.02,
        CollaborationMode::Broadcast => 0.05,
        CollaborationMode::Voting => 0.08,
        CollaborationMode::Routing => 0.03,
        CollaborationMode::ManagerWorker => 0.06,
        CollaborationMode::AgentAsTool => 0.04,
        CollaborationMode::Blackboard => 0.07,
    }
}

/// Returns the execution-result-id prefix for a collaboration mode.
pub(crate) fn mode_exec_prefix(mode: &CollaborationMode) -> &'static str {
    match mode {
        CollaborationMode::Chain => "chain_exec_",
        CollaborationMode::Broadcast => "broadcast_exec_",
        CollaborationMode::Voting => "voting_exec_",
        CollaborationMode::Routing => "routing_exec_",
        CollaborationMode::ManagerWorker => "mw_exec_",
        CollaborationMode::AgentAsTool => "aat_exec_",
        CollaborationMode::Blackboard => "bb_exec_",
    }
}
