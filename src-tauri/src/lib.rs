use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use morn::console::ConsoleBackend;
use morn::core::component_type::registry::TypeRegistry;
use morn::core::hub_seeder::seed_hub_data;
use morn::core::mcp::MCPServer;
use morn::core::plugin_manager::PluginManager;
use morn::core::storage::Storage;
use morn::core::supervisor::presets::seed_preset_agents;
use morn::core::supervisor::Supervisor;
use morn::studio::manager::StudioManager;
use morn::studio::publisher::StudioPublisher;
use morn::studio::tester::StudioTester;

mod autostart;
mod commands;

const DEFAULT_API_KEY: &str = "sk-zcFNOoh23DWQZxNdmCgXQnomTvc1jmPt";

pub struct AppState {
    pub supervisor: Mutex<Option<Supervisor>>,
    pub turn_count: Mutex<u64>,
    pub manager: Mutex<Option<StudioManager>>,
    pub publisher: Mutex<Option<StudioPublisher>>,
    pub tester: Mutex<Option<StudioTester>>,
    pub console: Mutex<Option<ConsoleBackend>>,
    pub storage: Mutex<Option<Storage>>,
    pub plugin_manager: Mutex<Option<PluginManager>>,
    pub type_registry: Mutex<TypeRegistry>,
    pub mcp_manager: Mutex<Vec<MCPServer>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api_key = std::env::var("MORN_API_KEY").ok();
    let effective_key = api_key.unwrap_or(DEFAULT_API_KEY.to_string());
    let storage = match Storage::new() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("Storage init failed: {}", e);
            None
        }
    };
    let supervisor = Some(Supervisor::new(storage.clone(), None));

    let registry = None;
    let manager = Some(StudioManager::new(registry.clone(), storage.clone(), None));

    if let Some(ref manager) = manager {
        seed_preset_agents(&storage, manager);
    }
    seed_hub_data(&storage);
    let publisher = Some(StudioPublisher::new(
        registry.clone(),
        storage.clone(),
        None,
    ));
    let tester = Some(StudioTester::new());
    let console = Some(ConsoleBackend::new(
        registry,
        storage.clone(),
        None,
        None,
        None,
        None,
    ));

    let plugin_dir = dirs::data_dir()
        .map(|d| d.join("morn").join("plugins"))
        .unwrap_or_else(|| PathBuf::from("./plugins"));
    let mut plugin_manager = PluginManager::new(plugin_dir);
    let _ = plugin_manager.scan();
    let plugin_manager = Some(plugin_manager);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            autostart::setup_autostart(app);

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon({
                    let img_bytes = include_bytes!("../icons/tray-icon.png");
                    let img =
                        image::load_from_memory(img_bytes).expect("failed to decode tray icon");
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    tauri::image::Image::new_owned(rgba.into_raw(), w, h)
                })
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
            plugin_manager: Mutex::new(plugin_manager),
            type_registry: Mutex::new(TypeRegistry::new()),
            mcp_manager: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_message,
            commands::chat::get_status,
            commands::chat::clear_history,
            commands::studio::list_components,
            commands::studio::get_component,
            commands::studio::create_component,
            commands::studio::update_component,
            commands::studio::delete_component,
            commands::studio::assemble_agent,
            commands::studio::test_component,
            commands::studio::test_component_rerun,
            commands::studio::list_component_types,
            commands::studio::publish_component,
            commands::console::get_system_status,
            commands::console::get_component_topology,
            commands::org::create_user,
            commands::org::list_users,
            commands::org::create_team,
            commands::org::list_teams,
            commands::org::add_member,
            commands::org::remove_member,
            commands::org::grant_permission,
            commands::org::revoke_permission,
            commands::market::create_agent_from_description,
            commands::studio::list_agent_templates,
            commands::org::get_audit_log,
            commands::market::get_preset_persona,
            commands::market::list_preset_personas,
            commands::market::get_market_listings,
            commands::market::list_bot_store,
            commands::market::install_bot_from_store,
            commands::market::get_agent_versions,
            commands::market::publish_agent_version,
            commands::market::list_themes,
            commands::market::apply_theme,
            commands::market::generate_plugin_from_nl,
            commands::market::sync_now,
            commands::market::test_notification,
            commands::analytics::get_usage_stats,
            commands::analytics::get_performance_metrics,
            commands::local_model::list_local_models,
            commands::local_model::download_model,
            commands::local_model::delete_local_model,
            commands::backup::export_mornpack,
            commands::backup::import_mornpack,
            commands::component_type::register_component_type,
            commands::component_type::unregister_component_type,
            commands::component_type::list_component_types,
            commands::mcp::mcp_connect,
            commands::mcp::mcp_disconnect,
            commands::mcp::mcp_list_servers,
            commands::mcp::mcp_serve,
            commands::sandbox::run_in_sandbox,
            commands::sandbox::sandbox_status,
            commands::notifications::send_notification,
            commands::notifications::list_notifications,
            commands::config::export_config,
            commands::config::import_config,
            commands::oauth::oauth_authorize,
            commands::oauth::oauth_list_providers,
            commands::memory::list_memories,
            commands::memory::search_memories,
            commands::memory::delete_memory,
            commands::whisper::transcribe_audio,
            commands::whisper::list_audio_devices,
            commands::cost::estimate_cost,
            commands::cost::get_cost_summary,
            commands::recovery::get_last_error,
            commands::recovery::retry_last_operation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
