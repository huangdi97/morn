use crate::AppState;
use crate::MornError;
use std::path::Path;
use tauri::State;

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
