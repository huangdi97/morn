//! Collaboration mode helpers — mode catalog, descriptions, and lightweight handlers.

use crate::core::orchestrator::CollaborationMode;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GroupCollaborationMode {
    Debate,
    Voting,
    RoundRobin,
    Broadcast,
    ManagerWorker,
    Consensus,
    Swarm,
}

impl GroupCollaborationMode {
    pub fn description(&self) -> &'static str {
        match self {
            GroupCollaborationMode::Debate => {
                "Agents argue opposing positions across rounds before synthesis."
            }
            GroupCollaborationMode::Voting => {
                "Agents evaluate independently and select the strongest answer by vote."
            }
            GroupCollaborationMode::RoundRobin => {
                "Agents take sequential turns, each receiving the previous context."
            }
            GroupCollaborationMode::Broadcast => {
                "The same input is sent to every agent for parallel independent work."
            }
            GroupCollaborationMode::ManagerWorker => {
                "A manager agent delegates subtasks to workers and collects their results."
            }
            GroupCollaborationMode::Consensus => {
                "Agents produce independent outputs, then converge on a shared synthesis."
            }
            GroupCollaborationMode::Swarm => {
                "Agents iteratively explore and refine work with lightweight coordination."
            }
        }
    }

    pub fn base_cost(&self) -> f64 {
        match self {
            GroupCollaborationMode::Debate => 0.09,
            GroupCollaborationMode::Voting => 0.08,
            GroupCollaborationMode::RoundRobin => 0.03,
            GroupCollaborationMode::Broadcast => 0.05,
            GroupCollaborationMode::ManagerWorker => 0.06,
            GroupCollaborationMode::Consensus => 0.10,
            GroupCollaborationMode::Swarm => 0.12,
        }
    }

    pub fn exec_prefix(&self) -> &'static str {
        match self {
            GroupCollaborationMode::Debate => "debate_exec_",
            GroupCollaborationMode::Voting => "voting_exec_",
            GroupCollaborationMode::RoundRobin => "rr_exec_",
            GroupCollaborationMode::Broadcast => "broadcast_exec_",
            GroupCollaborationMode::ManagerWorker => "mw_exec_",
            GroupCollaborationMode::Consensus => "consensus_exec_",
            GroupCollaborationMode::Swarm => "swarm_exec_",
        }
    }

    pub fn handle(&self, agent_ids: &[String], input: &str) -> Vec<String> {
        match self {
            GroupCollaborationMode::Debate => handle_debate(agent_ids, input),
            GroupCollaborationMode::Voting => handle_voting(agent_ids, input),
            GroupCollaborationMode::RoundRobin => handle_round_robin(agent_ids, input),
            GroupCollaborationMode::Broadcast => handle_broadcast(agent_ids, input),
            GroupCollaborationMode::ManagerWorker => handle_manager_worker(agent_ids, input),
            GroupCollaborationMode::Consensus => handle_consensus(agent_ids, input),
            GroupCollaborationMode::Swarm => handle_swarm(agent_ids, input),
        }
    }
}

impl From<&CollaborationMode> for GroupCollaborationMode {
    fn from(mode: &CollaborationMode) -> Self {
        match mode {
            CollaborationMode::Debate => GroupCollaborationMode::Debate,
            CollaborationMode::Voting => GroupCollaborationMode::Voting,
            CollaborationMode::RoundRobin | CollaborationMode::Chain => {
                GroupCollaborationMode::RoundRobin
            }
            CollaborationMode::Broadcast => GroupCollaborationMode::Broadcast,
            CollaborationMode::ManagerWorker
            | CollaborationMode::Routing
            | CollaborationMode::AgentAsTool => GroupCollaborationMode::ManagerWorker,
            CollaborationMode::Consensus => GroupCollaborationMode::Consensus,
            CollaborationMode::Swarm | CollaborationMode::Blackboard => {
                GroupCollaborationMode::Swarm
            }
        }
    }
}

fn format_results(mode: &GroupCollaborationMode, agent_ids: &[String], tag: &str) -> Vec<String> {
    agent_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| format!("{}{}:{}:{}", mode.exec_prefix(), id, tag, idx + 1))
        .collect()
}

pub fn handle_debate(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(&GroupCollaborationMode::Debate, agent_ids, "argument")
}

pub fn handle_voting(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(&GroupCollaborationMode::Voting, agent_ids, "vote")
}

pub fn handle_round_robin(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(&GroupCollaborationMode::RoundRobin, agent_ids, "turn")
}

pub fn handle_broadcast(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(&GroupCollaborationMode::Broadcast, agent_ids, "broadcast")
}

pub fn handle_manager_worker(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(
        &GroupCollaborationMode::ManagerWorker,
        agent_ids,
        "delegation",
    )
}

pub fn handle_consensus(agent_ids: &[String], _input: &str) -> Vec<String> {
    let mut results = format_results(&GroupCollaborationMode::Consensus, agent_ids, "proposal");
    if !agent_ids.is_empty() {
        results.push("consensus_exec_synthesis".to_string());
    }
    results
}

pub fn handle_swarm(agent_ids: &[String], _input: &str) -> Vec<String> {
    format_results(&GroupCollaborationMode::Swarm, agent_ids, "iteration")
}

/// Returns the base cost multiplier for a collaboration mode.
pub(crate) fn mode_base_cost(mode: &CollaborationMode) -> f64 {
    GroupCollaborationMode::from(mode).base_cost()
}

/// Returns the execution-result-id prefix for a collaboration mode.
#[allow(dead_code)]
pub(crate) fn mode_exec_prefix(mode: &CollaborationMode) -> &'static str {
    GroupCollaborationMode::from(mode).exec_prefix()
}

pub(crate) fn mode_description(mode: &CollaborationMode) -> &'static str {
    GroupCollaborationMode::from(mode).description()
}

pub(crate) fn handle_mode(
    mode: &CollaborationMode,
    agent_ids: &[String],
    input: &str,
) -> Vec<String> {
    GroupCollaborationMode::from(mode).handle(agent_ids, input)
}
