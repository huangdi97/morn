//! 注意：此通道需要 \[钉钉\] 真实应用注册才能使用
//! 配置方式：在钉钉开放平台创建应用，获取 Webhook URL
//! 环境变量：DINGTALK_WEBHOOK_URL

use crate::core::error::MornError;
use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct DingTalkChannel {
    webhook_url: String,
    webhook_token: Option<String>,
    app_key: Option<String>,
    app_secret: Option<String>,
}

impl DingTalkChannel {
    pub fn new(webhook_url: &str) -> Self {
        DingTalkChannel {
            webhook_url: webhook_url.to_string(),
            webhook_token: None,
            app_key: None,
            app_secret: None,
        }
    }

    pub fn with_webhook_config(
        webhook_url: &str,
        webhook_token: Option<String>,
        app_key: Option<String>,
        app_secret: Option<String>,
    ) -> Self {
        DingTalkChannel {
            webhook_url: webhook_url.to_string(),
            webhook_token,
            app_key,
            app_secret,
        }
    }

    pub fn from_env() -> Result<Self, MornError> {
        let url = std::env::var("DINGTALK_WEBHOOK_URL")
            .map_err(|_| "DINGTALK_WEBHOOK_URL not set".to_string())?;
        Ok(DingTalkChannel::with_webhook_config(
            &url,
            std::env::var("DINGTALK_WEBHOOK_TOKEN").ok(),
            std::env::var("DINGTALK_APP_KEY").ok(),
            std::env::var("DINGTALK_APP_SECRET").ok(),
        ))
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), MornError> {
        let payload = Self::build_payload(msg);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| MornError::Internal(format!("Failed to create HTTP client: {}", e)))?;
        let resp = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| MornError::Internal(format!("Failed to send DingTalk message: {}", e)))?;
        if !resp.status().is_success() {
            return Err(MornError::Internal(format!(
                "DingTalk webhook returned non-200 status: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    fn build_payload(msg: &ChannelMessage) -> serde_json::Value {
        serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": msg.content
            }
        })
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, MornError> {
        Err(MornError::Internal("DingTalk receive uses webhook_listen(adapter) for incoming callbacks".to_string()))
    }

    pub fn webhook_listen(&self, adapter: &mut ChannelAdapter) -> Result<(), MornError> {
        let listen_addr = self.listen_addr();
        let listener = TcpListener::bind(&listen_addr).map_err(|e| {
            format!(
                "Failed to bind DingTalk webhook listener {}: {}",
                listen_addr, e
            )
        })?;

        println!("[DingTalk] webhook listener started at {}", listen_addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(e) = self.handle_http_connection(&mut stream, adapter) {
                        tracing::error!("[DingTalk] webhook request failed: {}", e);
                        let _ = write_http_response(&mut stream, 500, "text/plain", "error");
                    }
                }
                Err(e) => {
                    tracing::error!("[DingTalk] webhook accept failed: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_http_connection(
        &self,
        stream: &mut TcpStream,
        adapter: &mut ChannelAdapter,
    ) -> Result<(), MornError> {
        let request = read_http_request(stream)?;

        if request.method != "POST" {
            write_http_response(stream, 405, "text/plain", "method not allowed")?;
            return Ok(());
        }

        if !self.token_matches(&request) {
            write_http_response(stream, 401, "text/plain", "unauthorized")?;
            return Ok(());
        }

        let incoming = parse_dingtalk_json(&request.body)?;
        let channel_msg = incoming.to_channel_message(self);
        let reply = adapter.handle_message(&channel_msg);
        let body = serde_json::to_string(&serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": reply,
            }
        }))
        .map_err(|e| MornError::Internal(format!("Failed to serialize DingTalk reply: {}", e)))?;

        write_http_response(stream, 200, "application/json; charset=utf-8", &body)
    }

    fn listen_addr(&self) -> String {
        std::env::var("MORN_CHANNEL_DINGTALK_WEBHOOK_ADDR")
            .or_else(|_| std::env::var("DINGTALK_WEBHOOK_ADDR"))
            .unwrap_or_else(|_| "0.0.0.0:8089".to_string())
    }

    fn token_matches(&self, request: &HttpRequest) -> bool {
        let Some(expected) = self
            .webhook_token
            .as_deref()
            .filter(|token| !token.is_empty())
        else {
            return true;
        };

        request
            .query_params
            .get("token")
            .or_else(|| request.query_params.get("access_token"))
            .map(|value| value == expected)
            .unwrap_or(false)
            || request
                .path
                .trim_matches('/')
                .split('/')
                .any(|segment| segment == expected)
    }
}

pub struct DingTalkServer {
    adapter: Option<ChannelAdapter>,
}

impl DingTalkServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        DingTalkServer { adapter }
    }

    pub fn handle_callback(&mut self, text: &str) -> Result<String, MornError> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "dingtalk".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}

struct HttpRequest {
    method: String,
    path: String,
    query_params: HashMap<String, String>,
    body: String,
}

#[derive(Debug, Clone)]
struct DingTalkIncomingMessage {
    content: String,
    msg_type: Option<String>,
    conversation_id: Option<String>,
    sender_id: Option<String>,
    chatbot_user_id: Option<String>,
}

