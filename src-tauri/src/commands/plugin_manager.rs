use crate::AppState;
use crate::MornError;
use std::path::Path;
use tauri::State;

use morn::bridge::chat_agent::ChatAgent;
use morn::core::plugin_generator;
use morn::core::plugin_manager::{MornPluginMeta, Plugin, PluginStatus};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct PluginEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub plugin_type: String,
    pub status: String,
}

impl From<&Plugin> for PluginEntry {
    fn from(p: &Plugin) -> Self {
        PluginEntry {
            name: p.manifest.name.clone(),
            version: p.manifest.version.clone(),
            description: p.manifest.description.clone(),
            author: p.manifest.author.clone(),
            plugin_type: p.manifest.plugin_type.clone(),
            status: match p.status {
                PluginStatus::Discovered => "discovered",
                PluginStatus::Loaded => "loaded",
                PluginStatus::Active => "active",
                PluginStatus::Error(_) => "error",
            }
            .to_string(),
        }
    }
}

#[tauri::command]
pub(crate) fn list_plugins(state: State<AppState>) -> Result<Vec<PluginEntry>, MornError> {
    let plugin_manager = state
        .plugin_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = plugin_manager
        .as_ref()
        .ok_or_else(|| MornError::Internal("PluginManager not initialized".to_string()))?;
    Ok(mgr.list().iter().map(PluginEntry::from).collect())
}

#[tauri::command]
pub(crate) fn toggle_plugin(
    name: String,
    enable: bool,
    state: State<AppState>,
) -> Result<(), MornError> {
    let mut plugin_manager = state
        .plugin_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = plugin_manager
        .as_mut()
        .ok_or_else(|| MornError::Internal("PluginManager not initialized".to_string()))?;

    if enable {
        // First ensure it's loaded
        let _ = mgr.load(&name);
        mgr.activate(&name)
            .map_err(|e| MornError::Internal(e.to_string()))?;
    } else {
        mgr.deactivate(&name)
            .map_err(|e| MornError::Internal(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn plugin_install(path: String, state: State<AppState>) -> Result<(), MornError> {
    let src = Path::new(&path);
    let src = std::fs::canonicalize(src)
        .map_err(|e| MornError::Internal(format!("Failed to resolve path '{}': {}", path, e)))?;

    if !src.is_dir() {
        return Err(format!("Path '{}' is not a directory", src.display()).into());
    }

    let manifest_path = src.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("No manifest.json found in '{}'", src.display()).into());
    }

    let plugin_dir = {
        let plugin_manager = state
            .plugin_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mgr = plugin_manager
            .as_ref()
            .ok_or_else(|| "PluginManager not initialized".to_string())?;
        mgr.plugin_dir.clone()
    };

    let dir_name = src
        .file_name()
        .ok_or_else(|| format!("Cannot determine directory name from '{}'", src.display()))?;
    let target = plugin_dir.join(dir_name);

    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }

    copy_dir_recursive(&src, &target).map_err(|e| {
        format!(
            "Failed to copy plugin from '{}' to '{}': {}",
            src.display(),
            target.display(),
            e
        )
    })?;

    {
        let mut plugin_manager = state
            .plugin_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(mgr) = plugin_manager.as_mut() {
            mgr.scan().map_err(|e| MornError::Internal(e.to_string()))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn list_morn_plugins() -> Result<Vec<MornPluginMeta>, MornError> {
    Ok(morn::core::plugin_manager::list_morn_plugin_metas())
}

#[tauri::command]
pub(crate) fn toggle_morn_plugin(id: String, enabled: bool) -> Result<(), MornError> {
    morn::core::plugin_manager::toggle_morn_plugin_enabled(&id, enabled)
        .map_err(|e| MornError::Internal(e))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn create_plugin_from_spec(
    description: String,
    state: State<'_, AppState>,
) -> Result<PluginEntry, MornError> {
    let api_key = std::env::var("MORN_API_KEY")
        .map_err(|_| MornError::Internal("MORN_API_KEY not set".to_string()))?;
    let chat_agent = ChatAgent::new(&api_key, "https://api.deepseek.com", "deepseek-chat");

    let plugin_dir = {
        let plugin_manager = state
            .plugin_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let mgr = plugin_manager
            .as_ref()
            .ok_or_else(|| MornError::Internal("PluginManager not initialized".to_string()))?;
        mgr.plugin_dir.clone()
    };

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);
    let spec = plugin_generator::parse_nl_to_spec(&description, &chat_fn)
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let manifest_path = plugin_generator::scaffold_plugin(&spec, &plugin_dir)
        .map_err(|e| MornError::Internal(e.to_string()))?;

    {
        let mut plugin_manager = state
            .plugin_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(mgr) = plugin_manager.as_mut() {
            let _ = mgr.scan();
        }
    }

    Ok(PluginEntry {
        name: spec.name.clone(),
        version: "0.1.0".to_string(),
        description: spec.description,
        author: None,
        plugin_type: spec.plugin_type,
        status: "active".to_string(),
    })
}
