//! 无 Agent 流水线 — 无需 Agent 参与的自动步骤处理
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type ToolFn = Box<dyn Fn(&str) -> Result<String, String>>;
type ToolRegistry = HashMap<String, ToolFn>;

#[derive(Serialize, Deserialize)]
pub struct AgentlessPipeline {
    steps: Vec<PipelineStep>,
    context: HashMap<String, String>,
    #[serde(skip)]
    tool_registry: Option<ToolRegistry>,
}

impl Clone for AgentlessPipeline {
    fn clone(&self) -> Self {
        Self {
            steps: self.steps.clone(),
            context: self.context.clone(),
            tool_registry: None,
        }
    }
}

impl std::fmt::Debug for AgentlessPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentlessPipeline")
            .field("steps", &self.steps)
            .field("context", &self.context)
            .field("tool_registry", &self.tool_registry.as_ref().map(|_| "..."))
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub command: String,
    pub timeout_secs: u64,
}

impl AgentlessPipeline {
    pub fn new(steps: Vec<PipelineStep>) -> Self {
        AgentlessPipeline {
            steps,
            context: HashMap::new(),
            tool_registry: None,
        }
    }

    pub fn with_tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
    }

    pub fn execute(&mut self) -> Result<Vec<String>, String> {
        let mut outputs = Vec::new();
        for step in &self.steps {
            if let Some(ref registry) = self.tool_registry {
                if let Some(tool_fn) = registry.get(&step.name) {
                    let result = tool_fn(&step.command)?;
                    self.context.insert(step.name.clone(), result.clone());
                    outputs.push(result);
                } else {
                    self.context.insert(step.name.clone(), step.command.clone());
                    outputs.push(format!("{}: {}", step.name, step.command));
                }
            } else {
                self.context.insert(step.name.clone(), step.command.clone());
                outputs.push(format!("{}: {}", step.name, step.command));
            }
        }
        Ok(outputs)
    }

    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}
