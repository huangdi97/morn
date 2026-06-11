//! 注意：此通道需要 \[微信小程序\] 真实应用注册才能使用
//! 配置方式：在微信公众平台注册小程序，获取 AppID
//! 环境变量：MINIPROGRAM_APPID, MINIPROGRAM_SECRET

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

#[allow(dead_code)] /* 预留：微信小程序通道真实接入 */
pub struct MiniProgramChannel {
    app_id: String,
    app_secret: String,
    token: Option<String>,
    access_token: Option<String>,
}

impl MiniProgramChannel {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        MiniProgramChannel {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            token: None,
            access_token: None,
        }
    }

    pub fn with_token(app_id: &str, app_secret: &str, token: Option<String>) -> Self {
        MiniProgramChannel {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            token,
            access_token: None,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let app_id = std::env::var("MINIPROGRAM_APPID")
            .map_err(|_| "MINIPROGRAM_APPID not set".to_string())?;
        let app_secret = std::env::var("MINIPROGRAM_SECRET")
            .map_err(|_| "MINIPROGRAM_SECRET not set".to_string())?;
        let token = std::env::var("MINIPROGRAM_TOKEN").ok();
        Ok(MiniProgramChannel::with_token(&app_id, &app_secret, token))
    }

    pub fn get_access_token(&mut self) -> Result<String, String> {
        if let Some(ref token) = self.access_token {
            return Ok(token.clone());
        }

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.app_id, self.app_secret
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("Failed to get access token: {}", e))?;
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse token response: {}", e))?;
        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| format!("access_token not found in response: {}", body))?
            .to_string();
        self.access_token = Some(token.clone());
        Ok(token)
    }

    pub fn send(&mut self, msg: &ChannelMessage) -> Result<(), String> {
        let touser = msg.metadata["touser"]
            .as_str()
            .ok_or_else(|| "Missing 'touser' in message metadata".to_string())?;
        let token = match &self.access_token {
            Some(t) => t.clone(),
            None => self.get_access_token()?,
        };
        let payload = serde_json::json!({
            "touser": touser,
            "msgtype": "text",
            "text": {
                "content": msg.content
            }
        });
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/message/custom/send?access_token={}",
            token
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .post(&url)
            .json(&payload)
            .send()
            .map_err(|e| format!("Failed to send MiniProgram message: {}", e))?;
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse send response: {}", e))?;
        if body["errcode"].as_i64() != Some(0) {
            return Err(format!(
                "MiniProgram API error: {} - {}",
                body["errcode"], body["errmsg"]
            ));
        }
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Option<ChannelMessage>, String> {
        Ok(self.fetch_messages()?.into_iter().next())
    }

    pub fn poll_messages(&mut self, adapter: &mut ChannelAdapter) -> Result<(), String> {
        loop {
            match self.fetch_messages() {
                Ok(messages) => {
                    for msg in messages {
                        let reply = adapter.handle_message(&msg);
                        let touser = msg
                            .metadata
                            .get("touser")
                            .or_else(|| msg.metadata.get("openid"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&msg.source);

                        if !reply.is_empty() && touser != "miniprogram" {
                            let reply_msg = ChannelMessage {
                                content: reply,
                                source: "miniprogram".into(),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                metadata: serde_json::json!({ "touser": touser }),
                            };
                            if let Err(e) = self.send(&reply_msg) {
                                tracing::warn!("[MiniProgram] failed to send reply: {}", e);
                            }
                        }
                    }
                }
                Err(e) => tracing::warn!("[MiniProgram] failed to poll messages: {}", e),
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn fetch_messages(&mut self) -> Result<Vec<ChannelMessage>, String> {
        let access_token = self.get_access_token()?;
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/message/custom/get?access_token={}",
            access_token
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("Failed to poll MiniProgram messages: {}", e))?;

        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Failed to parse MiniProgram poll response: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "MiniProgram API returned non-200 status {}: {}",
                status, body
            ));
        }

        if body.get("errcode").and_then(|v| v.as_i64()).unwrap_or(0) != 0 {
            return Err(format!(
                "MiniProgram API error: {} - {}",
                body.get("errcode").unwrap_or(&serde_json::Value::Null),
                body.get("errmsg").unwrap_or(&serde_json::Value::Null)
            ));
        }

        let values = body
            .get("messages")
            .or_else(|| body.get("msg_list"))
            .or_else(|| body.get("items"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_else(|| {
                if extract_text_content(&body).is_some() {
                    vec![body.clone()]
                } else {
                    Vec::new()
                }
            });

        values
            .into_iter()
            .filter_map(|value| match json_value_to_channel_message(value) {
                Ok(Some(msg)) => Some(Ok(msg)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }
}

pub struct MiniProgramServer {
    adapter: Option<ChannelAdapter>,
}

fn json_value_to_channel_message(
    value: serde_json::Value,
) -> Result<Option<ChannelMessage>, String> {
    let content = match extract_text_content(&value) {
        Some(content) if !content.trim().is_empty() => content,
        _ => return Ok(None),
    };

    let source = value
        .get("FromUserName")
        .or_else(|| value.get("from_user_name"))
        .or_else(|| value.get("openid"))
        .or_else(|| value.get("from"))
        .and_then(|v| v.as_str())
        .unwrap_or("miniprogram")
        .to_string();

    Ok(Some(ChannelMessage {
        content,
        source: source.clone(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        metadata: serde_json::json!({
            "type": "text",
            "openid": source,
            "touser": source,
            "raw": value,
        }),
    }))
}

fn extract_text_content(value: &serde_json::Value) -> Option<String> {
    value
        .get("Content")
        .or_else(|| value.get("content"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            value
                .get("text")
                .and_then(|text| text.get("content").or(Some(text)))
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("text"))
                .and_then(|v| v.as_str())
        })
        .map(|s| s.to_string())
}

#[cfg(test)]
fn parse_miniprogram_json_message(body: &str) -> Result<Option<ChannelMessage>, String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| format!("Failed to parse MiniProgram message JSON: {}", e))?;
    json_value_to_channel_message(value)
}

impl MiniProgramServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        MiniProgramServer { adapter }
    }

    pub fn handle_message(&mut self, text: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "miniprogram".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_missing_vars() {
        let result = MiniProgramChannel::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_new_channel() {
        let channel = MiniProgramChannel::new("test_appid", "test_secret");
        assert_eq!(channel.app_id, "test_appid");
        assert_eq!(channel.app_secret, "test_secret");
        assert!(channel.token.is_none());
        assert!(channel.access_token.is_none());
    }

    #[test]
    fn test_get_access_token_invalid_creds() {
        let mut channel = MiniProgramChannel::new("fake_id", "fake_secret");
        let result = channel.get_access_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_send_missing_touser() {
        let mut channel = MiniProgramChannel::new("test_id", "test_secret");
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "test".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let result = channel.send(&msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("touser"));
    }

    #[test]
    fn test_parse_json_text_message() {
        let msg = parse_miniprogram_json_message(
            r#"{"MsgType":"text","FromUserName":"openid_1","Content":"hello"}"#,
        )
        .unwrap()
        .unwrap();

        assert_eq!(msg.content, "hello");
        assert_eq!(msg.source, "openid_1");
        assert_eq!(msg.metadata["touser"], "openid_1");
    }

    #[test]
    fn test_parse_json_ignores_empty_text() {
        let msg = parse_miniprogram_json_message(r#"{"MsgType":"image","PicUrl":"x"}"#).unwrap();
        assert!(msg.is_none());
    }

    #[test]
    fn test_server_handle_message_no_adapter() {
        let mut server = MiniProgramServer::new(None);
        let result = server.handle_message("test");
        assert_eq!(result, Ok("No adapter configured".into()));
    }
}
