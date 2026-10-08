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

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use morn_kernel::error::{Error, Result};

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
        }
    }
}

impl PiRpcConfig {
    pub fn command_line(&self) -> Vec<String> {
        let mut args = self.args.clone();
        if let Some(provider) = &self.provider {
            args.push("--provider".to_string());
            args.push(provider.clone());
        }
        if let Some(model) = &self.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        args
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

pub struct PiRpcClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
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
        let mut command = Command::new(&config.command);
        command
            .args(config.command_line())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
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

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            buffered_events: Vec::new(),
        })
    }

    pub fn get_state(&mut self) -> Result<Value> {
        let response = self.command(PI_COMMAND_GET_STATE, Value::Object(Default::default()))?;
        Ok(response.data.unwrap_or(Value::Null))
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

        loop {
            let record = self.read_record()?;
            if is_response(&record) {
                let parsed = parse_response(&record)?;
                if parsed.id.as_deref() == Some(id.as_str()) {
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
            _ => return Err(Error::validation("Pi RPC command fields must be a JSON object")),
        };
        let object = record
            .as_object_mut()
            .expect("Pi command record is always an object");
        object.insert("id".to_string(), Value::String(id.clone()));
        object.insert("type".to_string(), Value::String(command.to_string()));
        self.write_record(record)?;

        loop {
            let incoming = self.read_record()?;
            if is_response(&incoming) {
                let response = parse_response(&incoming)?;
                if response.id.as_deref() == Some(id.as_str()) {
                    if response.success {
                        return Ok(response);
                    }
                    return Err(Error::external(format!(
                        "Pi RPC {command} failed: {}",
                        response.error.unwrap_or_else(|| "unknown error".to_string())
                    )));
                }
            } else {
                self.buffered_events.push(parse_event(incoming));
            }
        }
    }

    fn next_request_id(&mut self, command: &str) -> String {
        let id = format!("morn-{command}-{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn write_record(&mut self, record: Value) -> Result<()> {
        serde_json::to_writer(&mut self.stdin, &record)
            .map_err(|error| Error::external(format!("encode Pi RPC JSON: {error}")))?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| Error::external(format!("write Pi RPC JSONL: {error}")))
    }

    fn read_record(&mut self) -> Result<Value> {
        loop {
            let mut line = String::new();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .map_err(|error| Error::external(format!("read Pi RPC JSONL: {error}")))?;
            if bytes == 0 {
                return Err(Error::external("Pi RPC runtime closed stdout"));
            }
            let trimmed = line.trim_end_matches(|ch| ch == '\r' || ch == '\n');
            if trimmed.is_empty() {
                continue;
            }
            return serde_json::from_str(trimmed)
                .map_err(|error| Error::external(format!("invalid Pi RPC JSONL record: {error}")));
        }
    }
}

impl Drop for PiRpcClient {
    fn drop(&mut self) {
        // Closing stdin is Pi's orderly shutdown signal, but ChildStdin cannot
        // be moved out of Drop. If the child is still running, terminate it as
        // a best-effort cleanup; Work truth lives outside this process.
        let _ = self.child.kill();
        let _ = self.child.wait();
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
        error: value.get("error").and_then(Value::as_str).map(str::to_string),
    })
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
    response
        .data
        .as_ref()?
        .get("disposition")?
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_process_boundary_matches_current_pi_rpc_mode() {
        let config = PiRpcConfig::default();
        assert_eq!(config.command, "pi");
        assert_eq!(
            config.command_line(),
            vec!["--mode", "rpc", "--no-session"]
        );
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
