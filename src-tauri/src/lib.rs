use std::sync::Mutex;
use tauri::State;

use morn::console::ConsoleBackend;
use morn::core::assembler::AgentDef;
use morn::core::storage::Storage;
use morn::core::supervisor::Supervisor;
use morn::studio::manager::{CreateComponentDef, StudioManager, UpdateComponentDef};
use morn::studio::publisher::StudioPublisher;
use morn::studio::tester::StudioTester;

pub struct AppState {
    pub supervisor: Mutex<Option<Supervisor>>,
    pub turn_count: Mutex<u64>,
    pub manager: Mutex<Option<StudioManager>>,
    pub publisher: Mutex<Option<StudioPublisher>>,
    pub tester: Mutex<Option<StudioTester>>,
    pub console: Mutex<Option<ConsoleBackend>>,
}

#[tauri::command]
fn send_message(text: String, state: State<AppState>) -> Result<String, String> {
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    let api_key = std::env::var("MORN_API_KEY").map_err(|_| "MORN_API_KEY not set".to_string())?;

    let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
        &api_key,
        "https://api.deepseek.com",
        "deepseek-chat",
    );

    let mut supervisor = state.supervisor.lock().map_err(|e| e.to_string())?;
    let sup = supervisor
        .as_mut()
        .ok_or_else(|| "Supervisor not initialized.".to_string())?;

    let response = runtime.block_on(async {
        chat_agent
            .chat_async(&text, "You are Morn, a helpful AI assistant.")
            .await
    })?;

    sup.record_turn("user", &text);
    sup.record_turn("assistant", &response);

    let mut turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    *turn = sup.turn_count();

    Ok(response)
}

#[tauri::command]
fn get_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "turn_count": *turn,
        "version": "0.1.0"
    }))
}

#[tauri::command]
fn clear_history(state: State<AppState>) -> Result<(), String> {
    let mut supervisor = state.supervisor.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut sup) = *supervisor {
        sup.clear_history();
    }
    let mut turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    *turn = 0;
    Ok(())
}

#[tauri::command]
fn list_components(type_filter: Option<String>, state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    let components = mgr.list_components(type_filter.as_deref());
    Ok(serde_json::to_value(components).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_component(id: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    let detail = mgr.get_component(&id)?;
    Ok(serde_json::to_value(detail).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn create_component(name: String, component_type: String, config_json: Option<String>, state: State<AppState>) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    let id = mgr.create_component(CreateComponentDef {
        name,
        component_type,
        config_json,
    })?;
    Ok(id)
}

#[tauri::command]
fn update_component(id: String, name: Option<String>, config_json: Option<String>, status: Option<String>, state: State<AppState>) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.update_component(&id, UpdateComponentDef { name, config_json, status })
}

#[tauri::command]
fn delete_component(id: String, state: State<AppState>) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.delete_component(&id)
}

#[tauri::command]
fn assemble_agent(name: String, persona: String, model: String, tools: Vec<String>, knowledge: Vec<String>, skills: Vec<String>, state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;

    let persona_obj = match persona.as_str() {
        "researcher" => morn::component::persona::create_researcher_persona(),
        "analyst" => morn::component::persona::create_analyst_persona(),
        "writer" => morn::component::persona::create_writer_persona(),
        "coder" => morn::component::persona::create_coder_persona(),
        "translator" => morn::component::persona::create_translator_persona(),
        "reviewer" => morn::component::persona::create_reviewer_persona(),
        _ => morn::component::persona::create_assistant_persona(),
    };

    let model_obj = morn::component::model::ModelConfig {
        id: format!("model-{}", uuid::Uuid::new_v4()),
        provider: "deepseek".into(),
        model_name: model,
        base_url: "https://api.deepseek.com".into(),
        api_key: std::env::var("MORN_API_KEY").unwrap_or_default(),
        parameters: morn::component::model::ModelParameters::default(),
        fallback: None,
        cost_tier: morn::component::model::CostTier::Low,
    };

    let agent_id = mgr.assemble_agent(AgentDef {
        id: format!("agent-{}", uuid::Uuid::new_v4()),
        name,
        persona: persona_obj,
        model: model_obj,
        tools,
        knowledge,
        skills,
        memory: None,
    })?;

    Ok(serde_json::json!({ "agent_id": agent_id }))
}

#[tauri::command]
fn test_component(id: String, input: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager.as_ref().ok_or_else(|| "StudioManager not initialized".to_string())?;
    let data = morn::core::component::Data::text(&input);
    let result = mgr.test_component(&id, data)?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn publish_component(id: String, state: State<AppState>) -> Result<(), String> {
    let publisher = state.publisher.lock().map_err(|e| e.to_string())?;
    let pubr = publisher.as_ref().ok_or_else(|| "StudioPublisher not initialized".to_string())?;
    pubr.publish_agent(&id)
}

#[tauri::command]
fn get_system_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let console = state.console.lock().map_err(|e| e.to_string())?;
    let con = console.as_ref().ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
    let dashboard = con.get_dashboard();
    let system_info = con.get_system_info();
    Ok(serde_json::json!({
        "dashboard": dashboard,
        "system_info": system_info
    }))
}

#[tauri::command]
fn get_component_topology(state: State<AppState>) -> Result<serde_json::Value, String> {
    let console = state.console.lock().map_err(|e| e.to_string())?;
    let con = console.as_ref().ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
    let topology = con.get_topology();
    Ok(serde_json::to_value(topology).map_err(|e| e.to_string())?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api_key = std::env::var("MORN_API_KEY").ok();
    let storage = Storage::new_in_memory().ok();
    let supervisor = if api_key.is_some() {
        Some(Supervisor::new(storage.clone(), None))
    } else {
        None
    };

    let registry = None;
    let manager = Some(StudioManager::new(registry.clone(), storage.clone(), None));
    let publisher = Some(StudioPublisher::new(registry.clone(), storage.clone()));
    let tester = Some(StudioTester::new());
    let console = Some(ConsoleBackend::new(
        registry,
        storage,
        None,
        None,
        None,
        None,
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            supervisor: Mutex::new(supervisor),
            turn_count: Mutex::new(0),
            manager: Mutex::new(manager),
            publisher: Mutex::new(publisher),
            tester: Mutex::new(tester),
            console: Mutex::new(console),
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            get_status,
            clear_history,
            list_components,
            get_component,
            create_component,
            update_component,
            delete_component,
            assemble_agent,
            test_component,
            publish_component,
            get_system_status,
            get_component_topology,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}