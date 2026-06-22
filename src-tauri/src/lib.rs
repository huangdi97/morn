use chrono::Utc;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use morn::bridge::sync::SyncEngine;
use morn::console::ConsoleBackend;
use morn::core::capability::CapabilityRegistry;
use morn::core::component_type::registry::TypeRegistry;
pub use morn::core::error::MornError;
use morn::core::mcp::MCPServer;
use morn::core::oauth::OAuthManager;
use morn::core::plugin_manager::adapter::morn_plugin_to_plugin;
use morn::core::plugin_manager::CorePluginRegistry;
use morn::core::plugin_manager::PluginConfig;
use morn::core::plugin_manager::{register_morn_plugin, MornPluginMeta, PluginManager};
use morn::core::proactive::ProactiveEngine;
use morn::core::scheduler::Scheduler;
use morn::core::storage::InstalledItem;
use morn::core::storage::Storage;
use morn::core::supervisor::Supervisor;
use morn::core::{load_plugins, MornPlugin, PluginContext};
use morn::studio::manager::StudioManager;
use morn::studio::publisher::StudioPublisher;
use morn::studio::tester::StudioTester;

mod autostart;
mod commands;

const DEFAULT_API_KEY: &str = "sk-zcFNOoh23DWQZxNdmCgXQnomTvc1jmPt";

fn start_sync_loop(engine: Arc<Mutex<Option<SyncEngine>>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(300));
        if let Ok(mut guard) = engine.lock() {
            if let Some(ref mut engine) = *guard {
                if let Err(e) = engine.sync_once() {
                    tracing::warn!("Sync failed: {}", e);
                }
            }
        }
    });
}

/// 从 overrides.json 加载外部插件覆盖
fn load_plugin_overrides(registry: &mut CorePluginRegistry, plugin_dir: &Path) {
    let overrides_path = plugin_dir.join("overrides.json");
    if !overrides_path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&overrides_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to read overrides.json: {e}");
            return;
        }
    };
    let overrides: std::collections::HashMap<String, serde_json::Value> =
        match serde_json::from_str(&content) {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("Failed to parse overrides.json: {e}");
                return;
            }
        };
    for (plugin_id, _config) in &overrides {
        tracing::info!("Plugin override registered: {plugin_id}");
    }
}

pub struct AppState {
    pub supervisor: Mutex<Option<Arc<Mutex<Supervisor>>>>,
    pub turn_count: Mutex<u64>,
    pub manager: Mutex<Option<Arc<Mutex<StudioManager>>>>,
    pub publisher: Mutex<Option<Arc<Mutex<StudioPublisher>>>>,
    pub tester: Mutex<Option<Arc<Mutex<StudioTester>>>>,
    pub console: Mutex<Option<Arc<Mutex<ConsoleBackend>>>>,
    pub storage: Mutex<Option<Storage>>,
    pub plugin_manager: Mutex<Option<PluginManager>>,
    pub type_registry: Mutex<TypeRegistry>,
    pub mcp_manager: Mutex<Vec<MCPServer>>,
    pub scheduler: Mutex<Option<Scheduler>>,
    pub oauth_manager: Mutex<Option<OAuthManager>>,
    pub proactive_engine: Arc<Mutex<ProactiveEngine>>,
    pub sync_engine: Arc<Mutex<Option<SyncEngine>>>,
    pub capability_registry: Option<Arc<Mutex<CapabilityRegistry>>>,
}

