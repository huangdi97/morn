//! 注意：此通道需要 [微信公众号] 真实应用注册才能使用
//! 配置方式：在微信公众平台注册服务号，获取 AppID 和 AppSecret
//! 环境变量：WECHAT_MP_APPID, WECHAT_MP_SECRET

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct WeChatMpChannel {
    app_id: String,
    app_secret: String,
}

impl WeChatMpChannel {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        WeChatMpChannel {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let app_id =
            std::env::var("WECHAT_MP_APPID").map_err(|_| "WECHAT_MP_APPID not set".to_string())?;
        let app_secret = std::env::var("WECHAT_MP_SECRET")
            .map_err(|_| "WECHAT_MP_SECRET not set".to_string())?;
        Ok(WeChatMpChannel::new(&app_id, &app_secret))
    }

    pub fn get_access_token(&self) -> Result<String, String> {
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
            .map_err(|e| format!("Failed to parse access token response: {}", e))?;
        if let Some(token) = body.get("access_token").and_then(|v| v.as_str()) {
            Ok(token.to_string())
        } else {
            let err_msg = body
                .get("errmsg")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            Err(format!("WeChat API error: {}", err_msg))
        }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        let payload = Self::build_payload(msg);
        let access_token = self.get_access_token()?;
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/message/custom/send?access_token={}",
            access_token
        );
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .post(&url)
            .json(&payload)
            .send()
            .map_err(|e| format!("Failed to send WeChat MP message: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!(
                "WeChat MP API returned non-200 status: {}",
                resp.status()
            ));
        }
        Ok(())
    }

    fn build_payload(msg: &ChannelMessage) -> serde_json::Value {
        serde_json::json!({
            "touser": msg.source,
            "msgtype": "text",
            "text": {
                "content": msg.content
            }
        })
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(None)
    }

    pub fn handle_event(&self, event_type: &str, payload: &str) -> Result<String, String> {
        Ok(format!(
            "[WeChat MP] handled event '{}': {}",
            event_type, payload
        ))
    }
}

pub struct WeChatMpServer {
    adapter: Option<ChannelAdapter>,
}

impl WeChatMpServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        WeChatMpServer { adapter }
    }

    pub fn handle_callback(&mut self, text: &str) -> Result<String, String> {
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "wechat_mp".into(),
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
    fn test_wechat_mp_build_payload() {
        let msg = ChannelMessage {
            content: "Hello".into(),
            source: "openid_user".into(),
            timestamp: 0,
            metadata: serde_json::json!({}),
        };
        let payload = WeChatMpChannel::build_payload(&msg);
        assert_eq!(payload["touser"], "openid_user");
        assert_eq!(payload["msgtype"], "text");
        assert_eq!(payload["text"]["content"], "Hello");
    }

    #[test]
    fn test_wechat_mp_send_connection_error() {
        let channel = WeChatMpChannel::new("invalid_appid", "invalid_secret");
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
    fn test_wechat_mp_access_token_error() {
        let channel = WeChatMpChannel::new("bad_appid", "bad_secret");
        let result = channel.get_access_token();
        assert!(result.is_err());
    }
}
