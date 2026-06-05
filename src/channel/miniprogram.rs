//! 注意：此通道需要 [微信小程序] 真实应用注册才能使用
//! 配置方式：在微信公众平台注册小程序，获取 AppID
//! 环境变量：MINIPROGRAM_APPID, MINIPROGRAM_SECRET

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

#[allow(dead_code)]
pub struct MiniProgramChannel {
    app_id: String,
    app_secret: String,
    access_token: Option<String>,
}

impl MiniProgramChannel {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        MiniProgramChannel {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            access_token: None,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let app_id = std::env::var("MINIPROGRAM_APPID")
            .map_err(|_| "MINIPROGRAM_APPID not set".to_string())?;
        let app_secret = std::env::var("MINIPROGRAM_SECRET")
            .map_err(|_| "MINIPROGRAM_SECRET not set".to_string())?;
        Ok(MiniProgramChannel::new(&app_id, &app_secret))
    }

    pub fn get_access_token(&mut self) -> Result<String, String> {
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

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(None)
    }
}

pub struct MiniProgramServer {
    adapter: Option<ChannelAdapter>,
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
    fn test_receive_returns_none() {
        let channel = MiniProgramChannel::new("test_id", "test_secret");
        let result = channel.receive();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_server_handle_message_no_adapter() {
        let mut server = MiniProgramServer::new(None);
        let result = server.handle_message("test");
        assert_eq!(result, Ok("No adapter configured".into()));
    }
}
