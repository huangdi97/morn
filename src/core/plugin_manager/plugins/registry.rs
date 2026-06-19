use std::collections::HashMap;
use std::path::PathBuf;
use super::super::MornPlugin;
use super::{
    ChannelBusPlugin, DataLayerPlugin, EnginePlugin, RegistryPlugin,
    SandboxPlugin, StudioPlugin, SupervisorPlugin,
};
use super::super::bridge_plugin::BridgePlugin;

/// 内部插件注册表 — 知道所有核心插件的构造函数
pub struct CorePluginRegistry {
    builders: HashMap<&'static str, fn(PathBuf) -> Box<dyn MornPlugin>>,
}

impl CorePluginRegistry {
    /// 注册所有内置插件
    pub fn new() -> Self {
        let mut r = Self { builders: HashMap::new() };
        r.register("morn:data-layer", |_| Box::new(DataLayerPlugin(None)));
        r.register("morn:registry", |_| Box::new(RegistryPlugin(None)));
        r.register("morn:sandbox", |_| Box::new(SandboxPlugin(None)));
        r.register("morn:engine", |_| Box::new(EnginePlugin(None)));
        r.register("morn:channel-bus", |_| Box::new(ChannelBusPlugin(None)));
        r.register("morn:supervisor", |_| Box::new(SupervisorPlugin(None)));
        r.register("morn:studio", |_| Box::new(StudioPlugin(None, None)));
        r.register("morn:bridge", |p| Box::new(BridgePlugin::new(p)));
        r
    }

    fn register(&mut self, id: &'static str, builder: fn(PathBuf) -> Box<dyn MornPlugin>) {
        self.builders.insert(id, builder);
    }

    /// 按 ID 构造一个插件实例
    pub fn build(&self, id: &str, plugin_dir: PathBuf) -> Option<Box<dyn MornPlugin>> {
        self.builders.get(id).map(|f| f(plugin_dir))
    }

    /// 返回所有已知插件 ID
    pub fn known_ids(&self) -> Vec<&'static str> {
        self.builders.keys().copied().collect()
    }
}
