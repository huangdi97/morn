use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub author: Option<String>,
    pub plugin_type: String,
    #[serde(skip)]
    pub typed: Option<PluginType>,
    pub entry: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    #[serde(default)]
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Discovered,
    Loaded,
    Active,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub dir: PathBuf,
    pub runtime_type: String,
}

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

pub fn register_morn_plugin(meta: MornPluginMeta) {
    let mut map = get_plugin_meta().lock().expect("plugin meta lock");
    map.insert(meta.id.clone(), meta);
}

pub fn list_morn_plugin_metas() -> Vec<MornPluginMeta> {
    let map = get_plugin_meta().lock().expect("plugin meta lock");
    map.values().cloned().collect()
}

pub fn toggle_morn_plugin_enabled(id: &str, enabled: bool) -> Result<(), String> {
    let mut map = get_plugin_meta().lock().expect("plugin meta lock");
    let meta = map
        .get_mut(id)
        .ok_or_else(|| format!("Plugin '{}' not found", id))?;
    meta.enabled = enabled;
    Ok(())
}