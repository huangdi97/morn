//! 注意：此通道需要 \[企业微信\] 真实应用注册才能使用
//! 配置方式：在企业微信后台创建应用，获取 Webhook URL
//! 环境变量：WECOM_WEBHOOK_URL

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use crate::core::error::MornError;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct WeComChannel {
    webhook_url: String,
    token: Option<String>,
    encoding_aes_key: Option<String>,
    corp_id: Option<String>,
}

impl WeComChannel {
    pub fn new(webhook_url: &str) -> Self {
        WeComChannel {
            webhook_url: webhook_url.to_string(),
            token: None,
            encoding_aes_key: None,
            corp_id: None,
        }
    }

    pub fn with_webhook_config(
        webhook_url: &str,
        token: Option<String>,
        encoding_aes_key: Option<String>,
        corp_id: Option<String>,
    ) -> Self {
        WeComChannel {
            webhook_url: webhook_url.to_string(),
            token,
            encoding_aes_key,
            corp_id,
        }
    }

    pub fn from_env() -> Result<Self, MornError> {
        let url = std::env::var("WECOM_WEBHOOK_URL")
            .map_err(|_| "WECOM_WEBHOOK_URL not set".to_string())?;
        Ok(WeComChannel::with_webhook_config(
            &url,
            std::env::var("WECOM_TOKEN").ok(),
            std::env::var("WECOM_ENCODING_AES_KEY").ok(),
            std::env::var("WECOM_CORP_ID").ok(),
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
            .map_err(|e| MornError::Internal(format!("Failed to send WeCom message: {}", e)))?;
        if !resp.status().is_success() {
            return Err(MornError::Internal(format!(
                "WeCom webhook returned non-200 status: {}",
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
        Err(MornError::Internal(
            "WeCom receive uses webhook_listen(adapter) for incoming callbacks".to_string(),
        ))
    }

    pub fn webhook_listen(&self, adapter: &mut ChannelAdapter) -> Result<(), MornError> {
        let listen_addr = self.listen_addr()?;
        let listener = TcpListener::bind(&listen_addr).map_err(|e| {
            format!(
                "Failed to bind WeCom webhook listener {}: {}",
                listen_addr, e
            )
        })?;

        println!("[WeCom] webhook listener started at {}", listen_addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(e) = self.handle_http_connection(&mut stream, adapter) {
                        tracing::error!("[WeCom] webhook request failed: {}", e);
                        let _ = write_http_response(&mut stream, 500, "text/plain", "error");
                    }
                }
                Err(e) => {
                    tracing::error!("[WeCom] webhook accept failed: {}", e);
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

        let incoming = parse_wecom_xml(&request.body)?;
        let channel_msg = incoming.to_channel_message(self);
        let reply = adapter.handle_message(&channel_msg);
        let body = incoming.build_text_reply(&reply);

        write_http_response(stream, 200, "application/xml; charset=utf-8", &body)
    }

    fn listen_addr(&self) -> Result<String, MornError> {
        let url = reqwest::Url::parse(&self.webhook_url).map_err(|e| {
            MornError::Internal(format!(
                "Invalid WeCom webhook_url {}: {}",
                self.webhook_url, e
            ))
        })?;
        let host = match url.host_str() {
            Some("localhost") | Some("127.0.0.1") | Some("0.0.0.0") => {
                url.host_str().unwrap_or("localhost").to_string()
            }
            Some(_) => "0.0.0.0".to_string(),
            None => "0.0.0.0".to_string(),
        };
        let port = url
            .port()
            .or_else(|| match url.scheme() {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            })
            .unwrap_or(8088);

        Ok(format!("{}:{}", host, port))
    }
}

pub struct WeComServer {
    adapter: Option<ChannelAdapter>,
}

impl WeComServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        WeComServer { adapter }
    }

    pub fn handle_webhook(&mut self, text: &str) -> Result<String, MornError> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "wecom".into(),
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
    body: String,
}

#[derive(Debug, Clone)]
struct WeComIncomingMessage {
    to_user_name: String,
    from_user_name: String,
    create_time: Option<i64>,
    msg_type: String,
    content: String,
    msg_id: Option<String>,
    agent_id: Option<String>,
}

impl WeComIncomingMessage {
    fn to_channel_message(&self, channel: &WeComChannel) -> ChannelMessage {
        ChannelMessage {
            content: self.content.clone(),
            source: "wecom".into(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: serde_json::json!({
                "to_user_name": self.to_user_name,
                "from_user_name": self.from_user_name,
                "create_time": self.create_time,
                "msg_type": self.msg_type,
                "msg_id": self.msg_id,
                "agent_id": self.agent_id,
                "corp_id": channel.corp_id,
                "token_configured": channel.token.is_some(),
                "encoding_aes_key_configured": channel.encoding_aes_key.is_some(),
            }),
        }
    }

    fn build_text_reply(&self, content: &str) -> String {
        format!(
            "<xml>\
<ToUserName><![CDATA[{to}]]></ToUserName>\
<FromUserName><![CDATA[{from}]]></FromUserName>\
<CreateTime>{time}</CreateTime>\
<MsgType><![CDATA[text]]></MsgType>\
<Content><![CDATA[{content}]]></Content>\
</xml>",
            to = escape_cdata(&self.from_user_name),
            from = escape_cdata(&self.to_user_name),
            time = chrono::Utc::now().timestamp(),
            content = escape_cdata(content),
        )
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
            return Err(MornError::Internal(
                "Connection closed before HTTP headers were complete".to_string(),
            ));
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
        .map_err(|e| MornError::Internal(format!("WeCom webhook body is not UTF-8: {}", e)))?;

    Ok(HttpRequest { method, body })
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<(), MornError> {
    let reason = match status {
        200 => "OK",
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

fn parse_wecom_xml(xml: &str) -> Result<WeComIncomingMessage, MornError> {
    if extract_xml_text(xml, "Encrypt").is_some() {
        return Err(MornError::Internal(
            "Encrypted WeCom callbacks are configured but AES decrypt is not implemented yet"
                .to_string(),
        ));
    }

    let msg_type = extract_xml_text(xml, "MsgType").unwrap_or_else(|| "text".to_string());
    let content = match msg_type.as_str() {
        "text" => extract_xml_text(xml, "Content").unwrap_or_default(),
        _ => extract_xml_text(xml, "Content").unwrap_or_else(|| format!("[{} message]", msg_type)),
    };

    Ok(WeComIncomingMessage {
        to_user_name: extract_xml_text(xml, "ToUserName").unwrap_or_default(),
        from_user_name: extract_xml_text(xml, "FromUserName").unwrap_or_default(),
        create_time: extract_xml_text(xml, "CreateTime").and_then(|value| value.parse().ok()),
        msg_type,
        content,
        msg_id: extract_xml_text(xml, "MsgId"),
        agent_id: extract_xml_text(xml, "AgentID"),
    })
}

fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let start = format!("<{}>", tag);
    let end = format!("</{}>", tag);
    let start_idx = xml.find(&start)? + start.len();
    let end_idx = xml[start_idx..].find(&end)? + start_idx;
    Some(unescape_xml(trim_cdata(xml[start_idx..end_idx].trim())))
}

fn trim_cdata(value: &str) -> &str {
    value
        .strip_prefix("<![CDATA[")
        .and_then(|inner| inner.strip_suffix("]]>"))
        .unwrap_or(value)
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn escape_cdata(value: &str) -> String {
    value.replace("]]>", "]]]]><![CDATA[>")
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
    fn test_wecom_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let payload = WeComChannel::build_payload(&msg);
        assert_eq!(payload["msgtype"], "text");
        assert_eq!(payload["text"]["content"], "Hello");
    }

    #[test]
    fn test_wecom_send_connection_error() {
        let channel = WeComChannel::new("http://localhost:1/nonexistent");
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
    fn test_parse_wecom_text_xml() {
        let xml = r#"<xml>
<ToUserName><![CDATA[corp]]></ToUserName>
<FromUserName><![CDATA[user]]></FromUserName>
<CreateTime>1710000000</CreateTime>
<MsgType><![CDATA[text]]></MsgType>
<Content><![CDATA[hello]]></Content>
<MsgId>42</MsgId>
<AgentID>1000002</AgentID>
</xml>"#;

        let msg = parse_wecom_xml(xml).unwrap();
        assert_eq!(msg.to_user_name, "corp");
        assert_eq!(msg.from_user_name, "user");
        assert_eq!(msg.msg_type, "text");
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.msg_id.as_deref(), Some("42"));
        assert_eq!(msg.agent_id.as_deref(), Some("1000002"));
    }

    #[test]
    fn test_parse_wecom_encrypted_xml_is_explicitly_unsupported() {
        let xml = r#"<xml><Encrypt><![CDATA[cipher]]></Encrypt></xml>"#;
        let err = parse_wecom_xml(xml).unwrap_err();
        assert!(err.contains("AES decrypt is not implemented"));
    }
}
