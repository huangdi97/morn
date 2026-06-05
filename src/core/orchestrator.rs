use crate::bridge::a2a_discovery::A2ADiscovery;
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::supervisor::Supervisor;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

impl CollaborationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CollaborationMode::Chain => "chain",
            CollaborationMode::ManagerWorker => "manager_worker",
            CollaborationMode::Broadcast => "broadcast",
            CollaborationMode::Voting => "voting",
            CollaborationMode::Routing => "routing",
            CollaborationMode::AgentAsTool => "agent_as_tool",
            CollaborationMode::Blackboard => "blackboard",
        }
    }
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

#[allow(dead_code)]
pub struct Orchestrator {
    registry: Option<Registry>,
    supervisor: Option<Supervisor>,
    event_bus: Option<SimpleEventBus>,
    teams: HashMap<String, TeamDef>,
    a2a_discovery: Option<Arc<Mutex<A2ADiscovery>>>,
}

impl Orchestrator {
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
        }
    }

    pub fn with_a2a(mut self, discovery: Arc<Mutex<A2ADiscovery>>) -> Self {
        self.a2a_discovery = Some(discovery);
        self
    }

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

    pub fn get_team(&self, id: &str) -> Option<&TeamDef> {
        self.teams.get(id)
    }

    pub fn list_teams(&self) -> Vec<&TeamDef> {
        self.teams.values().collect()
    }

    pub fn delete_team(&mut self, id: &str) -> Result<(), String> {
        self.teams
            .remove(id)
            .ok_or_else(|| format!("Team '{}' not found", id))
            .map(|_| ())
    }

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

    fn simulate_agent_call(&self, agent_id: &str, input: &str) -> Result<TeamMemberOutput, String> {
        let confidence = 0.5 + (input.len() as f64 % 50.0) / 100.0;
        let output = format!("[{}] processed: {}", agent_id, input);
        Ok(TeamMemberOutput {
            agent_id: agent_id.to_string(),
            output,
            confidence,
        })
    }

    fn run_chain(&self, members: &[String], input: &str) -> Result<Vec<TeamMemberOutput>, String> {
        if members.is_empty() {
            return Err("No members in chain".to_string());
        }
        let mut outputs = Vec::new();
        let mut current = input.to_string();
        for member in members {
            let result = self.simulate_agent_call(member, &current)?;
            current = result.output.clone();
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn run_manager_worker(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        if members.is_empty() {
            return Err("No members".to_string());
        }
        let mut outputs = Vec::new();
        let manager = &members[0];
        let mgr = self.simulate_agent_call(manager, input)?;
        outputs.push(mgr);

        for worker in &members[1..] {
            let result =
                self.simulate_agent_call(worker, &format!("{} (from {})", input, manager))?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn run_broadcast(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut outputs = Vec::new();
        for member in members {
            let result = self.simulate_agent_call(member, &format!("[BROADCAST] {}", input))?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn run_voting(&self, members: &[String], input: &str) -> Result<Vec<TeamMemberOutput>, String> {
        if members.len() < 3 {
            return Err("Voting mode requires at least 3 members".to_string());
        }
        let mut outputs = Vec::new();
        for member in members {
            let result = self.simulate_agent_call(member, &format!("[EVALUATE] {}", input))?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn run_routing(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        if members.is_empty() {
            return Err("No members for routing".to_string());
        }
        let idx = input.len() % members.len();
        let selected = &members[idx];
        let result = self.simulate_agent_call(selected, &format!("[ROUTED] {}", input))?;
        Ok(vec![result])
    }

    fn run_agent_as_tool(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut outputs = Vec::new();
        let primary = if members.is_empty() {
            return Err("No members".to_string());
        } else {
            &members[0]
        };
        let primary_result = self.simulate_agent_call(primary, input)?;
        outputs.push(primary_result);

        for tool_agent in &members[1..] {
            let result = self.simulate_agent_call(
                tool_agent,
                &format!("[TOOL] {} called by {}", input, primary),
            )?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn run_blackboard(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut board = format!("[Blackboard] Initial: {}\n", input);
        let mut outputs = Vec::new();
        for member in members {
            let result = self.simulate_agent_call(member, &board)?;
            board.push_str(&format!("{}: {}\n", member, result.output));
            outputs.push(result);
        }
        Ok(outputs)
    }

    fn compute_consensus(
        &self,
        outputs: &[TeamMemberOutput],
        mechanism: &ConsensusMechanism,
    ) -> String {
        match mechanism {
            ConsensusMechanism::CeoDecides => outputs
                .first()
                .map(|o| o.output.clone())
                .unwrap_or_default(),
            ConsensusMechanism::Vote => {
                let best = outputs.iter().max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                best.map(|o| o.output.clone()).unwrap_or_default()
            }
            ConsensusMechanism::MungerVeto => {
                let worst = outputs.iter().min_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                worst
                    .map(|o| format!("[VETO] {}", o.output))
                    .unwrap_or_default()
            }
            ConsensusMechanism::AutoSynthesis => {
                let combined: Vec<String> = outputs.iter().map(|o| o.output.clone()).collect();
                format!(
                    "[Synthesis of {} opinions] {}",
                    outputs.len(),
                    combined.join(" | ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_team(id: &str, mode: CollaborationMode) -> TeamDef {
        TeamDef {
            id: id.to_string(),
            name: format!("Team {}", id),
            members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
            mode,
            consensus: ConsensusMechanism::CeoDecides,
        }
    }

    #[test]
    fn test_create_team() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = create_test_team("team-1", CollaborationMode::Chain);
        let id = orch.create_team(team).unwrap();
        assert_eq!(id, "team-1");
    }

    #[test]
    fn test_duplicate_team() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = create_test_team("team-1", CollaborationMode::Chain);
        orch.create_team(team).unwrap();
        let dup = create_test_team("team-1", CollaborationMode::Broadcast);
        assert!(orch.create_team(dup).is_err());
    }

    #[test]
    fn test_chain_mode() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = create_test_team("team-chain", CollaborationMode::Chain);
        orch.create_team(team).unwrap();
        let result = orch.run_team("team-chain", "hello").unwrap();
        assert_eq!(result.mode, "chain");
        assert_eq!(result.outputs.len(), 3);
    }

    #[test]
    fn test_broadcast_mode() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = create_test_team("team-bc", CollaborationMode::Broadcast);
        orch.create_team(team).unwrap();
        let result = orch.run_team("team-bc", "hello").unwrap();
        assert_eq!(result.outputs.len(), 3);
    }

    #[test]
    fn test_consensus_vote() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = TeamDef {
            id: "team-vote".into(),
            name: "Vote Team".into(),
            members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
            mode: CollaborationMode::Voting,
            consensus: ConsensusMechanism::Vote,
        };
        orch.create_team(team).unwrap();
        let result = orch.run_team("team-vote", "evaluate").unwrap();
        assert_eq!(result.outputs.len(), 3);
        assert!(!result.consensus_output.is_empty());
    }

    #[test]
    fn test_list_teams() {
        let mut orch = Orchestrator::new(None, None, None);
        orch.create_team(create_test_team("t1", CollaborationMode::Chain))
            .unwrap();
        orch.create_team(create_test_team("t2", CollaborationMode::Broadcast))
            .unwrap();
        assert_eq!(orch.list_teams().len(), 2);
    }

    #[test]
    fn test_delete_team() {
        let mut orch = Orchestrator::new(None, None, None);
        orch.create_team(create_test_team("t1", CollaborationMode::Chain))
            .unwrap();
        orch.delete_team("t1").unwrap();
        assert!(orch.get_team("t1").is_none());
    }

    #[test]
    fn test_routing_mode() {
        let mut orch = Orchestrator::new(None, None, None);
        let team = create_test_team("team-route", CollaborationMode::Routing);
        orch.create_team(team).unwrap();
        let result = orch.run_team("team-route", "route me").unwrap();
        assert_eq!(result.outputs.len(), 1);
    }

    #[test]
    fn test_consensus_mechanisms() {
        let mut orch = Orchestrator::new(None, None, None);
        for (mode_name, mechanism) in [
            ("ceo", ConsensusMechanism::CeoDecides),
            ("vote", ConsensusMechanism::Vote),
            ("veto", ConsensusMechanism::MungerVeto),
            ("synth", ConsensusMechanism::AutoSynthesis),
        ] {
            let team = TeamDef {
                id: format!("team-{}", mode_name),
                name: format!("{} Team", mode_name),
                members: vec!["agent-a".into(), "agent-b".into(), "agent-c".into()],
                mode: CollaborationMode::Voting,
                consensus: mechanism,
            };
            orch.create_team(team).unwrap();
            let result = orch
                .run_team(&format!("team-{}", mode_name), "test")
                .unwrap();
            assert!(!result.consensus_output.is_empty());
        }
    }
}
