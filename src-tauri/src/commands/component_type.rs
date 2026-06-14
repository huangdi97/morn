use crate::AppState;
use tauri::State;

use morn::core::component_type::def::ComponentTypeDef;
use morn::core::component_type::registry::TypeRegistry;

#[tauri::command]
pub(crate) fn register_component_type(
    type_name: String,
    interfaces: Vec<String>,
    config_schema: Option<serde_json::Value>,
    implements: Vec<String>,
    author: Option<String>,
    version: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let mut registry = state.type_registry.lock().map_err(|e| e.to_string())?;
    let def = ComponentTypeDef {
        type_name,
        interfaces,
        config_schema: config_schema.unwrap_or(serde_json::json!({})),
        implements,
        author: author.unwrap_or_else(|| "user".to_string()),
        version: version.unwrap_or_else(|| "0.1.0".to_string()),
    };
    registry.register(def)?;
    Ok(serde_json::json!({ "status": "ok" }))
}

#[tauri::command]
pub(crate) fn unregister_component_type(
    type_name: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let mut registry = state.type_registry.lock().map_err(|e| e.to_string())?;
    let removed = registry.unregister(&type_name);
    if removed {
        Ok(serde_json::json!({ "status": "removed" }))
    } else {
        Err(format!("type '{}' not found or is a built-in", type_name))
    }
}

#[tauri::command]
pub(crate) fn list_component_types(state: State<AppState>) -> Result<serde_json::Value, String> {
    let registry = state.type_registry.lock().map_err(|e| e.to_string())?;
    let types: Vec<&ComponentTypeDef> = registry.list();
    Ok(serde_json::to_value(types).map_err(|e| e.to_string())?)
}
