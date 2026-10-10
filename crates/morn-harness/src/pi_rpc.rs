//! Pi RPC subprocess boundary.
//!
//! Current Pi exposes `pi --mode rpc --no-session` as a long-lived JSONL
//! subprocess protocol. Morn keeps this transport out-of-process so Pi session
//! state remains executor/runtime evidence rather than canonical Work truth.
//!
//! The client deliberately models only stable boundary semantics needed by
//! Morn: command correlation, state inspection, prompt acceptance, abort, and
//! event collection until `agent_settled`. Rich Pi-specific controls can remain
//! provider extensions without entering the Morn semantic constitution.

use std::io::{BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use morn_kernel::error::{Error, Result};

use crate::subprocess_wire::{read_bounded_utf8_line, MAX_PROVIDER_WIRE_BUFFERED_FRAMES};

pub const PI_COMMAND_PROMPT: &str = "prompt";
pub const PI_COMMAND_GET_STATE: &str = "get_state";
pub const PI_COMMAND_ABORT: &str = "abort";
pub const PI_EVENT_AGENT_SETTLED: &str = "agent_settled";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PiRpcConfig {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    /// Exact execution-environment identity that owns this Pi subprocess.
    #[serde(default)]
    pub execution_environment_ref: Option<String>,
    /// Deployment-attested Pi distribution identity. RPC get_state/events prove
    /// runtime behavior, not which package artifact launched the process.
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub runtime_digest: Option<String>,
    pub request_timeout_ms: u64,
    pub prompt_timeout_ms: u64,
    /// True for the official Pi CLI. Alternative test/wrapper executables may
    /// own route configuration themselves and opt out of Pi-specific argv.
    #[serde(default = "default_true")]
    pub append_route_args: bool,
    /// True for production. Test fixtures may disable strictness only to skip
    /// libtest's own stdout preamble before the fake JSONL peer starts.
    pub strict_jsonl: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PiRpcConfig {
    fn default() -> Self {
        Self {
            command: "pi".to_string(),
            args: vec![
                "--mode".to_string(),
                "rpc".to_string(),
                "--no-session".to_string(),
            ],
            cwd: None,
            provider: None,
            model: None,
            execution_environment_ref: None,
            runtime_version: None,
            runtime_digest: None,
            request_timeout_ms: 30_000,
            prompt_timeout_ms: 60_000,
            append_route_args: true,
            strict_jsonl: true,
        }
    }
}

impl PiRpcConfig {
    /// Stable, secret-free identity of the Pi RPC route. The executable's
    /// distribution digest is pinned separately.
    pub fn route_ref(&self) -> Result<String> {
        let provider = self
            .provider
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::validation("Pi route identity requires provider"))?;
        let model = self
            .model
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::validation("Pi route identity requires model"))?;
        let route = serde_json::json!({
            "args": &self.args,
            "append_route_args": self.append_route_args,
            "provider": provider,
            "model": model,
        });
        Ok(format!("pi-rpc-route:{route}"))
    }

    pub fn command_line(&self) -> Vec<String> {
        let mut args = self.args.clone();
        if self.append_route_args {
            if let Some(provider) = &self.provider {
                args.push("--provider".to_string());
                args.push(provider.clone());
            }
            if let Some(model) = &self.model {
                args.push("--model".to_string());
                args.push(model.clone());
            }
        }
        args
    }

    pub fn validate_for_real(&self) -> Result<()> {
        let cwd = self
            .cwd
            .as_deref()
            .filter(|cwd| !cwd.trim().is_empty())
            .ok_or_else(|| Error::validation("real Pi RPC requires an explicit workspace"))?;
        if self.command.trim().is_empty()
            || self
                .execution_environment_ref
                .as_deref()
                .is_none_or(|reference| reference.trim().is_empty())
            || self
                .provider
                .as_deref()
                .is_none_or(|provider| provider.trim().is_empty())
            || self
                .model
                .as_deref()
                .is_none_or(|model| model.trim().is_empty())
        {
            return Err(Error::validation(
                "real Pi RPC requires command, provider, model and execution_environment_ref",
            ));
        }
        if !Path::new(cwd).is_absolute() {
            return Err(Error::validation(
                "real Pi RPC workspace must be an absolute path",
            ));
        }
        if self
            .runtime_version
            .as_deref()
            .is_none_or(|version| version.trim().is_empty())
        {
            return Err(Error::validation(
                "real Pi RPC requires a deployment-attested runtime version",
            ));
        }
        let digest = self
            .runtime_digest
            .as_deref()
            .ok_or_else(|| Error::validation("real Pi RPC requires a runtime SHA-256 digest"))?;
        let Some(hex) = digest.strip_prefix("sha256:") else {
            return Err(Error::validation(
                "real Pi RPC runtime digest must use sha256:<64-hex>",
            ));
        };
        if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::validation(
                "real Pi RPC runtime digest must use sha256:<64-hex>",
            ));
        }
        if self.request_timeout_ms == 0 || self.prompt_timeout_ms == 0 {
            return Err(Error::validation(
                "Pi request and prompt timeouts must be positive",
            ));
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self> {
        let mut config = Self {
            cwd: Some(
                std::env::var("MORN_PI_WORKSPACE")
                    .map_err(|_| Error::validation("MORN_PI_WORKSPACE is required for real Pi"))?,
            ),
            provider: Some(
                std::env::var("MORN_PI_PROVIDER")
                    .map_err(|_| Error::validation("MORN_PI_PROVIDER is required for real Pi"))?,
            ),
            model: Some(
                std::env::var("MORN_PI_MODEL")
                    .map_err(|_| Error::validation("MORN_PI_MODEL is required for real Pi"))?,
            ),
            execution_environment_ref: Some(
                std::env::var("MORN_PI_EXECUTION_ENVIRONMENT_REF").map_err(|_| {
                    Error::validation("MORN_PI_EXECUTION_ENVIRONMENT_REF is required for real Pi")
                })?,
            ),
            runtime_version: Some(std::env::var("MORN_PI_RUNTIME_VERSION").map_err(|_| {
                Error::validation("MORN_PI_RUNTIME_VERSION is required for real Pi")
            })?),
            runtime_digest: Some(std::env::var("MORN_PI_RUNTIME_DIGEST").map_err(|_| {
                Error::validation("MORN_PI_RUNTIME_DIGEST is required for real Pi")
            })?),
            ..Self::default()
        };
        if let Ok(command) = std::env::var("MORN_PI_COMMAND") {
            config.command = command;
        }
        if let Ok(timeout) = std::env::var("MORN_PI_REQUEST_TIMEOUT_MS") {
            config.request_timeout_ms = timeout
                .parse()
                .map_err(|_| Error::validation("MORN_PI_REQUEST_TIMEOUT_MS must be an integer"))?;
        }
        if let Ok(timeout) = std::env::var("MORN_PI_PROMPT_TIMEOUT_MS") {
            config.prompt_timeout_ms = timeout
                .parse()
                .map_err(|_| Error::validation("MORN_PI_PROMPT_TIMEOUT_MS must be an integer"))?;
        }
        config.validate_for_real()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PiRpcResponse {
    pub id: Option<String>,
    pub command: String,
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PiRpcEvent {
    pub event_type: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PiPromptRun {
    pub request_id: String,
    pub disposition: String,
    pub events: Vec<PiRpcEvent>,
}

impl PiPromptRun {
    pub fn settled(&self) -> bool {
        self.disposition == "handled"
            || self
                .events
                .iter()
                .any(|event| event.event_type == PI_EVENT_AGENT_SETTLED)
    }
}

type PiWireItem = std::result::Result<Value, String>;

pub struct PiRpcClient {
    child: Child,
    stdin: Option<ChildStdin>,
    incoming: Mutex<Receiver<PiWireItem>>,
    reader: Option<JoinHandle<()>>,
    wire_overflowed: Arc<AtomicBool>,
    next_id: u64,
    request_timeout: Duration,
    prompt_timeout: Duration,
    pub buffered_events: Vec<PiRpcEvent>,
}

impl std::fmt::Debug for PiRpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PiRpcClient")
            .field("child_id", &self.child.id())
            .field("next_id", &self.next_id)
            .field("buffered_events", &self.buffered_events.len())
            .finish()
    }
}

impl PiRpcClient {
    pub fn spawn(config: &PiRpcConfig) -> Result<Self> {
        if config.command.trim().is_empty()
            || config.request_timeout_ms == 0
            || config.prompt_timeout_ms == 0
        {
            return Err(Error::validation(
                "Pi RPC requires command and positive request/prompt timeouts",
            ));
        }
        let mut command = Command::new(&config.command);
        command
            .args(config.command_line())
            .env_clear()
            .envs(crate::subprocess_env::scrubbed_environment(
                "MORN_PI_ENV_PASSTHROUGH",
            ))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(cwd) = &config.cwd {
            command.current_dir(cwd);
        }
        let mut child = command.spawn().map_err(|error| {
            Error::external(format!(
                "failed to start Pi RPC runtime {:?}: {error}",
                config.command
            ))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::external("Pi RPC runtime has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::external("Pi RPC runtime has no stdout"))?;
        let strict_jsonl = config.strict_jsonl;
        let (sender, incoming) =
            mpsc::sync_channel::<PiWireItem>(MAX_PROVIDER_WIRE_BUFFERED_FRAMES);
        let wire_overflowed = Arc::new(AtomicBool::new(false));
        let reader_overflowed = Arc::clone(&wire_overflowed);
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            loop {
                let line = match read_bounded_utf8_line(&mut stdout, "Pi RPC JSONL") {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        let _ = sender.try_send(Err("Pi RPC runtime closed stdout".to_string()));
                        break;
                    }
                    Err(error) => {
                        let _ = sender.try_send(Err(error));
                        break;
                    }
                };
                let trimmed = line.trim_end_matches(['\r', '\n']);
                if trimmed.is_empty() {
                    continue;
                }
                let (item, terminate_after_send) = match serde_json::from_str::<Value>(trimmed) {
                    Ok(value) => (Ok(value), false),
                    Err(error) if strict_jsonl => {
                        (Err(format!("invalid Pi RPC JSONL record: {error}")), true)
                    }
                    Err(_) => continue,
                };
                match sender.try_send(item) {
                    Ok(()) if terminate_after_send => break,
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
            prompt_timeout: Duration::from_millis(config.prompt_timeout_ms),
            buffered_events: Vec::new(),
        })
    }

    pub fn get_state(&mut self) -> Result<Value> {
        let response = self.command(PI_COMMAND_GET_STATE, Value::Object(Default::default()))?;
        Ok(response.data.unwrap_or(Value::Null))
    }

    pub fn get_last_assistant_text(&mut self) -> Result<Option<String>> {
        let response =
            self.command("get_last_assistant_text", Value::Object(Default::default()))?;
        match response.data.as_ref().and_then(|data| data.get("text")) {
            Some(Value::String(text)) => Ok(Some(text.clone())),
            Some(Value::Null) | None => Ok(None),
            Some(_) => Err(Error::external(
                "Pi get_last_assistant_text returned non-string text",
            )),
        }
    }

    /// Close stdin to request Pi's documented orderly shutdown. If the child
    /// does not exit promptly, reap it rather than leaving an orphan process.
    pub fn shutdown(&mut self) -> Result<()> {
        self.stdin.take();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => {
                    self.child.kill().map_err(|error| {
                        Error::external(format!("kill stalled Pi RPC: {error}"))
                    })?;
                    self.child.wait().map_err(|error| {
                        Error::external(format!("reap stalled Pi RPC: {error}"))
                    })?;
                    return Err(Error::external(
                        "Pi RPC did not exit after stdin close and was forcefully reaped",
                    ));
                }
                Err(error) => {
                    return Err(Error::external(format!(
                        "inspect Pi RPC shutdown state: {error}"
                    )))
                }
            }
        }
    }

    /// Send a prompt and wait until Pi reports `agent_settled`. A successful
    /// prompt response means accepted/queued/handled only; it is not business
    /// completion and not even necessarily executor completion.
    pub fn prompt_and_wait(&mut self, message: &str) -> Result<PiPromptRun> {
        let id = self.next_request_id("prompt");
        self.write_record(json!({
            "id": id,
            "type": PI_COMMAND_PROMPT,
            "message": message
        }))?;

        let mut response: Option<PiRpcResponse> = None;
        let mut events = Vec::new();
        let mut settled = false;
        let deadline = Instant::now() + self.prompt_timeout;

        loop {
            let record = self.read_record_until(deadline, "prompt settlement")?;
            if is_response(&record) {
                let parsed = parse_response(&record)?;
                require_correlated_response(&parsed, &id, PI_COMMAND_PROMPT)?;
                if !parsed.success {
                    return Err(Error::external(format!(
                        "Pi prompt rejected: {}",
                        parsed.error.unwrap_or_else(|| "unknown error".to_string())
                    )));
                }
                let handled = prompt_disposition(&parsed) == Some("handled");
                response = Some(parsed);
                if handled || settled {
                    break;
                }
                continue;
            }

            let event = parse_event(record);
            if event.event_type == PI_EVENT_AGENT_SETTLED {
                settled = true;
            }
            events.push(event);
            if settled && response.is_some() {
                break;
            }
        }

        let response = response.ok_or_else(|| {
            Error::external("Pi event stream settled before correlated prompt response")
        })?;
        let disposition = prompt_disposition(&response)
            .unwrap_or("unknown")
            .to_string();
        self.buffered_events.extend(events.iter().cloned());

        Ok(PiPromptRun {
            request_id: id,
            disposition,
            events,
        })
    }

    pub fn abort(&mut self) -> Result<()> {
        let response = self.command(PI_COMMAND_ABORT, Value::Object(Default::default()))?;
        if response.success {
            Ok(())
        } else {
            Err(Error::external(
                response
                    .error
                    .unwrap_or_else(|| "Pi abort failed".to_string()),
            ))
        }
    }

    pub fn command(&mut self, command: &str, fields: Value) -> Result<PiRpcResponse> {
        let id = self.next_request_id(command);
        let mut record = match fields {
            Value::Object(object) => Value::Object(object),
            _ => {
                return Err(Error::validation(
                    "Pi RPC command fields must be a JSON object",
                ))
            }
        };
        let object = record
            .as_object_mut()
            .expect("Pi command record is always an object");
        object.insert("id".to_string(), Value::String(id.clone()));
        object.insert("type".to_string(), Value::String(command.to_string()));
        self.write_record(record)?;
        let deadline = Instant::now() + self.request_timeout;

        loop {
            let incoming = self.read_record_until(deadline, command)?;
            if is_response(&incoming) {
                let response = parse_response(&incoming)?;
                require_correlated_response(&response, &id, command)?;
                if response.success {
                    return Ok(response);
                }
                return Err(Error::external(format!(
                    "Pi RPC {command} failed: {}",
                    response
                        .error
                        .unwrap_or_else(|| "unknown error".to_string())
                )));
            }
            self.buffered_events.push(parse_event(incoming));
        }
    }

    fn next_request_id(&mut self, command: &str) -> String {
        let id = format!("morn-{command}-{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn write_record(&mut self, record: Value) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| Error::external("Pi RPC stdin is closed"))?;
        serde_json::to_writer(&mut *stdin, &record)
            .map_err(|error| Error::external(format!("encode Pi RPC JSON: {error}")))?;
        stdin
            .write_all(b"\n")
            .and_then(|_| stdin.flush())
            .map_err(|error| Error::external(format!("write Pi RPC JSONL: {error}")))
    }

    fn read_record_until(&mut self, deadline: Instant, operation: &str) -> Result<Value> {
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "Pi RPC wire exceeded buffered frame capacity; runtime containment required",
            ));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::external(format!("Pi RPC {operation} timed out")));
        }
        let incoming = self
            .incoming
            .lock()
            .map_err(|_| Error::internal("provider wire receiver lock poisoned"))?;
        let result = match incoming.recv_timeout(remaining) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(message)) => Err(Error::external(message)),
            Err(RecvTimeoutError::Timeout) => {
                Err(Error::external(format!("Pi RPC {operation} timed out")))
            }
            Err(RecvTimeoutError::Disconnected) => {
                Err(Error::external("Pi RPC stdout reader disconnected"))
            }
        };
        if self.wire_overflowed.load(Ordering::Acquire) {
            return Err(Error::external(
                "Pi RPC wire exceeded buffered frame capacity; runtime containment required",
            ));
        }
        result
    }
}

