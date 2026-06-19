//! plugin_manager — Scan, load, activate, and deactivate plugins.
use crate::core::error::MornError;
pub mod error;
pub use error::PluginError;
pub use error::PluginOrderError;

pub mod adapter;
pub mod bridge_plugin;
pub mod plugins;
#[cfg(feature = "sandbox")]
pub mod wasm_plugin;
pub use plugins::CorePluginRegistry;
pub mod config;
pub use config::PluginConfig;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Plugin category enum matching DESIGN.md §11.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PluginType {
    Theme,
    Channel,
    Tool,
    Knowledge,
    UiPanel,
    Protocol,
}

impl PluginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginType::Theme => "theme",
            PluginType::Channel => "channel",
            PluginType::Tool => "tool",
            PluginType::Knowledge => "knowledge",
            PluginType::UiPanel => "ui_panel",
            PluginType::Protocol => "protocol",
        }
    }
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Shared context that plugins can use to access registered services.
///
/// Services are registered by key and accessed by type. The `get` method
/// returns an owned clone because the underlying `Any` type cannot yield
/// references in a generic way.
pub struct PluginContext {
    services: RefCell<HashMap<&'static str, Box<dyn Any + Send + Sync>>>,
}

impl PluginContext {
    pub fn new() -> Self {
        PluginContext {
            services: RefCell::new(HashMap::new()),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&self, key: &'static str, val: T) {
        self.services.borrow_mut().insert(key, Box::new(val));
    }

    pub fn get<T: 'static + Clone>(&self, key: &str) -> Option<T> {
        self.services
            .borrow()
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    pub fn unregister(&self, key: &'static str) {
        self.services.borrow_mut().remove(key);
    }
}

/// Trait that every Morn plugin **with a typed PluginType** must implement.
///
/// This is the DESIGN.md §11 interface, kept alongside the existing
/// [`MornPlugin`] trait for backward compatibility (bridge pattern).
pub trait TypedPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn version(&self) -> &str;
    fn load(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn deactivate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn unload(&self) -> Result<(), PluginError>;
    fn hooks(&self) -> Vec<&str> {
        vec![]
    }
}

/// Trait that every Morn plugin must implement.
///
/// Provides lifecycle hooks (`init`, `activate`, `deactivate`) and metadata
/// (`id`, `deps`, `priority`) used by [`topological_sort`] and [`load_plugins`].
pub trait MornPlugin: Send + Sync {
    /// Unique identifier for this plugin.
    fn id(&self) -> &str;

    /// IDs of plugins this plugin depends on.
    fn deps(&self) -> Vec<&str> {
        vec![]
    }

    /// Higher values = loaded / activated first.
    fn priority(&self) -> i32 {
        0
    }

    /// Initialization phase (all plugins init before any activate).
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Activation phase (run in dependency order after all inits succeed).
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Deactivation / teardown.
    fn deactivate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
}

/// Kahn's algorithm — returns indices into `plugins` sorted so every plugin
/// appears after its dependencies. Higher-priority plugins are preferred when
/// multiple nodes have no remaining dependencies.
///
/// # Errors
///
/// Returns [`PluginOrderError`] when a dependency is missing or a cycle is
/// detected.
pub fn topological_sort(plugins: &[Box<dyn MornPlugin>]) -> Result<Vec<usize>, PluginOrderError> {
    let n = plugins.len();
    let mut in_degree = vec![0; n];
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

    let id_to_idx: HashMap<&str, usize> = plugins
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id(), i))
        .collect();

    for (i, plugin) in plugins.iter().enumerate() {
        for dep in plugin.deps() {
            match id_to_idx.get(dep) {
                Some(&j) => {
                    adj.entry(j).or_default().push(i);
                    in_degree[i] += 1;
                }
                None => {
                    return Err(PluginOrderError::MissingDependency(
                        plugin.id().to_string(),
                        dep.to_string(),
                    ));
                }
            }
        }
    }

    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result = Vec::with_capacity(n);

    while let Some(node) = queue.pop_front() {
        result.push(node);
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    if result.len() != n {
        let cycle: Vec<String> = (0..n)
            .filter(|&i| in_degree[i] > 0)
            .map(|i| plugins[i].id().to_string())
            .collect();
        return Err(PluginOrderError::CycleDetected(cycle));
    }

    Ok(result)
}

