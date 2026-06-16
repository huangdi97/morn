//! supervisor — Oversees task execution decisions, plans, and risk controls.
use crate::core::error::MornError;
pub mod auto_hands;
mod decision;
mod execution;
mod guided_builder;
mod learning;
pub mod presets;
mod rule_commands;
mod team_builder;
mod types;
mod workflow_approvals;

pub use auto_hands::*;
pub use decision::{parse_with_llm, Intent};
pub use guided_builder::*;
pub use learning::*;
pub use types::*;

use crate::core::agent_pool::{AgentPool, PoolConfig};
use crate::core::approval::WorkflowApproval;
pub(crate) use crate::core::dual_llm::{DualLlmGuard, DualLlmLog};
use crate::core::event_bus::SimpleEventBus;
use crate::core::memory::{MemoryHub, MemoryOrchestrator};
use crate::core::model_router::ModelRouter;
use crate::core::orchestrator::TeamDef;
use crate::core::security::AuditLog;
use crate::core::storage::Storage;
use crate::core::supervisor::execution::planner::Planner;
use crate::core::supervisor::execution::scheduler::Scheduler;
use crate::core::thread_pool::TaskPool;
use crate::core::trust_scorer::scorer::TrustScorer;

pub type ChatFn = dyn Fn(&str, &str) -> Result<String, MornError>;

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
    memory_orchestrator: Option<MemoryOrchestrator>,
    pub audit_log: AuditLog,
    trust_scorer: Option<TrustScorer>,
    agent_pool: AgentPool,
    model_router: ModelRouter,
    planner: Option<Planner>,
    scheduler: Option<Scheduler>,
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
            memory_orchestrator: Some(MemoryOrchestrator::new(MemoryHub::new())),
            audit_log: AuditLog::new(),
            trust_scorer: Some(TrustScorer::new()),
            agent_pool: AgentPool::new(PoolConfig::default()),
            model_router: ModelRouter::new(),
            planner: Some(Planner),
            scheduler: Some(Scheduler::new()),
        }
    }

    /// Creates a supervisor that uses the provided model router for chat routing.
    pub fn with_model_router(mut self, router: ModelRouter) -> Self {
        self.model_router = router;
        self
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

    /// Returns the shared task pool used by workflow execution paths.
    pub fn task_pool(&self) -> &TaskPool {
        &self.task_pool
    }

    pub fn agent_pool(&self) -> &AgentPool {
        &self.agent_pool
    }

    pub fn model_router(&self) -> &ModelRouter {
        &self.model_router
    }

    pub fn model_router_mut(&mut self) -> &mut ModelRouter {
        &mut self.model_router
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

    /// Returns a reference to the memory orchestrator if configured.
    pub fn memory(&self) -> Option<&MemoryOrchestrator> {
        self.memory_orchestrator.as_ref()
    }

    /// Returns a mutable reference to the memory orchestrator if configured.
    pub fn memory_mut(&mut self) -> Option<&mut MemoryOrchestrator> {
        self.memory_orchestrator.as_mut()
    }

    /// Queries all memory layers with the given context for decision support.
    pub fn query_memory(&mut self, context: &str) -> Result<(), MornError> {
        if let Some(ref mut mem) = self.memory_orchestrator {
            mem.decide_with_memory(context)?;
        }
        Ok(())
    }

    /// Builds a team from natural language using local keyword presets.
    pub fn build_team_from_nl(&self, nl: &str) -> Result<TeamDef, MornError> {
        crate::core::orchestrator::team_builder::nl_to_team(nl)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