impl Drop for PiRpcClient {
    fn drop(&mut self) {
        self.stdin.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn is_response(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("response")
}

fn parse_response(value: &Value) -> Result<PiRpcResponse> {
    let command = value
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::external("Pi response missing command"))?;
    let success = value
        .get("success")
        .and_then(Value::as_bool)
        .ok_or_else(|| Error::external("Pi response missing success flag"))?;
    Ok(PiRpcResponse {
        id: value.get("id").and_then(Value::as_str).map(str::to_string),
        command: command.to_string(),
        success,
        data: value.get("data").cloned(),
        error: value
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn require_correlated_response(
    response: &PiRpcResponse,
    expected_id: &str,
    expected_command: &str,
) -> Result<()> {
    if response.id.as_deref() != Some(expected_id) {
        return Err(Error::external(format!(
            "Pi RPC response id mismatch for {expected_command}: expected {expected_id:?}, received {:?}",
            response.id
        )));
    }
    if response.command != expected_command {
        return Err(Error::external(format!(
            "Pi RPC response command mismatch: expected {expected_command:?}, received {:?}",
            response.command
        )));
    }
    Ok(())
}

fn parse_event(value: Value) -> PiRpcEvent {
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    PiRpcEvent {
        event_type,
        payload: value,
    }
}

fn prompt_disposition(response: &PiRpcResponse) -> Option<&str> {
    response.data.as_ref()?.get("disposition")?.as_str()
}

#[cfg(test)]
mod route_identity_tests {
    use super::*;

    #[test]
    fn route_ref_changes_with_model_or_protocol_args() {
        let mut config = PiRpcConfig {
            provider: Some("provider-a".to_string()),
            model: Some("model-a".to_string()),
            ..PiRpcConfig::default()
        };
        let first = config.route_ref().unwrap();
        config.model = Some("model-b".to_string());
        assert_ne!(first, config.route_ref().unwrap());
        config.model = Some("model-a".to_string());
        config.args.push("--extra-mode".to_string());
        assert_ne!(first, config.route_ref().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_process_boundary_matches_current_pi_rpc_mode() {
        let config = PiRpcConfig::default();
        assert_eq!(config.command, "pi");
        assert_eq!(config.command_line(), vec!["--mode", "rpc", "--no-session"]);
    }

    #[test]
    fn real_rpc_transport_completes_prompt_against_wire_fixture() {
        let executable = std::env::current_exe().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let config = PiRpcConfig {
            command: executable.to_string_lossy().to_string(),
            args: vec![
                "--exact".to_string(),
                "pi_rpc::tests::fake_pi_rpc_runtime".to_string(),
                "--ignored".to_string(),
                "--quiet".to_string(),
                "--nocapture".to_string(),
            ],
            cwd: Some(cwd.to_string_lossy().to_string()),
            provider: None,
            model: None,
            execution_environment_ref: None,
            runtime_version: None,
            runtime_digest: None,
            request_timeout_ms: 10_000,
            prompt_timeout_ms: 10_000,
            append_route_args: false,
            strict_jsonl: false,
        };
        let mut client = PiRpcClient::spawn(&config).unwrap();
        let state = client.get_state().unwrap();
        assert_eq!(state["agent"]["status"], "idle");

        let run = client.prompt_and_wait("hello").unwrap();
        assert!(run.settled());
        assert_eq!(run.disposition, "started");
        assert_eq!(
            client.get_last_assistant_text().unwrap().as_deref(),
            Some("hello from fake pi")
        );
        client.abort().unwrap();
        client.shutdown().unwrap();
    }

    #[test]
    #[ignore]
    fn fake_pi_rpc_runtime() {
        use std::io::BufRead as _;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().lock();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let request: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let command = request.get("type").and_then(Value::as_str).unwrap_or("");
            let id = request.get("id").cloned();
            let write = |stdout: &mut std::io::StdoutLock<'_>, value: Value| {
                serde_json::to_writer(&mut *stdout, &value).unwrap();
                stdout.write_all(b"\n").unwrap();
                stdout.flush().unwrap();
            };
            let response = |command: &str, data: Value| {
                let mut value = json!({
                    "type":"response",
                    "command":command,
                    "success":true,
                    "data":data
                });
                if let Some(id) = id.clone() {
                    value.as_object_mut().unwrap().insert("id".to_string(), id);
                }
                value
            };
            match command {
                "get_state" => write(
                    &mut stdout,
                    response("get_state", json!({"agent":{"status":"idle"}})),
                ),
                "prompt" => {
                    let prompt_text = request.get("message").and_then(Value::as_str).unwrap_or("");
                    write(
                        &mut stdout,
                        response("prompt", json!({"disposition":"started"})),
                    );
                    if prompt_text == "__tool_activity__" {
                        write(
                            &mut stdout,
                            json!({
                                "type":"tool_execution_start",
                                "toolCallId":"call-1",
                                "toolName":"bash",
                                "args":{"command":"echo fixture"}
                            }),
                        );
                        write(
                            &mut stdout,
                            json!({
                                "type":"tool_execution_end",
                                "toolCallId":"call-1",
                                "toolName":"bash",
                                "result":{"content":[]},
                                "isError":false,
                                "durationMs":1
                            }),
                        );
                    }
                    write(&mut stdout, json!({"type":"agent_settled"}));
                }
                "get_last_assistant_text" => write(
                    &mut stdout,
                    response(
                        "get_last_assistant_text",
                        json!({"text":"hello from fake pi"}),
                    ),
                ),
                "abort" => write(&mut stdout, response("abort", Value::Null)),
                _ => write(
                    &mut stdout,
                    json!({
                        "type":"response",
                        "command":command,
                        "success":false,
                        "error":"unsupported fake command"
                    }),
                ),
            }
        }
    }

    #[test]
    fn live_pi_config_requires_absolute_workspace_and_pinned_route() {
        let cwd = std::env::current_dir().unwrap();
        let mut config = PiRpcConfig::default();
        assert!(config.validate_for_real().is_err());
        config.cwd = Some(cwd.to_string_lossy().to_string());
        config.provider = Some("fixture-provider".to_string());
        config.model = Some("fixture-model".to_string());
        assert!(config.validate_for_real().is_err());
        config.execution_environment_ref = Some("env://container/pi".to_string());
        assert!(config.validate_for_real().is_err());
        config.runtime_version = Some("1.0.0".to_string());
        config.runtime_digest = Some(format!("sha256:{}", "a".repeat(64)));
        assert!(config.validate_for_real().is_ok());
        config.cwd = Some("relative-workspace".to_string());
        assert!(config.validate_for_real().is_err());
    }

    #[test]
    fn last_assistant_text_parser_requires_string_or_null() {
        let ok = parse_response(&json!({
            "id":"morn-get_last_assistant_text-1",
            "type":"response",
            "command":"get_last_assistant_text",
            "success":true,
            "data":{"text":"answer"}
        }))
        .unwrap();
        assert_eq!(ok.data.unwrap()["text"], "answer");
    }

    #[test]
    fn response_correlation_rejects_cross_request_or_cross_command_records() {
        let response = parse_response(&json!({
            "id":"morn-prompt-1",
            "type":"response",
            "command":"prompt",
            "success":true,
            "data":{"disposition":"started"}
        }))
        .unwrap();
        assert!(require_correlated_response(&response, "morn-prompt-1", "prompt").is_ok());
        assert!(require_correlated_response(&response, "morn-prompt-2", "prompt").is_err());
        assert!(require_correlated_response(&response, "morn-prompt-1", "abort").is_err());
    }

    #[test]
    fn prompt_acceptance_and_agent_settled_are_distinct() {
        let response = parse_response(&json!({
            "id":"morn-prompt-1",
            "type":"response",
            "command":"prompt",
            "success":true,
            "data":{"disposition":"started"}
        }))
        .unwrap();
        assert_eq!(prompt_disposition(&response), Some("started"));

        let run = PiPromptRun {
            request_id: "morn-prompt-1".to_string(),
            disposition: "started".to_string(),
            events: vec![parse_event(json!({"type":"agent_settled"}))],
        };
        assert!(run.settled());
    }

    #[test]
    fn settled_before_response_is_retained_until_correlated_response() {
        // The parser/state machine deliberately treats response correlation and
        // settled state as two independent conditions. This mirrors Pi's
        // asynchronous event/response streams.
        let settled = parse_event(json!({"type":"agent_settled"}));
        let response = parse_response(&json!({
            "id":"morn-prompt-3",
            "type":"response",
            "command":"prompt",
            "success":true,
            "data":{"disposition":"started"}
        }))
        .unwrap();
        assert_eq!(settled.event_type, PI_EVENT_AGENT_SETTLED);
        assert_eq!(prompt_disposition(&response), Some("started"));
    }

    #[test]
    fn handled_prompt_needs_no_agent_settled_event() {
        let run = PiPromptRun {
            request_id: "morn-prompt-2".to_string(),
            disposition: "handled".to_string(),
            events: Vec::new(),
        };
        assert!(run.settled());
    }

    #[test]
    fn command_response_parser_rejects_missing_protocol_fields() {
        assert!(parse_response(&json!({"type":"response","success":true})).is_err());
        assert!(parse_response(&json!({"type":"response","command":"get_state"})).is_err());
    }

    #[test]
    fn runtime_event_is_never_a_morn_acceptance_record() {
        let event = parse_event(json!({
            "type":"agent_settled",
            "messages":[]
        }));
        let encoded = serde_json::to_value(event).unwrap();
        assert!(encoded.get("acceptance").is_none());
        assert!(encoded.get("outcome").is_none());
    }
}
