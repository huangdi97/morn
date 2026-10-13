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
use crate::dsh_acp::{DshAcpConfig, DshAcpControlHandle, DshAcpPromptResult, DshAcpStdioClient};
use crate::provider::DSH_REAL_E0_SCOPE_RESTRICTION;
use crate::scope::{CapabilityScope, ScopeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DshAcpLifecycleState {
    Active,
    /// Prompt settlement was lost or policy validation failed. No further work
    /// may be dispatched through this session; only quiescent cleanup remains.
    Faulted,
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
    pub scope_ref: String,
    pub route_ref: String,
    pub model_provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
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
    scope: CapabilityScope,
}

pub struct DshAcpLifecycleController {
    config: DshAcpConfig,
    client: Option<DshAcpStdioClient>,
    sessions: HashMap<String, BoundSession>,
    scopes: HashMap<String, CapabilityScope>,
    wire_agent_version: Option<String>,
}

impl std::fmt::Debug for DshAcpLifecycleController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshAcpLifecycleController")
            .field("configured", &true)
            .field("client_started", &self.client.is_some())
            .field("sessions", &self.sessions.len())
            .field("scopes", &self.scopes.len())
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
            scopes: HashMap::new(),
            wire_agent_version: None,
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::new(DshAcpConfig::from_env()?)
    }

    pub fn configured_route_ref(&self) -> Result<String> {
        self.config.route_ref()
    }

    pub fn configured_execution_environment_ref(&self) -> Option<&str> {
        self.config.execution_environment_ref.as_deref()
    }

    pub fn configured_profile_configuration_ref(&self) -> Option<&str> {
        self.config.profile_configuration_ref.as_deref()
    }

    pub fn configured_runtime_version(&self) -> Option<&str> {
        self.config.runtime_version.as_deref()
    }

    pub fn configured_runtime_digest(&self) -> Option<&str> {
        self.config.runtime_digest.as_deref()
    }

    pub fn wire_agent_version(&self) -> Option<&str> {
        self.wire_agent_version.as_deref()
    }

    pub fn session(&self, session_id: &str) -> Option<&DshAcpLifecycleSession> {
        self.sessions.get(session_id).map(|session| &session.public)
    }

    pub fn sessions(&self) -> Vec<DshAcpLifecycleSession> {
        let mut sessions: Vec<_> = self
            .sessions
            .values()
            .map(|session| session.public.clone())
            .collect();
        sessions.sort_by(|left, right| {
            left.opened_at
                .cmp(&right.opened_at)
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        sessions
    }

    pub fn mount_scope(&mut self, scope: CapabilityScope) -> Result<String> {
        if scope.kind != ScopeKind::ExecutionRun {
            return Err(Error::validation(
                "ACP lifecycle requires an ExecutionRun scope, not a broader reusable scope",
            ));
        }
        if !scope
            .restrictions
            .iter()
            .any(|restriction| restriction == DSH_REAL_E0_SCOPE_RESTRICTION)
        {
            return Err(Error::validation(
                "ACP lifecycle scope must explicitly declare morn.effects<=E0",
            ));
        }
        let id = scope.id.to_string();
        if self.scopes.contains_key(&id) {
            return Err(Error::conflict(format!(
                "ACP lifecycle scope {id} is already mounted"
            )));
        }
        self.scopes.insert(id.clone(), scope);
        Ok(id)
    }

    /// Revoke future prompt/resume admission immediately while preserving
    /// cancellation/close paths needed to quiesce an already-created session.
    pub fn unmount_scope(&mut self, scope_id: &str) -> Result<()> {
        if self.scopes.remove(scope_id).is_none() {
            return Err(Error::not_found(format!("ACP lifecycle scope {scope_id}")));
        }
        Ok(())
    }

    pub fn open_session(&mut self, context: &RuntimeContext) -> Result<DshAcpLifecycleSession> {
        self.validate_admission(context)?;
        let cwd = self.config.cwd.clone();
        let provider = self
            .config
            .provider
            .clone()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no model provider"))?;
        let model = self
            .config
            .model
            .clone()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no model"))?;
        let reasoning_effort = self.config.reasoning_effort.clone();
        let client = self.ensure_client()?;
        let session_id = client.new_session(&cwd)?;
        if let Err(error) =
            client.pin_model_route(&session_id, &provider, &model, reasoning_effort.as_deref())
        {
            let _ = client.close_session(&session_id);
            return Err(error);
        }
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
        if !matches!(
            session.public.state,
            DshAcpLifecycleState::Active | DshAcpLifecycleState::Faulted
        ) {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} is not open for cancellation"
            )));
        }
        self.client
            .as_ref()
            .ok_or_else(|| Error::invalid_state("ACP transport is not running"))?
            .control_handle(session_id)
    }

    pub fn prompt_text(
        &mut self,
        session_id: &str,
        context: &RuntimeContext,
        input: &str,
    ) -> Result<DshAcpPromptResult> {
        if input.trim().is_empty() {
            return Err(Error::validation("ACP prompt must be non-empty"));
        }
        self.validate_admission(context)?;
        self.validate_bound_session(session_id, context)?;
        let result = self
            .client
            .as_mut()
            .ok_or_else(|| Error::invalid_state("ACP transport is not running"))?
            .prompt_text(session_id, input);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?;
        session.public.prompt_count = session.public.prompt_count.saturating_add(1);
        match result {
            Ok(result) => {
                if !result.reached_protocol_settlement() {
                    session.public.state = DshAcpLifecycleState::Faulted;
                    return Err(Error::external(format!(
                        "ACP prompt returned unknown stop reason {:?}; session faulted and blind retry forbidden",
                        result.stop_reason
                    )));
                }
                session.public.last_stop_reason = Some(result.stop_reason.clone());
                // Deliberately do not create runtime-health or Work outcome state here.
                debug_assert!(!result.proves_executor_success());
                Ok(result)
            }
            Err(error) => {
                // A timeout/transport/policy error may leave activity in flight.
                // Request cancellation best-effort before returning, but never
                // interpret that write as proof of cancellation.
                if let Some(client) = self.client.as_mut() {
                    let _ = client.cancel_session(session_id);
                }
                session.public.state = DshAcpLifecycleState::Faulted;
                session.public.last_stop_reason = None;
                Err(error)
            }
        }
    }

    pub fn close_session(&mut self, session_id: &str) -> Result<()> {
        let state = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?
            .public
            .state;
        if !matches!(
            state,
            DshAcpLifecycleState::Active | DshAcpLifecycleState::Faulted
        ) {
            return Err(Error::invalid_state(format!(
                "ACP session {session_id} is not open for cleanup"
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

    pub fn resume_session(&mut self, session_id: &str, context: &RuntimeContext) -> Result<()> {
        self.validate_admission(context)?;
        let existing = self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::not_found(format!("ACP session {session_id}")))?
            .clone();
        match existing.public.state {
            DshAcpLifecycleState::Closed => {}
            DshAcpLifecycleState::TransportClosed => {
                return Err(Error::invalid_state(
                    "ACP transport restart uses a fresh one-shot DSH_HOME; cross-process resume is unavailable without a deployment-owned integrity-pinned durable session store",
                ));
            }
            DshAcpLifecycleState::Active | DshAcpLifecycleState::Faulted => {
                return Err(Error::invalid_state(format!(
                    "ACP session {session_id} must be cleanly closed before resume"
                )));
            }
        }
        self.ensure_same_binding(&existing, context)?;
        let cwd = self.config.cwd.clone();
        let provider = self
            .config
            .provider
            .clone()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no model provider"))?;
        let model = self
            .config
            .model
            .clone()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no model"))?;
        let reasoning_effort = self.config.reasoning_effort.clone();
        let client = self.ensure_client()?;
        client.resume_session(session_id, &cwd)?;
        if let Err(error) =
            client.pin_model_route(session_id, &provider, &model, reasoning_effort.as_deref())
        {
            let _ = client.close_session(session_id);
            return Err(error);
        }
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
        // The managed production ACP transport uses a fresh one-shot
        // DSH_HOME. Once that process is gone, even an explicitly closed
        // session cannot be truthfully advertised as resumable by a new
        // process because the persisted session store was intentionally reaped.
        for session in self.sessions.values_mut() {
            session.public.state = DshAcpLifecycleState::TransportClosed;
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
        let scope_ref = context
            .scope_id
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires mounted scope"))?;
        let scope = self
            .scopes
            .get(&scope_ref)
            .cloned()
            .ok_or_else(|| Error::validation("ACP session scope is not mounted"))?;
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
        let model_provider = self
            .config
            .provider
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires model provider"))?;
        let model = self
            .config
            .model
            .clone()
            .ok_or_else(|| Error::validation("ACP session requires model"))?;
        let reasoning_effort = self.config.reasoning_effort.clone();
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
                scope_ref,
                route_ref: self.config.route_ref()?,
                model_provider,
                model,
                reasoning_effort,
                runtime_version,
                runtime_digest,
                wire_agent_version,
                state: DshAcpLifecycleState::Active,
                prompt_count: 0,
                last_stop_reason: None,
                opened_at: Timestamp::now(),
            },
            context: context.clone(),
            scope,
        })
    }

    fn validate_admission(&self, context: &RuntimeContext) -> Result<()> {
        if !context.proves_pinned_work_execution() {
            return Err(Error::validation(
                "ACP lifecycle session requires exact Work generation and ExecutionBinding",
            ));
        }
        let runtime_version = self
            .config
            .runtime_version
            .as_deref()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no runtime version"))?;
        let runtime_digest = self
            .config
            .runtime_digest
            .as_deref()
            .ok_or_else(|| Error::validation("ACP lifecycle config has no runtime digest"))?;
        let route_ref = self.config.route_ref()?;
        if !context.proves_provider_identity(
            "deepseek-harness",
            runtime_version,
            Some(runtime_digest),
            Some(&route_ref),
        ) {
            return Err(Error::validation(
                "ACP lifecycle RuntimeContext does not match the exact DSH runtime/digest/route pinned by the ExecutionBinding",
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
        let scope_id = context
            .scope_id
            .as_deref()
            .ok_or_else(|| Error::validation("ACP RuntimeContext has no mounted scope id"))?;
        let scope = self
            .scopes
            .get(scope_id)
            .ok_or_else(|| Error::validation("ACP E0 scope is not currently mounted"))?;
        if scope.workspace_id != context.workspace_id
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
        self.ensure_same_binding(session, context)
    }

    fn ensure_same_binding(&self, session: &BoundSession, context: &RuntimeContext) -> Result<()> {
        let current_scope = context
            .scope_id
            .as_deref()
            .and_then(|scope_id| self.scopes.get(scope_id));
        if session.public.workspace_id != context.workspace_id
            || session.public.work_package_id != context.work_package_id
            || Some(session.public.work_generation) != context.work_generation
            || context.execution_binding_ref.as_ref() != Some(&session.public.execution_binding_ref)
            || context.provider_ref != session.context.provider_ref
            || context.provider_version != session.context.provider_version
            || context.provider_digest != session.context.provider_digest
            || context.provider_route_ref != session.context.provider_route_ref
            || context.execution_environment_ref.as_deref()
                != Some(session.public.execution_environment_ref.as_str())
            || context.scope_id.as_deref() != Some(session.public.scope_ref.as_str())
            || current_scope != Some(&session.scope)
            || session.public.route_ref != self.config.route_ref()?
        {
            return Err(Error::validation(
                "ACP session cannot be rebound to another Work, generation, binding, environment, scope snapshot or model route",
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
                .with_model_route("fixture-provider", "fixture-model")
                .with_reasoning_effort("high")
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
        let route_ref = config.route_ref().unwrap();
        let runtime_digest = format!("sha256:{}", "b".repeat(64));

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
        .with_provider_identity(
            "deepseek-harness",
            "1.0.0",
            Some(runtime_digest),
            Some(route_ref),
        )
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
        controller.mount_scope(scope.clone()).unwrap();
        let session = controller.open_session(&context).unwrap();
        assert_eq!(session.work_generation, 1);
        assert_eq!(session.scope_ref, scope.id.to_string());
        assert_eq!(session.model_provider, "fixture-provider");
        assert_eq!(session.model, "fixture-model");
        assert_eq!(session.reasoning_effort.as_deref(), Some("high"));
        assert_eq!(
            session.execution_binding_ref,
            context.execution_binding_ref.clone().unwrap()
        );
        let result = controller
            .prompt_text(&session.session_id, &context, "hello")
            .unwrap();
        assert_eq!(result.assistant_text, "hello ACP");
        assert!(!result.proves_executor_success());
        assert_eq!(
            controller
                .session(&session.session_id)
                .unwrap()
                .prompt_count,
            1
        );

        controller.close_session(&session.session_id).unwrap();
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::Closed
        );
        controller
            .resume_session(&session.session_id, &context)
            .unwrap();
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::Active
        );
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn scope_revocation_blocks_new_prompts_but_not_session_cleanup() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        controller.mount_scope(scope.clone()).unwrap();
        let session = controller.open_session(&context).unwrap();
        controller.unmount_scope(scope.id.as_str()).unwrap();

        assert!(controller
            .prompt_text(&session.session_id, &context, "must-not-run")
            .is_err());
        controller.close_session(&session.session_id).unwrap();
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unknown_prompt_settlement_faults_session_and_forbids_blind_retry() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        controller.mount_scope(scope).unwrap();
        let session = controller.open_session(&context).unwrap();

        assert!(controller
            .prompt_text(&session.session_id, &context, "unknown-stop-reason")
            .is_err());
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::Faulted
        );
        assert!(controller
            .prompt_text(&session.session_id, &context, "blind-retry")
            .is_err());

        assert!(controller.control_handle(&session.session_id).is_ok());
        controller.close_session(&session.session_id).unwrap();
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn transport_restart_does_not_fake_cross_process_resume_with_one_shot_home() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        controller.mount_scope(scope).unwrap();
        let session = controller.open_session(&context).unwrap();
        controller.close_session(&session.session_id).unwrap();
        controller.shutdown_transport().unwrap();
        assert_eq!(
            controller.session(&session.session_id).unwrap().state,
            DshAcpLifecycleState::TransportClosed
        );
        assert!(controller
            .resume_session(&session.session_id, &context)
            .is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn broader_or_recreated_scope_cannot_take_over_existing_session() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();

        let broad = CapabilityScope::new(
            ScopeKind::Workspace,
            None,
            context.workspace_id.clone(),
            "too-broad",
        )
        .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION);
        assert!(controller.mount_scope(broad).is_err());

        controller.mount_scope(scope.clone()).unwrap();
        let session = controller.open_session(&context).unwrap();
        controller.close_session(&session.session_id).unwrap();
        controller.unmount_scope(scope.id.as_str()).unwrap();

        let mut changed = scope.clone();
        changed
            .restrictions
            .push("morn.extra-restriction".to_string());
        controller.mount_scope(changed).unwrap();
        assert!(controller
            .resume_session(&session.session_id, &context)
            .is_err());

        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lifecycle_session_rejects_binding_id_reuse_with_provider_route_drift() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        controller.mount_scope(scope).unwrap();

        let mut forged_route = context.clone();
        forged_route.provider_route_ref = Some("dsh-acp-route:reinterpreted".to_string());
        assert!(controller.open_session(&forged_route).is_err());

        let session = controller.open_session(&context).unwrap();
        let mut forged_version = context.clone();
        forged_version.provider_version = Some("9.9.9".to_string());
        assert!(controller
            .prompt_text(&session.session_id, &forged_version, "must-not-run")
            .is_err());

        controller.close_session(&session.session_id).unwrap();
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lifecycle_session_rejects_context_rebinding() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let (mut controller, context, scope, root) = fixture();
        controller.mount_scope(scope.clone()).unwrap();
        let session = controller.open_session(&context).unwrap();
        controller.close_session(&session.session_id).unwrap();

        let mut forged = context.clone();
        forged.work_package_id = WorkPackageId::generate();
        assert!(controller
            .resume_session(&session.session_id, &forged)
            .is_err());
        controller.shutdown_transport().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
