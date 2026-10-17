//! DeepSeek Harness ACP lifecycle transport.
//!
//! This module speaks the official automation-only ACP JSON-RPC stdio surface.
//! It is intentionally separate from the production DSH SDK provider: ACP
//! exposes richer session lifecycle controls, but Morn must not claim those
//! controls through the synchronous HarnessProvider trait until out-of-band
//! interruption can be represented honestly.
//!
//! The client always rejects/cancels server permission requests. Tool execution
//! remains outside this transport's authority boundary.

#[cfg(test)]
use std::collections::BTreeMap;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use morn_kernel::error::{Error, Result};

#[cfg(test)]
use crate::dsh_sdk::DSH_QUALIFIED_DISTRIBUTION_VERSION;
use crate::dsh_sdk::{
    derive_morn_profile_configuration_ref, dsh_morn_e0_tool_policy_identity,
    validate_canonical_sha256_digest, validate_profile_configuration_ref,
    validate_qualified_dsh_distribution_version,
};
use crate::subprocess_wire::{read_bounded_utf8_line, MAX_PROVIDER_WIRE_BUFFERED_FRAMES};

pub const DSH_ACP_PROTOCOL_VERSION: u64 = 1;
pub const DSH_ACP_AGENT_NAME: &str = "deepseek-harness-acp";

pub const DSH_ACP_METHOD_INITIALIZE: &str = "initialize";
pub const DSH_ACP_METHOD_AUTHENTICATE: &str = "authenticate";
pub const DSH_ACP_METHOD_SESSION_NEW: &str = "session/new";
pub const DSH_ACP_METHOD_SESSION_LIST: &str = "session/list";
pub const DSH_ACP_METHOD_SESSION_RESUME: &str = "session/resume";
pub const DSH_ACP_METHOD_SESSION_CLOSE: &str = "session/close";
pub const DSH_ACP_METHOD_SESSION_PROMPT: &str = "session/prompt";
pub const DSH_ACP_METHOD_SESSION_SET_CONFIG_OPTION: &str = "session/set_config_option";
pub const DSH_ACP_METHOD_SESSION_CANCEL: &str = "session/cancel";
pub const DSH_ACP_METHOD_SESSION_UPDATE: &str = "session/update";
pub const DSH_ACP_METHOD_REQUEST_PERMISSION: &str = "session/request_permission";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshAcpConfig {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub dsh_home: String,
    /// Exact provider/model selection pinned through ACP session config.
    pub provider: Option<String>,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    /// Exact execution-environment identity that owns this ACP subprocess.
    pub execution_environment_ref: Option<String>,
    /// Secret-free identity for the exact DSH profile/home composition.
    pub profile_configuration_ref: Option<String>,
    /// Deployment-attested DSH distribution identity. ACP agentInfo.version
    /// is wire identity only and must never substitute for these fields.
    pub runtime_version: Option<String>,
    pub runtime_digest: Option<String>,
    pub request_timeout_ms: u64,
    pub prompt_timeout_ms: u64,
    /// Production ACP launches reuse Morn's final E0 DSH tool-deny policy and
    /// one-shot Harness home. Only the in-process wire fixture can disable it.
    enforce_morn_e0_tool_policy: bool,
    #[cfg(test)]
    test_env: BTreeMap<String, String>,
}

impl DshAcpConfig {
    pub fn profile_acp(cwd: impl Into<String>, dsh_home: impl Into<String>) -> Self {
        Self {
            command: "dsh".to_string(),
            args: vec!["--profile".to_string(), "acp".to_string()],
            cwd: cwd.into(),
            dsh_home: dsh_home.into(),
            provider: None,
            model: None,
            reasoning_effort: None,
            execution_environment_ref: None,
            profile_configuration_ref: None,
            runtime_version: None,
            runtime_digest: None,
            request_timeout_ms: 30_000,
            prompt_timeout_ms: 300_000,
            enforce_morn_e0_tool_policy: true,
            #[cfg(test)]
            test_env: BTreeMap::new(),
        }
    }

    pub fn with_model_route(
        mut self,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        self.provider = Some(provider.into());
        self.model = Some(model.into());
        self
    }

    pub fn with_reasoning_effort(mut self, reasoning_effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(reasoning_effort.into());
        self
    }

    pub fn with_execution_environment_ref(mut self, environment_ref: impl Into<String>) -> Self {
        self.execution_environment_ref = Some(environment_ref.into());
        self
    }

    pub fn with_profile_configuration_ref(
        mut self,
        profile_configuration_ref: impl Into<String>,
    ) -> Self {
        self.profile_configuration_ref = Some(profile_configuration_ref.into());
        self
    }

    pub fn with_runtime_identity(
        mut self,
        version: impl Into<String>,
        digest: impl Into<String>,
    ) -> Self {
        self.runtime_version = Some(version.into());
        self.runtime_digest = Some(digest.into());
        self
    }

