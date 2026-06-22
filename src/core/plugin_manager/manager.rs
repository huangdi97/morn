use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::error::MornError;

use super::error::PluginError;
use super::types::{Plugin, PluginManifest, PluginStatus, PluginType};

#[derive(Debug)]
pub struct PluginManager {
    pub plugins: Vec<Plugin>,
    pub plugin_dir: PathBuf,
    pub theme_css: HashMap<String, String>,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        PluginManager {
            plugins: Vec::new(),
            plugin_dir,
            theme_css: HashMap::new(),
        }
    }

    pub fn scan(&mut self) -> Result<Vec<String>, PluginError> {
        let dir = &self.plugin_dir;
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .inspect_err(|e| tracing::error!("[plugin_manager] scan create_dir_all: {e}"))?;
            return Ok(Vec::new());
        }
        let mut discovered = Vec::new();
        for entry in std::fs::read_dir(dir)
            .inspect_err(|e| tracing::error!("[plugin_manager] scan read_dir: {e}"))?
        {
            let entry =
                entry.inspect_err(|e| tracing::error!("[plugin_manager] scan entry: {e}"))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&manifest_path)
                .inspect_err(|e| tracing::error!("[plugin_manager] scan read manifest: {e}"))?;
            let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
                tracing::error!("[plugin_manager] scan parse manifest: {e}");
                PluginError::InvalidManifest(path.to_string_lossy().to_string(), e.to_string())
            })?;
            let name = manifest.name.clone();
            let entry = manifest.entry.as_str();
            let runtime_type = if entry.ends_with(".py") {
                "python"
            } else {
                "js"
            };
            let typed = match manifest.plugin_type.as_str() {
                "theme" => Some(PluginType::Theme),
                "channel" => Some(PluginType::Channel),
                "tool" => Some(PluginType::Tool),
                "knowledge" => Some(PluginType::Knowledge),
                "ui_panel" => Some(PluginType::UiPanel),
                "protocol" => Some(PluginType::Protocol),
                _ => None,
            };
            if !self.plugins.iter().any(|p| p.manifest.name == name) {
                let mut plugin = Plugin {
                    manifest,
                    status: PluginStatus::Discovered,
                    dir: path,
                    runtime_type: runtime_type.to_string(),
                };
                plugin.manifest.typed = typed;
                self.plugins.push(plugin);
                discovered.push(name);
            }
        }
        Ok(discovered)
    }

    pub fn load(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| {
                tracing::error!("[plugin_manager] load plugin not found: {}", name);
                PluginError::NotFound(name.to_string())
            })?;
        if plugin.status == PluginStatus::Discovered {
            plugin.status = PluginStatus::Loaded;
        }
        Ok(())
    }

    pub fn activate(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| {
                tracing::error!("[plugin_manager] activate plugin not found: {}", name);
                PluginError::NotFound(name.to_string())
            })?;
        match &plugin.status {
            PluginStatus::Loaded | PluginStatus::Discovered => {
                if plugin.manifest.plugin_type == "theme" {
                    let css_path = plugin.dir.join("theme.css");
                    if css_path.exists() {
                        let css = std::fs::read_to_string(&css_path).inspect_err(|e| {
                            tracing::error!("[plugin_manager] activate read theme.css: {e}")
                        })?;
                        self.theme_css.insert(name.to_string(), css);
                    }
                }
                if plugin.manifest.entry.ends_with(".py") {
                    plugin.runtime_type = "python".to_string();
                } else {
                    plugin.runtime_type = "js".to_string();
                }
                plugin.status = PluginStatus::Active;
                Ok(())
            }
            PluginStatus::Active => Ok(()),
            PluginStatus::Error(e) => {
                tracing::error!(
                    "[plugin_manager] activate plugin '{}' in error state: {}",
                    name,
                    e
                );
                Err(PluginError::Other(format!(
                    "Plugin '{}' is in error state: {}",
                    name, e
                )))
            }
        }
    }

    pub fn deactivate(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| {
                tracing::error!("[plugin_manager] deactivate plugin not found: {}", name);
                PluginError::NotFound(name.to_string())
            })?;
        if plugin.status == PluginStatus::Active {
            plugin.status = PluginStatus::Loaded;
        }
        Ok(())
    }

    pub fn list(&self) -> &[Plugin] {
        &self.plugins
    }

    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.manifest.name == name)
    }

    pub fn get_entry_path(&self, name: &str) -> Option<PathBuf> {
        self.plugins
            .iter()
            .find(|p| p.manifest.name == name)
            .map(|p| p.dir.join(&p.manifest.entry))
    }

    pub fn get_theme_css(&self, name: &str) -> Option<&str> {
        self.theme_css.get(name).map(|s| s.as_str())
    }

    pub fn list_themes(&self) -> Vec<&Plugin> {
        self.plugins
            .iter()
            .filter(|p| p.manifest.plugin_type == "theme")
            .collect()
    }

    pub fn load_plugin_sandboxed(&self, path: &str) -> Result<(), MornError> {
        let manifest_path = format!("{}/manifest.json", path);
        if !std::path::Path::new(&manifest_path).exists() {
            tracing::error!(
                "[plugin_manager] load_plugin_sandboxed manifest not found: {}",
                manifest_path
            );
            return Err(MornError::Internal(format!(
                "Manifest not found: {}",
                manifest_path
            )));
        }
        Ok(())
    }
}
