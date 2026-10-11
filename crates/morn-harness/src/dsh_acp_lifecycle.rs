//! Work-bound DeepSeek ACP lifecycle control.
//!
//! ACP is intentionally not a replacement for the DSH SDK execution provider.
//! The official ACP stop reason collapses multiple underlying Harness outcomes,
//! so this controller may prove session lifecycle settlement but never upgrades
//! an executor to Healthy or a Work to Delivered/Accepted.

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

use crate::context::RuntimeContext;
use crate::dsh_acp::{
    DshAcpConfig, DshAcpControlHandle, DshAcpPromptResult, DshAcpStdioClient,
};
use crate::provider::DSH_REAL_E0_SCOPE_RESTRICTION;
use crate::scope::CapabilityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DshAcpLifecycleState {
    Active,
    Closed,
    TransportClosed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DshAcpLifecycleSession {
    pub session_id: String,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub work_generation: u64,
    pub execution_binding_ref: RuntimeBindingId,
    pub execution_environment_ref: String,
    pub route_ref: String,
    pub runtime_version: String,
    pub runtime_digest: String,
    pub wire_agent_version: String,
    pub state: DshAcpLifecycleState,
    pub prompt_count: u64,
    pub last_stop_reason: Option<String>,
    pub opened_at: Timestamp,
}

#[derive(Debug, Clone)]
struct BoundSession {
    public: DshAcpLifecycleSession,
    context: RuntimeContext,
}

pub struct DshAcpLifecycleController {
    config: DshAcpConfig,
    client: Option<DshAcpStdioClient>,
    sessions: HashMap<String, BoundSession>,
    wire_agent_version: Option<String>,
}

impl std::fmt::Debug for DshAcpLifecycleController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshAcpLifecycleController")
            .field("configured", &true)
            .field("client_started", &self.client.is_some())
            .field("sessions", &self.sessions.len())
            .field("wire_agent_version", &self.wire_agent_version)
            .finish()
    }
}

