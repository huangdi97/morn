//! types — Data type definitions for task-related records.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub user_input: String,
    pub plan_json: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskRecord {
    pub id: String,
    pub task_id: String,
    pub agent_id: String,
    pub action: String,
    pub params_json: String,
    pub status: String,
    pub result_json: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub agent_id: String,
    pub task_id: String,
    pub action: String,
    pub status: String,
    pub latency_ms: Option<i64>,
    pub error_msg: Option<String>,
    pub token_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub task_id: String,
    pub decision_level: String,
    pub action: String,
    pub context_json: Option<String>,
    pub approved: bool,
    pub created_at: String,
}
