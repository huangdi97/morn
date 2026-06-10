//! workflow — Defines workflow actions, steps, templates, and execution helpers.
use serde_json::Value;
use std::collections::HashMap;

mod engine;
#[cfg(test)]
mod engine_tests;
pub mod templates;

pub use crate::core::thread_pool::TaskDef;
pub use engine::{ControlFlowNode, WorkflowEngine};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VarType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
}

impl VarType {
    pub fn detect(value: &Value) -> Self {
        match value {
            Value::String(_) => VarType::String,
            Value::Number(_) => VarType::Number,
            Value::Bool(_) => VarType::Boolean,
            Value::Object(_) => VarType::Object,
            Value::Array(_) => VarType::Array,
            Value::Null => VarType::Null,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: String,
    pub var_type: VarType,
    pub value: Value,
    pub source_step: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableStore {
    variables: HashMap<String, Variable>,
}

impl VariableStore {
    pub fn new() -> Self {
        VariableStore {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, step_id: &str, name: &str, value: Value) -> Result<(), String> {
        let var_type = VarType::detect(&value);
        let key = format!("{}.{}", step_id, name);
        self.variables.insert(
            name.to_string(),
            Variable {
                name: name.to_string(),
                var_type,
                value: value.clone(),
                source_step: Some(step_id.to_string()),
            },
        );
        // Also store with step prefix for disambiguation
        self.variables.insert(
            key,
            Variable {
                name: name.to_string(),
                var_type: VarType::detect(&value),
                value,
                source_step: Some(step_id.to_string()),
            },
        );
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Variable, String> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    pub fn convert(&self, value: &Variable, target_type: VarType) -> Result<Variable, String> {
        let converted_value = match (&value.var_type, &target_type) {
            (VarType::String, VarType::Number) => {
                let s = value.value.as_str().ok_or("expected string")?;
                let n: f64 = s
                    .parse()
                    .map_err(|e| format!("cannot parse as number: {}", e))?;
                Value::Number(
                    serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
                )
            }
            (VarType::String, VarType::Boolean) => {
                let s = value.value.as_str().ok_or("expected string")?;
                Value::Bool(s == "true" || s == "1" || s == "yes")
            }
            (VarType::Number, VarType::String) => Value::String(value.value.to_string()),
            (VarType::Number, VarType::Boolean) => {
                let n = value.value.as_f64().unwrap_or(0.0);
                Value::Bool(n != 0.0)
            }
            (VarType::Boolean, VarType::String) => {
                let b = value.value.as_bool().unwrap_or(false);
                Value::String(b.to_string())
            }
            (VarType::Boolean, VarType::Number) => {
                let b = value.value.as_bool().unwrap_or(false);
                Value::Number(serde_json::Number::from(if b { 1 } else { 0 }))
            }
            _ => {
                if std::mem::discriminant(&value.var_type) == std::mem::discriminant(&target_type) {
                    value.value.clone()
                } else {
                    return Err(format!(
                        "Unsupported conversion from {:?} to {:?}",
                        value.var_type, target_type
                    ));
                }
            }
        };
        Ok(Variable {
            name: value.name.clone(),
            var_type: target_type,
            value: converted_value,
            source_step: value.source_step.clone(),
        })
    }

    pub fn all(&self) -> Vec<&Variable> {
        self.variables
            .iter()
            .filter(|(k, _)| !k.contains('.'))
            .map(|(_, v)| v)
            .collect()
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JoinCondition {
    All,
    Any,
    NOf(u32),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WorkflowAction {
    LLMCall {
        system_prompt: String,
        model: String,
    },
    ToolCall {
        tool_id: String,
        params: Value,
    },
    AgentCall {
        agent_id: String,
        input: String,
    },
    TeamCall {
        team_id: String,
        input: String,
    },
    SubWorkflow {
        workflow_id: String,
    },
    CodeExec {
        language: String,
        script: String,
    },
    KnowledgeQuery {
        knowledge_id: String,
        query: String,
    },
    HumanApproval {
        message: String,
    },
    HumanInput {
        question: String,
    },
    Notification {
        channel: String,
        message: String,
    },
    Condition {
        expression: String,
        true_branch: Vec<WorkflowStep>,
        false_branch: Vec<WorkflowStep>,
    },
    Loop {
        iterator: String,
        body: Vec<WorkflowStep>,
        max_iterations: u32,
    },
    Wait {
        duration_secs: u32,
    },
    Fork {
        branches: Vec<Vec<WorkflowStep>>,
    },
    Join,
    PipelineExec {
        pipeline_json: Value,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub action: WorkflowAction,
    pub depends_on: Vec<String>,
    pub timeout_secs: u32,
    pub retry_count: u8,
    pub approval_required: bool,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub estimated_duration_secs: u64,
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub fork_from: Option<String>,
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
