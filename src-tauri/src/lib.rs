use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use morn::console::ConsoleBackend;
use morn::core::storage::Storage;
use morn::core::supervisor::Supervisor;
use morn::studio::manager::StudioManager;
use morn::studio::publisher::StudioPublisher;
use morn::studio::tester::StudioTester;

mod autostart;
mod commands;

pub struct AppState {
    pub supervisor: Mutex<Option<Supervisor>>,
    pub turn_count: Mutex<u64>,
    pub manager: Mutex<Option<StudioManager>>,
    pub publisher: Mutex<Option<StudioPublisher>>,
    pub tester: Mutex<Option<StudioTester>>,
    pub console: Mutex<Option<ConsoleBackend>>,
    pub storage: Mutex<Option<Storage>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api_key = std::env::var("MORN_API_KEY").ok();
    let storage = match Storage::new() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("Storage init failed: {}", e);
            None
        }
    };
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
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_message,
            commands::get_status,
            commands::clear_history,
            commands::list_components,
            commands::get_component,
            commands::create_component,
            commands::update_component,
            commands::delete_component,
            commands::assemble_agent,
            commands::test_component,
            commands::test_component_rerun,
            commands::list_component_types,
            commands::publish_component,
            commands::get_system_status,
            commands::get_component_topology,
            commands::create_user,
            commands::list_users,
            commands::create_team,
            commands::list_teams,
            commands::add_member,
            commands::remove_member,
            commands::grant_permission,
            commands::revoke_permission,
            commands::create_agent_from_description,
            commands::list_agent_templates,
            commands::get_audit_log,
            commands::get_preset_persona,
            commands::list_preset_personas,
            commands::get_market_listings,
            commands::list_bot_store,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