    pub fn route_ref(&self) -> Result<String> {
        let profile_configuration_ref = self
            .profile_configuration_ref
            .as_deref()
            .ok_or_else(|| Error::validation("DSH ACP route requires profile configuration"))?;
        validate_profile_configuration_ref(profile_configuration_ref)?;
        let provider = self
            .provider
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::validation("DSH ACP route requires provider"))?;
        let model = self
            .model
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::validation("DSH ACP route requires model"))?;
        let route = json!({
            "args": &self.args,
            "provider": provider,
            "model": model,
            "reasoning_effort": &self.reasoning_effort,
            "profile_configuration_ref": profile_configuration_ref,
            "tool_policy": if self.enforce_morn_e0_tool_policy {
                Some(dsh_morn_e0_tool_policy_identity())
            } else {
                None
            },
            "permission_policy": "reject-all",
            "transport": "acp-v1",
        });
        Ok(format!("dsh-acp-route:{route}"))
    }

    /// Bounded, deployment-attestable identity for the exact ACP semantic route.
    /// The raw route_ref remains available for diagnostics, but Work/runtime
    /// identity and execution-environment attestations use only this hash.
    pub fn route_attestation_ref(&self) -> Result<String> {
        let route = self.route_ref()?;
        let digest = Sha256::digest(route.as_bytes());
        Ok(format!("deepseek-harness-acp-route@v1#sha256:{digest:x}"))
    }

    pub fn validate_for_real(&self) -> Result<()> {
        self.validate()?;
        if self
            .provider
            .as_deref()
            .is_none_or(|provider| provider.trim().is_empty())
            || self
                .model
                .as_deref()
                .is_none_or(|model| model.trim().is_empty())
        {
            return Err(Error::validation(
                "real DSH ACP requires an exact provider/model route",
            ));
        }
        if self
            .reasoning_effort
            .as_deref()
            .is_some_and(|effort| effort.trim().is_empty())
        {
            return Err(Error::validation(
                "real DSH ACP reasoning effort must be non-empty when configured",
            ));
        }
        if self
            .execution_environment_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "real DSH ACP requires a pinned execution_environment_ref",
            ));
        }
        let runtime_version = self
            .runtime_version
            .as_deref()
            .filter(|version| !version.trim().is_empty())
            .ok_or_else(|| {
                Error::validation("real DSH ACP requires a deployment-attested runtime version")
            })?;
        // Protocol fixtures are compiled only under cfg(test) and do not
        // represent a qualified DeepSeek Harness distribution. Production
        // transports keep the exact distribution qualification gate.
        if self.enforce_morn_e0_tool_policy {
            validate_qualified_dsh_distribution_version(runtime_version)?;
        }
        let digest = self
            .runtime_digest
            .as_deref()
            .ok_or_else(|| Error::validation("real DSH ACP requires a runtime SHA-256 digest"))?;
        validate_canonical_sha256_digest(digest, "real DSH ACP runtime")?;
        let profile_configuration_ref = self
            .profile_configuration_ref
            .as_deref()
            .ok_or_else(|| Error::validation("real DSH ACP requires profile_configuration_ref"))?;
        validate_profile_configuration_ref(profile_configuration_ref)?;
        if self.enforce_morn_e0_tool_policy {
            let expected = derive_morn_profile_configuration_ref(runtime_version, digest, "acp")?;
            if profile_configuration_ref != expected {
                return Err(Error::validation(
                    "real DSH ACP profile_configuration_ref must be derived from the verified runtime artifact and Morn E0 composition",
                ));
            }
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self> {
        let cwd = std::env::var("MORN_DSH_ACP_WORKSPACE")
            .map_err(|_| Error::validation("MORN_DSH_ACP_WORKSPACE is required for real ACP"))?;
        let home = std::env::var("MORN_DSH_ACP_HOME")
            .map_err(|_| Error::validation("MORN_DSH_ACP_HOME is required for real ACP"))?;
        let environment_ref =
            std::env::var("MORN_DSH_ACP_EXECUTION_ENVIRONMENT_REF").map_err(|_| {
                Error::validation("MORN_DSH_ACP_EXECUTION_ENVIRONMENT_REF is required for real ACP")
            })?;
        let runtime_version = std::env::var("MORN_DSH_ACP_RUNTIME_VERSION").map_err(|_| {
            Error::validation("MORN_DSH_ACP_RUNTIME_VERSION is required for real ACP")
        })?;
        let runtime_digest = std::env::var("MORN_DSH_ACP_RUNTIME_DIGEST").map_err(|_| {
            Error::validation("MORN_DSH_ACP_RUNTIME_DIGEST is required for real ACP")
        })?;
        let derived_profile =
            derive_morn_profile_configuration_ref(&runtime_version, &runtime_digest, "acp")?;
        let profile_configuration_ref = match std::env::var(
            "MORN_DSH_ACP_PROFILE_CONFIGURATION_REF",
        ) {
            Ok(claimed) if claimed != derived_profile => {
                return Err(Error::validation(
                    "MORN_DSH_ACP_PROFILE_CONFIGURATION_REF does not match the verified runtime + Morn E0 composition",
                ));
            }
            Ok(claimed) => claimed,
            Err(_) => derived_profile,
        };
        let provider = std::env::var("MORN_DSH_ACP_PROVIDER")
            .unwrap_or_else(|_| "deepseek-official".to_string());
        let model =
            std::env::var("MORN_DSH_ACP_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string());
        let mut config = Self::profile_acp(cwd, home)
            .with_model_route(provider, model)
            .with_execution_environment_ref(environment_ref)
            .with_profile_configuration_ref(profile_configuration_ref)
            .with_runtime_identity(runtime_version, runtime_digest);
        if let Ok(command) = std::env::var("MORN_DSH_ACP_COMMAND") {
            config.command = command;
        }
        if let Ok(reasoning) = std::env::var("MORN_DSH_ACP_REASONING_EFFORT") {
            if !reasoning.trim().is_empty() {
                config = config.with_reasoning_effort(reasoning);
            }
        }
        if let Ok(timeout) = std::env::var("MORN_DSH_ACP_REQUEST_TIMEOUT_MS") {
            config.request_timeout_ms = timeout.parse().map_err(|_| {
                Error::validation("MORN_DSH_ACP_REQUEST_TIMEOUT_MS must be an integer")
            })?;
        }
        if let Ok(timeout) = std::env::var("MORN_DSH_ACP_PROMPT_TIMEOUT_MS") {
            config.prompt_timeout_ms = timeout.parse().map_err(|_| {
                Error::validation("MORN_DSH_ACP_PROMPT_TIMEOUT_MS must be an integer")
            })?;
        }
        config.validate_for_real()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.command.trim().is_empty()
            || self.cwd.trim().is_empty()
            || self.dsh_home.trim().is_empty()
        {
            return Err(Error::validation(
                "DSH ACP requires command, workspace and isolated DSH_HOME",
            ));
        }
        if self.request_timeout_ms == 0 || self.prompt_timeout_ms == 0 {
            return Err(Error::validation(
                "DSH ACP request and prompt timeouts must be positive",
            ));
        }
        let workspace = Path::new(&self.cwd);
        let home = Path::new(&self.dsh_home);
        if !workspace.is_absolute() || !home.is_absolute() {
            return Err(Error::validation(
                "DSH ACP workspace and DSH_HOME must be absolute",
            ));
        }
        if workspace == home || workspace.starts_with(home) || home.starts_with(workspace) {
            return Err(Error::validation(
                "DSH ACP workspace and DSH_HOME must be disjoint directory trees",
            ));
        }
        if self.enforce_morn_e0_tool_policy
            && !(self.args.len() == 2 && self.args[0] == "--profile" && self.args[1] == "acp")
        {
            return Err(Error::validation(
                "governed DSH ACP requires the exact official --profile acp launcher boundary",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn allow_protocol_fixture_transport(&mut self) {
        self.enforce_morn_e0_tool_policy = false;
    }

    #[cfg(test)]
    pub(crate) fn with_test_env(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.test_env.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshAcpServerInfo {
    pub protocol_version: u64,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DshAcpNotification {
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DshAcpPromptResult {
    pub session_id: String,
    pub stop_reason: String,
    pub assistant_text: String,
}

impl DshAcpPromptResult {
    /// ACP proves that the automation interval reached a protocol settlement,
    /// not that the underlying DSH turn succeeded. Upstream maps completed,
    /// blocked/error and some aborted turn endings to the same `end_turn`
    /// stop reason, so Morn must never promote this result to Healthy or Work
    /// truth without a stronger execution-evidence channel.
    pub const fn proves_executor_success(&self) -> bool {
        false
    }

    pub fn was_cancelled(&self) -> bool {
        self.stop_reason == "cancelled"
    }

    pub fn reached_protocol_settlement(&self) -> bool {
        matches!(
            self.stop_reason.as_str(),
            "end_turn" | "cancelled" | "max_tokens" | "refusal"
        )
    }
}

type AcpWireItem = std::result::Result<Value, String>;

#[derive(Clone)]
pub struct DshAcpControlHandle {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    session_id: String,
}

impl std::fmt::Debug for DshAcpControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshAcpControlHandle")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl DshAcpControlHandle {
    /// Send an ACP cancel notification for the one session this handle owns.
    ///
    /// Success means the notification was written to the live transport; it
    /// does not by itself prove cancellation. The execution path must still
    /// observe the prompt stop reason / durable updates before changing Work.
    pub fn request_cancel(&self) -> Result<()> {
        write_shared_frame(
            &self.stdin,
            json!({
                "jsonrpc":"2.0",
                "method":DSH_ACP_METHOD_SESSION_CANCEL,
                "params":{"sessionId":self.session_id}
            }),
        )
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

pub struct DshAcpStdioClient {
    child: Child,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    incoming: Mutex<Receiver<AcpWireItem>>,
    reader: Option<JoinHandle<()>>,
    wire_overflowed: Arc<AtomicBool>,
    next_id: u64,
    request_timeout: Duration,
    prompt_timeout: Duration,
    owned_policy_dir: Option<std::path::PathBuf>,
    owned_runtime_home: Option<std::path::PathBuf>,
    pub notifications: Vec<DshAcpNotification>,
}

impl std::fmt::Debug for DshAcpStdioClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshAcpStdioClient")
            .field("child_id", &self.child.id())
            .field("next_id", &self.next_id)
            .field("notifications", &self.notifications.len())
            .finish()
    }
}

fn dsh_acp_process_launch_cwd<'a>(
    workspace: &'a Path,
    owned_runtime_home: Option<&'a Path>,
) -> &'a Path {
    owned_runtime_home.unwrap_or(workspace)
}

impl DshAcpStdioClient {
    pub fn spawn(config: &DshAcpConfig) -> Result<Self> {
        config.validate()?;
        let expected_cwd = std::fs::canonicalize(&config.cwd).map_err(|error| {
            Error::validation(format!("canonicalize DSH ACP workspace: {error}"))
        })?;
        let mut command = Command::new(&config.command);
        let (launch_args, owned_policy_dir, owned_runtime_home) =
            if config.enforce_morn_e0_tool_policy {
                // Reuse the exact production SDK E0 overlay/one-shot-home
                // machinery. ACP adds lifecycle semantics, not a wider effect boundary.
                let mut policy = crate::dsh_sdk::DshSdkConfig::profile_sdk(
                    config.cwd.clone(),
                    "morn-acp-policy",
                    "morn-acp-policy",
                )
                .with_dsh_home(config.dsh_home.clone());
                policy.args = config.args.clone();
                let (launch_args, policy_dir) = policy.launch_args()?;
                let runtime_home = match policy.materialize_managed_runtime_home() {
                    Ok(home) => home,
                    Err(error) => {
                        if let Some(policy_dir) = &policy_dir {
                            let _ = std::fs::remove_dir_all(policy_dir);
                        }
                        return Err(error);
                    }
                };
                (launch_args, policy_dir, runtime_home)
            } else {
                (config.args.clone(), None, None)
            };
        // Match the SDK provider boundary: the DSH launcher itself runs from
        // Morn's fresh one-shot Home so a Work workspace .env cannot become an
        // ambient launcher/config layer. The ACP session still receives the
        // canonical Work workspace explicitly through session/new or resume.
        let launch_cwd = dsh_acp_process_launch_cwd(&expected_cwd, owned_runtime_home.as_deref());
        command
            .args(&launch_args)
            .current_dir(launch_cwd)
            .env_clear()
            .envs(crate::subprocess_env::scrubbed_environment(
                "MORN_DSH_ACP_ENV_PASSTHROUGH",
            ))
            .env(crate::dsh_sdk::DSH_MORN_WORKSPACE_ROOT_ENV, &expected_cwd)
            .env("DSH_PERMISSION_MODE", "read-only")
            .env("DSH_MAX_TOKENS_AS_SUCCESS", "false")
            .env("DSH_TELEMETRY_DISABLED", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(home) = &owned_runtime_home {
            command.env("DSH_HOME", home);
        } else {
            command.env("DSH_HOME", &config.dsh_home);
        }
        #[cfg(test)]
        command.envs(&config.test_env);

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                if let Some(policy_dir) = &owned_policy_dir {
                    let _ = std::fs::remove_dir_all(policy_dir);
                }
                if let Some(runtime_home) = &owned_runtime_home {
                    let _ = std::fs::remove_dir_all(runtime_home);
                }
                return Err(Error::external(format!(
                    "failed to start DSH ACP runtime {:?}: {error}",
                    config.command
                )));
            }
        };
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::external("DSH ACP runtime has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::external("DSH ACP runtime has no stdout"))?;

        let (sender, incoming) =
            mpsc::sync_channel::<AcpWireItem>(MAX_PROVIDER_WIRE_BUFFERED_FRAMES);
        let wire_overflowed = Arc::new(AtomicBool::new(false));
        let reader_overflowed = Arc::clone(&wire_overflowed);
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let line = match read_bounded_utf8_line(&mut stdout, "DSH ACP JSON-RPC") {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        let _ = sender.try_send(Err("DSH ACP runtime closed stdout".to_string()));
                        break;
                    }
                    Err(error) => {
                        let _ = sender.try_send(Err(error));
                        break;
                    }
                };
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let value = match serde_json::from_str::<Value>(trimmed) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                match sender.try_send(Ok(value)) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        reader_overflowed.store(true, Ordering::Release);
                        break;
                    }
                    Err(TrySendError::Disconnected(_)) => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin: Arc::new(Mutex::new(Some(stdin))),
            incoming: Mutex::new(incoming),
            reader: Some(reader),
            wire_overflowed,
            next_id: 1,
            request_timeout: Duration::from_millis(config.request_timeout_ms),
            prompt_timeout: Duration::from_millis(config.prompt_timeout_ms),
            owned_policy_dir,
            owned_runtime_home,
            notifications: Vec::new(),
        })
    }

    pub fn control_handle(&self, session_id: &str) -> Result<DshAcpControlHandle> {
        if session_id.trim().is_empty() {
            return Err(Error::validation(
                "DSH ACP control handle requires a non-empty session id",
            ));
        }
        Ok(DshAcpControlHandle {
            stdin: Arc::clone(&self.stdin),
            session_id: session_id.to_string(),
        })
    }

    pub fn initialize(&mut self) -> Result<DshAcpServerInfo> {
        let result = self.request(
            DSH_ACP_METHOD_INITIALIZE,
            Some(json!({
                "protocolVersion": DSH_ACP_PROTOCOL_VERSION,
                "clientCapabilities": {}
            })),
            self.request_timeout,
        )?;
        parse_acp_server_info(&result)
    }

    pub fn authenticate_noop(&mut self) -> Result<()> {
        self.request(
            DSH_ACP_METHOD_AUTHENTICATE,
            Some(json!({"methodId":"unused"})),
            self.request_timeout,
        )?;
        Ok(())
    }

    pub fn new_session(&mut self, cwd: &str) -> Result<String> {
        let result = self.request(
            DSH_ACP_METHOD_SESSION_NEW,
            Some(json!({"cwd":cwd,"mcpServers":[]})),
            self.request_timeout,
        )?;
        required_string(&result, "sessionId", "DSH ACP session/new result")
    }

    pub fn list_sessions(&mut self, cwd: Option<&str>) -> Result<Value> {
        self.list_sessions_page(cwd, None)
    }

    pub fn list_sessions_page(&mut self, cwd: Option<&str>, cursor: Option<&str>) -> Result<Value> {
        let mut params = serde_json::Map::new();
        if let Some(cwd) = cwd {
            params.insert("cwd".to_string(), json!(cwd));
        }
        if let Some(cursor) = cursor {
            params.insert("cursor".to_string(), json!(cursor));
        }
        self.request(
            DSH_ACP_METHOD_SESSION_LIST,
            Some(Value::Object(params)),
            self.request_timeout,
        )
    }

    pub fn resume_session(&mut self, session_id: &str, cwd: &str) -> Result<()> {
        self.request(
            DSH_ACP_METHOD_SESSION_RESUME,
            Some(json!({"sessionId":session_id,"cwd":cwd,"mcpServers":[]})),
            self.request_timeout,
        )?;
        Ok(())
    }

    pub fn close_session(&mut self, session_id: &str) -> Result<()> {
        self.request(
            DSH_ACP_METHOD_SESSION_CLOSE,
            Some(json!({"sessionId":session_id})),
            self.request_timeout,
        )?;
        Ok(())
    }

    pub fn set_session_config_option(
        &mut self,
        session_id: &str,
        config_id: &str,
        value: Value,
    ) -> Result<Value> {
        self.request(
            DSH_ACP_METHOD_SESSION_SET_CONFIG_OPTION,
            Some(json!({
                "sessionId": session_id,
                "configId": config_id,
                "value": value
            })),
            self.request_timeout,
        )
    }

    /// Pin the exact model route before any prompt is admitted and verify the
    /// ACP server reports the same resulting currentValue.
    pub fn pin_model_route(
        &mut self,
        session_id: &str,
        provider: &str,
        model: &str,
        reasoning_effort: Option<&str>,
    ) -> Result<()> {
        if provider.trim().is_empty() || model.trim().is_empty() {
            return Err(Error::validation(
                "DSH ACP model route requires non-empty provider and model",
            ));
        }
        let model_value = serde_json::to_string(&[provider, model])
            .map_err(|error| Error::internal(format!("encode ACP model route: {error}")))?;
        let result = self.set_session_config_option(session_id, "model", json!(model_value))?;
        require_config_option_value(&result, "model", &json!(model_value))?;

        if let Some(reasoning) = reasoning_effort {
            if reasoning.trim().is_empty() {
                return Err(Error::validation(
                    "DSH ACP reasoning effort must be non-empty when configured",
                ));
            }
            let result =
                self.set_session_config_option(session_id, "reasoning_effort", json!(reasoning))?;
            require_config_option_value(&result, "reasoning_effort", &json!(reasoning))?;
        }
        Ok(())
    }

    pub fn cancel_session(&mut self, session_id: &str) -> Result<()> {
        self.control_handle(session_id)?.request_cancel()
    }

    pub fn prompt_text(&mut self, session_id: &str, text: &str) -> Result<DshAcpPromptResult> {
        if session_id.trim().is_empty() || text.trim().is_empty() {
            return Err(Error::validation(
                "DSH ACP prompt requires non-empty session id and text",
            ));
        }
        // ACP lifecycle controllers can stay alive for many sessions. Keep
        // only the current prompt interval in memory; prior notifications have
        // already served their caller and must not become an unbounded transcript.
        self.notifications.clear();
        let result = self.request(
            DSH_ACP_METHOD_SESSION_PROMPT,
            Some(json!({
                "sessionId":session_id,
                "prompt":[{"type":"text","text":text}]
            })),
            self.prompt_timeout,
        );
        let updates = std::mem::take(&mut self.notifications);
        if updates.iter().any(acp_update_reports_tool_activity) {
            // Policy evidence dominates a later protocol or transport failure.
            return Err(Error::external(
                "DSH ACP reported tool activity on a transport that has no Morn authority grant",
            ));
        }
        let result = result?;
        let stop_reason = required_string(&result, "stopReason", "DSH ACP prompt result")?;
        Ok(DshAcpPromptResult {
            session_id: session_id.to_string(),
            stop_reason,
            assistant_text: assistant_text_from_acp_updates(&updates, session_id),
        })
    }

    pub fn shutdown_process(&mut self) -> Result<()> {
        self.stdin
            .lock()
            .map_err(|_| Error::internal("DSH ACP stdin lock poisoned"))?
            .take();
        let deadline = Instant::now() + Duration::from_secs(6);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return Err(Error::external(
                        "DSH ACP runtime did not exit after stdin EOF; process was force-reaped",
                    ));
                }
                Err(error) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return Err(Error::external(format!(
                        "inspect DSH ACP exit after stdin EOF: {error}"
                    )));
                }
            }
        }
    }

    fn request(&mut self, method: &str, params: Option<Value>, timeout: Duration) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.write_frame(request_frame(id, method, params))?;
        let deadline = Instant::now() + timeout;
        loop {
            let incoming = self.read_frame_until(deadline, method)?;
            require_jsonrpc_v2(&incoming)?;
            let has_id = incoming.get("id").is_some();
            let has_method = incoming.get("method").is_some();
            if has_id && has_method {
                self.answer_server_request(&incoming)?;
                continue;
            }
            if has_id {
                return response_result_for_id(&incoming, id, method);
            }
            if has_method {
                let method = incoming
                    .get("method")
                    .and_then(Value::as_str)
                    .filter(|method| !method.trim().is_empty())
                    .ok_or_else(|| Error::external("DSH ACP notification method is empty"))?;
                let params = incoming.get("params").cloned().unwrap_or(Value::Null);
                // ACP deliberately carries committed agent thoughts. Morn never
                // needs private reasoning for Work truth, lifecycle control, or
                // acceptance, so discard it at the transport boundary instead
                // of retaining it in the in-memory notification transcript.
                if acp_notification_is_private_thought(method, &params) {
                    continue;
                }
                self.notifications.push(DshAcpNotification {
                    method: method.to_string(),
                    params,
                });
                continue;
            }
            return Err(Error::external(
                "DSH ACP emitted a JSON-RPC frame with neither id nor method",
            ));
        }
    }

    fn answer_server_request(&mut self, request: &Value) -> Result<()> {
        let id = request
            .get("id")
            .cloned()
            .ok_or_else(|| Error::external("DSH ACP server request has no id"))?;
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::external("DSH ACP server request has no method"))?;
        let response = if method == DSH_ACP_METHOD_REQUEST_PERMISSION {
            json!({
                "jsonrpc":"2.0",
                "id":id,
                "result":permission_denial_result(
                    request.get("params").unwrap_or(&Value::Null)
                )
            })
        } else {
            json!({
                "jsonrpc":"2.0",
                "id":id,
                "error":{"code":-32601,"message":"Method not supported by Morn ACP client"}
            })
        };
        self.write_frame(response)
    }

    fn write_frame(&self, frame: Value) -> Result<()> {
        write_shared_frame(&self.stdin, frame)
    }

    fn read_frame_until(&mut self, deadline: Instant, operation: &str) -> Result<Value> {
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "DSH ACP wire exceeded buffered frame capacity",
            ));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::external(format!("DSH ACP {operation} timed out")));
        }
        let receiver = self
            .incoming
            .lock()
            .map_err(|_| Error::internal("DSH ACP receiver lock poisoned"))?;
        let result = match receiver.recv_timeout(remaining) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(message)) => Err(Error::external(message)),
            Err(RecvTimeoutError::Timeout) => {
                Err(Error::external(format!("DSH ACP {operation} timed out")))
            }
            Err(RecvTimeoutError::Disconnected) => {
                Err(Error::external("DSH ACP stdout reader disconnected"))
            }
        };
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "DSH ACP wire exceeded buffered frame capacity",
            ));
        }
        result
    }
}