impl DshAcpLifecycleController {
    pub fn new(config: DshAcpConfig) -> Result<Self> {
        config.validate_for_real()?;
        Ok(Self {
            config,
            client: None,
            sessions: HashMap::new(),
            wire_agent_version: None,
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::new(DshAcpConfig::from_env()?)
    }

    pub fn configured_route_ref(&self) -> Result<String> {
        self.config.route_ref()
    }

    pub fn wire_agent_version(&self) -> Option<&str> {
        self.wire_agent_version.as_deref()
    }

    pub fn session(&self, session_id: &str) -> Option<&DshAcpLifecycleSession> {
        self.sessions.get(session_id).map(|session| &session.public)
    }

    pub fn open_session(
        &mut self,
        context: &RuntimeContext,
        scope: &CapabilityScope,
    ) -> Result<DshAcpLifecycleSession> {
        self.validate_admission(context, scope)?;
        let cwd = self.config.cwd.clone();
        let session_id = self.ensure_client()?.new_session(&cwd)?;
        let session = self.bound_session(session_id.clone(), context)?;
        self.sessions.insert(session_id, session.clone());
        Ok(session.public)
    }

    /// Return a cloneable transport handle before a blocking prompt begins.
    /// A successful write proves only that a cancellation request reached the
    /// live ACP transport. The prompt's later stop reason remains the evidence
    /// for whether that activity actually settled as cancelled.
    pub fn control_handle(&self, session_id: &str) -> Result<DshAcpControlHandle> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?;
        if session.public.state != DshAcpLifecycleState::Active {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} is not active"
            )));
        }
        self.client
            .as_ref()
            .ok_or_else(|| Error::invalid_state("ACP transport is not running"))?
            .control_handle(session_id)
    }

    pub fn prompt_text(&mut self, session_id: &str, input: &str) -> Result<DshAcpPromptResult> {
        if input.trim().is_empty() {
            return Err(Error::validation("ACP prompt must be non-empty"));
        }
        let context = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?
            .context
            .clone();
        self.validate_bound_session(session_id, &context)?;
        let result = self
            .client
            .as_mut()
            .ok_or_else(|| Error::invalid_state("ACP transport is not running"))?
            .prompt_text(session_id, input)?;
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?;
        session.public.prompt_count = session.public.prompt_count.saturating_add(1);
        session.public.last_stop_reason = Some(result.stop_reason.clone());
        // Deliberately do not create runtime-health or Work outcome state here.
        debug_assert!(!result.proves_executor_success());
        Ok(result)
    }

    pub fn close_session(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?
            .public
            .state;
        if state != DshAcpLifecycleState::Active {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} is not active"
            )));
        }
        self.client
            .as_mut()
            .ok_or_else(|| Error::invalid_state("ACP transport is not running"))?
            .close_session(session_id)?;
        self.sessions
            .get_mut(session_id)
            .expect("session checked above")
            .public
            .state = DshAcpLifecycleState::Closed;
        Ok(())
    }

    pub fn resume_session(
        &mut self,
        session_id: &str,
        context: &RuntimeContext,
        scope: &CapabilityScope,
    ) -> Result<()> {
        self.validate_admission(context, scope)?;
        let existing = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?
            .clone();
        if !matches!(
            existing.public.state,
            DshAcpLifecycleState::Closed | DshAcpLifecycleState::TransportClosed
        ) {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} must be inactive before resume"
            )));
        }
        self.ensure_same_binding(&existing.public, context)?;
        let cwd = self.config.cwd.clone();
        self.ensure_client()?.resume_session(session_id, &cwd)?;
        let resumed = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?;
        resumed.context = context.clone();
        resumed.public.state = DshAcpLifecycleState::Active;
        Ok(())
    }

    pub fn shutdown_transport(&mut self) -> Result<()> {
        let result = match self.client.take() {
            Some(mut client) => client.shutdown_process(),
            None => Ok(()),
        };
        for session in self.sessions.values_mut() {
            if session.public.state == DshAcpLifecycleState::Active {
                session.public.state = DshAcpLifecycleState::TransportClosed;
            }
        }
        result
    }

    fn ensure_client(&mut self) -> Result<&mut DshAcpStdioClient> {
        if self.client.is_none() {
            self.config.validate_for_real()?;
            let mut client = DshAcpStdioClient::spawn(&self.config)?;
            let info = client.initialize()?;
            client.authenticate_noop()?;
            self.wire_agent_version = Some(info.version);
            self.client = Some(client);
        }
        self.client
            .as_mut()
            .ok_or_else(|| Error::internal("ACP client missing after initialization"))
    }

    fn bound_session(&self, session_id: String, context: &RuntimeContext) -> Result<BoundSession> {
        let work_generation = context
            .work_generation
            .ok_or_else(|| Error::validation("ACP session requires Work generation"))?;
        let execution_binding_ref = context
            .execution_binding_ref
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires ExecutionBinding"))?;
        let execution_environment_ref = context
            .execution_environment_ref
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires execution environment"))?;
        let runtime_version = self
            .config
            .runtime_version
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires runtime version"))?;
        let runtime_digest = self
            .config
            .runtime_digest
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires runtime digest"))?;
        let wire_agent_version = self
            .wire_agent_version
            .clone()
            .ok_or_else(|| Error::invalid_state("ACP initialize identity is unavailable"))?;
        Ok(BoundSession {
            public: DshAcpLifecycleSession {
                session_id,
                workspace_id: context.workspace_id.clone(),
                work_package_id: context.work_package_id.clone(),
                work_generation,
                execution_binding_ref,
                execution_environment_ref,
                route_ref: self.config.route_ref()?,
                runtime_version,
                runtime_digest,
                wire_agent_version,
                state: DshAcpLifecycleState::Active,
                prompt_count: 0,
                last_stop_reason: None,
                opened_at: Timestamp::now(),
            },
            context: context.clone(),
        })
    }

    fn validate_admission(&self, context: &RuntimeContext, scope: &CapabilityScope) -> Result<()> {
        if !context.proves_pinned_work_execution() {
            return Err(Error::validation(
                "ACP lifecycle session requires exact Work generation and ExecutionBinding",
            ));
        }
        if !context.proves_real_harness_environment() {
            return Err(Error::validation(
                "ACP lifecycle session requires runtime-attested container-or-stronger execution environment",
            ));
        }
        let configured_environment = self
            .config
            .execution_environment_ref
            .as_deref()
            .ok_or_else(|| Error::validation("ACP config has no execution environment"))?;
        if context.execution_environment_ref.as_deref() != Some(configured_environment) {
            return Err(Error::validation(
                "ACP RuntimeContext environment does not match the pinned launch environment",
            ));
        }
        if scope.workspace_id != context.workspace_id
            || context.scope_id.as_deref() != Some(scope.id.as_str())
            || !scope
                .restrictions
                .iter()
                .any(|restriction| restriction == DSH_REAL_E0_SCOPE_RESTRICTION)
        {
            return Err(Error::validation(
                "ACP lifecycle session requires the exact mounted Morn E0 scope",
            ));
        }
        Ok(())
    }

    fn validate_bound_session(&self, session_id: &str, context: &RuntimeContext) -> Result<()> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?;
        if session.public.state != DshAcpLifecycleState::Active {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} is not active"
            )));
        }
        self.ensure_same_binding(&session.public, context)
    }

    fn ensure_same_binding(
        &self,
        session: &DshAcpLifecycleSession,
        context: &RuntimeContext,
    ) -> Result<()> {
        if session.workspace_id != context.workspace_id
            || session.work_package_id != context.work_package_id
            || Some(session.work_generation) != context.work_generation
            || context.execution_binding_ref.as_ref() != Some(&session.execution_binding_ref)
            || context.execution_environment_ref.as_deref()
                != Some(session.execution_environment_ref.as_str())
            || session.route_ref != self.config.route_ref()?
        {
            return Err(Error::validation(
                "ACP session cannot be rebound to another Work, generation, binding, environment or route",
            ));
        }
        Ok(())
    }
}

