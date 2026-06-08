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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_templates() {
        let templates = WorkflowTemplate::list_builtin();
        assert_eq!(templates.len(), 8);
    }

    #[test]
    fn test_get_template_by_id() {
        let template = WorkflowTemplate::get_by_id("workflow-task-execution");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Task Execution");
    }

    #[test]
    fn test_template_categories() {
        let templates = WorkflowTemplate::list_builtin();
        let categories: Vec<&str> = templates.iter().map(|t| t.category.as_str()).collect();
        assert!(categories.contains(&"general"));
        assert!(categories.contains(&"research"));
        assert!(categories.contains(&"development"));
        assert!(categories.contains(&"operations"));
    }

    #[test]
    fn test_task_execution_has_six_steps() {
        let t = WorkflowTemplate::get_by_id("workflow-task-execution").unwrap();
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn test_code_delivery_has_seven_steps() {
        let t = WorkflowTemplate::get_by_id("workflow-code-delivery").unwrap();
        assert_eq!(t.steps.len(), 7);
    }

    #[test]
    fn test_all_templates_have_steps() {
        for t in WorkflowTemplate::list_builtin() {
            assert!(!t.steps.is_empty(), "Template '{}' has no steps", t.id);
        }
    }

    #[test]
    fn test_workflow_action_serialization() {
        let action = WorkflowAction::ToolCall {
            tool_id: "web_search".into(),
            params: serde_json::json!({"q": "test"}),
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: WorkflowAction = serde_json::from_str(&json).unwrap();
        match deserialized {
            WorkflowAction::ToolCall { tool_id, .. } => assert_eq!(tool_id, "web_search"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_variable_store_set_and_get() {
        let mut store = VariableStore::new();
        store
            .set("step1", "result", serde_json::json!("hello"))
            .unwrap();
        let var = store.get("result").unwrap();
        assert_eq!(var.value, "hello");
        assert_eq!(var.source_step.unwrap(), "step1");
    }

    #[test]
    fn test_variable_store_get_missing() {
        let store = VariableStore::new();
        let result = store.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_store_type_detection() {
        let mut store = VariableStore::new();
        store.set("s1", "str", serde_json::json!("text")).unwrap();
        store.set("s1", "num", serde_json::json!(42)).unwrap();
        store.set("s1", "flag", serde_json::json!(true)).unwrap();
        store
            .set("s1", "arr", serde_json::json!([1, 2, 3]))
            .unwrap();
        store
            .set("s1", "obj", serde_json::json!({"k": "v"}))
            .unwrap();
        store.set("s1", "null", serde_json::json!(null)).unwrap();

        assert!(matches!(
            store.get("str").unwrap().var_type,
            VarType::String
        ));
        assert!(matches!(
            store.get("num").unwrap().var_type,
            VarType::Number
        ));
        assert!(matches!(
            store.get("flag").unwrap().var_type,
            VarType::Boolean
        ));
        assert!(matches!(store.get("arr").unwrap().var_type, VarType::Array));
        assert!(matches!(
            store.get("obj").unwrap().var_type,
            VarType::Object
        ));
        assert!(matches!(store.get("null").unwrap().var_type, VarType::Null));
    }

    #[test]
    fn test_variable_store_convert_string_to_number() {
        let var = Variable {
            name: "score".into(),
            var_type: VarType::String,
            value: serde_json::json!("95.5"),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::Number).unwrap();
        assert_eq!(converted.value, 95.5);
    }

    #[test]
    fn test_variable_store_convert_number_to_string() {
        let var = Variable {
            name: "count".into(),
            var_type: VarType::Number,
            value: serde_json::json!(42),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::String).unwrap();
        assert_eq!(converted.value, "42");
    }

    #[test]
    fn test_variable_store_convert_bool_to_string() {
        let var = Variable {
            name: "flag".into(),
            var_type: VarType::Boolean,
            value: serde_json::json!(true),
            source_step: None,
        };
        let store = VariableStore::new();
        let converted = store.convert(&var, VarType::String).unwrap();
        assert_eq!(converted.value, "true");
    }

    #[test]
    fn test_variable_store_all() {
        let mut store = VariableStore::new();
        store.set("a", "x", serde_json::json!(1)).unwrap();
        store.set("b", "y", serde_json::json!(2)).unwrap();
        assert_eq!(store.all().len(), 2);
    }

    #[test]
    fn test_variable_store_clear() {
        let mut store = VariableStore::new();
        store.set("a", "x", serde_json::json!(1)).unwrap();
        store.clear();
        assert!(store.get("x").is_err());
    }
}
