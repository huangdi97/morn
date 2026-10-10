//! DeepSeek Harness SDK stdio boundary.
//!
//! DSH currently exposes newline-delimited JSON-RPC 2.0 over stdio. The
//! stable request methods at the researched upstream boundary are
//! `initialize`, `session/prompt`, and `shutdown`; notifications are
//! `session.event`, `session.status`, `subagent.started`, and
//! `subagent.finished`. The current wire has no cancel/session-close method,
//! so Morn must not fabricate those semantics.

use std::fs;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use morn_kernel::error::{Error, Result};

use crate::subprocess_wire::{read_bounded_utf8_line, MAX_PROVIDER_WIRE_BUFFERED_FRAMES};

pub const DSH_METHOD_INITIALIZE: &str = "initialize";
pub const DSH_METHOD_SESSION_PROMPT: &str = "session/prompt";
pub const DSH_METHOD_SHUTDOWN: &str = "shutdown";
pub const DSH_SDK_SERVER_NAME: &str = "deepseek-harness-sdk-runtime";

pub const DSH_NOTIFICATION_SESSION_EVENT: &str = "session.event";
pub const DSH_NOTIFICATION_SESSION_STATUS: &str = "session.status";
pub const DSH_NOTIFICATION_SUBAGENT_STARTED: &str = "subagent.started";
pub const DSH_NOTIFICATION_SUBAGENT_FINISHED: &str = "subagent.finished";

/// Morn-owned final DSH overlay for the real E0 provider path.
///
/// DSH's stock SDK profile intentionally ships useful model-facing tools.
/// Morn's HarnessProvider contract is stricter: direct Harness tools are not
/// the Authority/ExternalAction boundary. This final overlay disables the
/// shipped direct tool producers before a model turn. Runtime event checks
/// remain a second line of defence against added/unknown tool plugins.
pub const DSH_MORN_E0_TOOL_POLICY_REF: &str = "morn.dsh-e0-no-direct-tools/v1";
const DSH_MORN_E0_TOOL_POLICY_FILE: &str = "morn-e0-no-direct-tools.patch.yml";
const DSH_MORN_E0_TOOL_GUARD_FILE: &str = "morn-e0-tool-guard.mjs";
const DSH_MORN_E0_TOOL_GUARD_PLUGIN: &str = r#"export const name = 'morn-e0-tool-guard-v1'
export const inject = ['tools']

export function apply(ctx) {
  ctx.tools.guard(() => 'Morn E0 Harness policy denies direct DSH tool execution.')
}
"#;
const DSH_MORN_E0_TOOL_POLICY_PATCH: &str = r#"# Generated/owned by Morn. Applied LAST to the official DSH SDK profile.
# This plugin installs a global monotonic ToolRuntime guard. It denies unknown
# tools contributed by earlier profile/home layers as well as the shipped set.
- insert:
    - id: morn-e0-tool-guard-v1
      name: ./morn-e0-tool-guard.mjs

# Independent containment: a DSH sandbox remains read-only even though direct
# model-facing tools are denied by the guard.
- id: sandbox-policy
  config:
    mode: read-only
    workspaceRoot: !!js process.cwd()

# Defence-in-depth and prompt hygiene: hide shipped direct tool producers so
# the model is not invited to issue calls that the global guard will reject.
- id: tool-plugin-manager
  disabled: true
- id: tool-working-directory
  disabled: true
- id: tool-bash
  disabled: true
- id: tool-pwsh
  disabled: true
- id: tool-jobs
  disabled: true
- id: tool-fs
  disabled: true
- id: tool-fs-search
  disabled: true
- id: user-questions
  disabled: true
- id: tool-skill
  disabled: true
- id: plan-mode
  disabled: true
- id: goal-round-driver
  disabled: true
- id: command-goal
  disabled: true
- id: tool-subagent-control
  disabled: true
- id: tool-subagent-list-agents
  disabled: true
- id: tool-subagent
  disabled: true
- id: tool-subagent-fork
  disabled: true
- id: tool-workflow
  disabled: true
- id: tool-todo
  disabled: true
- id: tool-goal
  disabled: true
- id: tool-web
  disabled: true
- id: mcp-resources
  disabled: true
- id: workspace-dependencies
  disabled: true
- id: skill-office
  disabled: true
"#;

/// Secret-free cryptographic identity for the exact Morn policy composition.
fn dsh_morn_e0_tool_policy_identity() -> String {
    let mut digest = Sha256::new();
    digest.update(DSH_MORN_E0_TOOL_POLICY_PATCH.as_bytes());
    digest.update([0]);
    digest.update(DSH_MORN_E0_TOOL_GUARD_PLUGIN.as_bytes());
    format!(
        "{DSH_MORN_E0_TOOL_POLICY_REF}#sha256:{:x}",
        digest.finalize()
    )
}

#[derive(Debug)]
struct DshOwnedToolPolicy {
    directory: PathBuf,
    patch_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DshSdkConfig {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub max_tokens: Option<u64>,
    /// Isolated Harness home. Live deployments require this explicitly;
    /// protocol fixtures may leave it unset.
    pub dsh_home: Option<String>,
    /// Exact execution-environment identity that owns this subprocess launch.
    /// It must match RuntimeContext/ExecutionBinding; a caller-supplied
    /// isolation label alone is insufficient.
    #[serde(default)]
    pub execution_environment_ref: Option<String>,
    /// Deployment-attested DSH distribution identity. The SDK serverInfo
    /// version is a wire identity and must never be substituted for these.
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub runtime_digest: Option<String>,
    pub request_timeout_ms: u64,
    pub turn_timeout_ms: u64,
    /// Production DSH SDK launches apply a Morn-owned last-layer patch that
    /// disables shipped direct tools before the first model request. Protocol
    /// fixtures may opt out because they are not DSH compositions.
    #[serde(skip_serializing, skip_deserializing, default = "default_true")]
    enforce_morn_e0_tool_policy: bool,
}

const fn default_true() -> bool {
    true
}

impl DshSdkConfig {
    /// Stable, secret-free identity for the execution route selected inside
    /// one DSH distribution. Runtime artifact identity alone is insufficient:
    /// changing profile/model/reasoning can reinterpret the same Work.
    pub fn route_ref(&self) -> Result<String> {
        if self.provider.trim().is_empty() || self.model.trim().is_empty() {
            return Err(Error::validation(
                "DSH route identity requires provider and model",
            ));
        }
        let route = json!({
            "args": &self.args,
            "provider": &self.provider,
            "model": &self.model,
            "reasoning_effort": &self.reasoning_effort,
            "max_tokens": self.max_tokens,
            "tool_policy": if self.enforce_morn_e0_tool_policy {
                Some(dsh_morn_e0_tool_policy_identity())
            } else {
                None
            },
            "max_tokens_as_success": if self.enforce_morn_e0_tool_policy {
                Some(false)
            } else {
                None
            },
            "telemetry": if self.enforce_morn_e0_tool_policy {
                Some("disabled")
            } else {
                None
            },
        });
        Ok(format!("dsh-sdk-route:{route}"))
    }

