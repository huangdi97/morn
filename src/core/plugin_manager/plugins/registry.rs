use super::super::bridge_plugin::BridgePlugin;
use super::super::MornPlugin;
use super::channel_plugins::*;
use super::{
    BackupPlugin, ChannelBusPlugin, DataLayerPlugin, EnginePlugin, HubPlugin, ObservabilityPlugin,
    RegistryPlugin, SandboxPlugin, StudioPlugin, SupervisorPlugin, SyncPlugin, VoicePlugin,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// 内置插件构造器类型
type PluginBuilder = Box<dyn Fn(PathBuf) -> Box<dyn MornPlugin>>;

/// 内部插件注册表 — 知道所有核心插件的构造函数
pub struct CorePluginRegistry {
    builders: HashMap<String, PluginBuilder>,
}

impl Default for CorePluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CorePluginRegistry {
    /// 注册所有内置插件
    pub fn new() -> Self {
        let mut r = Self {
            builders: HashMap::new(),
        };
        r.register("morn:data-layer", Box::new(|_| Box::new(DataLayerPlugin(None))));
        r.register("morn:registry", Box::new(|_| Box::new(RegistryPlugin(None))));
        r.register("morn:sandbox", Box::new(|_| Box::new(SandboxPlugin(None))));
        r.register("morn:engine", Box::new(|_| Box::new(EnginePlugin(None))));
        r.register("morn:channel-bus", Box::new(|_| Box::new(ChannelBusPlugin(None))));
        r.register("morn:supervisor", Box::new(|_| Box::new(SupervisorPlugin(None))));
        r.register("morn:studio", Box::new(|_| Box::new(StudioPlugin(None, None))));
        r.register("morn:hub", Box::new(|_| Box::new(HubPlugin)));
        r.register("morn:bridge", Box::new(|p| Box::new(BridgePlugin::new(p))));
        r.register("morn:observability", Box::new(|_| Box::new(ObservabilityPlugin(None))));
        r.register("morn:voice", Box::new(|_| Box::new(VoicePlugin)));
        r.register("morn:sync", Box::new(|_| Box::new(SyncPlugin(None))));
        r.register("morn:backup", Box::new(|_| Box::new(BackupPlugin::new())));

        // 固定渠道
        r.register("morn:channel-cli", Box::new(|_| Box::new(CliChannelPlugin)));
        r.register("morn:channel-desktop", Box::new(|_| Box::new(DesktopChannelPlugin)));
        r.register("morn:channel-rest-api", Box::new(|_| Box::new(RestApiChannelPlugin)));

        // feature-gated 渠道
        #[cfg(feature = "channels-full")]
        {
            use super::channel_full::*;
            r.register("morn:channel-telegram", Box::new(|_| Box::new(TelegramChannelPlugin)));
            r.register("morn:channel-wecom", Box::new(|_| Box::new(WecomChannelPlugin)));
            r.register("morn:channel-feishu", Box::new(|_| Box::new(FeishuChannelPlugin)));
            r.register("morn:channel-dingtalk", Box::new(|_| Box::new(DingtalkChannelPlugin)));
            r.register("morn:channel-miniprogram", Box::new(|_| Box::new(MiniprogramChannelPlugin)));
            r.register("morn:channel-qqbot", Box::new(|_| Box::new(QqbotChannelPlugin)));
            r.register("morn:channel-pushplus", Box::new(|_| Box::new(PushplusChannelPlugin)));
            r.register("morn:channel-serverchan", Box::new(|_| Box::new(ServerchanChannelPlugin)));
            r.register("morn:channel-webhook", Box::new(|_| Box::new(WebhookChannelPlugin)));
            r.register("morn:channel-wechat-mp", Box::new(|_| Box::new(WechatMpChannelPlugin)));
            r.register("morn:channel-browser-ext", Box::new(|_| Box::new(BrowserExtChannelPlugin)));
            r.register("morn:channel-smtp", Box::new(|_| Box::new(SmtpChannelPlugin)));
        }
        r
    }

    fn register(&mut self, id: &str, builder: PluginBuilder) {
        self.builders.insert(id.to_string(), builder);
    }

    pub fn register_external(&mut self, id: &str, builder: PluginBuilder) {
        self.builders.insert(id.to_string(), builder);
    }

    /// 按 ID 构造一个插件实例
    pub fn build(&self, id: &str, plugin_dir: PathBuf) -> Option<Box<dyn MornPlugin>> {
        self.builders.get(id).map(|f| f(plugin_dir))
    }

    /// 返回所有已知插件 ID
    pub fn known_ids(&self) -> Vec<String> {
        self.builders.keys().cloned().collect()
    }
}
