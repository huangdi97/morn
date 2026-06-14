use crate::AppState;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use morn::bridge::chat_agent::ChatAgent;
use morn::core::plugin_generator;
use morn::market::Marketplace;
use morn::studio::manager::CreateComponentDef;

#[tauri::command]
pub(crate) fn list_themes(state: State<AppState>) -> Result<Vec<String>, String> {
    let mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
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
pub(crate) fn apply_theme(name: String, state: State<AppState>) -> Result<String, String> {
    let mut mgr = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let mgr = mgr
        .as_mut()
        .ok_or_else(|| "PluginManager not initialized".to_string())?;
    // Activate the theme plugin if not already active, which caches its CSS
    mgr.activate(&name).map_err(|e| e.to_string())?;
    mgr.get_theme_css(&name)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No CSS cached for theme '{}'", name))
}

#[tauri::command]
pub(crate) fn get_market_listings(
    type_filter: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let listings = marketplace.list(type_filter.as_deref());
    serde_json::to_value(listings).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn list_bot_store() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": "b1", "name": "Data Analyst", "icon": "📊", "description": "Turn raw data into actionable insights with statistical analysis and visualization", "category": "analysis", "rating": 4.8, "installs": 3420, "author": "Morn Labs", "price": 0, "template_id": "preset-analyst"}),
        serde_json::json!({"id": "b2", "name": "Research Assistant", "icon": "🔬", "description": "Multi-source research with cross-validation and citation management", "category": "research", "rating": 4.7, "installs": 2890, "author": "Morn Labs", "price": 0, "template_id": "preset-researcher"}),
        serde_json::json!({"id": "b3", "name": "Content Writer", "icon": "✍️", "description": "Create engaging content from blog posts to technical documentation", "category": "writing", "rating": 4.6, "installs": 2150, "author": "Morn Labs", "price": 0, "template_id": "preset-writer"}),
        serde_json::json!({"id": "b4", "name": "Code Engineer", "icon": "💻", "description": "Full-stack development with testing and best practices", "category": "coding", "rating": 4.9, "installs": 4560, "author": "Morn Labs", "price": 0, "template_id": "preset-coder"}),
        serde_json::json!({"id": "b5", "name": "Translator Pro", "icon": "🌐", "description": "Professional translation with cultural adaptation and terminology management", "category": "translation", "rating": 4.5, "installs": 1870, "author": "Morn Labs", "price": 0.001, "template_id": "preset-translator"}),
        serde_json::json!({"id": "b6", "name": "System Assistant", "icon": "🤖", "description": "All-purpose AI assistant for daily tasks and workflow automation", "category": "assistant", "rating": 4.4, "installs": 5230, "author": "Morn Labs", "price": 0, "template_id": "preset-assistant"}),
        serde_json::json!({"id": "b7", "name": "Code Reviewer", "icon": "🔍", "description": "Thorough code review with actionable improvement suggestions", "category": "review", "rating": 4.7, "installs": 1560, "author": "Morn Labs", "price": 0, "template_id": "preset-reviewer"}),
        serde_json::json!({"id": "b8", "name": "Customer Support", "icon": "🎧", "description": "Patient and empathetic customer service agent", "category": "support", "rating": 4.3, "installs": 980, "author": "Morn Labs", "price": 0, "template_id": "preset-cs-agent"}),
        serde_json::json!({"id": "b9", "name": "Financial Analyst", "icon": "💰", "description": "Financial data analysis, trend prediction and investment research", "category": "analysis", "rating": 4.6, "installs": 1340, "author": "Morn Labs", "price": 0.002, "template_id": "preset-analyst"}),
        serde_json::json!({"id": "b10", "name": "DevOps Bot", "icon": "⚙️", "description": "Infrastructure management, deployment automation and monitoring", "category": "coding", "rating": 4.5, "installs": 870, "author": "Morn Labs", "price": 0, "template_id": "preset-coder"}),
    ]
}

#[tauri::command]
pub(crate) fn install_bot_from_store(
    bot_id: String,
    template_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;

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
pub(crate) fn get_preset_persona(name: String) -> Result<serde_json::Value, String> {
    match morn::component::persona::get_preset_persona(&name) {
        Some(persona) => serde_json::to_value(persona).map_err(|e| e.to_string()),
        None => Err(format!("Preset persona '{}' not found", name)),
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
) -> Result<String, String> {
    let api_key = std::env::var("MORN_API_KEY").map_err(|_| "MORN_API_KEY not set".to_string())?;
    let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
        &api_key,
        "https://api.deepseek.com",
        "deepseek-chat",
    );

    let supervisor = state.supervisor.lock().map_err(|e| e.to_string())?;
    let sup = supervisor
        .as_ref()
        .ok_or_else(|| "Supervisor not initialized.".to_string())?;

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);
    let nl_def = sup.create_agent_from_nl(&nl, &chat_fn, None)?;
    serde_json::to_string(&nl_def).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn get_agent_versions(
    listing_id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let versions = marketplace.get_version_history(&listing_id);
    serde_json::to_value(versions).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn publish_agent_version(
    listing_id: String,
    data_json: String,
    changelog: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let marketplace = Marketplace::new(s.clone());
    let ver = marketplace.publish_new_version(&listing_id, &data_json, &changelog)?;
    serde_json::to_value(ver).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn generate_plugin_from_nl(
    nl: String,
    state: State<AppState>,
) -> Result<String, String> {
    let api_key = std::env::var("MORN_API_KEY").map_err(|_| "MORN_API_KEY not set".to_string())?;
    let chat_agent = ChatAgent::new(&api_key, "https://api.deepseek.com", "deepseek-chat");

    let plugin_manager = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    let mgr = plugin_manager
        .as_ref()
        .ok_or_else(|| "PluginManager not initialized".to_string())?;

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);
    let path = plugin_generator::generate_plugin_from_nl(&nl, &mgr.plugin_dir, &chat_fn)
        .map_err(|e| e.to_string())?;
    Ok(path)
}

#[tauri::command]
pub(crate) fn sync_now(state: State<AppState>) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;

    let unsynced = s.list_unsynced_events()?;
    let ids: Vec<String> = unsynced.iter().map(|e| e.id.clone()).collect();
    if !ids.is_empty() {
        s.mark_events_synced(&ids)?;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    Ok(timestamp)
}

#[tauri::command]
pub(crate) fn test_notification(state: State<AppState>) -> Result<String, String> {
    tracing::info!("test notification");
    let _ = state;
    Ok("Notification sent (placeholder)".to_string())
}