    pub fn profile_sdk(
        cwd: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            command: "dsh".to_string(),
            args: vec!["--profile".to_string(), "sdk".to_string()],
            cwd: cwd.into(),
            provider: provider.into(),
            model: model.into(),
            reasoning_effort: None,
            max_tokens: None,
            dsh_home: None,
            execution_environment_ref: None,
            runtime_version: None,
            runtime_digest: None,
            request_timeout_ms: 30_000,
            turn_timeout_ms: 300_000,
            enforce_morn_e0_tool_policy: true,
        }
    }

    pub fn with_dsh_home(mut self, path: impl Into<String>) -> Self {
        self.dsh_home = Some(path.into());
        self
    }

    pub fn with_execution_environment_ref(mut self, environment_ref: impl Into<String>) -> Self {
        self.execution_environment_ref = Some(environment_ref.into());
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

    #[cfg(test)]
    pub(crate) fn allow_protocol_fixture_transport(&mut self) {
        self.enforce_morn_e0_tool_policy = false;
    }

    pub fn validate_for_real(&self) -> Result<()> {
        if self.command.trim().is_empty()
            || self.cwd.trim().is_empty()
            || self.provider.trim().is_empty()
            || self.model.trim().is_empty()
        {
            return Err(Error::validation(
                "real DSH SDK requires command, workspace, provider and model",
            ));
        }
        let home = self
            .dsh_home
            .as_deref()
            .filter(|home| !home.trim().is_empty())
            .ok_or_else(|| Error::validation("real DSH SDK requires isolated DSH_HOME"))?;
        if self
            .execution_environment_ref
            .as_deref()
            .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "real DSH SDK requires a pinned execution_environment_ref",
            ));
        }
        if self
            .runtime_version
            .as_deref()
            .is_none_or(|version| version.trim().is_empty())
        {
            return Err(Error::validation(
                "real DSH SDK requires a deployment-attested runtime version",
            ));
        }
        let digest = self
            .runtime_digest
            .as_deref()
            .ok_or_else(|| Error::validation("real DSH SDK requires a runtime SHA-256 digest"))?;
        let Some(hex) = digest.strip_prefix("sha256:") else {
            return Err(Error::validation(
                "real DSH SDK runtime digest must use sha256:<64-hex>",
            ));
        };
        if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::validation(
                "real DSH SDK runtime digest must use sha256:<64-hex>",
            ));
        }
        let workspace = Path::new(&self.cwd);
        let home_path = Path::new(home);
        if !workspace.is_absolute() || !home_path.is_absolute() {
            return Err(Error::validation(
                "real DSH SDK workspace and DSH_HOME must be absolute paths",
            ));
        }
        if workspace == home_path
            || home_path.starts_with(workspace)
            || workspace.starts_with(home_path)
        {
            return Err(Error::validation(
                "real DSH SDK workspace and DSH_HOME must be disjoint directory trees",
            ));
        }
        if self.request_timeout_ms == 0 || self.turn_timeout_ms == 0 {
            return Err(Error::validation(
                "DSH request and turn timeout must be positive",
            ));
        }
        if self.max_tokens == Some(0) {
            return Err(Error::validation(
                "DSH max_tokens must be positive when set",
            ));
        }
        if self.enforce_morn_e0_tool_policy
            && !(self.args.len() == 2 && self.args[0] == "--profile" && self.args[1] == "sdk")
        {
            return Err(Error::validation(
                "Morn E0 tool policy requires the exact official DSH --profile sdk launcher boundary",
            ));
        }
        Ok(())
    }

    fn materialize_morn_e0_tool_policy(&self) -> Result<Option<DshOwnedToolPolicy>> {
        if !self.enforce_morn_e0_tool_policy {
            return Ok(None);
        }
        let home = self
            .dsh_home
            .as_deref()
            .ok_or_else(|| Error::validation("Morn DSH tool policy requires DSH_HOME"))?;
        let workspace = fs::canonicalize(&self.cwd)
            .map_err(|error| Error::validation(format!("canonicalize DSH workspace: {error}")))?;
        if fs::symlink_metadata(home).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(Error::validation(
                "real DSH_HOME must not be a symbolic link",
            ));
        }
        fs::create_dir_all(home)
            .map_err(|error| Error::external(format!("create isolated DSH_HOME: {error}")))?;
        let canonical_home = fs::canonicalize(home)
            .map_err(|error| Error::validation(format!("canonicalize DSH_HOME: {error}")))?;
        if workspace == canonical_home
            || canonical_home.starts_with(&workspace)
            || workspace.starts_with(&canonical_home)
        {
            return Err(Error::validation(
                "real DSH SDK workspace and DSH_HOME must be physically disjoint directory trees",
            ));
        }

        let policy_root = canonical_home.join("morn-policy");
        match fs::symlink_metadata(&policy_root) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(Error::validation(
                    "Morn DSH policy root must be a real directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&policy_root).map_err(|error| {
                    Error::external(format!("create Morn DSH policy root: {error}"))
                })?;
            }
            Err(error) => {
                return Err(Error::external(format!(
                    "inspect Morn DSH policy root: {error}"
                )));
            }
        }

        let directory = policy_root.join(uuid::Uuid::new_v4().to_string());
        fs::create_dir(&directory).map_err(|error| {
            Error::external(format!(
                "create one-shot Morn DSH policy directory: {error}"
            ))
        })?;
        let patch_path = directory.join(DSH_MORN_E0_TOOL_POLICY_FILE);
        let guard_path = directory.join(DSH_MORN_E0_TOOL_GUARD_FILE);
        let write_result = (|| {
            create_exclusive_policy_file(
                &guard_path,
                DSH_MORN_E0_TOOL_GUARD_PLUGIN.as_bytes(),
                "guard plugin",
            )?;
            create_exclusive_policy_file(
                &patch_path,
                DSH_MORN_E0_TOOL_POLICY_PATCH.as_bytes(),
                "overlay",
            )?;
            Ok::<(), Error>(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_dir_all(&directory);
            return Err(error);
        }
        Ok(Some(DshOwnedToolPolicy {
            directory,
            patch_path,
        }))
    }

    fn launch_args(&self) -> Result<(Vec<String>, Option<PathBuf>)> {
        let mut args = self.args.clone();
        let policy = self.materialize_morn_e0_tool_policy()?;
        if let Some(policy) = &policy {
            args.push("--patch".to_string());
            args.push(policy.patch_path.to_string_lossy().to_string());
        }
        Ok((args, policy.map(|policy| policy.directory)))
    }

    pub fn from_env() -> Result<Self> {
        let cwd = std::env::var("MORN_DSH_WORKSPACE")
            .map_err(|_| Error::validation("MORN_DSH_WORKSPACE is required for real DSH"))?;
        let home = std::env::var("MORN_DSH_HOME")
            .map_err(|_| Error::validation("MORN_DSH_HOME is required for real DSH"))?;
        let provider =
            std::env::var("MORN_DSH_PROVIDER").unwrap_or_else(|_| "deepseek-official".to_string());
        let model =
            std::env::var("MORN_DSH_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string());
        let environment_ref =
            std::env::var("MORN_DSH_EXECUTION_ENVIRONMENT_REF").map_err(|_| {
                Error::validation("MORN_DSH_EXECUTION_ENVIRONMENT_REF is required for real DSH")
            })?;
        let runtime_version = std::env::var("MORN_DSH_RUNTIME_VERSION")
            .map_err(|_| Error::validation("MORN_DSH_RUNTIME_VERSION is required for real DSH"))?;
        let runtime_digest = std::env::var("MORN_DSH_RUNTIME_DIGEST")
            .map_err(|_| Error::validation("MORN_DSH_RUNTIME_DIGEST is required for real DSH"))?;
        let mut config = Self::profile_sdk(cwd, provider, model)
            .with_dsh_home(home)
            .with_execution_environment_ref(environment_ref)
            .with_runtime_identity(runtime_version, runtime_digest);
        if let Ok(command) = std::env::var("MORN_DSH_COMMAND") {
            config.command = command;
        }
        if let Ok(reasoning) = std::env::var("MORN_DSH_REASONING_EFFORT") {
            if !reasoning.trim().is_empty() {
                config.reasoning_effort = Some(reasoning);
            }
        }
        if let Ok(max_tokens) = std::env::var("MORN_DSH_MAX_TOKENS") {
            config.max_tokens = Some(
                max_tokens
                    .parse()
                    .map_err(|_| Error::validation("MORN_DSH_MAX_TOKENS must be an integer"))?,
            );
        }
        if let Ok(timeout) = std::env::var("MORN_DSH_REQUEST_TIMEOUT_MS") {
            config.request_timeout_ms = timeout
                .parse()
                .map_err(|_| Error::validation("MORN_DSH_REQUEST_TIMEOUT_MS must be an integer"))?;
        }
        if let Ok(timeout) = std::env::var("MORN_DSH_TURN_TIMEOUT_MS") {
            config.turn_timeout_ms = timeout
                .parse()
                .map_err(|_| Error::validation("MORN_DSH_TURN_TIMEOUT_MS must be an integer"))?;
        }
        config.validate_for_real()?;
        Ok(config)
    }
}
fn create_exclusive_policy_file(path: &Path, bytes: &[u8], label: &str) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|error| Error::external(format!("create Morn DSH {label}: {error}")))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| Error::external(format!("write Morn DSH {label}: {error}")))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DshSdkServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DshNotification {
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DshSdkRunResult {
    pub session_id: String,
    pub message_id: String,
    pub final_response: String,
    pub finish_reason: Option<String>,
    pub notifications: Vec<DshNotification>,
}

impl DshSdkRunResult {
    /// Only an explicit durable `turn/end: completed` is a successful executor turn.
    /// Idle, text output, or a transport-level response alone are insufficient.
    pub fn completed_successfully(&self) -> bool {
        self.finish_reason.as_deref() == Some("completed")
    }
}

type DshWireItem = std::result::Result<Value, String>;

pub struct DshSdkStdioClient {
    child: Child,
    stdin: Option<ChildStdin>,
    incoming: Mutex<Receiver<DshWireItem>>,
    reader: Option<JoinHandle<()>>,
    wire_overflowed: Arc<AtomicBool>,
    next_id: u64,
    request_timeout: Duration,
    turn_timeout: Duration,
    owned_policy_dir: Option<PathBuf>,
    pub notifications: Vec<DshNotification>,
}

impl std::fmt::Debug for DshSdkStdioClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshSdkStdioClient")
            .field("child_id", &self.child.id())
            .field("next_id", &self.next_id)
            .field("notifications", &self.notifications.len())
            .finish()
    }
}

