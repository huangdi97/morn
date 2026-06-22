//! WasmPlugin — 加载 .wasm 文件作为 MornPlugin
//!
//! 通过 wasmtime 运行时加载 wasm 插件，插件需导出以下函数：
//! - `morn_init` (无参数，无返回值)
//! - `morn_activate` (无参数，无返回值)
//! - `morn_deactivate` (无参数，无返回值)
//!
//! 使用 `#[cfg(feature = "sandbox")]` 控制编译（sandbox 特性已包含 wasmtime 依赖）。

use crate::core::plugin_manager::{MornPlugin, PluginContext, PluginError};

#[cfg(feature = "sandbox")]
use crate::sandbox::wasm::Sandbox;

#[cfg(feature = "sandbox")]
use std::sync::OnceLock;

#[cfg(feature = "sandbox")]
static SANDBOX: OnceLock<Sandbox> = OnceLock::new();

#[cfg(feature = "sandbox")]
fn get_sandbox() -> &'static Sandbox {
    SANDBOX.get_or_init(|| Sandbox::new().expect("failed to create sandbox"))
}

/// 一个从 .wasm 文件加载的 MornPlugin 包装
pub struct WasmPlugin {
    id: String,
    _wasm_path: String,
}

impl WasmPlugin {
    /// 从 .wasm 文件路径创建 WasmPlugin
    ///
    /// 文件名的 stem 作为插件 id: `my-plugin.wasm` → `my-plugin`
    pub fn new(path: &str) -> Self {
        let id = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        WasmPlugin {
            id,
            _wasm_path: path.to_string(),
        }
    }
}

impl MornPlugin for WasmPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn deps(&self) -> Vec<&str> {
        // TODO: 从 wasm custom section 或 metadata 读取 deps
        vec![]
    }

    fn priority(&self) -> i32 {
        // TODO: 从 wasm custom section 读取 priority
        0
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        #[cfg(feature = "sandbox")]
        {
            let wasm_bytes = std::fs::read(&self._wasm_path)
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm init read {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
            get_sandbox()
                .execute_func(&wasm_bytes, "morn_init")
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm init exec {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
        }
        Ok(())
    }

    fn activate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        #[cfg(feature = "sandbox")]
        {
            let wasm_bytes = std::fs::read(&self._wasm_path)
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm activate read {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
            get_sandbox()
                .execute_func(&wasm_bytes, "morn_activate")
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm activate exec {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
        }
        Ok(())
    }

    fn deactivate(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        #[cfg(feature = "sandbox")]
        {
            let wasm_bytes = std::fs::read(&self._wasm_path)
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm deactivate read {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
            get_sandbox()
                .execute_func(&wasm_bytes, "morn_deactivate")
                .map_err(|e| {
                    tracing::error!("[plugin_manager] wasm deactivate exec {}: {}", self.id, e);
                    PluginError::LoadFailed(self.id.clone(), e.to_string())
                })?;
        }
        Ok(())
    }
}

/// 从目录扫描 .wasm 文件并加载为 WasmPlugin
///
/// 只返回有效文件（存在、扩展名是 .wasm），不校验 wasm 内容
pub fn load_wasm_plugins_from_dir(dir: &std::path::Path) -> Vec<Box<dyn MornPlugin>> {
    let mut plugins: Vec<Box<dyn MornPlugin>> = Vec::new();
    if !dir.exists() {
        return plugins;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "wasm").unwrap_or(false) {
                plugins.push(Box::new(WasmPlugin::new(path.to_string_lossy().as_ref())));
            }
        }
    }
    plugins
}

/// 非 sandbox 特性下的桩（空）
#[cfg(not(feature = "sandbox"))]
pub fn load_wasm_plugins_from_dir(_dir: &std::path::Path) -> Vec<Box<dyn MornPlugin>> {
    Vec::new()
}
