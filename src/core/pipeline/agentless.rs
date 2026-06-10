use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentlessPipeline {
    steps: Vec<PipelineStep>,
    context: HashMap<String, String>,
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
        }
    }

    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
    }

    pub fn execute(&mut self) -> Result<Vec<String>, String> {
        let mut outputs = Vec::new();
        for step in &self.steps {
            self.context.insert(step.name.clone(), step.command.clone());
            outputs.push(format!("{}: {}", step.name, step.command));
        }
        Ok(outputs)
    }

    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}