impl AppState {
    pub fn from_ctx(ctx: &PluginContext) -> Self {
        Self {
            supervisor: Mutex::new(ctx.get::<Arc<Mutex<Supervisor>>>("morn:supervisor")),
            turn_count: Mutex::new(0),
            manager: Mutex::new(ctx.get::<Arc<Mutex<StudioManager>>>("morn:studio-manager")),
            publisher: Mutex::new(ctx.get::<Arc<Mutex<StudioPublisher>>>("morn:studio-publisher")),
            tester: Mutex::new(ctx.get::<Arc<Mutex<StudioTester>>>("morn:studio-tester")),
            console: Mutex::new(ctx.get::<Arc<Mutex<ConsoleBackend>>>("morn:console")),
            storage: Mutex::new(ctx.get::<Storage>("morn:storage")),
            plugin_manager: Mutex::new(None),
            type_registry: Mutex::new(
                ctx.get::<TypeRegistry>("morn:type-registry")
                    .unwrap_or_default(),
            ),
            mcp_manager: Mutex::new(Vec::new()),
            scheduler: Mutex::new(Some(Scheduler::new())),
            oauth_manager: Mutex::new(None),
            proactive_engine: Arc::new(Mutex::new(ProactiveEngine::new(None))),
            sync_engine: Arc::new(Mutex::new(
                ctx.get::<Arc<Mutex<SyncEngine>>>("morn:sync-engine")
                    .and_then(|arc| arc.lock().ok().map(|guard| (*guard).clone())),
            )),
            capability_registry: ctx.capability_registry().cloned(),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let plugin_dir = dirs::data_dir()
        .map(|d| d.join("morn").join("plugins"))
        .unwrap_or_else(|| PathBuf::from("./plugins"));

    let config_path = plugin_dir.join("plugins.json");
    let config = PluginConfig::load(&config_path);
    let mut registry = CorePluginRegistry::new();
    load_plugin_overrides(&mut registry, &plugin_dir);

    let mut plugins: Vec<Box<dyn MornPlugin>> = config
        .plugins_to_load(&registry)
        .iter()
        .filter_map(|id| registry.build(id, plugin_dir.clone()))
        .collect();

    if plugins.is_empty() {
        panic!("No plugins loaded. Check plugins.json in: {:?}", plugin_dir);
    }
    let cap_registry = Arc::new(Mutex::new(CapabilityRegistry::new()));
    let ctx = PluginContext::new().with_capability_registry(cap_registry.clone());
    if let Err(e) = load_plugins(&mut plugins, &ctx) {
        let crash_log = dirs::data_dir()
            .map(|d| d.join("morn").join("crash.log"))
            .unwrap_or_else(|| PathBuf::from("./crash.log"));
        let _ = std::fs::write(
            &crash_log,
            format!(
                "Morn crash at {}\nPlugin loading failed: {:#?}\n",
                chrono::Utc::now().to_rfc3339(),
                e
            ),
        );
        // On Windows, try to show an error dialog via PowerShell
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("powershell")
                .args([
                    "-WindowStyle",
                    "Hidden",
                    "-Command",
                    &format!(
                        "[System.Windows.Forms.MessageBox]::Show(\
                            'Morn 启动失败，插件加载出错。\n\n{0}\n\n详情已写入：\n{1}',\
                            'Morn Error', 'OK', 'Error')",
                        e,
                        crash_log.display()
                    ),
                ])
                .output();
        }
        panic!(
            "Plugin loading failed: {:#?}. Crash log written to: {:?}",
            e, crash_log
        );
    }

    for plugin in &plugins {
        register_morn_plugin(MornPluginMeta {
            id: plugin.id().to_string(),
            deps: plugin.deps().into_iter().map(|s| s.to_string()).collect(),
            priority: plugin.priority(),
            enabled: true,
        });
    }

    // Build PluginManager with all plugins
    let mut pm = PluginManager::new(plugin_dir);
    let _ = pm.scan();
    for plugin in &plugins {
        let adapter = morn_plugin_to_plugin(plugin.as_ref());
        if !pm
            .plugins
            .iter()
            .any(|p| p.manifest.name == adapter.manifest.name)
        {
            pm.plugins.push(adapter);
        }
    }

    let mut state = AppState::from_ctx(&ctx);
    state.plugin_manager = Mutex::new(Some(pm));

    // 自动将系统插件注册到 installed_items 生命周期表
    if let Ok(guard) = state.storage.lock() {
        if let Some(ref storage) = *guard {
            for plugin_id in registry.known_ids() {
                let item = InstalledItem {
                    id: plugin_id.clone(),
                    item_type: "system_plugin".to_string(),
                    name: plugin_id.clone(),
                    description: String::new(),
                    enabled: true,
                    installed_at: Utc::now().to_rfc3339(),
                };
                let _ = storage.upsert_installed_item(&item);
            }
        }
    }

    // Initialize OAuthManager with stored provider credentials
    if let Some(storage) = state.storage.lock().ok().and_then(|s| s.clone()) {
        let oauth = OAuthManager::new(Arc::new(storage.clone()));
        state.oauth_manager = Mutex::new(Some(oauth));

        // Initialize ProactiveEngine with storage
        let engine = ProactiveEngine::new(Some(Arc::new(storage)));
        state.proactive_engine = Arc::new(Mutex::new(engine));
    }

    // Start background sync loop (engine initialized by SyncPlugin)
    if state
        .sync_engine
        .lock()
        .ok()
        .map(|g| g.is_some())
        .unwrap_or(false)
    {
        let sync_engine = state.sync_engine.clone();
        start_sync_loop(sync_engine);
    }

    // Clone engine reference for background tick thread
    let bg_engine = state.proactive_engine.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            autostart::setup_autostart(app);

            // Background tick thread for proactive engine
            let engine = bg_engine.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(60));
                if let Ok(mut engine) = engine.lock() {
                    let ready = engine.tick();
                    for agent in &ready {
                        tracing::info!(
                            "Proactive rule triggered: {} (action: {})",
                            agent.id,
                            agent.action
                        );
                    }
                }
            });

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
        .manage(state)
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
            commands::component_type::list_component_types,
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
            commands::hub::create_agent_from_description,
            commands::studio::list_agent_templates,
            commands::hub::get_preset_persona,
            commands::hub::list_preset_personas,
            commands::hub::get_hub_listings,
            commands::hub::list_bot_store,
            commands::hub::install_bot_from_store,
            commands::hub::hub_publish,
            commands::hub::get_agent_versions,
            commands::hub::publish_agent_version,
            commands::hub::list_themes,
            commands::hub::apply_theme,
            commands::hub::generate_plugin_from_nl,
            commands::hub::sync_now,
            commands::hub::test_notification,
            commands::hub_search::search_hub_listings,
            commands::hub_search::submit_review,
            commands::hub_search::get_listing_reviews,
            commands::analytics::get_analytics_data,
            commands::journey::get_user_journey,
            commands::local_model::list_local_models,
            commands::local_model::download_model,
            commands::local_model::delete_local_model,
            commands::backup::export_mornpack,
            commands::backup::import_mornpack,
            commands::component_type::register_component_type,
            commands::component_type::unregister_component_type,
            commands::mcp::mcp_connect,
            commands::mcp::mcp_disconnect,
            commands::mcp::mcp_list_servers,
            commands::mcp::mcp_call_tool,
            commands::mcp::mcp_serve,
            commands::sandbox::run_in_sandbox,
            commands::sandbox::sandbox_status,
            commands::notifications::send_notification,
            commands::notifications::list_notifications,
            commands::config::export_config,
            commands::config::import_config,
            commands::oauth::oauth_authorize,
            commands::oauth::oauth_callback,
            commands::oauth::oauth_list_providers,
            commands::oauth::oauth_save_config,
            commands::memory::list_memories,
            commands::memory::search_memories,
            commands::memory::delete_memory,
            commands::whisper::transcribe_audio,
            commands::whisper::list_audio_devices,
            commands::cost::estimate_cost,
            commands::cost::get_cost_summary,
            commands::cost::get_cost_details,
            commands::recovery::get_last_error,
            commands::recovery::retry_last_operation,
            commands::proactive::list_proactive_rules,
            commands::proactive::toggle_proactive_rule,
            commands::proactive::create_proactive_rule,
            commands::proactive::delete_proactive_rule,
            commands::earnings::get_creator_earnings,
            commands::metrics::get_reliability_metrics,
            commands::checkup::run_system_check,
            commands::scheduler::schedule_task,
            commands::scheduler::list_scheduled_tasks,
            commands::scheduler::cancel_task,
            commands::execution::get_recent_logs,
            commands::team_templates::list_team_templates,
            commands::hub::rollback_agent,
            commands::git::git_info,
            commands::plugin_manager::plugin_install,
            commands::plugin_manager::list_plugins,
            commands::plugin_manager::toggle_plugin,
            commands::plugin_manager::create_plugin_from_spec,
            commands::plugin_manager::list_morn_plugins,
            commands::plugin_manager::toggle_morn_plugin,
            commands::workflow::list_workflow_templates,
            commands::workflow::save_workflow_template,
            commands::workflow::delete_workflow_template,
            commands::workflow::execute_workflow,
            commands::workflow::list_workflow_node_types,
            commands::sync::list_sync_devices,
            commands::sync::get_sync_status,
            commands::sync::set_sync_server_url,
            commands::lifecycle::list_installed_items,
            commands::lifecycle::toggle_installed_item,
            commands::lifecycle::uninstall_installed_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
