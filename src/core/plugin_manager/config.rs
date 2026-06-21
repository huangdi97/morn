use std::fs;
use std::path::Path;

/// 插件加载配置，从 plugins.json 读取
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PluginConfig {
    /// 是否加载所有已知核心插件。true 时忽略 enabled 列表。
    #[serde(default = "default_true")]
    pub enable_all: bool,
    /// 显式启用的插件 ID 列表（仅 enable_all=false 时生效）
    #[serde(default)]
    pub enabled: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enable_all: true,
            enabled: Vec::new(),
        }
    }
}

impl PluginConfig {
    /// 从文件加载配置，文件不存在时返回默认（所有插件启用）
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// 计算实际需要加载的插件 ID 列表
    pub fn plugins_to_load(
        &self,
        registry: &crate::core::plugin_manager::CorePluginRegistry,
    ) -> Vec<String> {
        if self.enable_all {
            registry.known_ids()
        } else {
            self.enabled.clone()
        }
    }
}