impl DshSdkStdioClient {
    pub fn spawn(config: &DshSdkConfig) -> Result<Self> {
        if config.cwd.trim().is_empty()
            || config.provider.trim().is_empty()
            || config.model.trim().is_empty()
            || config.request_timeout_ms == 0
            || config.turn_timeout_ms == 0
        {
            return Err(Error::validation(
                "DSH SDK requires cwd/provider/model and positive timeouts",
            ));
        }
        let mut command = Command::new(&config.command);
        let (launch_args, owned_policy_dir) = config.launch_args()?;
        command
            .args(&launch_args)
            .current_dir(&config.cwd)
            .env_clear()
            .envs(crate::subprocess_env::scrubbed_environment(
                "MORN_DSH_ENV_PASSTHROUGH",
            ))
            .env("DSH_PERMISSION_MODE", "read-only")
            .env("DSH_MAX_TOKENS_AS_SUCCESS", "false")
            .env("DSH_TELEMETRY_DISABLED", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(home) = &config.dsh_home {
            command.env("DSH_HOME", home);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                if let Some(policy_dir) = &owned_policy_dir {
                    let _ = fs::remove_dir_all(policy_dir);
                }
                return Err(Error::external(format!(
                    "failed to start DSH SDK runtime {:?}: {error}",
                    config.command
                )));
            }
        };
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::external("DSH SDK runtime has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::external("DSH SDK runtime has no stdout"))?;
        let (sender, incoming) =
            mpsc::sync_channel::<DshWireItem>(MAX_PROVIDER_WIRE_BUFFERED_FRAMES);
        let wire_overflowed = Arc::new(AtomicBool::new(false));
        let reader_overflowed = Arc::clone(&wire_overflowed);
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let line = match read_bounded_utf8_line(&mut stdout, "DSH SDK JSON-RPC") {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        let _ = sender.try_send(Err("DSH SDK runtime closed stdout".to_string()));
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
                    Err(_) => {
                        // Upstream explicitly permits malformed lines to be ignored.
                        continue;
                    }
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
            stdin: Some(stdin),
            incoming: Mutex::new(incoming),
            reader: Some(reader),
            wire_overflowed,
            next_id: 1,
            request_timeout: Duration::from_millis(config.request_timeout_ms),
            turn_timeout: Duration::from_millis(config.turn_timeout_ms),
            owned_policy_dir,
            notifications: Vec::new(),
        })
    }

