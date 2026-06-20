//! BridgePlugin — 外部 Python/JS 插件适配器。
//!
//! BridgePlugin 实现 MornPlugin trait，在 activate() 时扫描 plugins/ 目录，
//! 为每个发现的外部插件创建代理 MornPluginWrapper。
//! 外部脚本通过 stdin/stdout JSON-RPC 与 Bridge 通信。

use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError, PluginManifest};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// BridgePlugin — 外部脚本插件的宿主。
pub struct BridgePlugin {
    /// 外部插件发现目录
    plugin_dir: PathBuf,
    /// 已发现的外部插件列表
    pub external_plugins: Vec<MornPluginWrapper>,
}

/// 外部插件的 MornPlugin 包装
#[derive(Debug, Clone)]
pub struct MornPluginWrapper {
    pub id: String,
    pub deps: Vec<String>,
    pub priority: i32,
    pub dir: PathBuf,
    pub manifest: PluginManifest,
    pub runtime: String,
}

impl BridgePlugin {
    pub fn new(plugin_dir: PathBuf) -> Self {
        BridgePlugin {
            plugin_dir,
            external_plugins: Vec::new(),
        }
    }

    /// 扫描 plugins/ 目录，发现外部插件
    pub fn discover(&mut self) -> Result<Vec<String>, PluginError> {
        let mut discovered = Vec::new();
        if !self.plugin_dir.exists() {
            return Ok(discovered);
        }
        for entry in std::fs::read_dir(&self.plugin_dir)
            .map_err(|e| PluginError::Other(format!("bridge: read dir failed: {}", e)))?
        {
            let entry =
                entry.map_err(|e| PluginError::Other(format!("bridge: entry failed: {}", e)))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&manifest_path)
                .map_err(|e| PluginError::Other(format!("bridge: read manifest failed: {}", e)))?;
            let manifest: PluginManifest = serde_json::from_str(&content)
                .map_err(|e| PluginError::Other(format!("bridge: parse manifest failed: {}", e)))?;

            let name = manifest.name.clone();
            let runtime = if manifest.entry.ends_with(".py") {
                "python"
            } else {
                "js"
            };

            let wrapper = MornPluginWrapper {
                id: format!("external:{}", name),
                deps: manifest.dependencies.clone().unwrap_or_default(),
                priority: 0,
                dir: path,
                manifest,
                runtime: runtime.to_string(),
            };
            self.external_plugins.push(wrapper);
            discovered.push(name);
        }
        Ok(discovered)
    }

    pub fn external_list(&self) -> &[MornPluginWrapper] {
        &self.external_plugins
    }
}

impl MornPlugin for BridgePlugin {
    fn id(&self) -> &str {
        "morn:bridge"
    }

    fn deps(&self) -> Vec<&str> {
        vec!["morn:data-layer", "morn:engine"]
    }

    fn priority(&self) -> i32 {
        130
    }

    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        tracing::info!("bridge: scanning plugins/ at {:?}", self.plugin_dir);
        let discovered = self.discover()?;
        let _bridge = Arc::new(Mutex::new(ctx as *const PluginContext as usize));
        ctx.register("morn:bridge", discovered.len());
        for name in &discovered {
            tracing::info!("bridge: discovered external plugin '{}'", name);
        }
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        for plugin in &self.external_plugins {
            tracing::info!("bridge: activated external plugin '{}'", plugin.id);
        }
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.external_plugins.clear();
        Ok(())
    }
}
