//! MornPlugin → PluginManager 适配器。
//! 把 Box<dyn MornPlugin> 包装成 PluginManager 的 Plugin 结构体。

use crate::core::plugin_manager::{MornPlugin, Plugin, PluginManifest, PluginStatus};
use std::path::PathBuf;

/// 将 MornPlugin 转换为 PluginManager 可管理的 Plugin。
pub fn morn_plugin_to_plugin(plugin: &dyn MornPlugin) -> Plugin {
    let plugin_type = match plugin.id() {
        "morn:data-layer" => "storage",
        "morn:registry" => "tool",
        "morn:sandbox" => "tool",
        "morn:engine" => "tool",
        "morn:channel-bus" => "channel",
        "morn:supervisor" => "tool",
        "morn:studio" => "tool",
        "morn:bridge" => "bridge",
        _ => "tool",
    };
    Plugin {
        manifest: PluginManifest {
            name: plugin.id().to_string(),
            version: "built-in".to_string(),
            description: format!("Built-in Morn plugin: {}", plugin.id()),
            author: Some("Morn".to_string()),
            plugin_type: plugin_type.to_string(),
            typed: None,
            entry: String::new(),
            icon: None,
            homepage: None,
            repository: None,
            permissions: None,
            dependencies: Some(plugin.deps().into_iter().map(String::from).collect()),
        },
        status: PluginStatus::Active,
        dir: PathBuf::from("built-in"),
        runtime_type: "rust".to_string(),
    }
}
