//! orchestrator — Coordinates multi-agent teams, routing modes, and shared execution.
use crate::bridge::a2a_discovery::A2ADiscovery;
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::supervisor::Supervisor;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod collaboration;
pub mod group;
mod helpers;
mod manager_worker;
mod modes;
pub mod team_builder;
pub mod team_presets;

#[allow(unused_imports)]
pub use manager_worker::*;
#[allow(unused_imports)]
pub use modes::*;
#[allow(unused_imports)]
pub use team_builder::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CollaborationMode {
    Chain,
    ManagerWorker,
    Broadcast,
    Voting,
    Routing,
    AgentAsTool,
    Blackboard,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExpertSpec {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub persona_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConsensusMechanism {
    Vote,
    CeoDecides,
    MungerVeto,
    AutoSynthesis,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamDef {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub mode: CollaborationMode,
    pub consensus: ConsensusMechanism,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamResult {
    pub team_id: String,
    pub outputs: Vec<TeamMemberOutput>,
    pub consensus_output: String,
    pub mode: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemberOutput {
    pub agent_id: String,
    pub output: String,
    pub confidence: f64,
}

#[allow(dead_code)] /* 预留：多 agent 编排器运行态暂未全部接入 */
pub struct Orchestrator {
    registry: Option<Registry>,
    supervisor: Option<Supervisor>,
    event_bus: Option<SimpleEventBus>,
    teams: HashMap<String, TeamDef>,
    a2a_discovery: Option<Arc<Mutex<A2ADiscovery>>>,
    experts: HashMap<String, ExpertSpec>,
}

impl Orchestrator {
    /// Creates an orchestrator with optional registry, supervisor, and event bus integrations.
    pub fn new(
        registry: Option<Registry>,
        supervisor: Option<Supervisor>,
        event_bus: Option<SimpleEventBus>,
    ) -> Self {
        Orchestrator {
            registry,
            supervisor,
            event_bus,
            teams: HashMap::new(),
            a2a_discovery: None,
            experts: HashMap::new(),
        }
    }

    /// Attaches A2A discovery to the orchestrator and returns the updated instance.
    pub fn with_a2a(mut self, discovery: Arc<Mutex<A2ADiscovery>>) -> Self {
        self.a2a_discovery = Some(discovery);
        self
    }

    /// Registers an expert specification and returns its id on success.
    pub fn register_expert(&mut self, expert: ExpertSpec) -> Result<String, String> {
        let id = expert.id.clone();
        if self.experts.contains_key(&id) {
            return Err(format!("Expert '{}' already registered", id));
        }
        self.experts.insert(id.clone(), expert);
        Ok(id)
    }

    /// Removes an expert by id and returns an error if it is not registered.
    pub fn unregister_expert(&mut self, id: &str) -> Result<(), String> {
        self.experts
            .remove(id)
            .ok_or_else(|| format!("Expert '{}' not found", id))
            .map(|_| ())
    }

    /// Returns references to all registered experts.
    pub fn list_experts(&self) -> Vec<&ExpertSpec> {
        self.experts.values().collect()
    }

    /// Creates a team definition and returns its id when no duplicate exists.
    pub fn create_team(&mut self, def: TeamDef) -> Result<String, String> {
        let id = def.id.clone();
        if self.teams.contains_key(&id) {
            return Err(format!("Team '{}' already exists", id));
        }
        self.teams.insert(id.clone(), def);
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                "orchestrator.team.created",
                "orchestrator",
                serde_json::json!({"team_id": id}),
            );
        }
        Ok(id)
    }

    /// Looks up a team by id and returns it when found.
    pub fn get_team(&self, id: &str) -> Option<&TeamDef> {
        self.teams.get(id)
    }

    /// Returns references to all registered teams.
    pub fn list_teams(&self) -> Vec<&TeamDef> {
        self.teams.values().collect()
    }

    /// Deletes a team by id and returns an error if it does not exist.
    pub fn delete_team(&mut self, id: &str) -> Result<(), String> {
        self.teams
            .remove(id)
            .ok_or_else(|| format!("Team '{}' not found", id))
            .map(|_| ())
    }

    /// Runs a team against input text and returns member outputs plus the consensus result.
    pub fn run_team(&self, team_id: &str, input: &str) -> Result<TeamResult, String> {
        let team = self
            .teams
            .get(team_id)
            .ok_or_else(|| format!("Team '{}' not found", team_id))?;

        let outputs = match team.mode {
            CollaborationMode::Chain => self.run_chain(&team.members, input)?,
            CollaborationMode::ManagerWorker => self.run_manager_worker(&team.members, input)?,
            CollaborationMode::Broadcast => self.run_broadcast(&team.members, input)?,
            CollaborationMode::Voting => self.run_voting(&team.members, input)?,
            CollaborationMode::Routing => self.run_routing(&team.members, input)?,
            CollaborationMode::AgentAsTool => self.run_agent_as_tool(&team.members, input)?,
            CollaborationMode::Blackboard => self.run_blackboard(&team.members, input)?,
        };

        let consensus_output = self.compute_consensus(&outputs, &team.consensus);

        let result = TeamResult {
            team_id: team_id.to_string(),
            outputs,
            consensus_output,
            mode: team.mode.as_str().to_string(),
        };

        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
