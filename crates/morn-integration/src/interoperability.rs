//! Standards-first interoperability boundaries.
//!
//! MCP/A2A objects are transport/runtime state. They may contribute capability
//! interfaces and execution evidence, but they never replace canonical Morn
//! Work, Authority, Outcome or Acceptance records.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{Id, RuntimeBindingId, WorkPackageId};
use morn_kernel::time::Timestamp;
use morn_runtime::ExecutionBinding;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExternalTaskObservationTag;
pub type ExternalTaskObservationId = Id<ExternalTaskObservationTag>;

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

/// HTTP MCP authorization binding. The concrete bearer token never appears in
/// this structure: Morn carries an opaque CredentialProvider handle and the
/// RFC-8707 target resource/audience that the credential must be issued for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpHttpAuthorizationBinding {
    pub server_resource_uri: String,
    pub credential_handle_ref: String,
    pub scopes: Vec<String>,
    pub audience_bound: bool,
}

impl McpHttpAuthorizationBinding {
    pub fn validate(&self) -> Result<()> {
        if !(self.server_resource_uri.starts_with("https://")
            || self.server_resource_uri.starts_with("http://"))
        {
            return Err(Error::validation(
                "HTTP MCP authorization requires an explicit http(s) resource URI",
            ));
        }
        if self.credential_handle_ref.trim().is_empty() {
            return Err(Error::validation(
                "MCP authorization requires an opaque credential handle",
            ));
        }
        if !self.audience_bound {
            return Err(Error::validation(
                "MCP credential must be bound to the target resource/audience",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum McpTaskState {
    Working,
    InputRequired,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpTaskEvidence {
    pub server_ref: String,
    pub task_id: String,
    pub state: McpTaskState,
    pub status_message: Option<String>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl McpTaskEvidence {
    /// MCP Tasks are durable executor state for an augmented MCP request. Even
    /// a completed Task remains provider/runtime evidence until Morn observes
    /// a domain Outcome and evaluates Acceptance independently.
    pub fn is_executor_terminal(&self) -> bool {
        matches!(
            self.state,
            McpTaskState::Completed | McpTaskState::Cancelled | McpTaskState::Failed
        )
    }

    pub fn proves_morn_acceptance(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "protocol", content = "task", rename_all = "kebab-case")]
pub enum ExternalTaskSnapshot {
    Mcp(McpTaskEvidence),
    A2a(A2aTaskEvidence),
}

impl ExternalTaskSnapshot {
    pub fn task_id(&self) -> &str {
        match self {
            Self::Mcp(task) => &task.task_id,
            Self::A2a(task) => &task.task_id,
        }
    }

    pub fn protocol(&self) -> InteropProtocol {
        match self {
            Self::Mcp(_) => InteropProtocol::Mcp,
            Self::A2a(_) => InteropProtocol::A2a,
        }
    }

    pub fn is_executor_terminal(&self) -> bool {
        match self {
            Self::Mcp(task) => task.is_executor_terminal(),
            Self::A2a(task) => task.is_executor_terminal(),
        }
    }
}

/// Append-only observation of a durable external protocol task.
///
/// The task handle is provider/runtime evidence tied to the exact Morn Work
/// generation and ExecutionBinding that created or resumed it. It never owns
/// Work phase, Outcome or Acceptance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernedExternalTaskObservation {
    pub id: ExternalTaskObservationId,
    pub work_id: WorkPackageId,
    pub work_generation: u64,
    pub execution_binding_ref: RuntimeBindingId,
    pub endpoint: ExternalEndpoint,
    pub snapshot: ExternalTaskSnapshot,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
}

impl GovernedExternalTaskObservation {
    pub fn new(
        work_id: WorkPackageId,
        work_generation: u64,
        execution_binding_ref: RuntimeBindingId,
        endpoint: ExternalEndpoint,
        snapshot: ExternalTaskSnapshot,
        evidence_refs: Vec<String>,
    ) -> Result<Self> {
        let observation = Self {
            id: ExternalTaskObservationId::generate_with("external-task-observation"),
            work_id,
            work_generation,
            execution_binding_ref,
            endpoint,
            snapshot,
            evidence_refs,
            observed_at: Timestamp::now(),
        };
        observation.validate()?;
        Ok(observation)
    }

    pub fn validate(&self) -> Result<()> {
        self.endpoint.validate()?;
        if self.work_generation == 0
            || self.snapshot.task_id().trim().is_empty()
            || self.evidence_refs.is_empty()
        {
            return Err(Error::validation(
                "governed external task observation requires Work generation, task id and evidence",
            ));
        }
        if self.endpoint.protocol != self.snapshot.protocol() {
            return Err(Error::validation(
                "external task snapshot protocol must match the bound endpoint",
            ));
        }
        match &self.snapshot {
            ExternalTaskSnapshot::Mcp(task)
                if task.server_ref.trim().is_empty()
                    || task.server_ref != self.endpoint.endpoint_ref =>
            {
                return Err(Error::validation(
                    "MCP task server must match the bound endpoint",
                ));
            }
            ExternalTaskSnapshot::A2a(task) if task.agent_ref.trim().is_empty() => {
                return Err(Error::validation("A2A task requires agent identity"));
            }
            _ => {}
        }
        Ok(())
    }

    pub fn validate_against_binding(&self, binding: &ExecutionBinding) -> Result<()> {
        self.validate()?;
        if self.work_id != binding.work_id
            || self.work_generation != binding.work_generation
            || self.execution_binding_ref != binding.id
        {
            return Err(Error::validation(
                "external task observation must match the exact Work generation and ExecutionBinding",
            ));
        }
        Ok(())
    }

    pub fn proves_morn_acceptance(&self) -> bool {
        false
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
    /// A2A protocol compatibility is negotiated at Major.Minor granularity
    /// (for example "1.0"). Patch release numbers are not protocol versions.
    pub protocol_version: String,
    pub skills: Vec<String>,
}

impl A2aAgentCardRef {
    pub fn validate(&self) -> Result<()> {
        if self.agent_ref.trim().is_empty() || self.card_url.trim().is_empty() {
            return Err(Error::validation("A2A Agent Card identity/url is required"));
        }
        let parts: Vec<&str> = self.protocol_version.split('.').collect();
        if parts.len() != 2
            || parts
                .iter()
                .any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()))
        {
            return Err(Error::validation(
                "A2A protocol version must use Major.Minor form such as 1.0",
            ));
        }
        Ok(())
    }
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

    pub fn validate_against_execution_binding(&self, binding: &ExecutionBinding) -> Result<()> {
        self.validate()?;
        if self.work_ref != binding.work_id.to_string()
            || self.execution_binding_ref != binding.id.to_string()
            || self.capability_ref != binding.capability_manifest_ref
        {
            return Err(Error::validation(
                "interop binding must match the exact Work, ExecutionBinding and capability manifest",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mcp_http_authorization_requires_resource_bound_opaque_credential() {
        let binding = McpHttpAuthorizationBinding {
            server_resource_uri: "https://mcp.example.com".to_string(),
            credential_handle_ref: "credential://work-1042/mcp-example".to_string(),
            scopes: vec!["tools.call".to_string()],
            audience_bound: true,
        };
        binding.validate().unwrap();

        let encoded = serde_json::to_value(&binding).unwrap();
        assert!(encoded.get("access_token").is_none());
        assert!(encoded.get("bearer").is_none());

        let mut unsafe_binding = binding.clone();
        unsafe_binding.audience_bound = false;
        assert!(unsafe_binding.validate().is_err());
    }

    #[test]
    fn mcp_task_completion_does_not_equal_morn_outcome_or_acceptance() {
        let task = McpTaskEvidence {
            server_ref: "https://mcp.example.com".to_string(),
            task_id: "task-1".to_string(),
            state: McpTaskState::Completed,
            status_message: Some("tool computation complete".to_string()),
            result: Some(json!({"resultType":"complete","value":42})),
            error: None,
        };
        assert!(task.is_executor_terminal());
        assert!(!task.proves_morn_acceptance());
    }

    #[test]
    fn durable_external_task_observation_is_pinned_but_never_acceptance() {
        use morn_kernel::ids::{RuntimeBindingId, WorkPackageId};

        let work_id = WorkPackageId::generate_with("work");
        let binding_id = RuntimeBindingId::generate_with("binding");
        let observation = GovernedExternalTaskObservation::new(
            work_id.clone(),
            3,
            binding_id.clone(),
            ExternalEndpoint {
                protocol: InteropProtocol::Mcp,
                endpoint_ref: "https://mcp.example.com".to_string(),
                protocol_version: Some("2026-07-28".to_string()),
                identity_ref: None,
            },
            ExternalTaskSnapshot::Mcp(McpTaskEvidence {
                server_ref: "https://mcp.example.com".to_string(),
                task_id: "task-42".to_string(),
                state: McpTaskState::Completed,
                status_message: Some("executor completed".to_string()),
                result: Some(json!({"artifact":"result"})),
                error: None,
            }),
            vec!["mcp://task-42/status/1".to_string()],
        )
        .unwrap();

        assert_eq!(observation.work_id, work_id);
        assert_eq!(observation.execution_binding_ref, binding_id);
        assert!(observation.snapshot.is_executor_terminal());
        assert!(!observation.proves_morn_acceptance());
    }

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
    fn a2a_protocol_version_uses_major_minor_negotiation() {
        let card = A2aAgentCardRef {
            agent_ref: "a2a://planner".to_string(),
            card_url: "https://agent.example/.well-known/agent-card.json".to_string(),
            protocol_version: "1.0".to_string(),
            skills: vec!["plan".to_string()],
        };
        card.validate().unwrap();

        let mut invalid = card;
        invalid.protocol_version = "1.0.0".to_string();
        assert!(invalid.validate().is_err());
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
    fn interop_binding_cannot_relabel_a_different_execution_binding_or_capability() {
        use morn_kernel::ids::{WorkPackageId, WorkspaceId};
        use morn_work::control::{WorkResource, WorkSpec};

        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "external task",
                "morn.lite@1.0.0",
            ),
        );
        let binding = ExecutionBinding::for_work(
            &work,
            "capability://orders",
            "mcp-provider",
            "2026-07-28",
        );
        let mut interop = InteropBinding {
            work_ref: work.id.to_string(),
            execution_binding_ref: binding.id.to_string(),
            endpoint: ExternalEndpoint {
                protocol: InteropProtocol::Mcp,
                endpoint_ref: "https://mcp.example.com".to_string(),
                protocol_version: Some("2026-07-28".to_string()),
                identity_ref: None,
            },
            capability_ref: binding.capability_manifest_ref.clone(),
        };
        interop.validate_against_execution_binding(&binding).unwrap();

        interop.capability_ref = "capability://other".to_string();
        assert!(interop.validate_against_execution_binding(&binding).is_err());
        interop.capability_ref = binding.capability_manifest_ref.clone();
        interop.execution_binding_ref = "binding://other".to_string();
        assert!(interop.validate_against_execution_binding(&binding).is_err());
    }

    #[test]
    fn interop_binding_requires_morn_execution_binding() {
        let binding = InteropBinding {
            work_ref: "work-1".to_string(),
            execution_binding_ref: "binding-1".to_string(),
            endpoint: ExternalEndpoint {
                protocol: InteropProtocol::A2a,
                endpoint_ref: "https://agent.example/a2a".to_string(),
                protocol_version: Some("1.0".to_string()),
                identity_ref: Some("workload://agent-example".to_string()),
            },
            capability_ref: "capability://planner".to_string(),
        };
        binding.validate().unwrap();
    }
}
