use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

// ===== 总是可用的渠道（3个） =====

macro_rules! make_always_channel_plugin {
    ($name:ident, $id:expr) => {
        pub struct $name;
        impl MornPlugin for $name {
            fn id(&self) -> &str { $id }
            fn deps(&self) -> Vec<&str> { vec!["morn:channel-bus", "morn:data-layer"] }
            fn priority(&self) -> i32 { 60 }
            fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
            fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
            fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
        }
        impl Default for $name { fn default() -> Self { Self } }
    };
}

make_always_channel_plugin!(CliChannelPlugin, "morn:channel-cli");
make_always_channel_plugin!(DesktopChannelPlugin, "morn:channel-desktop");
make_always_channel_plugin!(RestApiChannelPlugin, "morn:channel-rest-api");

// ===== feature-gated 渠道（12个） =====

macro_rules! make_channel_plugin {
    ($name:ident, $id:expr) => {
        pub struct $name;
        impl MornPlugin for $name {
            fn id(&self) -> &str { $id }
            fn deps(&self) -> Vec<&str> { vec!["morn:channel-bus", "morn:data-layer"] }
            fn priority(&self) -> i32 { 60 }
            fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
            fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
            fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
        }
        impl Default for $name { fn default() -> Self { Self } }
    };
}

#[cfg(feature = "channels-full")]
pub(crate) mod channel_full {
    use super::*;
    make_channel_plugin!(TelegramChannelPlugin, "morn:channel-telegram");
    make_channel_plugin!(WecomChannelPlugin, "morn:channel-wecom");
    make_channel_plugin!(FeishuChannelPlugin, "morn:channel-feishu");
    make_channel_plugin!(DingtalkChannelPlugin, "morn:channel-dingtalk");
    make_channel_plugin!(MiniprogramChannelPlugin, "morn:channel-miniprogram");
    make_channel_plugin!(QqbotChannelPlugin, "morn:channel-qqbot");
    make_channel_plugin!(PushplusChannelPlugin, "morn:channel-pushplus");
    make_channel_plugin!(ServerchanChannelPlugin, "morn:channel-serverchan");
    make_channel_plugin!(WebhookChannelPlugin, "morn:channel-webhook");
    make_channel_plugin!(WechatMpChannelPlugin, "morn:channel-wechat-mp");
    make_channel_plugin!(BrowserExtChannelPlugin, "morn:channel-browser-ext");
    make_channel_plugin!(SmtpChannelPlugin, "morn:channel-smtp");
}