impl Drop for DshAcpStdioClient {
    fn drop(&mut self) {
        if let Ok(mut stdin) = self.stdin.lock() {
            stdin.take();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        if let Some(policy_dir) = self.owned_policy_dir.take() {
            let _ = std::fs::remove_dir_all(policy_dir);
        }
        if let Some(runtime_home) = self.owned_runtime_home.take() {
            let _ = std::fs::remove_dir_all(runtime_home);
        }
    }
}

fn write_shared_frame(stdin: &Arc<Mutex<Option<ChildStdin>>>, frame: Value) -> Result<()> {
    let mut guard = stdin
        .lock()
        .map_err(|_| Error::internal("DSH ACP stdin lock poisoned"))?;
    let stream = guard
        .as_mut()
        .ok_or_else(|| Error::external("DSH ACP stdin is closed"))?;
    serde_json::to_writer(&mut *stream, &frame)
        .map_err(|error| Error::external(format!("encode DSH ACP JSON-RPC: {error}")))?;
    stream
        .write_all(b"\n")
        .and_then(|_| stream.flush())
        .map_err(|error| Error::external(format!("write DSH ACP JSON-RPC: {error}")))
}

fn request_frame(id: u64, method: &str, params: Option<Value>) -> Value {
    let mut frame = json!({"jsonrpc":"2.0","id":id,"method":method});
    if let Some(params) = params {
        frame
            .as_object_mut()
            .expect("ACP request frame is always an object")
            .insert("params".to_string(), params);
    }
    frame
}

fn require_jsonrpc_v2(value: &Value) -> Result<()> {
    if value.get("jsonrpc").and_then(Value::as_str) == Some("2.0") {
        Ok(())
    } else {
        Err(Error::external(
            "DSH ACP emitted a frame without jsonrpc=\"2.0\"",
        ))
    }
}

fn response_result_for_id(value: &Value, expected_id: u64, operation: &str) -> Result<Value> {
    let response_id = value
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::external("DSH ACP response id must be an unsigned integer"))?;
    if response_id != expected_id {
        return Err(Error::external(format!(
            "DSH ACP response id mismatch: expected {expected_id}, received {response_id}"
        )));
    }
    let has_result = value.get("result").is_some();
    let has_error = value.get("error").is_some();
    if has_result == has_error {
        return Err(Error::external(
            "DSH ACP response must contain exactly one of result or error",
        ));
    }
    if let Some(error) = value.get("error") {
        return Err(Error::external(format!(
            "DSH ACP JSON-RPC {operation} failed: {error}"
        )));
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| Error::external("DSH ACP response result disappeared"))
}