impl DingTalkIncomingMessage {
    fn to_channel_message(&self, channel: &DingTalkChannel) -> ChannelMessage {
        ChannelMessage {
            content: self.content.clone(),
            source: "dingtalk".into(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: serde_json::json!({
                "msg_type": self.msg_type,
                "conversation_id": self.conversation_id,
                "sender_id": self.sender_id,
                "chatbot_user_id": self.chatbot_user_id,
                "app_key": channel.app_key,
                "webhook_token_configured": channel.webhook_token.is_some(),
                "app_secret_configured": channel.app_secret.is_some(),
            }),
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, MornError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| MornError::Internal(format!("Failed to set read timeout: {}", e)))?;

    let mut buffer = Vec::new();
    let header_end = loop {
        if let Some(pos) = find_subsequence(&buffer, b"\r\n\r\n") {
            break pos + 4;
        }

        let mut chunk = [0u8; 4096];
        let bytes_read = stream
            .read(&mut chunk)
            .map_err(|e| MornError::Internal(format!("Failed to read HTTP request: {}", e)))?;
        if bytes_read == 0 {
            return Err(MornError::Internal("Connection closed before HTTP headers were complete".to_string()))
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = headers.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "Missing HTTP request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "Missing HTTP method".to_string())?
        .to_string();
    let target = request_parts
        .next()
        .ok_or_else(|| "Missing HTTP request target".to_string())?;
    let (path, query_params) = parse_request_target(target);

    let mut header_map = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            header_map.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let content_length = header_map
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    while buffer.len() < header_end + content_length {
        let mut chunk = [0u8; 4096];
        let bytes_read = stream
            .read(&mut chunk)
            .map_err(|e| MornError::Internal(format!("Failed to read HTTP body: {}", e)))?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
    }

    let body_bytes = &buffer[header_end..buffer.len().min(header_end + content_length)];
    let body = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| MornError::Internal(format!("DingTalk webhook body is not UTF-8: {}", e)))?;

    Ok(HttpRequest {
        method,
        path,
        query_params,
        body,
    })
}

fn parse_request_target(target: &str) -> (String, HashMap<String, String>) {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let query_params = query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (key.to_string(), value.to_string())
        })
        .collect();

    (path.to_string(), query_params)
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<(), MornError> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        reason,
        content_type,
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|e| MornError::Internal(format!("Failed to write HTTP response: {}", e)))
}

fn parse_dingtalk_json(body: &str) -> Result<DingTalkIncomingMessage, MornError> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|e| MornError::Internal(format!("Invalid DingTalk JSON body: {}", e)))?;
    let content = value
        .get("text")
        .and_then(|text| text.get("content"))
        .and_then(|content| content.as_str())
        .ok_or_else(|| "DingTalk JSON body missing text.content".to_string())?
        .to_string();

    Ok(DingTalkIncomingMessage {
        content,
        msg_type: value
            .get("msgtype")
            .and_then(|msg_type| msg_type.as_str())
            .map(str::to_string),
        conversation_id: value
            .get("conversationId")
            .and_then(|conversation_id| conversation_id.as_str())
            .map(str::to_string),
        sender_id: value
            .get("senderId")
            .and_then(|sender_id| sender_id.as_str())
            .map(str::to_string),
        chatbot_user_id: value
            .get("chatbotUserId")
            .and_then(|chatbot_user_id| chatbot_user_id.as_str())
            .map(str::to_string),
    })
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dingtalk_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let payload = DingTalkChannel::build_payload(&msg);
        assert_eq!(payload["msgtype"], "text");
        assert_eq!(payload["text"]["content"], "Hello");
    }

    #[test]
    fn test_dingtalk_send_connection_error() {
        let channel = DingTalkChannel::new("http://localhost:1/nonexistent");
        let msg = ChannelMessage {
            content: "test".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let result = channel.send(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dingtalk_text_json() {
        let body = r#"{
            "msgtype": "text",
            "text": {"content": "hello"},
            "conversationId": "cid",
            "senderId": "sid",
            "chatbotUserId": "bot"
        }"#;

        let msg = parse_dingtalk_json(body).unwrap();
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.msg_type.as_deref(), Some("text"));
        assert_eq!(msg.conversation_id.as_deref(), Some("cid"));
        assert_eq!(msg.sender_id.as_deref(), Some("sid"));
        assert_eq!(msg.chatbot_user_id.as_deref(), Some("bot"));
    }

    #[test]
    fn test_parse_dingtalk_json_requires_text_content() {
        let err = parse_dingtalk_json(r#"{"msgtype":"text","text":{}}"#).unwrap_err();
        assert!(err.contains("text.content"));
    }

    #[test]
    fn test_dingtalk_token_matches_query_or_path() {
        let channel = DingTalkChannel::with_webhook_config("", Some("secret".into()), None, None);
        let (_, query_params) = parse_request_target("/dingtalk?token=secret");
        let query_request = HttpRequest {
            method: "POST".into(),
            path: "/dingtalk".into(),
            query_params,
            body: String::new(),
        };
        assert!(channel.token_matches(&query_request));

        let (path, query_params) = parse_request_target("/dingtalk/secret");
        let path_request = HttpRequest {
            method: "POST".into(),
            path,
            query_params,
            body: String::new(),
        };
        assert!(channel.token_matches(&path_request));
    }
}
