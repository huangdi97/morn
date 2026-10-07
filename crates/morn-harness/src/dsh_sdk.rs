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
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
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
        self.request(DSH_METHOD_INITIALIZE, Some(params))
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