/// Two-phase plugin loading:
/// 1. Call `init` on every plugin (order independent).
/// 2. Topologically sort plugins, then call `activate` in dependency order.
///
/// # Errors
///
/// Returns [`PluginError::LoadFailed`] if any `init` fails,  
/// [`PluginError::ActivateFailed`] if any `activate` fails, or
/// [`PluginOrderError`] converted to [`PluginError::OrderError`] on dependency
/// issues.
pub fn load_plugins(
    plugins: &mut [Box<dyn MornPlugin>],
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    for plugin in plugins.iter_mut() {
        safe_init(plugin.as_mut(), ctx)?;
    }

    let order = topological_sort(plugins).map_err(|e| PluginError::OrderError(e.to_string()))?;

    for &i in &order {
        safe_activate(plugins[i].as_mut(), ctx)?;
    }

    Ok(())
}

/// Safe init a plugin — uses catch_unwind to isolate crashes.
pub fn safe_init(plugin: &mut dyn MornPlugin, ctx: &PluginContext) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.init(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            PluginError::LoadFailed(id, msg)
        },
    )?
}

/// Safe activate a plugin — uses catch_unwind to isolate crashes.
pub fn safe_activate(plugin: &mut dyn MornPlugin, ctx: &PluginContext) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.activate(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            PluginError::ActivateFailed(id, msg)
        },
    )?
}

/// Safe deactivate a plugin — uses catch_unwind to isolate crashes.
pub fn safe_deactivate(
    plugin: &mut dyn MornPlugin,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.deactivate(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            PluginError::Other(format!("{} deactivate panicked: {}", id, msg))
        },
    )?
}

/// Enable a plugin at runtime — calls init then activate.
pub fn enable_plugin(
    plugin: &mut Box<dyn MornPlugin>,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    safe_init(plugin.as_mut(), ctx)?;
    safe_activate(plugin.as_mut(), ctx)?;
    Ok(())
}

/// Disable a plugin at runtime — calls deactivate.
pub fn disable_plugin(
    plugin: &mut Box<dyn MornPlugin>,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    safe_deactivate(plugin.as_mut(), ctx)
}

/// Metadata for a [`MornPlugin`] registered in the global plugin registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MornPluginMeta {
    pub id: String,
    pub deps: Vec<String>,
    pub priority: i32,
    pub enabled: bool,
}

static MORN_PLUGIN_META: OnceLock<Mutex<HashMap<String, MornPluginMeta>>> = OnceLock::new();

fn get_plugin_meta() -> &'static Mutex<HashMap<String, MornPluginMeta>> {
    MORN_PLUGIN_META.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a [`MornPlugin`]'s metadata into the global registry.
pub fn register_morn_plugin(meta: MornPluginMeta) {
    let mut map = get_plugin_meta().lock().expect("plugin meta lock");
    map.insert(meta.id.clone(), meta);
}

/// List all registered [`MornPlugin`] metadata entries.
pub fn list_morn_plugin_metas() -> Vec<MornPluginMeta> {
    let map = get_plugin_meta().lock().expect("plugin meta lock");
    map.values().cloned().collect()
}

/// Toggle the enabled flag of a registered [`MornPlugin`].
pub fn toggle_morn_plugin_enabled(id: &str, enabled: bool) -> Result<(), String> {
    let mut map = get_plugin_meta().lock().expect("plugin meta lock");
    let meta = map
        .get_mut(id)
        .ok_or_else(|| format!("Plugin '{}' not found", id))?;
    meta.enabled = enabled;
    Ok(())
}

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
    /// Plugin category (e.g. `PluginType::Tool`, `PluginType::Theme`).
    pub plugin_type: String,
    /// Typed plugin category parsed from `plugin_type` string.
    #[serde(skip)]
    pub typed: Option<PluginType>,
    /// Path (relative to the plugin directory) to the entry script.
    pub entry: String,
    /// Optional icon path or URL.
    #[serde(default)]
    pub icon: Option<String>,
    /// Optional homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Optional source repository URL.
    #[serde(default)]
    pub repository: Option<String>,
    /// Optional list of required permissions (e.g. ['network', 'filesystem']).
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    /// Optional list of dependency plugin names.
    #[serde(default)]
    pub dependencies: Option<Vec<String>>,
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
            let typed = match manifest.plugin_type.as_str() {
                "theme" => Some(PluginType::Theme),
                "channel" => Some(PluginType::Channel),
                "tool" => Some(PluginType::Tool),
                "knowledge" => Some(PluginType::Knowledge),
                "ui_panel" => Some(PluginType::UiPanel),
                "protocol" => Some(PluginType::Protocol),
                _ => None,
            };
            // Avoid duplicates
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