    pub fn initialize(&mut self, config: &DshSdkConfig) -> Result<DshSdkServerInfo> {
        let mut params = json!({
            "cwd": config.cwd,
            "provider": config.provider,
            "model": config.model,
        });
        let object = params
            .as_object_mut()
            .expect("initialize params are always an object");
        if let Some(reasoning) = &config.reasoning_effort {
            object.insert("reasoningEffort".to_string(), json!(reasoning));
        }
        if let Some(max_tokens) = config.max_tokens {
            object.insert("maxTokens".to_string(), json!(max_tokens));
        }
        let result = self.request(DSH_METHOD_INITIALIZE, Some(params))?;
        parse_server_info(&result)
    }

    pub fn enqueue_text_prompt(&mut self, session_id: &str, text: &str) -> Result<String> {
        let result = self.request(
            DSH_METHOD_SESSION_PROMPT,
            Some(json!({
                "sessionId": session_id,
                "contentBlocks": [{"type":"text","text":text}],
            })),
        )?;
        result
            .get("messageId")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| Error::external("DSH session/prompt result missing messageId"))
    }

    /// Own one SDK activity interval: durable inbox receipt -> root-session idle.
    /// A successful run is executor evidence only; callers must not treat this
    /// text as a Morn ObservedOutcome or AcceptanceDecision.
    pub fn run_text_prompt(&mut self, session_id: &str, text: &str) -> Result<DshSdkRunResult> {
        if session_id.trim().is_empty() || text.trim().is_empty() {
            return Err(Error::validation(
                "DSH SDK run requires non-empty session id and prompt text",
            ));
        }
        let start = self.notifications.len();
        let message_id = self.enqueue_text_prompt(session_id, text)?;
        let deadline = Instant::now() + self.turn_timeout;
        let mut received = false;
        let mut final_response = String::new();
        let mut finish_reason = None;
        let mut cursor = start;

        loop {
            while cursor < self.notifications.len() {
                let notification = self.notifications[cursor].clone();
                cursor += 1;
                if !received && inbox_receipt_matches(&notification, session_id, &message_id) {
                    received = true;
                }
                if received {
                    if notification.method == DSH_NOTIFICATION_SESSION_EVENT {
                        if let Some(text) =
                            assistant_text_from_session_event(&notification.params, session_id)
                        {
                            final_response = text;
                        }
                        if let Some(reason) =
                            finish_reason_from_session_event(&notification.params, session_id)?
                        {
                            finish_reason = Some(reason);
                        }
                    }
                    if session_idle_matches(&notification, session_id) {
                        if finish_reason.is_none() {
                            return Err(Error::external(
                                "DSH session became idle without a durable turn/end reason",
                            ));
                        }
                        return Ok(DshSdkRunResult {
                            session_id: session_id.to_string(),
                            message_id,
                            final_response,
                            finish_reason,
                            notifications: self.notifications[start..cursor].to_vec(),
                        });
                    }
                }
            }

            let incoming = self.read_frame_until(deadline, "prompt activity")?;
            if incoming.get("id").is_none() {
                if let Some(method) = incoming.get("method").and_then(Value::as_str) {
                    self.notifications.push(DshNotification {
                        method: method.to_string(),
                        params: incoming.get("params").cloned().unwrap_or(Value::Null),
                    });
                    continue;
                }
            }
            // The SDK currently sends no server->client requests. Fail closed
            // rather than silently ignoring a future permission/request surface.
            if incoming.get("id").is_some() && incoming.get("method").is_some() {
                return Err(Error::external(
                    "DSH SDK sent an unsupported server-to-client request",
                ));
            }
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        let protocol_result = self.request(DSH_METHOD_SHUTDOWN, None);
        // Match the official SDK client's ownership semantics: after the
        // shutdown response, close stdin to deliver EOF and allow the runtime
        // to flush persistence and exit on its own. Kill is only a bounded
        // containment fallback, never the normal successful path.
        self.stdin.take();

        if protocol_result.is_ok() {
            let deadline = Instant::now() + Duration::from_secs(6);
            loop {
                match self.child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if Instant::now() < deadline => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Ok(None) => {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        return Err(Error::external(
                            "DSH SDK acknowledged shutdown but did not exit before the grace deadline; process was force-reaped",
                        ));
                    }
                    Err(error) => {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        return Err(Error::external(format!(
                            "inspect DSH SDK exit after shutdown: {error}"
                        )));
                    }
                }
            }
        } else {
            // A transport that could not acknowledge shutdown is no longer
            // trusted to remain alive. Reap it immediately and preserve the
            // original protocol/transport error for the caller.
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        protocol_result.map(|_| ())
    }

    pub fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let mut frame = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
        });
        if let Some(params) = params {
            frame
                .as_object_mut()
                .expect("request frame is always object")
                .insert("params".to_string(), params);
        }
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| Error::external("DSH SDK stdin is closed"))?;
        serde_json::to_writer(&mut *stdin, &frame)
            .map_err(|error| Error::external(format!("encode DSH JSON-RPC: {error}")))?;
        stdin
            .write_all(b"\n")
            .and_then(|_| stdin.flush())
            .map_err(|error| Error::external(format!("write DSH JSON-RPC: {error}")))?;

        let deadline = Instant::now() + self.request_timeout;
        loop {
            let incoming = self.read_frame_until(deadline, method)?;
            require_jsonrpc_v2(&incoming)?;

            let has_id = incoming.get("id").is_some();
            let has_method = incoming.get("method").is_some();
            if has_id && has_method {
                return Err(Error::external(
                    "DSH SDK sent an unsupported server-to-client request",
                ));
            }

            if has_id {
                return response_result_for_id(&incoming, id, method);
            }

            if has_method {
                let notification_method = incoming
                    .get("method")
                    .and_then(Value::as_str)
                    .filter(|method| !method.trim().is_empty())
                    .ok_or_else(|| Error::external("DSH SDK notification method is empty"))?;
                self.notifications.push(DshNotification {
                    method: notification_method.to_string(),
                    params: incoming.get("params").cloned().unwrap_or(Value::Null),
                });
                continue;
            }

            return Err(Error::external(
                "DSH SDK emitted a JSON-RPC frame with neither id nor method",
            ));
        }
    }

    fn read_frame_until(&mut self, deadline: Instant, operation: &str) -> Result<Value> {
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "DSH SDK wire exceeded buffered frame capacity; runtime containment required",
            ));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::external(format!("DSH {operation} timed out")));
        }
        let incoming = self
            .incoming
            .lock()
            .map_err(|_| Error::internal("provider wire receiver lock poisoned"))?;
        let result = match incoming.recv_timeout(remaining) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(message)) => Err(Error::external(message)),
            Err(RecvTimeoutError::Timeout) => {
                Err(Error::external(format!("DSH {operation} timed out")))
            }
            Err(RecvTimeoutError::Disconnected) => {
                Err(Error::external("DSH SDK stdout reader disconnected"))
            }
        };
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "DSH SDK wire exceeded buffered frame capacity; runtime containment required",
            ));
        }
        result
    }
}

