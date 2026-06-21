use crate::AppState;
use crate::MornError;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use morn::bridge::chat_agent::ChatAgent;
use morn::core::plugin_generator;
use morn::market::{Listing, Marketplace};
use morn::studio::manager::CreateComponentDef;

#[tauri::command]
pub(crate) fn list_themes(state: State<AppState>) -> Result<Vec<String>, MornError> {
    let mgr = state
        .plugin_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = mgr
        .as_ref()
        .ok_or_else(|| "PluginManager not initialized".to_string())?;
    Ok(mgr
        .list_themes()
        .iter()
        .map(|p| p.manifest.name.clone())
        .collect())
}

#[tauri::command]
pub(crate) fn apply_theme(name: String, state: State<AppState>) -> Result<String, MornError> {
    let mut mgr = state
        .plugin_manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = mgr
        .as_mut()
        .ok_or_else(|| "PluginManager not initialized".to_string())?;
    mgr.activate(&name)
        .map_err(|e| MornError::Internal(e.to_string()))?;
    mgr.get_theme_css(&name)
        .map(|s| s.to_string())
        .ok_or_else(|| MornError::Internal(format!("No CSS cached for theme '{}'", name)))
}

#[tauri::command]
pub(crate) fn get_market_listings(
    type_filter: Option<String>,
    price_filter: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let listings = marketplace.list(type_filter.as_deref());
    let filtered = match price_filter.as_deref() {
        Some("free") => listings
            .into_iter()
            .filter(|l| l.price == Some(0.0))
            .collect(),
        Some("paid") => listings
            .into_iter()
            .filter(|l| l.price > Some(0.0))
            .collect(),
        _ => listings,
    };
    serde_json::to_value(filtered).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn list_bot_store(state: State<AppState>) -> Result<Vec<serde_json::Value>, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let marketplace = Marketplace::new(s.clone());
    let listings = marketplace.list(None);

    let bot_listings: Vec<serde_json::Value> = listings
        .iter()
        .map(|l| {
            serde_json::json!({
                "id": l.id,
                "name": l.name,
                "icon": "🤖",
                "description": l.description,
                "category": l.category,
                "rating": l.rating,
                "installs": l.downloads,
                "author": l.author,
                "price": l.price,
                "template_id": format!("preset-{}", l.name.to_lowercase().replace(' ', "-")),
            })
        })
        .collect();

    Ok(bot_listings)
}

#[tauri::command]
pub(crate) fn install_bot_from_store(
    bot_id: String,
    template_id: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let mgr = mgr.lock().map_err(|e| MornError::Internal(e.to_string()))?;

    let name = template_id
        .strip_prefix("preset-")
        .unwrap_or(&template_id)
        .replace('-', " ")
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let id = mgr.create_component(CreateComponentDef {
        name,
        component_type: "agent".to_string(),
        config_json: Some(
            serde_json::json!({"template_id": template_id, "bot_id": bot_id}).to_string(),
        ),
    })?;
    Ok(id)
}

#[tauri::command]
pub(crate) fn hub_publish(
    name: String,
    description: String,
    item_type: String,
    price: f64,
    author: String,
    version: String,
    screenshots: String,
    category: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let id = format!("hub-{}", uuid::Uuid::new_v4());
    let listing = Listing {
        id: id.clone(),
        item_type,
        name,
        description,
        price: Some(price),
        price_model: String::new(),
        requires: vec![],
        verified: false,
        updated_at: chrono::Utc::now().to_rfc3339(),
        author,
        rating: 0.0,
        downloads: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        version,
        screenshots,
        category,
    };
    marketplace.publish(listing)?;
    Ok(id)
}

#[tauri::command]
pub(crate) fn get_preset_persona(name: String) -> Result<serde_json::Value, MornError> {
    match morn::component::persona::get_preset_persona(&name) {
        Some(persona) => {
            serde_json::to_value(persona).map_err(|e| MornError::Internal(e.to_string()))
        }
        None => Err(format!("Preset persona '{}' not found", name).into()),
    }
}

#[tauri::command]
pub(crate) fn list_preset_personas() -> Vec<std::collections::HashMap<String, String>> {
    morn::component::persona::list_preset_personas()
}

#[tauri::command]
pub(crate) fn create_agent_from_description(
    nl: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    let api_key = std::env::var("MORN_API_KEY")
        .map_err(|_| MornError::Internal("MORN_API_KEY not set".to_string()))?;
    let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
        &api_key,
        "https://api.deepseek.com",
        "deepseek-chat",
    );

    let supervisor = state
        .supervisor
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let sup = supervisor
        .as_ref()
        .ok_or_else(|| "Supervisor not initialized.".to_string())?;
    let sup = sup.lock().map_err(|e| MornError::Internal(e.to_string()))?;

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);
    let nl_def = sup.create_team_from_nl(&nl, &chat_fn)?;
    serde_json::to_string(&nl_def).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn get_agent_versions(
    listing_id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let versions = marketplace.get_version_history(&listing_id);
    serde_json::to_value(versions).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn publish_agent_version(
    listing_id: String,
    data_json: String,
    changelog: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let ver = marketplace.publish_new_version(&listing_id, &data_json, &changelog)?;
    serde_json::to_value(ver).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn rollback_agent(
    listing_id: String,
    version: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let data = marketplace.restore_version(&listing_id, &version)?;
    serde_json::to_value(data).map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn generate_plugin_from_nl(
    nl: String,
    state: State<AppState>,
) -> Result<String, MornError> {
    let api_key = std::env::var("MORN_API_KEY").map_err(|_| "MORN_API_KEY not set".to_string())?;
    let chat_agent = ChatAgent::new(&api_key, "https://api.deepseek.com", "deepseek-chat");

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

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);
    let path = plugin_generator::generate_plugin_from_nl(&nl, &plugin_dir, &chat_fn)
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

    Ok(path)
}

#[tauri::command]
pub(crate) fn sync_now(state: State<AppState>) -> Result<String, MornError> {
    let mut guard = state
        .sync_engine
        .lock()
        .map_err(|e| MornError::Internal(format!("lock error: {}", e)))?;
    if let Some(ref mut engine) = *guard {
        let report = engine.sync_once()?;
        Ok(format!(
            "Synced: {} pushed, {} pulled, {} applied",
            report.pushed_events, report.pulled_events, report.applied_events
        ))
    } else {
        Err(MornError::Internal("Sync engine not initialized".into()))
    }
}

#[tauri::command]
pub(crate) fn test_notification(state: State<AppState>) -> Result<String, MornError> {
    tracing::info!("test notification");
    let _ = state;
    Ok("Notification sent (placeholder)".to_string())
}
