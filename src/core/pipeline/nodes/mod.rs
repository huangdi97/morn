//! Node type definitions and related data types for the pipeline system.
mod execution;
pub use execution::{PipelineContext, PipelineNodeExecutor};

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortSchema {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
}

impl PortSchema {
    pub fn new(name: &str, data_type: &str, required: bool, description: &str) -> Self {
        PortSchema {
            name: name.to_string(),
            data_type: data_type.to_string(),
            required,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodeSchema {
    pub node_type: PipelineNodeType,
    pub description: String,
    pub inputs: Vec<PortSchema>,
    pub outputs: Vec<PortSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PipelineNodeType {
    Start,
    End,
    LLM,
    Tool,
    Code,
    Condition,
    Loop,
    Parallel,
    Merge,
    Wait,
    HumanInput,
    Email,
    Webhook,
    Transform,
    Log,
    SubWorkflow,
}

impl PipelineNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipelineNodeType::Start => "start",
            PipelineNodeType::End => "end",
            PipelineNodeType::LLM => "llm",
            PipelineNodeType::Tool => "tool",
            PipelineNodeType::Code => "code",
            PipelineNodeType::Condition => "condition",
            PipelineNodeType::Loop => "loop",
            PipelineNodeType::Parallel => "parallel",
            PipelineNodeType::Merge => "merge",
            PipelineNodeType::Wait => "wait",
            PipelineNodeType::HumanInput => "human_input",
            PipelineNodeType::Email => "email",
            PipelineNodeType::Webhook => "webhook",
            PipelineNodeType::Transform => "transform",
            PipelineNodeType::Log => "log",
            PipelineNodeType::SubWorkflow => "sub_workflow",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            PipelineNodeType::Start,
            PipelineNodeType::End,
            PipelineNodeType::LLM,
            PipelineNodeType::Tool,
            PipelineNodeType::Code,
            PipelineNodeType::Condition,
            PipelineNodeType::Loop,
            PipelineNodeType::Parallel,
            PipelineNodeType::Merge,
            PipelineNodeType::Wait,
            PipelineNodeType::HumanInput,
            PipelineNodeType::Email,
            PipelineNodeType::Webhook,
            PipelineNodeType::Transform,
            PipelineNodeType::Log,
            PipelineNodeType::SubWorkflow,
        ]
    }

    pub fn schema(&self) -> NodeSchema {
        let any_in = || PortSchema::new("input", "any", false, "Input payload");
        let any_out = || PortSchema::new("output", "any", true, "Output payload");
        match self {
            PipelineNodeType::Start => NodeSchema {
                node_type: self.clone(),
                description: "Entry point for a workflow run.".to_string(),
                inputs: vec![],
                outputs: vec![any_out()],
            },
            PipelineNodeType::End => NodeSchema {
                node_type: self.clone(),
                description: "Terminal node that returns the final workflow result.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![PortSchema::new("result", "any", true, "Final result")],
            },
            PipelineNodeType::LLM => NodeSchema {
                node_type: self.clone(),
                description: "Calls a language model with prompt and context.".to_string(),
                inputs: vec![
                    PortSchema::new("prompt", "string", true, "Prompt text"),
                    PortSchema::new("context", "object", false, "Additional model context"),
                ],
                outputs: vec![PortSchema::new(
                    "response",
                    "string",
                    true,
                    "Model response",
                )],
            },
            PipelineNodeType::Tool => NodeSchema {
                node_type: self.clone(),
                description: "Invokes a registered tool with JSON arguments.".to_string(),
                inputs: vec![PortSchema::new("args", "object", false, "Tool arguments")],
                outputs: vec![PortSchema::new("result", "any", true, "Tool result")],
            },
            PipelineNodeType::Code => NodeSchema {
                node_type: self.clone(),
                description: "Runs code in a configured execution environment.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![
                    PortSchema::new("stdout", "string", false, "Standard output"),
                    PortSchema::new("stderr", "string", false, "Standard error"),
                    PortSchema::new("result", "any", true, "Execution result"),
                ],
            },
            PipelineNodeType::Condition => NodeSchema {
                node_type: self.clone(),
                description: "Routes data according to a boolean expression.".to_string(),
                inputs: vec![PortSchema::new("value", "any", true, "Value to test")],
                outputs: vec![
                    PortSchema::new("true", "any", false, "True branch"),
                    PortSchema::new("false", "any", false, "False branch"),
                ],
            },
            PipelineNodeType::Loop => NodeSchema {
                node_type: self.clone(),
                description: "Repeats work over an array of items.".to_string(),
                inputs: vec![PortSchema::new("items", "array", true, "Items to iterate")],
                outputs: vec![
                    PortSchema::new("item", "any", false, "Current item"),
                    PortSchema::new("results", "array", true, "Iteration results"),
                ],
            },
            PipelineNodeType::Parallel => NodeSchema {
                node_type: self.clone(),
                description: "Fans out work to multiple branches.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![PortSchema::new("branches", "array", true, "Branch inputs")],
            },
            PipelineNodeType::Merge => NodeSchema {
                node_type: self.clone(),
                description: "Combines branch outputs into one payload.".to_string(),
                inputs: vec![PortSchema::new("sources", "array", true, "Values to merge")],
                outputs: vec![PortSchema::new("merged", "any", true, "Merged value")],
            },
            PipelineNodeType::Wait => NodeSchema {
                node_type: self.clone(),
                description: "Waits for a duration or external condition.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![
                    any_out(),
                    PortSchema::new("timeout", "boolean", false, "Timed out"),
                ],
            },
            PipelineNodeType::HumanInput => NodeSchema {
                node_type: self.clone(),
                description: "Pauses for a human-provided response.".to_string(),
                inputs: vec![PortSchema::new(
                    "request",
                    "string",
                    true,
                    "Question or task",
                )],
                outputs: vec![PortSchema::new(
                    "response",
                    "string",
                    true,
                    "Human response",
                )],
            },
            PipelineNodeType::Email => NodeSchema {
                node_type: self.clone(),
                description: "Sends or prepares an email message.".to_string(),
                inputs: vec![
                    PortSchema::new("to", "string", true, "Recipient address"),
                    PortSchema::new("body", "string", true, "Email body"),
                ],
                outputs: vec![PortSchema::new(
                    "delivery",
                    "object",
                    true,
                    "Delivery status",
                )],
            },
            PipelineNodeType::Webhook => NodeSchema {
                node_type: self.clone(),
                description: "Receives or sends webhook payloads.".to_string(),
                inputs: vec![PortSchema::new(
                    "payload",
                    "object",
                    false,
                    "Webhook payload",
                )],
                outputs: vec![PortSchema::new(
                    "response",
                    "object",
                    true,
                    "Webhook response",
                )],
            },
            PipelineNodeType::Transform => NodeSchema {
                node_type: self.clone(),
                description: "Transforms payloads with a named operation or expression."
                    .to_string(),
                inputs: vec![any_in()],
                outputs: vec![PortSchema::new(
                    "transformed",
                    "any",
                    true,
                    "Transformed value",
                )],
            },
            PipelineNodeType::Log => NodeSchema {
                node_type: self.clone(),
                description: "Records a log event and passes through input.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![any_out()],
            },
            PipelineNodeType::SubWorkflow => NodeSchema {
                node_type: self.clone(),
                description: "Runs another workflow as a nested step.".to_string(),
                inputs: vec![any_in()],
                outputs: vec![PortSchema::new(
                    "result",
                    "any",
                    true,
                    "Sub-workflow result",
                )],
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineData {
    Text(String),
    Bytes(Vec<u8>),
    Number(f64),
    Table(Vec<Vec<Value>>),
    Json(Value),
}

impl PipelineData {
    pub fn as_text(&self) -> Result<String, String> {
        match self {
            PipelineData::Text(t) => Ok(t.clone()),
            PipelineData::Json(v) => Ok(v.to_string()),
            PipelineData::Number(n) => Ok(n.to_string()),
            _ => Err("cannot convert to text".to_string()),
        }
    }

    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            PipelineData::Number(n) => Ok(*n),
            PipelineData::Text(t) => t.parse::<f64>().map_err(|e| e.to_string()),
            _ => Err("cannot convert to number".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PipelineNode {
    Start {
        id: String,
        config: Value,
    },
    End {
        id: String,
    },
    LLM {
        id: String,
        model: String,
        prompt: String,
        params: Value,
    },
    Tool {
        id: String,
        tool_name: String,
        args: Value,
    },
    Code {
        id: String,
        language: String,
        code: String,
        timeout_secs: u64,
    },
    Condition {
        id: String,
        expression: String,
    },
    Loop {
        id: String,
        max_iterations: u64,
    },
    Parallel {
        id: String,
        branch_ids: Vec<String>,
    },
    Merge {
        id: String,
        strategy: String,
    },
    Wait {
        id: String,
        duration_secs: u64,
    },
    HumanInput {
        id: String,
        prompt: String,
    },
    Email {
        id: String,
        to: String,
        subject: String,
        body: String,
    },
    Webhook {
        id: String,
        url: String,
        method: String,
    },
    Input {
        id: String,
        source: String,
    },
    Transform {
        id: String,
        operation: String,
        params: Value,
    },
    Output {
        id: String,
        target: String,
    },
    Timer {
        id: String,
        interval_secs: u64,
    },
    Log {
        id: String,
        level: String,
        message: String,
    },
    SubWorkflow {
        id: String,
        workflow_id: String,
        input: Value,
    },
}

impl PipelineNode {
    pub fn id(&self) -> &str {
        match self {
            PipelineNode::Start { id, .. } => id,
            PipelineNode::End { id, .. } => id,
            PipelineNode::LLM { id, .. } => id,
            PipelineNode::Tool { id, .. } => id,
            PipelineNode::Code { id, .. } => id,
            PipelineNode::Condition { id, .. } => id,
            PipelineNode::Loop { id, .. } => id,
            PipelineNode::Parallel { id, .. } => id,
            PipelineNode::Merge { id, .. } => id,
            PipelineNode::Wait { id, .. } => id,
            PipelineNode::HumanInput { id, .. } => id,
            PipelineNode::Email { id, .. } => id,
            PipelineNode::Webhook { id, .. } => id,
            PipelineNode::Input { id, .. } => id,
            PipelineNode::Transform { id, .. } => id,
            PipelineNode::Output { id, .. } => id,
            PipelineNode::Timer { id, .. } => id,
            PipelineNode::Log { id, .. } => id,
            PipelineNode::SubWorkflow { id, .. } => id,
        }
    }

    pub fn node_type(&self) -> &str {
        match self {
            PipelineNode::Start { .. } => "start",
            PipelineNode::End { .. } => "end",
            PipelineNode::LLM { .. } => "llm",
            PipelineNode::Tool { .. } => "tool",
            PipelineNode::Code { .. } => "code",
            PipelineNode::Condition { .. } => "condition",
            PipelineNode::Loop { .. } => "loop",
            PipelineNode::Parallel { .. } => "parallel",
            PipelineNode::Merge { .. } => "merge",
            PipelineNode::Wait { .. } => "wait",
            PipelineNode::HumanInput { .. } => "human_input",
            PipelineNode::Email { .. } => "email",
            PipelineNode::Webhook { .. } => "webhook",
            PipelineNode::Input { .. } => "input",
            PipelineNode::Transform { .. } => "transform",
            PipelineNode::Output { .. } => "output",
            PipelineNode::Timer { .. } => "timer",
            PipelineNode::Log { .. } => "log",
            PipelineNode::SubWorkflow { .. } => "sub_workflow",
        }
    }

    pub fn input_schema(&self) -> Vec<PortSchema> {
        self.schema().inputs
    }

    pub fn output_schema(&self) -> Vec<PortSchema> {
        self.schema().outputs
    }

    pub fn schema(&self) -> NodeSchema {
        match self.pipeline_node_type() {
            Some(node_type) => node_type.schema(),
            None => NodeSchema {
                node_type: PipelineNodeType::Transform,
                description: format!("Legacy {} node.", self.node_type()),
                inputs: vec![PortSchema::new("input", "any", false, "Input payload")],
                outputs: vec![PortSchema::new("output", "any", true, "Output payload")],
            },
        }
    }

    pub fn pipeline_node_type(&self) -> Option<PipelineNodeType> {
        match self {
            PipelineNode::Start { .. } => Some(PipelineNodeType::Start),
            PipelineNode::End { .. } => Some(PipelineNodeType::End),
            PipelineNode::LLM { .. } => Some(PipelineNodeType::LLM),
            PipelineNode::Tool { .. } => Some(PipelineNodeType::Tool),
            PipelineNode::Code { .. } => Some(PipelineNodeType::Code),
            PipelineNode::Condition { .. } => Some(PipelineNodeType::Condition),
            PipelineNode::Loop { .. } => Some(PipelineNodeType::Loop),
            PipelineNode::Parallel { .. } => Some(PipelineNodeType::Parallel),
            PipelineNode::Merge { .. } => Some(PipelineNodeType::Merge),
            PipelineNode::Wait { .. } => Some(PipelineNodeType::Wait),
            PipelineNode::HumanInput { .. } => Some(PipelineNodeType::HumanInput),
            PipelineNode::Email { .. } => Some(PipelineNodeType::Email),
            PipelineNode::Webhook { .. } => Some(PipelineNodeType::Webhook),
            PipelineNode::Transform { .. } => Some(PipelineNodeType::Transform),
            PipelineNode::Log { .. } => Some(PipelineNodeType::Log),
            PipelineNode::SubWorkflow { .. } => Some(PipelineNodeType::SubWorkflow),
            PipelineNode::Input { .. }
            | PipelineNode::Output { .. }
            | PipelineNode::Timer { .. } => None,
        }
    }

    pub fn available_node_schemas() -> Vec<NodeSchema> {
        PipelineNodeType::all()
            .into_iter()
            .map(|node_type| node_type.schema())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub from_port: String,
    pub to_port: String,
}
