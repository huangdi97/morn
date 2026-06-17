//! plugin_manager — Scan, load, activate, and deactivate plugins.
use crate::core::error::MornError;
pub mod error;
pub use error::PluginError;

use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata describing a plugin, typically read from a `manifest.json` file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    /// Display name of the plugin.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Human-readable description of the plugin's purpose.
    pub description: String,
    /// Optional author name or identifier.
    #[serde(default)]
    pub author: Option<String>,
    /// Plugin category (e.g. `"theme"`, `"channel"`, `"tool"`).
    pub plugin_type: String,
    /// Path (relative to the plugin directory) to the entry script.
    pub entry: String,
}

/// The lifecycle state of a plugin within the [`PluginManager`].
#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    /// Plugin has been discovered via a directory scan but not yet validated.
    Discovered,
    /// Manifest has been parsed and the plugin is ready for activation.
    Loaded,
    /// Plugin is currently active (hooks registered, scripts loaded).
    Active,
    /// Plugin encountered an error; the inner string provides details.
    Error(String),
}

/// A loaded plugin with its manifest, current status, and filesystem location.
#[derive(Debug, Clone)]
pub struct Plugin {
    /// Parsed manifest metadata.
    pub manifest: PluginManifest,
    /// Current lifecycle status.
    pub status: PluginStatus,
    /// Absolute path to the plugin's directory on disk.
    pub dir: PathBuf,
    /// Runtime type (e.g. "js", "python").
    pub runtime_type: String,
}

/// Manages discovery, loading, activation, and deactivation of plugins.
///
/// Plugins live in subdirectories under a single root directory. Each plugin
/// must contain a `manifest.json` file that is parsed into a [`PluginManifest`].
#[derive(Debug)]
pub struct PluginManager {
    /// All discovered/loaded/active plugins.
    pub plugins: Vec<Plugin>,
    /// Root directory that is scanned for plugin subdirectories.
    pub plugin_dir: PathBuf,
    /// Cached CSS content for theme plugins, keyed by plugin name.
    pub theme_css: HashMap<String, String>,
}

impl PluginManager {
    /// Creates a new `PluginManager` that scans the given directory.
    pub fn new(plugin_dir: PathBuf) -> Self {
        PluginManager {
            plugins: Vec::new(),
            plugin_dir,
            theme_css: HashMap::new(),
        }
    }

