use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri::State;

use morn::console::ConsoleBackend;
use morn::core::assembler::AgentDef;
use morn::core::storage::Storage;
use morn::core::supervisor::{NLAgentDef, Supervisor};
use morn::org::audit::AuditLogger;
use morn::org::permissions::PermissionChecker;
use morn::org::team::{TeamManager, UserManager};
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
    pub storage: Mutex<Option<Storage>>,
}

fn setup_autostart(app: &tauri::App) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let autostart_dir = std::path::PathBuf::from(&home).join(".config/autostart");
            if std::fs::create_dir_all(&autostart_dir).is_ok() {
                if let Ok(exe) = std::env::current_exe() {
                    let desktop_entry = format!(
                        "[Desktop Entry]\nType=Application\nName=Morn\nExec={}\nX-GNOME-Autostart-enabled=true\n",
                        exe.display()
                    );
                    let _ =
                        std::fs::write(autostart_dir.join("morn-desktop.desktop"), desktop_entry);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new("reg")
                .args([
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "Morn",
                    "/d",
                    &exe.display().to_string(),
                    "/f",
                ])
                .output();
        }
    }
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
fn list_components(
    type_filter: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let components = mgr.list_components(type_filter.as_deref());
    Ok(serde_json::to_value(components).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn get_component(id: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let detail = mgr.get_component(&id)?;
    Ok(serde_json::to_value(detail).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn create_component(
    name: String,
    component_type: String,
    config_json: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let id = mgr.create_component(CreateComponentDef {
        name,
        component_type,
        config_json,
    })?;
    Ok(id)
}

#[tauri::command]
fn update_component(
    id: String,
    name: Option<String>,
    config_json: Option<String>,
    status: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.update_component(
        &id,
        UpdateComponentDef {
            name,
            config_json,
            status,
        },
    )
}

#[tauri::command]
fn delete_component(id: String, state: State<AppState>) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.delete_component(&id)
}

#[tauri::command]
fn assemble_agent(
    name: String,
    persona: String,
    model: String,
    tools: Vec<String>,
    knowledge: Vec<String>,
    skills: Vec<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;

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
fn list_agent_templates(state: State<AppState>) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let templates = mgr.list_templates();
    Ok(serde_json::to_value(templates).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn test_component(
    id: String,
    input: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let data = morn::core::component::Data::text(&input);
    let result = mgr.test_component(&id, data)?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn publish_component(id: String, state: State<AppState>) -> Result<(), String> {
    let publisher = state.publisher.lock().map_err(|e| e.to_string())?;
    let pubr = publisher
        .as_ref()
        .ok_or_else(|| "StudioPublisher not initialized".to_string())?;
    pubr.publish_agent(&id)
}

#[tauri::command]
fn get_system_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let console = state.console.lock().map_err(|e| e.to_string())?;
    let con = console
        .as_ref()
        .ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
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
    let con = console
        .as_ref()
        .ok_or_else(|| "ConsoleBackend not initialized".to_string())?;
    let topology = con.get_topology();
    Ok(serde_json::to_value(topology).map_err(|e| e.to_string())?)
}

// --- Org Management Commands ---

#[tauri::command]
fn create_user(
    username: String,
    display_name: String,
    role: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let um = UserManager::new(s.clone());
    let user = um.register(&username, &display_name, &role)?;
    Ok(serde_json::to_string(&user).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn list_users(state: State<AppState>) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let um = UserManager::new(s.clone());
    let users = um.list_users()?;
    Ok(serde_json::to_value(users).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn create_team(
    name: String,
    description: String,
    owner_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    let team = tm.create_team(&name, &description, &owner_id)?;
    Ok(serde_json::to_string(&team).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn list_teams(state: State<AppState>) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let teams = s.list_teams()?;
    Ok(serde_json::to_value(teams).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn add_member(
    team_id: String,
    user_id: String,
    role: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    let member = tm.add_member(&team_id, &user_id, &role)?;
    Ok(serde_json::to_string(&member).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn remove_member(team_id: String, user_id: String, state: State<AppState>) -> Result<(), String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    tm.remove_member(&team_id, &user_id)
}

#[tauri::command]
fn grant_permission(
    user_id: String,
    agent_id: String,
    permission: String,
    team_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let pc = PermissionChecker::new(s.clone());
    let perm = pc.grant(&user_id, &agent_id, &permission, team_id.as_deref())?;
    Ok(serde_json::to_string(&perm).map_err(|e| e.to_string())?)
}

#[tauri::command]
fn revoke_permission(
    user_id: String,
    agent_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let pc = PermissionChecker::new(s.clone());
    pc.revoke(&user_id, &agent_id)
}

#[tauri::command]
fn create_agent_from_description(nl: String, state: State<AppState>) -> Result<String, String> {
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
    let nl_def = sup.create_agent_from_nl(&nl, &chat_fn)?;
    serde_json::to_string(&nl_def).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_audit_log(
    user_id: Option<String>,
    action_type: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let audit = AuditLogger::new(s.clone());
    let logs = audit.query(
        user_id.as_deref(),
        action_type.as_deref(),
        limit.unwrap_or(50),
    )?;
    Ok(serde_json::to_value(logs).map_err(|e| e.to_string())?)
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
        storage.clone(),
        None,
        None,
        None,
        None,
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            setup_autostart(app);

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray-icon.png"
                ))?)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .manage(AppState {
            supervisor: Mutex::new(supervisor),
            turn_count: Mutex::new(0),
            manager: Mutex::new(manager),
            publisher: Mutex::new(publisher),
            tester: Mutex::new(tester),
            console: Mutex::new(console),
            storage: Mutex::new(storage),
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
            create_user,
            list_users,
            create_team,
            list_teams,
            add_member,
            remove_member,
            grant_permission,
            revoke_permission,
            create_agent_from_description,
            list_agent_templates,
            get_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
