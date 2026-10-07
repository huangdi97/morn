//! Standards-first interoperability boundaries.
//!
//! MCP/A2A objects are transport/runtime state. They may contribute capability
//! interfaces and execution evidence, but they never replace canonical Morn
//! Work, Authority, Outcome or Acceptance records.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum InteropProtocol {
    Mcp,
    A2a,
    OpenApi,
    Grpc,
    JsonRpc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalEndpoint {
    pub protocol: InteropProtocol,
    pub endpoint_ref: String,
    pub protocol_version: Option<String>,
    pub identity_ref: Option<String>,
}

impl ExternalEndpoint {
    pub fn validate(&self) -> Result<()> {
        if self.endpoint_ref.trim().is_empty() {
            return Err(Error::validation("external endpoint reference is required"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub server_ref: String,
    pub tool_name: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub annotations: Value,
}

impl McpToolDescriptor {
    pub fn validate(&self) -> Result<()> {
        if self.server_ref.trim().is_empty() || self.tool_name.trim().is_empty() {
            return Err(Error::validation("MCP server/tool identity is required"));
        }
        if !self.input_schema.is_object() {
            return Err(Error::validation("MCP input schema must be an object"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct A2aAgentCardRef {
    pub agent_ref: String,
    pub card_url: String,
    pub protocol_version: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum A2aTaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Canceled,
    Failed,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct A2aTaskEvidence {
    pub agent_ref: String,
    pub task_id: String,
    pub context_id: Option<String>,
    pub state: A2aTaskState,
    pub artifact_refs: Vec<String>,
    pub raw_status: Option<Value>,
}

impl A2aTaskEvidence {
    /// A completed A2A Task is still only executor evidence. The caller must
    /// separately derive/verify a Morn Outcome and Acceptance.
    pub fn is_executor_terminal(&self) -> bool {
        matches!(
            self.state,
            A2aTaskState::Completed
                | A2aTaskState::Canceled
                | A2aTaskState::Failed
                | A2aTaskState::Rejected
        )
    }

    pub fn proves_morn_acceptance(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteropBinding {
    pub work_ref: String,
    pub execution_binding_ref: String,
    pub endpoint: ExternalEndpoint,
    pub capability_ref: String,
}

impl InteropBinding {
    pub fn validate(&self) -> Result<()> {
        if self.work_ref.trim().is_empty()
            || self.execution_binding_ref.trim().is_empty()
            || self.capability_ref.trim().is_empty()
        {
            return Err(Error::validation(
                "interop binding requires Work, ExecutionBinding and capability references",
            ));
        }
        self.endpoint.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mcp_tool_descriptor_is_interface_metadata_not_authority() {
        let tool = McpToolDescriptor {
            server_ref: "mcp://cmms".to_string(),
            tool_name: "create_order".to_string(),
            input_schema: json!({"type":"object"}),
            output_schema: None,
            annotations: json!({"destructiveHint": true}),
        };
        tool.validate().unwrap();
        // No authority/grant/token fields exist on the descriptor by design.
        let encoded = serde_json::to_value(tool).unwrap();
        assert!(encoded.get("authority").is_none());
        assert!(encoded.get("token").is_none());
    }

    #[test]
    fn a2a_task_completion_does_not_equal_morn_acceptance() {
        let evidence = A2aTaskEvidence {
            agent_ref: "a2a://planner".to_string(),
            task_id: "task-1".to_string(),
            context_id: Some("ctx-1".to_string()),
            state: A2aTaskState::Completed,
            artifact_refs: vec!["a2a-artifact://result".to_string()],
            raw_status: None,
        };
        assert!(evidence.is_executor_terminal());
        assert!(!evidence.proves_morn_acceptance());
    }

    #[test]
    fn interop_binding_requires_morn_execution_binding() {
        let binding = InteropBinding {
            work_ref: "work-1".to_string(),
            execution_binding_ref: "binding-1".to_string(),
            endpoint: ExternalEndpoint {
                protocol: InteropProtocol::A2a,
                endpoint_ref: "https://agent.example/a2a".to_string(),
                protocol_version: Some("0.3.0".to_string()),
                identity_ref: Some("workload://agent-example".to_string()),
            },
            capability_ref: "capability://planner".to_string(),
        };
        binding.validate().unwrap();
    }
}