impl Drop for DshAcpLifecycleController {
    fn drop(&mut self) {
        let _ = self.shutdown_transport();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use morn_kernel::ids::{ActorInstanceId, RuntimeBindingId, WorkPackageId, WorkspaceId};
    use morn_kernel::{ExecutionClass, ExecutionGuarantee};

    use crate::scope::ScopeKind;

    fn fixture() -> (
        DshAcpLifecycleController,
        RuntimeContext,
        CapabilityScope,
        std::path::PathBuf,
    ) {
        let root =
            std::env::temp_dir().join(format!("morn-acp-lifecycle-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        let home = root.join("home");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&home).unwrap();

        let mut config =
            DshAcpConfig::profile_acp(workspace.to_string_lossy(), home.to_string_lossy())
                .with_execution_environment_ref("env://container/acp")
                .with_profile_configuration_ref(format!(
                    "deepseek-harness-profile@1.0.0#sha256:{}",
                    "a".repeat(64)
                ))
                .with_runtime_identity("1.0.0", format!("sha256:{}", "b".repeat(64)))
                .with_test_env("MORN_ACP_FIXTURE_SERVER", "1");
        config.allow_protocol_fixture_transport();
        config.command = std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .to_string();
        config.args = vec![
            "--exact".to_string(),
            "dsh_acp::tests::fixture_acp_server_process".to_string(),
            "--nocapture".to_string(),
        ];
        config.request_timeout_ms = 5_000;
        config.prompt_timeout_ms = 5_000;

        let workspace_id = WorkspaceId::generate();
        let scope = CapabilityScope::new(
            ScopeKind::ExecutionRun,
            None,
            workspace_id.clone(),
            "acp-lifecycle",
        )
        .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
        let context = RuntimeContext::new(
            workspace_id,
            ActorInstanceId::generate(),
            WorkPackageId::generate(),
        )
        .with_work_binding(1, RuntimeBindingId::generate_with("binding"))
        .unwrap()
        .with_scope_id(scope.id.to_string())
        .unwrap()
        .with_execution_environment(
            "env://container/acp",
            ExecutionClass::Container,
            vec![
                ExecutionGuarantee::FilesystemReadPolicy,
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::ProcessBoundary,
                ExecutionGuarantee::ResourceLimits,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
                ExecutionGuarantee::RuntimeAttestation,
                ExecutionGuarantee::ToolMediation,
            ],
        )
        .unwrap();
        (
            DshAcpLifecycleController::new(config).unwrap(),
            context,
            scope,
            root,
        )
    }

    #[test]
    fn lifecycle_session_is_work_bound_and_never_proves_executor_success() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        let session = controller.open_session(&context, &scope).unwrap();
        assert_eq!(session.work_generation, 1);
        assert_eq!(
            session.execution_binding_ref,
            context.execution_binding_ref.clone().unwrap()
        );
        let result = controller.prompt_text(&session.session_id, "hello").unwrap();
        assert_eq!(result.assistant_text, "hello ACP");
        assert!(!result.proves_executor_success());
        assert_eq!(
            controller.session(&session.session_id).unwrap().prompt_count,
            1
        );

        controller.close_session(&session.session_id).unwrap();
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::Closed
        );
        controller
            .resume_session(&session.session_id, &context, &scope)
            .unwrap();
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::Active
        );
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lifecycle_session_rejects_context_rebinding() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        let session = controller.open_session(&context, &scope).unwrap();
        controller.close_session(&session.session_id).unwrap();

        let mut forged = context.clone();
        forged.work_package_id = WorkPackageId::generate();
        assert!(controller
            .resume_session(&session.session_id, &forged, &scope)
            .is_err());
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
