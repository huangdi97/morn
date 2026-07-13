//! 多 Agent 协作模式 — 投票/路由/AgentAsTool/黑板等 7 种模式
use super::Orchestrator;
use crate::core::error::MornError;

pub struct DebateMode {
    pub agents: Vec<String>,
    pub rounds: u32,
    pub consensus_required: bool,
    pub orchestrator: Orchestrator,
}

pub struct VotingMode {
    pub agents: Vec<String>,
    pub threshold: f64,
    pub orchestrator: Orchestrator,
}

pub struct HierarchyMode {
    pub levels: Vec<String>,
    pub current_level: usize,
    pub orchestrator: Orchestrator,
}

pub struct SwarmMode {
    pub agents: Vec<String>,
    pub max_iterations: u32,
    pub convergence_threshold: f64,
    pub orchestrator: Orchestrator,
}

impl DebateMode {
    pub fn new(
        agents: Vec<String>,
        rounds: u32,
        consensus_required: bool,
        orchestrator: Orchestrator,
    ) -> Self {
        DebateMode {
            agents,
            rounds,
            consensus_required,
            orchestrator,
        }
    }

    pub fn execute(&self, task: &str) -> Result<String, MornError> {
        self.orchestrator
            .run_debate(&self.agents, task)
            .map(|outputs| {
                outputs
                    .into_iter()
                    .map(|o| o.output)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }
}

impl VotingMode {
    pub fn new(agents: Vec<String>, threshold: f64, orchestrator: Orchestrator) -> Self {
        VotingMode {
            agents,
            threshold,
            orchestrator,
        }
    }

    pub fn execute(&self, task: &str) -> Result<String, MornError> {
        self.orchestrator
            .run_voting(&self.agents, task)
            .map(|outputs| {
                outputs
                    .into_iter()
                    .map(|o| o.output)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }
}

impl HierarchyMode {
    pub fn new(levels: Vec<String>, current_level: usize, orchestrator: Orchestrator) -> Self {
        HierarchyMode {
            levels,
            current_level,
            orchestrator,
        }
    }

    pub fn execute(&self, task: &str) -> Result<String, MornError> {
        self.orchestrator
            .run_chain(&self.levels, task)
            .map(|outputs| {
                outputs
                    .into_iter()
                    .map(|o| o.output)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }
}

impl SwarmMode {
    pub fn new(
        agents: Vec<String>,
        max_iterations: u32,
        convergence_threshold: f64,
        orchestrator: Orchestrator,
    ) -> Self {
        SwarmMode {
            agents,
            max_iterations,
            convergence_threshold,
            orchestrator,
        }
    }

    pub fn execute(&self, task: &str) -> Result<String, MornError> {
        self.orchestrator
            .run_swarm(&self.agents, task)
            .map(|outputs| {
                outputs
                    .into_iter()
                    .map(|o| o.output)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
    }
}
