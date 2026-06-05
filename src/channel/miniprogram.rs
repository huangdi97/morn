//! 注意：此通道需要 [微信小程序] 真实应用注册才能使用
//! 配置方式：在微信公众平台注册小程序，获取 AppID
//! 环境变量：MINIPROGRAM_APPID

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub struct MiniProgramChannel {
    adapter: Option<ChannelAdapter>,
}

impl MiniProgramChannel {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        MiniProgramChannel { adapter }
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
