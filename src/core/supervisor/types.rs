//! types — Defines supervisor plans, decisions, and execution metadata.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLAgentDef {
    pub name: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskDef {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub params: Value,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub user_input: String,
    pub subtasks: Vec<SubTaskDef>,
    pub estimated_secs: u64,
    pub decision_level: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskResult {
    pub id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub subtask_results: Vec<SubTaskResult>,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurnRecord {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DecisionLevel {
    L1DirectAnswer,
    L2SingleTool,
    L3SingleAgent,
    L4Team,
    L5Workflow,
    L6JumpToStudio,
}

impl DecisionLevel {
    /// Returns the stable string identifier for this decision level.
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "direct_answer",
            DecisionLevel::L2SingleTool => "single_tool",
            DecisionLevel::L3SingleAgent => "single_agent",
            DecisionLevel::L4Team => "team",
            DecisionLevel::L5Workflow => "workflow",
            DecisionLevel::L6JumpToStudio => "jump_studio",
        }
    }

    /// Returns the approximate cost tier label associated with this decision level.
    pub fn cost_tier(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "¥0.001/0.5s",
            DecisionLevel::L2SingleTool => "¥0.003/1s",
            DecisionLevel::L3SingleAgent => "¥0.02/5s",
            DecisionLevel::L4Team => "¥0.05/15s",
            DecisionLevel::L5Workflow => "¥0.03/10s",
            DecisionLevel::L6JumpToStudio => "variable",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CooMode {
    Active,
    Safe,
    Auto,
}

impl CooMode {
    /// Returns the stable string identifier for this COO mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            CooMode::Active => "active",
            CooMode::Safe => "safe",
            CooMode::Auto => "auto",
        }
    }
}
