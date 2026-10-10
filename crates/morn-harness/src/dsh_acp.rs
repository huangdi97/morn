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

use morn_kernel::error::{Error, Result};

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
            request_timeout_ms: 30_000,
            prompt_timeout_ms: 300_000,
            enforce_morn_e0_tool_policy: true,
            #[cfg(test)]
            test_env: BTreeMap::new(),
        }
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
    fn allow_protocol_fixture_transport(&mut self) {
        self.enforce_morn_e0_tool_policy = false;
    }

    #[cfg(test)]
    fn with_test_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DshAcpPromptResult {
    pub session_id: String,
    pub stop_reason: String,
    pub assistant_text: String,
}

type AcpWireItem = std::result::Result<Value, String>;

#[derive(Clone)]
pub struct DshAcpControlHandle {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
}

impl std::fmt::Debug for DshAcpControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DshAcpControlHandle")
            .finish_non_exhaustive()
    }
}

impl DshAcpControlHandle {
    /// Cancel is the only ACP lifecycle action intentionally exposed on this
    /// cloneable handle. It is a notification, so it can be written while the
    /// owning execution thread is blocked awaiting session/prompt settlement.
    pub fn cancel_session(&self, session_id: &str) -> Result<()> {
        if session_id.trim().is_empty() {
            return Err(Error::validation(
                "DSH ACP cancel requires a non-empty session id",
            ));
        }
        write_shared_frame(
            &self.stdin,
            json!({
                "jsonrpc":"2.0",
                "method":DSH_ACP_METHOD_SESSION_CANCEL,
                "params":{"sessionId":session_id}
            }),
        )
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

impl DshAcpStdioClient {
    pub fn spawn(config: &DshAcpConfig) -> Result<Self> {
        config.validate()?;
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
        command
            .args(&launch_args)
            .current_dir(&config.cwd)
            .env_clear()
            .envs(crate::subprocess_env::scrubbed_environment(
                "MORN_DSH_ACP_ENV_PASSTHROUGH",
            ))
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

    pub fn control_handle(&self) -> DshAcpControlHandle {
        DshAcpControlHandle {
            stdin: Arc::clone(&self.stdin),
        }
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

    pub fn cancel_session(&mut self, session_id: &str) -> Result<()> {
        self.control_handle().cancel_session(session_id)
    }

    pub fn prompt_text(&mut self, session_id: &str, text: &str) -> Result<DshAcpPromptResult> {
        if session_id.trim().is_empty() || text.trim().is_empty() {
            return Err(Error::validation(
                "DSH ACP prompt requires non-empty session id and text",
            ));
        }
        let start = self.notifications.len();
        let result = self.request(
            DSH_ACP_METHOD_SESSION_PROMPT,
            Some(json!({
                "sessionId":session_id,
                "prompt":[{"type":"text","text":text}]
            })),
            self.prompt_timeout,
        )?;
        let stop_reason = required_string(&result, "stopReason", "DSH ACP prompt result")?;
        let updates = &self.notifications[start..];
        if updates.iter().any(acp_update_reports_tool_activity) {
            return Err(Error::external(
                "DSH ACP reported tool activity on a transport that has no Morn authority grant",
            ));
        }
        Ok(DshAcpPromptResult {
            session_id: session_id.to_string(),
            stop_reason,
            assistant_text: assistant_text_from_acp_updates(updates, session_id),
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
                self.notifications.push(DshAcpNotification {
                    method: method.to_string(),
                    params: incoming.get("params").cloned().unwrap_or(Value::Null),
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
                    writeln!(
                        output,
                        "{}",
                        json!({
                            "jsonrpc":"2.0",
                            "id":id,
                            "result":{"configOptions":[{"id":"reasoning_effort","currentValue":"high"}]}
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
        let control = client.control_handle();
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
        control.cancel_session(&session).unwrap();

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
        let result = client.prompt_text(&session, "hello").unwrap();
        assert_eq!(result.stop_reason, "end_turn");
        assert_eq!(result.assistant_text, "hello ACP");
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