fn parse_acp_server_info(result: &Value) -> Result<DshAcpServerInfo> {
    let protocol_version = result
        .get("protocolVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::external("DSH ACP initialize missing protocolVersion"))?;
    if protocol_version != DSH_ACP_PROTOCOL_VERSION {
        return Err(Error::external(format!(
            "DSH ACP negotiated unexpected protocol version {protocol_version}"
        )));
    }
    let info = result
        .get("agentInfo")
        .ok_or_else(|| Error::external("DSH ACP initialize missing agentInfo"))?;
    let name = required_string(info, "name", "DSH ACP agentInfo")?;
    let version = required_string(info, "version", "DSH ACP agentInfo")?;
    if name != DSH_ACP_AGENT_NAME {
        return Err(Error::external(format!(
            "unexpected DSH ACP agent identity {name:?}"
        )));
    }
    Ok(DshAcpServerInfo {
        protocol_version,
        name,
        version,
    })
}

fn require_config_option_value(result: &Value, config_id: &str, expected: &Value) -> Result<()> {
    let current = result
        .get("configOptions")
        .and_then(Value::as_array)
        .and_then(|options| {
            options
                .iter()
                .find(|option| option.get("id").and_then(Value::as_str) == Some(config_id))
        })
        .and_then(|option| option.get("currentValue"))
        .ok_or_else(|| {
            Error::external(format!(
                "DSH ACP set_config_option did not report currentValue for {config_id}"
            ))
        })?;
    if current != expected {
        return Err(Error::external(format!(
            "DSH ACP config option {config_id} drifted: expected {expected}, received {current}"
        )));
    }
    Ok(())
}

fn required_string(value: &Value, key: &str, context: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::external(format!("{context} missing {key}")))
}