impl Drop for DshSdkStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        if let Some(policy_dir) = self.owned_policy_dir.take() {
            let _ = fs::remove_dir_all(policy_dir);
        }
    }
}

fn require_jsonrpc_v2(value: &Value) -> Result<()> {
    if value.get("jsonrpc").and_then(Value::as_str) == Some("2.0") {
        Ok(())
    } else {
        Err(Error::external(
            "DSH SDK emitted a frame without jsonrpc=\"2.0\"",
        ))
    }
}

fn response_result_for_id(value: &Value, expected_id: u64, operation: &str) -> Result<Value> {
    require_jsonrpc_v2(value)?;
    let response_id = value
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::external("DSH SDK response id must be an unsigned integer"))?;
    if response_id != expected_id {
        return Err(Error::external(format!(
            "DSH SDK response id mismatch: expected {expected_id}, received {response_id}"
        )));
    }
    let has_result = value.get("result").is_some();
    let has_error = value.get("error").is_some();
    if has_result == has_error {
        return Err(Error::external(
            "DSH SDK response must contain exactly one of result or error",
        ));
    }
    if let Some(error) = value.get("error") {
        return Err(Error::external(format!(
            "DSH JSON-RPC {operation} failed: {error}"
        )));
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| Error::external("DSH SDK response result disappeared"))
}

fn parse_server_info(result: &Value) -> Result<DshSdkServerInfo> {
    let info = result
        .get("serverInfo")
        .ok_or_else(|| Error::external("DSH initialize result missing serverInfo"))?;
    let name = info
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::external("DSH initialize result missing serverInfo.name"))?;
    let version = info
        .get("version")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::external("DSH initialize result missing serverInfo.version"))?;
    if name != DSH_SDK_SERVER_NAME {
        return Err(Error::external(format!(
            "unexpected DSH SDK server identity {name:?}"
        )));
    }
    Ok(DshSdkServerInfo {
        name: name.to_string(),
        version: version.to_string(),
    })
}

fn inbox_receipt_matches(
    notification: &DshNotification,
    session_id: &str,
    message_id: &str,
) -> bool {
    if notification.method != DSH_NOTIFICATION_SESSION_EVENT
        || notification.params.get("sessionId").and_then(Value::as_str) != Some(session_id)
    {
        return false;
    }
    let inserted = notification
        .params
        .get("event")
        .filter(|event| event.get("type").and_then(Value::as_str) == Some("agent/inbox/spliced"))
        .and_then(|event| event.get("data"))
        .and_then(|data| data.get("inserted"))
        .and_then(Value::as_array);
    inserted.is_some_and(|messages| {
        messages
            .iter()
            .any(|message| message.get("id").and_then(Value::as_str) == Some(message_id))
    })
}

fn session_idle_matches(notification: &DshNotification, session_id: &str) -> bool {
    notification.method == DSH_NOTIFICATION_SESSION_STATUS
        && notification.params.get("sessionId").and_then(Value::as_str) == Some(session_id)
        && notification.params.get("status").and_then(Value::as_str) == Some("idle")
}

