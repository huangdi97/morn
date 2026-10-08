//! DeepSeek Harness SDK stdio boundary.
//!
//! DSH currently exposes newline-delimited JSON-RPC 2.0 over stdio. The
//! stable request methods at the researched upstream boundary are
//! `initialize`, `session/prompt`, and `shutdown`; notifications are
//! `session.event`, `session.status`, `subagent.started`, and
//! `subagent.finished`. The current wire has no cancel/session-close method,
//! so Morn must not fabricate those semantics.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use morn_kernel::error::{Error, Result};

pub const DSH_METHOD_INITIALIZE: &str = "initialize";
pub const DSH_METHOD_SESSION_PROMPT: &str = "session/prompt";
pub const DSH_METHOD_SHUTDOWN: &str = "shutdown";

pub const DSH_NOTIFICATION_SESSION_EVENT: &str = "session.event";
pub const DSH_NOTIFICATION_SESSION_STATUS: &str = "session.status";
pub const DSH_NOTIFICATION_SUBAGENT_STARTED: &str = "subagent.started";
pub const DSH_NOTIFICATION_SUBAGENT_FINISHED: &str = "subagent.finished";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DshSdkConfig {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub max_tokens: Option<u64>,
}

impl DshSdkConfig {
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
        }
    }
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

pub struct DshSdkStdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
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
        {
            return Err(Error::validation(
                "DSH SDK requires non-empty cwd, provider and model",
            ));
        }
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .current_dir(&config.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = command.spawn().map_err(|error| {
            Error::external(format!(
                "failed to start DSH SDK runtime {:?}: {error}",
                config.command
            ))
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::external("DSH SDK runtime has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::external("DSH SDK runtime has no stdout"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            notifications: Vec::new(),
        })
    }

    pub fn initialize(&mut self, config: &DshSdkConfig) -> Result<Value> {
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
        let name = result
            .get("serverInfo")
            .and_then(|info| info.get("name"))
            .and_then(Value::as_str)
            .ok_or_else(|| Error::external("DSH initialize result missing serverInfo.name"))?;
        if name != "deepseek-harness-sdk-runtime" {
            return Err(Error::external(format!(
                "unexpected DSH SDK server identity {name:?}"
            )));
        }
        Ok(result)
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
    pub fn run_text_prompt(
        &mut self,
        session_id: &str,
        text: &str,
    ) -> Result<DshSdkRunResult> {
        if session_id.trim().is_empty() || text.trim().is_empty() {
            return Err(Error::validation(
                "DSH SDK run requires non-empty session id and prompt text",
            ));
        }
        let start = self.notifications.len();
        let message_id = self.enqueue_text_prompt(session_id, text)?;
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

            let incoming = self.read_frame()?;
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
        self.request(DSH_METHOD_SHUTDOWN, None)?;
        Ok(())
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
        serde_json::to_writer(&mut self.stdin, &frame)
            .map_err(|error| Error::external(format!("encode DSH JSON-RPC: {error}")))?;
        self.stdin
            .write_all(b"\n")
            .and_then(|_| self.stdin.flush())
            .map_err(|error| Error::external(format!("write DSH JSON-RPC: {error}")))?;

        loop {
            let incoming = self.read_frame()?;
            if incoming.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(error) = incoming.get("error") {
                    return Err(Error::external(format!(
                        "DSH JSON-RPC {method} failed: {error}"
                    )));
                }
                return Ok(incoming.get("result").cloned().unwrap_or(Value::Null));
            }
            if incoming.get("id").is_none() {
                if let Some(method) = incoming.get("method").and_then(Value::as_str) {
                    self.notifications.push(DshNotification {
                        method: method.to_string(),
                        params: incoming.get("params").cloned().unwrap_or(Value::Null),
                    });
                }
            }
        }
    }

    fn read_frame(&mut self) -> Result<Value> {
        loop {
            let mut line = String::new();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .map_err(|error| Error::external(format!("read DSH JSON-RPC: {error}")))?;
            if bytes == 0 {
                return Err(Error::external("DSH SDK runtime closed stdout"));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                return Ok(value);
            }
            // Upstream protocol specifies malformed lines are ignored.
        }
    }
}

impl Drop for DshSdkStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
            finish_reason_from_session_event(&end, "session-1").unwrap().as_deref(),
            Some("completed")
        );
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
        };
        let mut client = DshSdkStdioClient::spawn(&config).unwrap();
        let initialized = client.initialize(&config).unwrap();
        assert_eq!(
            initialized["serverInfo"]["name"],
            "deepseek-harness-sdk-runtime"
        );

        let run = client.run_text_prompt("session-1", "hello").unwrap();
        assert_eq!(run.message_id, "message-1");
        assert_eq!(run.final_response, "hello from fake sdk");
        assert_eq!(run.finish_reason.as_deref(), Some("completed"));
        assert!(run.notifications.iter().any(|notification| {
            notification.method == DSH_NOTIFICATION_SESSION_STATUS
                && session_idle_matches(notification, "session-1")
        }));
        client.shutdown().unwrap();
    }

    /// A tiny protocol peer used only by the parent test above. It is ignored
    /// in normal test discovery and launched as a child test process.
    #[test]
    #[ignore]
    fn fake_sdk_runtime() {
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
                                "sessionId":"session-1",
                                "event":{
                                    "type":"agent/inbox/spliced",
                                    "data":{"inserted":[{"id":"message-1"}]}
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
                                "sessionId":"session-1",
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
                                "sessionId":"session-1",
                                "event":{
                                    "type":"turn/end",
                                    "data":{"reason":{"kind":"completed"}}
                                }
                            }
                        }),
                    );
                    write(
                        &mut stdout,
                        json!({
                            "jsonrpc":"2.0",
                            "method":"session.status",
                            "params":{"sessionId":"session-1","status":"idle"}
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
