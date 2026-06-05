//! 注意：此通道需要 [微信公众号] 真实应用注册才能使用
//! 配置方式：在微信公众平台注册服务号，获取 AppID 和 AppSecret
//! 环境变量：WECHAT_MP_APPID, WECHAT_MP_SECRET

use crate::channel::adapter::ChannelMessage;

#[allow(dead_code)]
pub struct WeChatMpChannel {
    app_id: String,
    app_secret: String,
    adapter_ref: bool,
}

impl WeChatMpChannel {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        WeChatMpChannel {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            adapter_ref: false,
        }
    }

    pub fn send(&self, msg: &ChannelMessage) -> Result<(), String> {
        println!(
            "[WeChat MP] Sending message: {} (to user via template msg)",
            msg.content
        );
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<ChannelMessage>, String> {
        Ok(Some(ChannelMessage {
            content: "[WeChat MP] simulated incoming message".into(),
            source: "wechat_mp".into(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: serde_json::json!({"type": "text", "msg_id": "wx_12345"}),
        }))
    }

    pub fn handle_event(&self, event_type: &str, payload: &str) -> Result<String, String> {
        Ok(format!(
            "[WeChat MP] handled event '{}': {}",
            event_type, payload
        ))
    }
}
