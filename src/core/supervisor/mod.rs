//! supervisor — Oversees task execution decisions, plans, and risk controls.
mod decision;
mod execution;
mod types;

pub use types::*;

use crate::core::event_bus::SimpleEventBus;
use crate::core::storage::Storage;

pub struct Supervisor {
    storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
    history: Vec<TurnRecord>,
    max_history: usize,
    turn_count: u64,
    mode: CooMode,
    user_id: Option<String>,
    user_teams: Vec<String>,
}

impl Supervisor {
    /// Creates a supervisor with optional storage and event bus integrations and returns the initialized instance.
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        Supervisor {
            storage,
            event_bus,
            history: Vec::new(),
            max_history: 10,
            turn_count: 0,
            mode: CooMode::Active,
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
    pub fn mode(&self) -> &CooMode {
        &self.mode
    }
    /// Sets the COO execution mode that controls approval and automation behavior.
    pub fn set_mode(&mut self, mode: CooMode) {
        self.mode = mode;
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
        assert_eq!(*supervisor.mode(), CooMode::Active);
        supervisor.set_mode(CooMode::Safe);
        assert_eq!(*supervisor.mode(), CooMode::Safe);
    }

    #[test]
    fn test_decide_reasoning() {
        let supervisor = Supervisor::new(None, None);
        let (level, _reasoning) = supervisor.decide("complex multi-step task");
        assert_eq!(level, DecisionLevel::L4Team);
    }
}