fn finish_reason_from_session_event(params: &Value, session_id: &str) -> Result<Option<String>> {
    if params.get("sessionId").and_then(Value::as_str) != Some(session_id) {
        return Ok(None);
    }
    let event = match params.get("event") {
        Some(event) if event.get("type").and_then(Value::as_str) == Some("turn/end") => event,
        _ => return Ok(None),
    };
    let reason = event
        .get("data")
        .and_then(|data| data.get("reason"))
        .and_then(|reason| reason.get("kind"))
        .and_then(Value::as_str)
        .ok_or_else(|| Error::external("DSH turn/end is missing data.reason.kind"))?;
    Ok(Some(reason.to_string()))
}

/// Extract concatenated text from an upstream `assistant/message` session event.
pub fn assistant_text_from_session_event(params: &Value, session_id: &str) -> Option<String> {
    if params.get("sessionId").and_then(Value::as_str) != Some(session_id) {
        return None;
    }
    let event = params.get("event")?;
    if event.get("type").and_then(Value::as_str) != Some("assistant/message") {
        return None;
    }
    let blocks = event
        .get("data")?
        .get("message")?
        .get("content")?
        .as_array()?;
    let text: String = blocks
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect();
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_sdk_rejects_overlapping_workspace_and_harness_home() {
        let root = std::env::temp_dir().join("morn-dsh-boundary");
        let workspace = root.join("workspace");
        let nested_home = workspace.join(".dsh");
        let mut config = DshSdkConfig::profile_sdk(
            workspace.to_string_lossy(),
            "deepseek-official",
            "deepseek-v4-flash",
        )
        .with_dsh_home(nested_home.to_string_lossy())
        .with_execution_environment_ref("env://container/dsh")
        .with_runtime_identity(
            "fixture-runtime-1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(config.validate_for_real().is_err());

        config.dsh_home = Some(root.join("dsh-home").to_string_lossy().to_string());
        assert!(config.validate_for_real().is_ok());

        config.cwd = root.to_string_lossy().to_string();
        assert!(config.validate_for_real().is_err());
    }

    #[test]
    fn route_ref_changes_when_execution_route_changes_without_exposing_credentials() {
        let mut config = DshSdkConfig::profile_sdk("/tmp/work", "deepseek-official", "model-a");
        config.reasoning_effort = Some("high".to_string());
        let first = config.route_ref().unwrap();
        config.model = "model-b".to_string();
        let second = config.route_ref().unwrap();
        assert_ne!(first, second);
        assert!(first.contains("deepseek-official"));
        assert!(!first.contains("dsh_home"));
    }

    #[test]
    fn real_sdk_materializes_last_layer_no_direct_tools_policy_and_pins_it_in_route() {
        let root = std::env::temp_dir().join(format!("morn-dsh-policy-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("workspace")).unwrap();
        let mut config = DshSdkConfig::profile_sdk(
            root.join("workspace").to_string_lossy(),
            "deepseek-official",
            "deepseek-v4-flash",
        )
        .with_dsh_home(root.join("dsh-home").to_string_lossy())
        .with_execution_environment_ref("env://container/dsh")
        .with_runtime_identity(
            "fixture-runtime-1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(config.validate_for_real().is_ok());
        let (args, owned_policy_dir) = config.launch_args().unwrap();
        let patch_position = args.iter().position(|arg| arg == "--patch").unwrap();
        assert_eq!(
            patch_position + 2,
            args.len(),
            "Morn policy must be the final launcher overlay"
        );
        let policy_path = PathBuf::from(&args[patch_position + 1]);
        assert_eq!(
            owned_policy_dir.as_deref(),
            policy_path.parent(),
            "one-shot policy ownership must include the guard and overlay"
        );
        let policy = fs::read_to_string(&policy_path).unwrap();
        let guard = fs::read_to_string(
            policy_path
                .parent()
                .unwrap()
                .join(DSH_MORN_E0_TOOL_GUARD_FILE),
        )
        .unwrap();
        assert!(policy.contains("name: ./morn-e0-tool-guard.mjs"));
        assert!(policy.contains("mode: read-only"));
        assert!(guard.contains("ctx.tools.guard"));
        for required in [
            "id: tool-bash",
            "id: tool-pwsh",
            "id: tool-fs",
            "id: tool-subagent",
            "id: tool-web",
            "id: mcp-resources",
        ] {
            assert!(
                policy.contains(required),
                "{required} must be hidden before launch"
            );
        }
        let route = config.route_ref().unwrap();
        assert!(route.contains(DSH_MORN_E0_TOOL_POLICY_REF));
        assert!(route.contains("#sha256:"));
        assert!(route.contains(r#""max_tokens_as_success":false"#));
        assert!(route.contains(r#""telemetry":"disabled""#));

        config.args = vec!["--profile".to_string(), "web".to_string()];
        assert!(config.validate_for_real().is_err());
        config.args = vec![
            "--profile".to_string(),
            "sdk".to_string(),
            "--help".to_string(),
        ];
        assert!(config.validate_for_real().is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn production_policy_cannot_be_disabled_through_serialized_config() {
        let config = DshSdkConfig::profile_sdk("/tmp/work", "deepseek-official", "model-a");
        let mut encoded = serde_json::to_value(&config).unwrap();
        assert!(
            encoded.get("enforce_morn_e0_tool_policy").is_none(),
            "the policy toggle is not part of the public serialized contract"
        );
        encoded
            .as_object_mut()
            .unwrap()
            .insert("enforce_morn_e0_tool_policy".to_string(), json!(false));
        let decoded: DshSdkConfig = serde_json::from_value(encoded).unwrap();
        assert!(decoded.enforce_morn_e0_tool_policy);
        assert!(decoded
            .route_ref()
            .unwrap()
            .contains(DSH_MORN_E0_TOOL_POLICY_REF));
    }

    #[test]
    fn sdk_method_set_matches_current_upstream_boundary() {
        assert_eq!(DSH_METHOD_INITIALIZE, "initialize");
        assert_eq!(DSH_METHOD_SESSION_PROMPT, "session/prompt");
        assert_eq!(DSH_METHOD_SHUTDOWN, "shutdown");
        let notifications = [
            DSH_NOTIFICATION_SESSION_EVENT,
            DSH_NOTIFICATION_SESSION_STATUS,
            DSH_NOTIFICATION_SUBAGENT_STARTED,
            DSH_NOTIFICATION_SUBAGENT_FINISHED,
        ];
        assert_eq!(notifications.len(), 4);
    }

    #[test]
    fn jsonrpc_response_shape_fails_closed() {
        assert!(
            response_result_for_id(&json!({"jsonrpc":"1.0","id":1,"result":{}}), 1, "test")
                .is_err()
        );
        assert!(
            response_result_for_id(&json!({"jsonrpc":"2.0","id":2,"result":{}}), 1, "test")
                .is_err()
        );
        assert!(response_result_for_id(
            &json!({"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-1}}),
            1,
            "test"
        )
        .is_err());
        assert!(response_result_for_id(&json!({"jsonrpc":"2.0","id":1}), 1, "test").is_err());
        assert_eq!(
            response_result_for_id(
                &json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}}),
                1,
                "test"
            )
            .unwrap(),
            json!({"ok":true})
        );
    }

    #[test]
    fn sdk_server_identity_requires_wire_name_and_non_empty_version() {
        assert!(parse_server_info(&json!({
            "serverInfo":{"name":DSH_SDK_SERVER_NAME,"version":"0.0.1"}
        }))
        .is_ok());
        assert!(parse_server_info(&json!({
            "serverInfo":{"name":"lookalike-runtime","version":"0.0.1"}
        }))
        .is_err());
        assert!(parse_server_info(&json!({
            "serverInfo":{"name":DSH_SDK_SERVER_NAME,"version":""}
        }))
        .is_err());
    }

    #[test]
    fn identifies_owned_inbox_receipt_idle_and_finish_reason() {
        let receipt = DshNotification {
            method: DSH_NOTIFICATION_SESSION_EVENT.to_string(),
            params: json!({
                "sessionId":"session-1",
                "event":{
                    "type":"agent/inbox/spliced",
                    "data":{"inserted":[{"id":"message-1"}]}
                }
            }),
        };
        assert!(inbox_receipt_matches(&receipt, "session-1", "message-1"));
        assert!(!inbox_receipt_matches(&receipt, "session-2", "message-1"));

        let idle = DshNotification {
            method: DSH_NOTIFICATION_SESSION_STATUS.to_string(),
            params: json!({"sessionId":"session-1","status":"idle"}),
        };
        assert!(session_idle_matches(&idle, "session-1"));

        let end = json!({
            "sessionId":"session-1",
            "event":{"type":"turn/end","data":{"reason":{"kind":"completed"}}}
        });
        assert_eq!(
            finish_reason_from_session_event(&end, "session-1")
                .unwrap()
                .as_deref(),
            Some("completed")
        );
    }

    #[test]
    fn finish_reasons_distinguish_completed_from_non_successful_settlement() {
        let completed = DshSdkRunResult {
            session_id: "s".to_string(),
            message_id: "m".to_string(),
            final_response: "ok".to_string(),
            finish_reason: Some("completed".to_string()),
            notifications: vec![],
        };
        assert!(completed.completed_successfully());

        for reason in [
            "aborted",
            "blocked",
            "error",
            "max-tokens",
            "interrupted",
            "forked",
        ] {
            let run = DshSdkRunResult {
                finish_reason: Some(reason.to_string()),
                ..completed.clone()
            };
            assert!(
                !run.completed_successfully(),
                "{reason} must not be promoted as success"
            );
        }
        let missing = DshSdkRunResult {
            finish_reason: None,
            ..completed
        };
        assert!(!missing.completed_successfully());
    }

    #[test]
    fn sdk_subprocess_environment_is_explicitly_scrubbed() {
        let names = crate::subprocess_env::allowed_environment_names(Some(
            "CUSTOM_PROXY_TOKEN,MY_PROVIDER_KEY",
        ));
        for secret in [
            "GITHUB_TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            "DATABASE_URL",
            "OPENAI_API_KEY",
            "DEEPSEEK_API_KEY",
        ] {
            assert!(
                !names.contains(secret),
                "{secret} must require explicit passthrough"
            );
        }
        assert!(names.contains("PATH"));
        assert!(names.contains("CUSTOM_PROXY_TOKEN"));
        assert!(names.contains("MY_PROVIDER_KEY"));
    }

    #[test]
    fn live_config_requires_isolated_absolute_home() {
        let cwd = std::env::current_dir().unwrap();
        let config = DshSdkConfig::profile_sdk(
            cwd.to_string_lossy(),
            "deepseek-official",
            "deepseek-v4-flash",
        );
        assert!(config.validate_for_real().is_err());
        let root = std::env::temp_dir().join("morn-dsh-live-config");
        let configured = DshSdkConfig::profile_sdk(
            root.join("workspace").to_string_lossy(),
            "deepseek-official",
            "deepseek-v4-flash",
        )
        .with_dsh_home(root.join("dsh-home").to_string_lossy())
        .with_execution_environment_ref("env://container/dsh")
        .with_runtime_identity(
            "fixture-runtime-1",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        assert!(configured.validate_for_real().is_ok());
    }

    #[test]
    fn real_sdk_transport_completes_one_owned_turn_against_wire_fixture() {
        let executable = std::env::current_exe().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let config = DshSdkConfig {
            command: executable.to_string_lossy().to_string(),
            args: vec![
                "--exact".to_string(),
                "dsh_sdk::tests::fake_sdk_runtime".to_string(),
                "--ignored".to_string(),
                "--nocapture".to_string(),
            ],
            cwd: cwd.to_string_lossy().to_string(),
            provider: "fixture-provider".to_string(),
            model: "fixture-model".to_string(),
            reasoning_effort: None,
            max_tokens: Some(64),
            dsh_home: None,
            execution_environment_ref: None,
            runtime_version: None,
            runtime_digest: None,
            request_timeout_ms: 10_000,
            turn_timeout_ms: 10_000,
            enforce_morn_e0_tool_policy: false,
        };
        let mut client = DshSdkStdioClient::spawn(&config).unwrap();
        let initialized = client.initialize(&config).unwrap();
        assert_eq!(initialized.name, DSH_SDK_SERVER_NAME);
        assert_eq!(initialized.version, "0.0.1");

        let run = client.run_text_prompt("session-1", "hello").unwrap();
        assert_eq!(run.message_id, "message-1");
        assert_eq!(run.final_response, "hello from fake sdk");
        assert_eq!(run.finish_reason.as_deref(), Some("completed"));
        assert!(run.notifications.iter().any(|notification| {
            notification.method == DSH_NOTIFICATION_SESSION_STATUS
                && session_idle_matches(notification, "session-1")
        }));
        client.shutdown().unwrap();
        assert!(
            client.child.try_wait().unwrap().is_some(),
            "successful shutdown must reap the SDK child before Drop"
        );
        assert!(client.stdin.is_none(), "successful shutdown must close stdin");
    }

    /// A tiny protocol peer used only by the parent test above. It is ignored
    /// in normal test discovery and launched as a child test process.
    #[test]
    #[ignore]
    fn fake_sdk_runtime() {
        use std::io::BufRead as _;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().lock();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let request: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let Some(method) = request.get("method").and_then(Value::as_str) else {
                continue;
            };
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            let write = |stdout: &mut std::io::StdoutLock<'_>, value: Value| {
                serde_json::to_writer(&mut *stdout, &value).unwrap();
                stdout.write_all(b"\n").unwrap();
                stdout.flush().unwrap();
            };
            match method {
                DSH_METHOD_INITIALIZE => write(
                    &mut stdout,
                    json!({
                        "jsonrpc":"2.0",
                        "id":id,
                        "result":{
                            "serverInfo":{
                                "name":"deepseek-harness-sdk-runtime",
                                "version":"0.0.1"
                            }
                        }
                    }),
                ),
                DSH_METHOD_SESSION_PROMPT => {
                    let session_id = request
                        .get("params")
                        .and_then(|params| params.get("sessionId"))
                        .and_then(Value::as_str)
                        .unwrap_or("session-1");
                    let prompt_text = request
                        .get("params")
                        .and_then(|params| params.get("contentBlocks"))
                        .and_then(Value::as_array)
                        .and_then(|blocks| blocks.first())
                        .and_then(|block| block.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let finish_kind = if prompt_text == "__non_success__" {
                        "error"
                    } else {
                        "completed"
                    };
                    write(
                        &mut stdout,
                        json!({"jsonrpc":"2.0","id":id,"result":{"messageId":"message-1"}}),
                    );
                    write(
                        &mut stdout,
                        json!({
                            "jsonrpc":"2.0",
                            "method":"session.event",
                            "params":{
                                "sessionId":session_id,
                                "event":{
                                    "type":"agent/inbox/spliced",
                                    "data":{"inserted":[{"id":"message-1"}]}
                                }
                            }
                        }),
                    );
                    if prompt_text == "__hang__" {
                        // Parent timeout/containment tests intentionally leave this
                        // owned turn unsettled until the subprocess is reaped.
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        continue;
                    }
                    if prompt_text == "__tool_activity__" {
                        write(
                            &mut stdout,
                            json!({
                                "jsonrpc":"2.0",
                                "method":"session.event",
                                "params":{
                                    "sessionId":session_id,
                                    "event":{
                                        "type":"tool/call",
                                        "data":{"callId":"call-1","name":"bash","arguments":"{}"}
                                    }
                                }
                            }),
                        );
                        write(
                            &mut stdout,
                            json!({
                                "jsonrpc":"2.0",
                                "method":"session.event",
                                "params":{
                                    "sessionId":session_id,
                                    "event":{
                                        "type":"tool/result",
                                        "data":{
                                            "callId":"call-1",
                                            "message":{"toolCallId":"call-1","isError":false}
                                        }
                                    }
                                }
                            }),
                        );
                    }
                    write(
                        &mut stdout,
                        json!({
                            "jsonrpc":"2.0",
                            "method":"session.event",
                            "params":{
                                "sessionId":session_id,
                                "event":{
                                    "type":"assistant/message",
                                    "data":{"message":{"content":[
                                        {"type":"text","text":"hello from fake sdk"}
                                    ]}}
                                }
                            }
                        }),
                    );
                    write(
                        &mut stdout,
                        json!({
                            "jsonrpc":"2.0",
                            "method":"session.event",
                            "params":{
                                "sessionId":session_id,
                                "event":{
                                    "type":"turn/end",
                                    "data":{"reason":{"kind":finish_kind}}
                                }
                            }
                        }),
                    );
                    write(
                        &mut stdout,
                        json!({
                            "jsonrpc":"2.0",
                            "method":"session.status",
                            "params":{"sessionId":session_id,"status":"idle"}
                        }),
                    );
                }
                DSH_METHOD_SHUTDOWN => {
                    write(&mut stdout, json!({"jsonrpc":"2.0","id":id,"result":{}}));
                    break;
                }
                _ => write(
                    &mut stdout,
                    json!({
                        "jsonrpc":"2.0",
                        "id":id,
                        "error":{"code":-32601,"message":"method not found"}
                    }),
                ),
            }
        }
    }

    #[test]
    fn extracts_assistant_text_without_treating_session_as_work_truth() {
        let notification = json!({
            "sessionId":"dsh-1",
            "event":{
                "type":"assistant/message",
                "data":{
                    "message":{
                        "content":[
                            {"type":"text","text":"hello "},
                            {"type":"reasoning","text":"hidden"},
                            {"type":"text","text":"world"}
                        ]
                    }
                }
            }
        });
        assert_eq!(
            assistant_text_from_session_event(&notification, "dsh-1").as_deref(),
            Some("hello world")
        );
    }
}