    /// Scans the plugin directory for subdirectories containing `manifest.json`.
    ///
    /// Returns a list of newly discovered plugin names. Plugins that have already
    /// been registered (by name) are skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created (when missing), read,
    /// or if any `manifest.json` is invalid JSON or does not match [`PluginManifest`].
    pub fn scan(&mut self) -> Result<Vec<String>, PluginError> {
        let dir = &self.plugin_dir;
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
            return Ok(Vec::new());
        }
        let mut discovered = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&manifest_path)?;
            let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
                PluginError::InvalidManifest(path.to_string_lossy().to_string(), e.to_string())
            })?;
            let name = manifest.name.clone();
            let entry = manifest.entry.as_str();
            let runtime_type = if entry.ends_with(".py") {
                "python"
            } else {
                "js"
            };
            // Avoid duplicates
            if !self.plugins.iter().any(|p| p.manifest.name == name) {
                self.plugins.push(Plugin {
                    manifest,
                    status: PluginStatus::Discovered,
                    dir: path,
                    runtime_type: runtime_type.to_string(),
                });
                discovered.push(name);
            }
        }
        Ok(discovered)
    }

    /// Transitions a discovered plugin to the `Loaded` status.
    ///
    /// # Errors
    ///
    /// Returns an error if no plugin with the given `name` has been registered
    /// (e.g. `scan()` has not been called or the plugin directory is missing).
    pub fn load(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        if plugin.status == PluginStatus::Discovered {
            plugin.status = PluginStatus::Loaded;
        }
        Ok(())
    }

    /// Activates a loaded (or discovered) plugin, setting its status to `Active`.
    ///
    /// If the plugin is already active this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is in an `Error` state or if it has not
    /// been registered.
    pub fn activate(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        match &plugin.status {
            PluginStatus::Loaded | PluginStatus::Discovered => {
                if plugin.manifest.plugin_type == "theme" {
                    let css_path = plugin.dir.join("theme.css");
                    if css_path.exists() {
                        let css = std::fs::read_to_string(&css_path)?;
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
            PluginStatus::Error(e) => Err(PluginError::Other(format!(
                "Plugin '{}' is in error state: {}",
                name, e
            ))),
        }
    }

    /// Deactivates an active plugin, returning it to `Loaded` status.
    ///
    /// If the plugin is not active, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if no plugin with the given `name` has been registered.
    pub fn deactivate(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.name == name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;
        if plugin.status == PluginStatus::Active {
            plugin.status = PluginStatus::Loaded;
        }
        Ok(())
    }

    /// Returns a slice of all registered plugins.
    pub fn list(&self) -> &[Plugin] {
        &self.plugins
    }

    /// Looks up a plugin by name.
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.manifest.name == name)
    }

    /// Returns the full path to the entry file for a plugin.
    pub fn get_entry_path(&self, name: &str) -> Option<PathBuf> {
        self.plugins
            .iter()
            .find(|p| p.manifest.name == name)
            .map(|p| p.dir.join(&p.manifest.entry))
    }

    /// Returns the cached CSS content for a theme plugin, if any.
    pub fn get_theme_css(&self, name: &str) -> Option<&str> {
        self.theme_css.get(name).map(|s| s.as_str())
    }

    /// Returns all plugins whose `plugin_type` is `"theme"`.
    pub fn list_themes(&self) -> Vec<&Plugin> {
        self.plugins
            .iter()
            .filter(|p| p.manifest.plugin_type == "theme")
            .collect()
    }

    pub fn load_plugin_sandboxed(&self, path: &str) -> Result<(), MornError> {
        let manifest_path = format!("{}/manifest.json", path);
        if !std::path::Path::new(&manifest_path).exists() {
            return Err(MornError::Internal(format!(
                "Manifest not found: {}",
                manifest_path
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_plugin_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let plugin_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        (dir, plugin_dir)
    }

    fn write_manifest(dir: &std::path::Path, name: &str, plugin_type: &str) {
        let manifest = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "description": format!("Test {}", name),
            "author": "Morn Labs",
            "plugin_type": plugin_type,
            "entry": "main.js"
        });
        std::fs::create_dir_all(dir.join(name)).unwrap();
        std::fs::write(
            dir.join(name).join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_scan_finds_manifests() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        write_manifest(&plugin_dir, "theme-alpha", "theme");
        write_manifest(&plugin_dir, "channel-beta", "channel");
        let mut mgr = PluginManager::new(plugin_dir);
        let discovered = mgr.scan().unwrap();
        assert_eq!(discovered.len(), 2);
        assert!(discovered.contains(&"theme-alpha".to_string()));
        assert!(discovered.contains(&"channel-beta".to_string()));
    }

    #[test]
    fn test_load_valid_manifest() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        write_manifest(&plugin_dir, "test-plugin", "theme");
        let mut mgr = PluginManager::new(plugin_dir);
        mgr.scan().unwrap();
        mgr.load("test-plugin").unwrap();
        let p = mgr.get("test-plugin").unwrap();
        assert_eq!(p.status, PluginStatus::Loaded);
    }

    #[test]
    fn test_activate_deactivate_cycle() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        write_manifest(&plugin_dir, "cycle-plugin", "channel");
        let mut mgr = PluginManager::new(plugin_dir);
        mgr.scan().unwrap();
        mgr.load("cycle-plugin").unwrap();
        mgr.activate("cycle-plugin").unwrap();
        assert_eq!(
            mgr.get("cycle-plugin").unwrap().status,
            PluginStatus::Active
        );
        mgr.deactivate("cycle-plugin").unwrap();
        assert_eq!(
            mgr.get("cycle-plugin").unwrap().status,
            PluginStatus::Loaded
        );
    }

    #[test]
    fn test_load_nonexistent_plugin() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        let mut mgr = PluginManager::new(plugin_dir);
        let result = mgr.load("ghost");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_empty_dir() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        let mut mgr = PluginManager::new(plugin_dir);
        let discovered = mgr.scan().unwrap();
        assert!(discovered.is_empty());
    }

    #[test]
    fn test_scan_ignores_dirs_without_manifest() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        std::fs::create_dir_all(plugin_dir.join("no-manifest")).unwrap();
        let mut mgr = PluginManager::new(plugin_dir);
        let discovered = mgr.scan().unwrap();
        assert!(discovered.is_empty());
    }

    #[test]
    fn test_activate_theme_loads_css() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        let plugin_name = "dark-theme";
        let plugin_path = plugin_dir.join(plugin_name);
        std::fs::create_dir_all(&plugin_path).unwrap();

        let manifest = serde_json::json!({
            "name": plugin_name,
            "version": "1.0.0",
            "description": "A dark theme",
            "author": "Morn Labs",
            "plugin_type": "theme",
            "entry": "main.js"
        });
        std::fs::write(
            plugin_path.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let css_content = "body { background: #000; color: #fff; }";
        std::fs::write(plugin_path.join("theme.css"), css_content).unwrap();

        let mut mgr = PluginManager::new(plugin_dir);
        mgr.scan().unwrap();
        mgr.activate(plugin_name).unwrap();

        assert_eq!(mgr.get_theme_css(plugin_name), Some(css_content));
    }

    #[test]
    fn test_get_entry_path_returns_correct_path() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        write_manifest(&plugin_dir, "entry-test", "tool");
        let plugin_dir_clone = plugin_dir.clone();
        let mut mgr = PluginManager::new(plugin_dir);
        mgr.scan().unwrap();

        let entry = mgr.get_entry_path("entry-test");
        assert!(entry.is_some());
        let expected = plugin_dir_clone.join("entry-test").join("main.js");
        assert_eq!(entry.unwrap(), expected);
    }

    #[test]
    fn test_list_themes_returns_only_themes() {
        let (_tmp, plugin_dir) = temp_plugin_dir();
        write_manifest(&plugin_dir, "theme-one", "theme");
        write_manifest(&plugin_dir, "theme-two", "theme");
        write_manifest(&plugin_dir, "channel-one", "channel");

        let mut mgr = PluginManager::new(plugin_dir);
        mgr.scan().unwrap();

        let themes = mgr.list_themes();
        assert_eq!(themes.len(), 2);
        assert!(themes.iter().any(|p| p.manifest.name == "theme-one"));
        assert!(themes.iter().any(|p| p.manifest.name == "theme-two"));
        assert!(!themes.iter().any(|p| p.manifest.name == "channel-one"));
    }
}