fn permission_denial_result(params: &Value) -> Value {
    let reject = params
        .get("options")
        .and_then(Value::as_array)
        .and_then(|options| {
            options
                .iter()
                .find(|option| option.get("kind").and_then(Value::as_str) == Some("reject_once"))
        })
        .and_then(|option| option.get("optionId"))
        .and_then(Value::as_str);
    match reject {
        Some(option_id) => json!({
            "outcome":{"outcome":"selected","optionId":option_id}
        }),
        None => json!({"outcome":{"outcome":"cancelled"}}),
    }
}

fn acp_notification_is_private_thought(method: &str, params: &Value) -> bool {
    method == DSH_ACP_METHOD_SESSION_UPDATE
        && params
            .get("update")
            .and_then(|update| update.get("sessionUpdate"))
            .and_then(Value::as_str)
            == Some("agent_thought_chunk")
}

fn acp_update_reports_tool_activity(notification: &DshAcpNotification) -> bool {
    notification.method == DSH_ACP_METHOD_SESSION_UPDATE
        && matches!(
            notification
                .params
                .get("update")
                .and_then(|update| update.get("sessionUpdate"))
                .and_then(Value::as_str),
            Some("tool_call" | "tool_call_update")
        )
}

pub fn assistant_text_from_acp_updates(
    notifications: &[DshAcpNotification],
    session_id: &str,
) -> String {
    notifications
        .iter()
        .filter(|notification| {
            notification.method == DSH_ACP_METHOD_SESSION_UPDATE
                && notification.params.get("sessionId").and_then(Value::as_str) == Some(session_id)
        })
        .filter_map(|notification| notification.params.get("update"))
        .filter(|update| {
            update.get("sessionUpdate").and_then(Value::as_str) == Some("agent_message_chunk")
        })
        .filter_map(|update| update.get("content"))
        .filter(|content| content.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    fn governed_acp_profile(runtime_version: &str, runtime_digest: &str) -> String {
        derive_morn_profile_configuration_ref(runtime_version, runtime_digest, "acp").unwrap()
    }

    #[test]
    fn governed_acp_process_launch_isolated_from_work_workspace() {
        let workspace = Path::new("work-workspace");
        let runtime_home = Path::new("one-shot-runtime-home");

        assert_eq!(
            dsh_acp_process_launch_cwd(workspace, Some(runtime_home)),
            runtime_home,
            "governed ACP must launch DSH from the one-shot runtime Home, not the Work workspace",
        );
        assert_eq!(
            dsh_acp_process_launch_cwd(workspace, None),
            workspace,
            "protocol fixtures without a managed runtime Home keep their explicit fixture cwd",
        );
    }

    #[test]
    fn permission_requests_fail_closed_to_reject_once_or_cancelled() {
        assert_eq!(
            permission_denial_result(&json!({
                "options":[
                    {"optionId":"allow-once","kind":"allow_once"},
                    {"optionId":"reject-once","kind":"reject_once"}
                ]
            })),
            json!({"outcome":{"outcome":"selected","optionId":"reject-once"}})
        );
        assert_eq!(
            permission_denial_result(&json!({"options":[]})),
            json!({"outcome":{"outcome":"cancelled"}})
        );
    }

    #[test]
    fn extracts_only_visible_agent_message_updates_for_one_session() {
        let updates = vec![
            DshAcpNotification {
                method: DSH_ACP_METHOD_SESSION_UPDATE.to_string(),
                params: json!({
                    "sessionId":"a",
                    "update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"hello "}}
                }),
            },
            DshAcpNotification {
                method: DSH_ACP_METHOD_SESSION_UPDATE.to_string(),
                params: json!({
                    "sessionId":"a",
                    "update":{"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"private"}}
                }),
            },
            DshAcpNotification {
                method: DSH_ACP_METHOD_SESSION_UPDATE.to_string(),
                params: json!({
                    "sessionId":"a",
                    "update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"world"}}
                }),
            },
            DshAcpNotification {
                method: DSH_ACP_METHOD_SESSION_UPDATE.to_string(),
                params: json!({
                    "sessionId":"b",
                    "update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"other"}}
                }),
            },
        ];
        assert_eq!(
            assistant_text_from_acp_updates(&updates, "a"),
            "hello world"
        );
        assert!(acp_notification_is_private_thought(
            &updates[1].method,
            &updates[1].params
        ));
        assert!(
            !acp_notification_is_private_thought(DSH_ACP_METHOD_SESSION_UPDATE, &updates[0].params),
            "visible assistant messages must remain available to the caller"
        );
    }

    #[test]
    fn parses_only_the_expected_acp_identity_and_protocol() {
        let info = parse_acp_server_info(&json!({
            "protocolVersion":1,
            "agentInfo":{"name":"deepseek-harness-acp","version":"0.0.1"}
        }))
        .unwrap();
        assert_eq!(info.protocol_version, 1);
        assert_eq!(info.name, DSH_ACP_AGENT_NAME);
        assert!(parse_acp_server_info(&json!({
            "protocolVersion":1,
            "agentInfo":{"name":"other","version":"0.0.1"}
        }))
        .is_err());
    }

    #[test]
    fn acp_end_turn_is_settlement_not_executor_success_proof() {
        let result = DshAcpPromptResult {
            session_id: "acp-1".to_string(),
            stop_reason: "end_turn".to_string(),
            assistant_text: "candidate response".to_string(),
        };
        assert!(result.reached_protocol_settlement());
        assert!(!result.proves_executor_success());
        assert!(!result.was_cancelled());

        let cancelled = DshAcpPromptResult {
            stop_reason: "cancelled".to_string(),
            ..result
        };
        assert!(cancelled.reached_protocol_settlement());
        assert!(cancelled.was_cancelled());
        assert!(!cancelled.proves_executor_success());
    }

    #[test]
    fn tool_updates_are_never_silently_treated_as_read_only_output() {
        let tool = DshAcpNotification {
            method: DSH_ACP_METHOD_SESSION_UPDATE.to_string(),
            params: json!({
                "sessionId":"a",
                "update":{"sessionUpdate":"tool_call","toolCallId":"call-1"}
            }),
        };
        assert!(acp_update_reports_tool_activity(&tool));
    }

    #[test]
    fn acp_config_requires_disjoint_absolute_workspace_and_home() {
        let root = std::env::temp_dir().join(format!("morn-acp-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        let home = root.join("home");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let config = DshAcpConfig::profile_acp(workspace.to_string_lossy(), home.to_string_lossy());
        assert!(config.validate().is_ok());
        let mut wrong_profile = config.clone();
        wrong_profile.args = vec!["--profile".to_string(), "web".to_string()];
        assert!(wrong_profile.validate().is_err());
        let nested = DshAcpConfig::profile_acp(
            workspace.to_string_lossy(),
            workspace.join(".dsh").to_string_lossy(),
        );
        assert!(nested.validate().is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn real_acp_config_requires_deployment_identity_and_pins_route() {
        let root = std::env::temp_dir().join(format!("morn-acp-real-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        let home = root.join("home");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let base = DshAcpConfig::profile_acp(workspace.to_string_lossy(), home.to_string_lossy());
        assert!(base.validate().is_ok());
        assert!(base.validate_for_real().is_err());

        let configured = base
            .with_model_route("fixture-provider", "fixture-model")
            .with_reasoning_effort("high")
            .with_execution_environment_ref("env://container/acp")
            .with_profile_configuration_ref(governed_acp_profile(
                DSH_QUALIFIED_DISTRIBUTION_VERSION,
                &format!("sha256:{}", "b".repeat(64)),
            ))
            .with_runtime_identity(
                DSH_QUALIFIED_DISTRIBUTION_VERSION,
                format!("sha256:{}", "b".repeat(64)),
            );
        assert!(configured.validate_for_real().is_ok());
        let route = configured.route_ref().unwrap();
        assert!(route.contains("dsh-acp-route:"));
        assert!(route.contains("permission_policy"));
        assert!(route.contains("fixture-provider"));
        assert!(route.contains("fixture-model"));
        assert!(!route.contains("env://container/acp"));
        assert!(!route.contains(&"b".repeat(64)));

        let attestation_ref = configured.route_attestation_ref().unwrap();
        assert!(attestation_ref.starts_with("deepseek-harness-acp-route@v1#sha256:"));
        assert_eq!(
            attestation_ref.len(),
            "deepseek-harness-acp-route@v1#sha256:".len() + 64
        );
        assert!(!attestation_ref.contains("fixture-provider"));
        let mut changed = configured.clone();
        changed.model = Some("other-model".to_string());
        assert_ne!(
            attestation_ref,
            changed.route_attestation_ref().unwrap(),
            "semantic route changes must require a new deployment attestation"
        );

        let mut bad_digest = configured.clone();
        bad_digest.runtime_digest = Some("sha256:short".to_string());
        assert!(bad_digest.validate_for_real().is_err());

        let mut uppercase_digest = configured.clone();
        uppercase_digest.runtime_digest = Some(format!("sha256:{}", "B".repeat(64)));
        assert!(
            uppercase_digest.validate_for_real().is_err(),
            "artifact identity must have one canonical lowercase spelling"
        );

        let mut uppercase_profile = configured.clone();
        uppercase_profile.profile_configuration_ref = Some(format!(
            "deepseek-harness-profile@1.0.0#sha256:{}",
            "A".repeat(64)
        ));
        assert!(
            uppercase_profile.validate_for_real().is_err(),
            "profile identity must have one canonical lowercase spelling"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn fixture_acp_server_process() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() != Some("1") {
            return;
        }
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let stdout = std::io::stdout();
        let mut output = stdout.lock();
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line).unwrap() == 0 {
                break;
            }
            let Ok(frame) = serde_json::from_str::<Value>(line.trim()) else {
                continue;
            };
            let method = frame.get("method").and_then(Value::as_str).unwrap_or("");
            let id = frame.get("id").cloned();
            match (method, id) {
                (DSH_ACP_METHOD_INITIALIZE, Some(id)) => {
                    writeln!(output, "{}", json!({
                        "jsonrpc":"2.0","id":id,"result":{
                            "protocolVersion":1,
                            "agentInfo":{"name":"deepseek-harness-acp","version":"0.0.1"},
                            "agentCapabilities":{"sessionCapabilities":{"close":{},"list":{},"resume":{}}},
                            "authMethods":[]
                        }
                    })).unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_AUTHENTICATE, Some(id)) => {
                    writeln!(output, "{}", json!({"jsonrpc":"2.0","id":id,"result":{}})).unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_NEW, Some(id)) => {
                    writeln!(output, "{}", json!({
                        "jsonrpc":"2.0","id":id,"result":{"sessionId":"acp-fixture-1","configOptions":[]}
                    })).unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_PROMPT, Some(id)) => {
                    let prompt = frame
                        .pointer("/params/prompt/0/text")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if prompt == "unknown-stop-reason" {
                        writeln!(
                            output,
                            "{}",
                            json!({
                                "jsonrpc":"2.0","id":id,"result":{"stopReason":"future-unknown-reason"}
                            })
                        )
                        .unwrap();
                        output.flush().unwrap();
                        continue;
                    }
                    if prompt == "wait-for-out-of-band-cancel" {
                        if let Ok(path) = std::env::var("MORN_ACP_CANCEL_READY_FILE") {
                            std::fs::write(path, b"ready").unwrap();
                        }
                        let mut cancel = String::new();
                        input.read_line(&mut cancel).unwrap();
                        let cancel: Value = serde_json::from_str(cancel.trim()).unwrap();
                        assert_eq!(
                            cancel.get("method").and_then(Value::as_str),
                            Some(DSH_ACP_METHOD_SESSION_CANCEL)
                        );
                        assert_eq!(
                            cancel.pointer("/params/sessionId").and_then(Value::as_str),
                            Some("acp-fixture-1")
                        );
                        writeln!(
                            output,
                            "{}",
                            json!({
                                "jsonrpc":"2.0","id":id,"result":{"stopReason":"cancelled"}
                            })
                        )
                        .unwrap();
                        output.flush().unwrap();
                        continue;
                    }

                    writeln!(output, "{}", json!({
                        "jsonrpc":"2.0","id":900,"method":"session/request_permission","params":{
                            "sessionId":"acp-fixture-1",
                            "toolCall":{"toolCallId":"call-1"},
                            "options":[
                                {"optionId":"allow-once","kind":"allow_once"},
                                {"optionId":"reject-once","kind":"reject_once"}
                            ]
                        }
                    })).unwrap();
                    output.flush().unwrap();

                    let mut permission = String::new();
                    input.read_line(&mut permission).unwrap();
                    let permission: Value = serde_json::from_str(permission.trim()).unwrap();
                    assert_eq!(permission.get("id"), Some(&json!(900)));
                    assert_eq!(
                        permission
                            .pointer("/result/outcome/optionId")
                            .and_then(Value::as_str),
                        Some("reject-once")
                    );

                    writeln!(output, "{}", json!({
                        "jsonrpc":"2.0","method":"session/update","params":{
                            "sessionId":"acp-fixture-1",
                            "update":{"sessionUpdate":"agent_message_chunk","messageId":"m1","content":{"type":"text","text":"hello ACP"}}
                        }
                    })).unwrap();
                    writeln!(
                        output,
                        "{}",
                        json!({
                            "jsonrpc":"2.0","id":id,"result":{"stopReason":"end_turn"}
                        })
                    )
                    .unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_LIST, Some(id)) => {
                    writeln!(output, "{}", json!({
                        "jsonrpc":"2.0","id":id,"result":{"sessions":[{"sessionId":"acp-fixture-1","cwd":std::env::current_dir().unwrap()}]}
                    })).unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_SET_CONFIG_OPTION, Some(id)) => {
                    let config_id = frame
                        .pointer("/params/configId")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let value = frame
                        .pointer("/params/value")
                        .cloned()
                        .unwrap_or(Value::Null);
                    writeln!(
                        output,
                        "{}",
                        json!({
                            "jsonrpc":"2.0",
                            "id":id,
                            "result":{"configOptions":[{"id":config_id,"currentValue":value}]}
                        })
                    )
                    .unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_CLOSE, Some(id))
                | (DSH_ACP_METHOD_SESSION_RESUME, Some(id)) => {
                    writeln!(
                        output,
                        "{}",
                        json!({"jsonrpc":"2.0","id":id,"result":{"configOptions":[]}})
                    )
                    .unwrap();
                    output.flush().unwrap();
                }
                (DSH_ACP_METHOD_SESSION_CANCEL, None) => {}
                _ => {}
            }
        }
        std::process::exit(0);
    }

    #[test]
    fn cloneable_control_handle_cancels_a_prompt_while_execution_waits_for_settlement() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let root = std::env::temp_dir().join(format!("morn-acp-cancel-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        let home = root.join("home");
        let ready = root.join("prompt-ready");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&home).unwrap();

        let mut config =
            DshAcpConfig::profile_acp(workspace.to_string_lossy(), home.to_string_lossy())
                .with_test_env("MORN_ACP_FIXTURE_SERVER", "1")
                .with_test_env("MORN_ACP_CANCEL_READY_FILE", ready.to_string_lossy());
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

        let mut client = DshAcpStdioClient::spawn(&config).unwrap();
        client.initialize().unwrap();
        client.authenticate_noop().unwrap();
        let session = client.new_session(&config.cwd).unwrap();
        let control = client.control_handle(&session).unwrap();
        assert_eq!(control.session_id(), session);
        let thread_session = session.clone();
        let worker = std::thread::spawn(move || {
            let result = client.prompt_text(&thread_session, "wait-for-out-of-band-cancel");
            (client, result)
        });

        let deadline = Instant::now() + Duration::from_secs(3);
        while !ready.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            ready.exists(),
            "fixture prompt never reached cancellable state"
        );
        control.request_cancel().unwrap();

        let (mut client, result) = worker.join().unwrap();
        let result = result.unwrap();
        assert_eq!(result.stop_reason, "cancelled");
        assert_eq!(result.assistant_text, "");
        client.shutdown_process().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn stdio_fixture_covers_initialize_prompt_permission_reject_close_resume_and_list() {
        if std::env::var("MORN_ACP_FIXTURE_SERVER").ok().as_deref() == Some("1") {
            return;
        }
        let root = std::env::temp_dir().join(format!("morn-acp-wire-{}", uuid::Uuid::new_v4()));
        let workspace = root.join("workspace");
        let home = root.join("home");
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&home).unwrap();

        let mut config =
            DshAcpConfig::profile_acp(workspace.to_string_lossy(), home.to_string_lossy())
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

        let mut client = DshAcpStdioClient::spawn(&config).unwrap();
        let info = client.initialize().unwrap();
        assert_eq!(info.name, DSH_ACP_AGENT_NAME);
        client.authenticate_noop().unwrap();
        let session = client.new_session(&config.cwd).unwrap();
        client
            .pin_model_route(&session, "fixture-provider", "fixture-model", Some("high"))
            .unwrap();
        let result = client.prompt_text(&session, "hello").unwrap();
        assert_eq!(result.stop_reason, "end_turn");
        assert_eq!(result.assistant_text, "hello ACP");
        assert!(
            client.notifications.is_empty(),
            "settled ACP prompt notifications must be consumed after projection"
        );
        let options = client
            .set_session_config_option(
                &session,
                "reasoning_effort",
                Value::String("high".to_string()),
            )
            .unwrap();
        assert_eq!(
            options
                .pointer("/configOptions/0/currentValue")
                .and_then(Value::as_str),
            Some("high")
        );
        client.cancel_session(&session).unwrap();
        client.close_session(&session).unwrap();
        let listed = client.list_sessions(Some(&config.cwd)).unwrap();
        assert_eq!(
            listed
                .pointer("/sessions/0/sessionId")
                .and_then(Value::as_str),
            Some("acp-fixture-1")
        );
        client.resume_session(&session, &config.cwd).unwrap();
        client.shutdown_process().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
