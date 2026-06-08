//! supervisor — Oversees task execution decisions, plans, and risk controls.
mod decision;
mod execution;
mod guided_builder;
mod learning;
mod rule_commands;
mod types;
mod workflow_approvals;

pub use guided_builder::*;
pub use learning::*;
pub use types::*;

use crate::core::approval::WorkflowApproval;
use crate::core::event_bus::SimpleEventBus;
use crate::core::orchestrator::team_presets;
use crate::core::orchestrator::{CollaborationMode, ConsensusMechanism, TeamDef};
use crate::core::storage::Storage;
use crate::core::thread_pool::TaskPool;

pub type ChatFn = dyn Fn(&str, &str) -> Result<String, String>;

pub struct Supervisor {
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
    history: Vec<TurnRecord>,
    max_history: usize,
    turn_count: u64,
    mode: Mode,
    decision_override: Option<DecisionOverride>,
    task_pool: TaskPool,
    workflow_approvals: Vec<WorkflowApproval>,
    guided_builder: Option<GuidedBuilder>,
    learning_engine: Option<LearningEngine>,
    user_id: Option<String>,
    user_teams: Vec<String>,
}

impl Supervisor {
    /// Creates a supervisor with optional storage and event bus integrations and returns the initialized instance.
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        let learning_engine = storage
            .as_ref()
            .map(|storage| LearningEngine::new(Some(storage.clone()), 50.0));
        Supervisor {
            storage,
            event_bus,
            history: Vec::new(),
            max_history: 10,
            turn_count: 0,
            mode: Mode::Proactive,
            decision_override: None,
            task_pool: TaskPool::default(),
            workflow_approvals: Vec::new(),
            guided_builder: None,
            learning_engine,
            user_id: None,
            user_teams: Vec::new(),
        }
    }

    /// Attaches a user id and team list to the supervisor and returns the updated instance.
    pub fn with_user(mut self, user_id: &str, teams: &[String]) -> Self {
        self.user_id = Some(user_id.to_string());
        self.user_teams = teams.to_vec();
        self
    }

    /// Sets the active user id and team list used for visibility and governance checks.
    pub fn set_user(&mut self, user_id: &str, teams: &[String]) {
        self.user_id = Some(user_id.to_string());
        self.user_teams = teams.to_vec();
    }

    /// Returns the active user id if one has been configured.
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    /// Returns the team ids associated with the active user.
    pub fn user_teams(&self) -> &[String] {
        &self.user_teams
    }

    /// Returns the number of chat or plan turns executed by this supervisor.
    pub fn turn_count(&self) -> u64 {
        self.turn_count
    }
    /// Returns the retained conversation history records.
    pub fn history(&self) -> &[TurnRecord] {
        &self.history
    }
    /// Returns the current COO execution mode.
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the current COO execution mode.
    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the shared task pool used by workflow execution paths.
    pub fn task_pool(&self) -> &TaskPool {
        &self.task_pool
    }

    /// Sets the COO execution mode that controls approval and automation behavior.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Forces a decision level until its configured scope expires.
    pub fn override_decision(&mut self, level: DecisionLevel, scope: OverrideScope) {
        self.decision_override = Some(DecisionOverride { level, scope });
    }

    /// Returns the active decision override if one has been configured.
    pub fn decision_override(&self) -> Option<&DecisionOverride> {
        self.decision_override.as_ref()
    }

    pub(crate) fn take_next_turn_override(&mut self) -> Option<DecisionOverride> {
        match self.decision_override.as_ref() {
            Some(override_) if override_.scope == OverrideScope::NextTurn => {
                self.decision_override.take()
            }
            Some(override_) => Some(override_.clone()),
            None => None,
        }
    }

    /// Suggests the likely next decision level from recent behavior in Proactive mode.
    pub fn live_suggestion(&self) -> Option<DecisionLevel> {
        if self.mode != Mode::Proactive {
            return None;
        }
        self.history
            .iter()
            .rev()
            .find(|turn| turn.role == "user")
            .map(|turn| self.decide_level(&turn.content))
    }

    /// Lists registry capabilities visible to the configured user and teams.
    pub fn list_available_agents<'a>(
        &self,
        registry: &'a crate::core::registry::Registry,
    ) -> Vec<&'a crate::core::registry::Capability> {
        registry.list_available(self.user_id.as_deref(), &self.user_teams)
    }

    /// Builds a prompt context from recent history and the current input, returning the formatted prompt string.
    pub fn build_context(&self, current_input: &str) -> String {
        let mut context = String::from(
            "[System]\nYou are Morn, a helpful AI assistant.\n\n[Conversation History]\n",
        );

        let start = if self.history.len() > self.max_history {
            self.history.len() - self.max_history
        } else {
            0
        };

        for turn in &self.history[start..] {
            let role = if turn.role == "user" {
                "User"
            } else {
                "Assistant"
            };
            context.push_str(&format!("{}: {}\n", role, turn.content));
        }

        context.push_str(&format!(
            "\n[Current]\nUser: {}\nAssistant:\n",
            current_input
        ));
        context
    }

    /// Records one conversation turn with a role and content, trimming old history when needed.
    pub fn record_turn(&mut self, role: &str, content: &str) {
        self.history.push(TurnRecord {
            role: role.to_string(),
            content: content.to_string(),
        });
        if self.history.len() > self.max_history * 2 {
            self.history.remove(0);
        }
    }

    /// Clears all retained conversation history and resets the turn counter.
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.turn_count = 0;
    }

    /// Determines whether the user needs a single agent or a full team,
    /// then creates a TeamDef via keyword matching on presets or LLM generation.
    pub fn create_team_from_nl(&self, nl: &str, chat_fn: &ChatFn) -> Result<TeamDef, String> {
        let system_prompt = "You are a team configuration assistant. Determine if the user needs a single agent or a multi-agent team. Reply with exactly one line: either 'SINGLE' or 'TEAM'.";
        let response = chat_fn(nl, system_prompt)?;
        let trimmed = response.trim().to_uppercase();

        if trimmed.starts_with("SINGLE") {
            return Err("Single agent sufficient, no team needed".to_string());
        }

        if let Some(preset) = team_presets::find_preset(nl) {
            return Ok(preset);
        }

        let gen_prompt = format!(
            r#"The user wants to form a team for this task:
{}
Generate a TeamDef JSON. Available collaboration modes: Chain, ManagerWorker, Broadcast, Voting, Routing, AgentAsTool, Blackboard.
Available consensus mechanisms: Vote, CeoDecides, MungerVeto, AutoSynthesis.
Return only valid JSON with fields: id, name, members (string array of agent IDs), mode, consensus.
Example:
{{"id":"team-custom","name":"Custom Team","members":["agent-a","agent-b"],"mode":"Chain","consensus":"CeoDecides"}}"#,
            nl
        );
        let gen_response = chat_fn(&gen_prompt, "Only return valid JSON, no markdown.")?;
        let cleaned = gen_response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        #[derive(serde::Deserialize)]
        struct GenTeamDef {
            id: String,
            name: String,
            members: Vec<String>,
            mode: String,
            consensus: String,
        }

        let gen: GenTeamDef = serde_json::from_str(cleaned)
            .map_err(|e| format!("Failed to parse LLM team response: {}. Raw: {}", e, cleaned))?;

        let mode = match gen.mode.to_lowercase().as_str() {
            "chain" => CollaborationMode::Chain,
            "managerworker" | "manager_worker" => CollaborationMode::ManagerWorker,
            "broadcast" => CollaborationMode::Broadcast,
            "voting" => CollaborationMode::Voting,
            "routing" => CollaborationMode::Routing,
            "agentastool" | "agent_as_tool" => CollaborationMode::AgentAsTool,
            "blackboard" => CollaborationMode::Blackboard,
            _ => CollaborationMode::Chain,
        };

        let consensus = match gen.consensus.to_lowercase().as_str() {
            "vote" => ConsensusMechanism::Vote,
            "ceodecides" | "ceo_decides" => ConsensusMechanism::CeoDecides,
            "mungerveto" | "munger_veto" => ConsensusMechanism::MungerVeto,
            "autosynthesis" | "auto_synthesis" => ConsensusMechanism::AutoSynthesis,
            _ => ConsensusMechanism::CeoDecides,
        };

        Ok(TeamDef {
            id: gen.id,
            name: gen.name,
            members: gen.members,
            mode,
            consensus,
        })
    }

    /// Builds a team from natural language using local keyword presets.
    pub fn build_team_from_nl(&self, nl: &str) -> Result<TeamDef, String> {
        crate::core::orchestrator::team_builder::nl_to_team(nl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_build_context() {
        let supervisor = Supervisor::new(None, None);
        let context = supervisor.build_context("hello");
        assert!(context.contains("You are Morn"));
        assert!(context.contains("[Current]"));
        assert!(context.contains("hello"));
    }

    #[test]
    fn test_supervisor_record_and_context() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.record_turn("user", "hi");
        supervisor.record_turn("assistant", "hello!");
        let context = supervisor.build_context("how are you?");
        assert!(context.contains("hi"));
        assert!(context.contains("hello!"));
        assert!(context.contains("how are you?"));
    }

    #[test]
    fn test_decide_level_simple() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("hello"),
            DecisionLevel::L1DirectAnswer
        );
        assert_eq!(
            supervisor.decide_level("thanks"),
            DecisionLevel::L1DirectAnswer
        );
    }

    #[test]
    fn test_decide_level_tool() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("search for AI news"),
            DecisionLevel::L2SingleTool
        );
        assert_eq!(
            supervisor.decide_level("calculate 2+2"),
            DecisionLevel::L2SingleTool
        );
    }

    #[test]
    fn test_decide_level_workflow() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("create a report"),
            DecisionLevel::L5Workflow
        );
        assert_eq!(
            supervisor.decide_level("analysis"),
            DecisionLevel::L5Workflow
        );
    }

    #[test]
    fn test_decide_level_studio() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("create an agent"),
            DecisionLevel::L6JumpToStudio
        );
    }

    #[test]
    fn test_decide_level_default() {
        let supervisor = Supervisor::new(None, None);
        assert_eq!(
            supervisor.decide_level("tell me about quantum physics"),
            DecisionLevel::L3SingleAgent
        );
    }

    #[test]
    fn test_coo_mode() {
        let mut supervisor = Supervisor::new(None, None);
        assert_eq!(*supervisor.mode(), Mode::Proactive);
        supervisor.set_mode(Mode::Safe);
        assert_eq!(*supervisor.mode(), Mode::Safe);
    }

    #[test]
    fn test_decision_override_is_recorded() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.override_decision(DecisionLevel::L4Team, OverrideScope::Session);

        assert_eq!(
            supervisor.decision_override().map(|o| &o.level),
            Some(&DecisionLevel::L4Team)
        );
    }

    #[test]
    fn test_live_suggestion_uses_recent_user_turn_in_proactive_mode() {
        let mut supervisor = Supervisor::new(None, None);
        supervisor.record_turn("user", "please search docs");

        assert_eq!(
            supervisor.live_suggestion(),
            Some(DecisionLevel::L2SingleTool)
        );
    }

    #[test]
    fn test_decide_reasoning() {
        let supervisor = Supervisor::new(None, None);
        let (level, _reasoning) = supervisor.decide("complex multi-step task");
        assert_eq!(level, DecisionLevel::L4Team);
    }

    #[test]
    fn test_create_team_from_nl_single_agent() {
        let supervisor = Supervisor::new(None, None);
        let chat_fn = |_prompt: &str, _system: &str| Ok("SINGLE".to_string());
        let result = supervisor.create_team_from_nl("simple greeting", &chat_fn);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Single agent"));
    }

    #[test]
    fn test_create_team_from_nl_preset_research() {
        let supervisor = Supervisor::new(None, None);
        let chat_fn = |_prompt: &str, _system: &str| Ok("TEAM".to_string());
        let result = supervisor.create_team_from_nl("need research and analysis", &chat_fn);
        assert!(result.is_ok());
        let team = result.unwrap();
        assert_eq!(team.name, "Research Team");
    }

    #[test]
    fn test_create_team_from_nl_preset_code() {
        let supervisor = Supervisor::new(None, None);
        let chat_fn = |_prompt: &str, _system: &str| Ok("TEAM".to_string());
        let result = supervisor.create_team_from_nl("build a web app", &chat_fn);
        assert!(result.is_ok());
        let team = result.unwrap();
        assert_eq!(team.name, "Development Team");
    }

    #[test]
    fn test_create_team_from_nl_llm_generated() {
        let supervisor = Supervisor::new(None, None);
        let json_response = r#"{"id":"team-custom","name":"Custom Team","members":["agent-a","agent-b"],"mode":"Chain","consensus":"CeoDecides"}"#;
        let chat_fn = move |prompt: &str, _system: &str| {
            if prompt.contains("SINGLE") || prompt.contains("TEAM") {
                Ok("TEAM".to_string())
            } else {
                Ok(json_response.to_string())
            }
        };
        let result =
            supervisor.create_team_from_nl("something totally unique and custom", &chat_fn);
        assert!(result.is_ok());
        let team = result.unwrap();
        assert_eq!(team.id, "team-custom");
        assert_eq!(team.members.len(), 2);
    }

    #[test]
    fn test_build_team_from_nl_uses_local_builder() {
        let supervisor = Supervisor::new(None, None);
        let team = supervisor.build_team_from_nl("devops deploy monitor").unwrap();

        assert_eq!(team.id, "preset-devops");
        assert_eq!(team.members.len(), 3);
    }

    #[test]
    fn test_modify_rule_from_nl_add() {
        let storage = Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage), None);
        let result = supervisor
            .modify_rule_from_nl("add | deploy | L4 | contains 'deploy' | require_approval");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Rule added");
    }

    #[test]
    fn test_modify_rule_from_nl_list() {
        let storage = Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage), None);
        supervisor
            .modify_rule_from_nl("add | search | L2 | contains 'search' | auto_execute")
            .unwrap();
        let result = supervisor.modify_rule_from_nl("list all");
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("search"));
    }

    #[test]
    fn test_modify_rule_from_nl_find() {
        let storage = Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage), None);
        supervisor
            .modify_rule_from_nl("add | search | L2 | contains 'search' | auto_execute")
            .unwrap();
        let result = supervisor.modify_rule_from_nl("find search");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("L2"));
    }

    #[test]
    fn test_modify_rule_from_nl_delete() {
        let storage = Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage), None);
        supervisor
            .modify_rule_from_nl("add | test | L1 | test | none")
            .unwrap();
        supervisor
            .modify_rule_from_nl("add | test2 | L2 | test2 | none")
            .unwrap();
        let rules_before = supervisor.modify_rule_from_nl("list all").unwrap();
        let rules: Vec<crate::core::decision_rules::DecisionRule> =
            serde_json::from_str(&rules_before).unwrap();
        assert_eq!(rules.len(), 2);
        supervisor
            .modify_rule_from_nl(&format!("delete {}", rules[0].id))
            .unwrap();
        let rules_after = supervisor.modify_rule_from_nl("list all").unwrap();
        let remaining: Vec<crate::core::decision_rules::DecisionRule> =
            serde_json::from_str(&rules_after).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_modify_rule_from_nl_unknown() {
        let storage = Storage::new_in_memory().unwrap();
        let supervisor = Supervisor::new(Some(storage), None);
        let result = supervisor.modify_rule_from_nl("unknown command");
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_rule_from_nl_no_storage() {
        let supervisor = Supervisor::new(None, None);
        let result = supervisor.modify_rule_from_nl("list all");
        assert!(result.is_err());
    }
}
