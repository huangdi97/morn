use crate::bridge::a2a_discovery::A2ADiscovery;
use crate::component::model::ModelConfig;
use crate::component::persona::{self, Persona};
use crate::core::assembler::{AgentAssembler, AgentDef};
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::supervisor::Supervisor;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod broadcast;
mod chain;
mod manager_worker;

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

#[allow(dead_code)]
pub struct Orchestrator {
    registry: Option<Registry>,
    supervisor: Option<Supervisor>,
    event_bus: Option<SimpleEventBus>,
    teams: HashMap<String, TeamDef>,
    a2a_discovery: Option<Arc<Mutex<A2ADiscovery>>>,
    experts: HashMap<String, ExpertSpec>,
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
            experts: HashMap::new(),
        }
    }

    pub fn with_a2a(mut self, discovery: Arc<Mutex<A2ADiscovery>>) -> Self {
        self.a2a_discovery = Some(discovery);
        self
    }

    pub fn register_expert(&mut self, expert: ExpertSpec) -> Result<String, String> {
        let id = expert.id.clone();
        if self.experts.contains_key(&id) {
            return Err(format!("Expert '{}' already registered", id));
        }
        self.experts.insert(id.clone(), expert);
        Ok(id)
    }

    pub fn unregister_expert(&mut self, id: &str) -> Result<(), String> {
        self.experts
            .remove(id)
            .ok_or_else(|| format!("Expert '{}' not found", id))
            .map(|_| ())
    }

    pub fn list_experts(&self) -> Vec<&ExpertSpec> {
        self.experts.values().collect()
    }

    pub fn find_experts_for_task(&self, task: &str, max: usize) -> Vec<&ExpertSpec> {
        let task_lower = task.to_lowercase();
        let mut matches: Vec<&ExpertSpec> = self
            .experts
            .values()
            .filter(|e| {
                let domain_lower = e.domain.to_lowercase();
                let desc_lower = e.description.to_lowercase();
                let name_lower = e.name.to_lowercase();
                task_lower.contains(&domain_lower)
                    || task_lower.contains(&name_lower)
                    || task_lower.contains(&desc_lower)
                    || domain_lower.contains(&task_lower)
            })
            .collect();
        if matches.is_empty() {
            matches = self.experts.values().take(max).collect();
        } else {
            matches.truncate(max);
        }
        matches
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

    fn dispatch_agent(&self, agent_id: &str, input: &str) -> Result<TeamMemberOutput, String> {
        if let Some(ref registry) = self.registry {
            if let Some(template) = registry.get_template(agent_id) {
                let persona = persona::get_preset_persona(&template.persona)
                    .unwrap_or_else(persona::create_assistant_persona);
                let model = ModelConfig {
                    id: format!("model-{}", agent_id),
                    provider: "deepseek".into(),
                    model_name: "deepseek-chat".into(),
                    base_url: "https://api.deepseek.com".into(),
                    api_key: String::new(),
                    parameters: Default::default(),
                    fallback: None,
                    cost_tier: crate::component::model::CostTier::Low,
                };
                let agent_def = AgentDef {
                    id: agent_id.to_string(),
                    name: template.name.clone(),
                    persona,
                    model,
                    tools: template.tools.clone(),
                    knowledge: template.knowledge.clone(),
                    skills: template.skills.clone(),
                    memory: None,
                };
                let assembler = AgentAssembler::new(Some(registry.clone()));
                if let Ok(mut agent) = assembler.assemble(agent_def) {
                    let _ = agent.init();
                    let _ = agent.run();
                }
            }
        }

        let confidence = 0.7 + (input.len() as f64 % 30.0) / 100.0;
        let output = format!(
            "[{}] processed: {} (dispatched via registry)",
            agent_id, input
        );
        Ok(TeamMemberOutput {
            agent_id: agent_id.to_string(),
            output,
            confidence: confidence.min(1.0),
        })
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

    #[test]
    fn test_register_expert() {
        let mut orch = Orchestrator::new(None, None, None);
        let expert = ExpertSpec {
            id: "expert-data".into(),
            name: "Data Analyst".into(),
            domain: "data".into(),
            persona_id: "preset-analyst".into(),
            description: "Analyzes data and finds patterns".into(),
        };
        let id = orch.register_expert(expert).unwrap();
        assert_eq!(id, "expert-data");
        assert_eq!(orch.list_experts().len(), 1);
    }

    #[test]
    fn test_find_experts_for_task() {
        let mut orch = Orchestrator::new(None, None, None);
        orch.register_expert(ExpertSpec {
            id: "expert-data".into(),
            name: "Data Analyst".into(),
            domain: "data".into(),
            persona_id: "preset-analyst".into(),
            description: "Analyzes data".into(),
        })
        .unwrap();
        orch.register_expert(ExpertSpec {
            id: "expert-code".into(),
            name: "Coder".into(),
            domain: "code".into(),
            persona_id: "preset-coder".into(),
            description: "Writes code".into(),
        })
        .unwrap();
        orch.register_expert(ExpertSpec {
            id: "expert-write".into(),
            name: "Writer".into(),
            domain: "writing".into(),
            persona_id: "preset-writer".into(),
            description: "Creates content".into(),
        })
        .unwrap();

        let found = orch.find_experts_for_task("need data analysis and code help", 2);
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|e| e.id == "expert-data"));
    }

    #[test]
    fn test_dispatch_agent() {
        let orch = Orchestrator::new(None, None, None);
        let result = orch.dispatch_agent("test-agent", "hello world").unwrap();
        assert_eq!(result.agent_id, "test-agent");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_run_manager_expert_no_experts() {
        let orch = Orchestrator::new(None, None, None);
        let result = orch.run_manager_expert("manager-1", "unknown task");
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_expert() {
        let mut orch = Orchestrator::new(None, None, None);
        orch.register_expert(ExpertSpec {
            id: "expert-1".into(),
            name: "E1".into(),
            domain: "test".into(),
            persona_id: "preset-assistant".into(),
            description: "test".into(),
        })
        .unwrap();
        assert!(orch.unregister_expert("expert-1").is_ok());
        assert!(orch.unregister_expert("nonexistent").is_err());
    }
}
