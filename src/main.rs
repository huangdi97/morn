//! Morn CLI entry point — REPL, ASCII banner, and protocol selection.
//! Use --cli flag to force CLI mode; otherwise defaults to CLI.
//! Tauri mode is activated when launched via src-tauri.

use std::env;
use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};

use morn::channel::adapter::ChannelAdapter;
use morn::channel::cli;
use morn::channel::dingtalk::DingTalkChannel;
use morn::channel::miniprogram::MiniProgramChannel;
use morn::channel::rest_api::{serve, ApiState};
use morn::channel::telegram::TelegramChannel;
use morn::channel::wecom::WeComChannel;
use morn::computer::app_ops::{launch, list};
use morn::computer::desktop_ops::{keyboard, mouse};
use morn::computer::sys_ops;
use morn::config::MornConfig;
use morn::console::cost::CostCenter;
use morn::console::ConsoleBackend;
use morn::core::assembler::AgentAssembler;
use morn::core::assembly::AssemblyBuilder;
use morn::core::computer_control::ComputerControl;
use morn::core::event_bus::SimpleEventBus;
use morn::core::model_router::ModelRouter;
use morn::core::registry::Registry;
use morn::core::security::SecurityGuard;
use morn::core::storage::Storage;
use morn::core::supervisor::Supervisor;
use morn::market::Marketplace;
use morn::mcp::adapter;
use morn::org::team::TeamManager;
use morn::protocol::a2a::router::A2ARouter;
use morn::studio::manager::StudioManager;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let is_daemon = args.iter().any(|a| a == "--daemon");

    if args.iter().any(|a| a == "--execute-task") {
        let mut task_json = String::new();
        std::io::stdin()
            .read_to_string(&mut task_json)
            .map_err(|e| e.to_string())?;
        let task = serde_json::from_str::<serde_json::Value>(&task_json)
            .unwrap_or_else(|_| serde_json::json!({ "raw": task_json }));
        let result = serde_json::json!({
            "success": true,
            "task": task,
        });
        println!("{}", serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {}\"}}", e)));
        return Ok(());
    }

    if let Ok(exec_task) = std::env::var("EXECUTE_TASK") {
        let mut child = morn::core::task_engine::child_process::ChildProcess::new();
        let result = child.spawn(&exec_task, 300)?;
        println!("{}", serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"error\":\"serialize failed: {}\"}}", e)));
        return Ok(());
    }

    let config = MornConfig::load()?;
    let is_cli_mode = args.iter().any(|a| a == "--cli");

    if is_daemon {
        return run_daemon(config);
    }

    if !is_cli_mode {
        // Tauri mode is launched via src-tauri binary, not from here
    }

    let api_key = config
        .model
        .api_key
        .clone()
        .or_else(|| env::var("MORN_API_KEY").ok());

    let event_bus = SimpleEventBus::new();
    let storage = match Storage::new() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("Storage init failed: {}", e);
            None
        }
    };
    let security = Arc::new(Mutex::new(SecurityGuard::new()));
    let registry = Arc::new(Mutex::new(Registry::new(storage.clone(), Some(event_bus))));

    let version = env!("CARGO_PKG_VERSION");

    println!("╔══════════════════════════════════╗");
    println!("║         Morn Desktop v{}         ║", version);
    println!("║    Single Agent · Desktop · CLI   ║");
    println!("╚══════════════════════════════════╝");

    match &api_key {
        Some(key) => {
            let mut model_config = config.model.clone();
            model_config.api_key = Some(key.clone());
            let router = ModelRouter::from_model_config(&model_config);
            let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
                key,
                &config.model.base_url,
                &config.model.name,
            );

            println!(
                "[Morn] ChatAgent initialized ({}:{})",
                config.model.provider, config.model.name
            );

            let supervisor = Supervisor::new(storage.clone(), None).with_model_router(router);
            let _assembler =
                AgentAssembler::new(Some(registry.lock().map_err(|e| e.to_string())?.clone()));

            let chat_fn = Arc::new(
                move |prompt: &str, system: &str| -> Result<String, String> {
                    chat_agent.chat(prompt, system)
                },
            );

            run_cli(supervisor, chat_fn, security, storage, registry, config)?;
        }
        None => {
            println!("[Morn] MORN_API_KEY not set.");
            println!("[Morn] Usage: MORN_API_KEY=your_key cargo run -- cli");
            println!();
            println!("Available commands:");
            println!("  /exit     - Exit Morn");
            println!("  /clear    - Clear conversation history");
            println!("  /status   - Show system status");
            println!("  /help     - Show this help");
            println!("  /mode     - Set COO mode (active/safe/auto)");
        }
    }
    Ok(())
}

fn run_daemon(config: MornConfig) -> Result<(), String> {
    let api_key = config
        .model
        .api_key
        .clone()
        .or_else(|| env::var("MORN_API_KEY").ok())
        .ok_or_else(|| "MORN_API_KEY not set and config.model.api_key is empty".to_string())?;

    if env::var("API_PORT").is_err() {
        env::set_var("API_PORT", config.server.port.to_string());
    }

    if let Some(parent) = config.daemon.pid_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create PID directory {}: {}", parent.display(), e))?;
    }

    fs::write(&config.daemon.pid_file, std::process::id().to_string()).map_err(|e| {
        format!(
            "Failed to write PID file {}: {}",
            config.daemon.pid_file.display(),
            e
        )
    })?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to initialize daemon runtime: {}", e))?;

    let pid_file = config.daemon.pid_file.clone();
    let result = runtime.block_on(async move {
        let event_bus = SimpleEventBus::new();
        let storage = match Storage::new() {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!("Storage init failed: {}", e);
                None
            }
        };
        let registry = Arc::new(tokio::sync::Mutex::new(Registry::new(
            storage.clone(),
            Some(event_bus.clone()),
        )));
        let mut model_config = config.model.clone();
        model_config.api_key = Some(api_key.clone());
        let router = ModelRouter::from_model_config(&model_config);
        let supervisor = Arc::new(tokio::sync::Mutex::new(
            Supervisor::new(storage.clone(), Some(event_bus.clone())).with_model_router(router),
        ));

        let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
            &api_key,
            &config.model.base_url,
            &config.model.name,
        );
        let chat_fn: ChatFn = Arc::new(
            move |prompt: &str, system: &str| -> Result<String, String> {
                chat_agent.chat(prompt, system)
            },
        );

        if config.channels.telegram.enabled {
            let bot_token = config
                .channels
                .telegram
                .bot_token
                .clone()
                .or_else(|| env::var("TELEGRAM_BOT_TOKEN").ok())
                .ok_or_else(|| {
                    "Telegram channel enabled but TELEGRAM_BOT_TOKEN is not configured".to_string()
                })?;
            let chat_id = if config.channels.telegram.chat_id == "0" {
                env::var("TELEGRAM_CHAT_ID").unwrap_or_else(|_| "0".to_string())
            } else {
                config.channels.telegram.chat_id.clone()
            };
            let telegram_storage = storage.clone();
            let telegram_event_bus = event_bus.clone();
            let telegram_chat_fn = chat_fn.clone();

            std::thread::spawn(move || {
                let mut telegram = TelegramChannel::new(&bot_token, &chat_id);
                let mut adapter = ChannelAdapter::new(Some(Supervisor::new(
                    telegram_storage,
                    Some(telegram_event_bus),
                )))
                .with_chat_fn(telegram_chat_fn);
                telegram.poll_updates(&mut adapter);
            });
        }

        if config.channels.wecom.enabled {
            let webhook_url = config
                .channels
                .wecom
                .webhook_url
                .clone()
                .or_else(|| env::var("WECOM_WEBHOOK_URL").ok())
                .ok_or_else(|| {
                    "WeCom channel enabled but channels.wecom.webhook_url is not configured"
                        .to_string()
                })?;
            let token = config.channels.wecom.token.clone();
            let encoding_aes_key = config.channels.wecom.encoding_aes_key.clone();
            let corp_id = config.channels.wecom.corp_id.clone();
            let wecom_storage = storage.clone();
            let wecom_event_bus = event_bus.clone();
            let wecom_chat_fn = chat_fn.clone();

            std::thread::spawn(move || {
                let wecom = WeComChannel::with_webhook_config(
                    &webhook_url,
                    token,
                    encoding_aes_key,
                    corp_id,
                );
                let mut adapter = ChannelAdapter::new(Some(Supervisor::new(
                    wecom_storage,
                    Some(wecom_event_bus),
                )))
                .with_chat_fn(wecom_chat_fn);

                if let Err(e) = wecom.webhook_listen(&mut adapter) {
                    tracing::error!("[WeCom] webhook listener stopped: {}", e);
                }
            });
        }

        if config.channels.dingtalk.enabled {
            let webhook_token = config.channels.dingtalk.webhook_token.clone();
            let app_key = config.channels.dingtalk.app_key.clone();
            let app_secret = config.channels.dingtalk.app_secret.clone();
            let dingtalk_storage = storage.clone();
            let dingtalk_event_bus = event_bus.clone();
            let dingtalk_chat_fn = chat_fn.clone();

            std::thread::spawn(move || {
                let dingtalk =
                    DingTalkChannel::with_webhook_config("", webhook_token, app_key, app_secret);
                let mut adapter = ChannelAdapter::new(Some(Supervisor::new(
                    dingtalk_storage,
                    Some(dingtalk_event_bus),
                )))
                .with_chat_fn(dingtalk_chat_fn);

                if let Err(e) = dingtalk.webhook_listen(&mut adapter) {
                    tracing::error!("[DingTalk] webhook listener stopped: {}", e);
                }
            });
        }

        if config.channels.miniprogram.enabled {
            let appid = config
                .channels
                .miniprogram
                .appid
                .clone()
                .or_else(|| env::var("MINIPROGRAM_APPID").ok())
                .ok_or_else(|| {
                    "MiniProgram channel enabled but channels.miniprogram.appid is not configured"
                        .to_string()
                })?;
            let secret = config
                .channels
                .miniprogram
                .secret
                .clone()
                .or_else(|| env::var("MINIPROGRAM_SECRET").ok())
                .ok_or_else(|| {
                    "MiniProgram channel enabled but channels.miniprogram.secret is not configured"
                        .to_string()
                })?;
            let token = config.channels.miniprogram.token.clone();
            let miniprogram_storage = storage.clone();
            let miniprogram_event_bus = event_bus.clone();
            let miniprogram_chat_fn = chat_fn.clone();

            std::thread::spawn(move || {
                let mut miniprogram = MiniProgramChannel::with_token(&appid, &secret, token);
                let mut adapter = ChannelAdapter::new(Some(Supervisor::new(
                    miniprogram_storage,
                    Some(miniprogram_event_bus),
                )))
                .with_chat_fn(miniprogram_chat_fn);

                if let Err(e) = miniprogram.poll_messages(&mut adapter) {
                    tracing::error!("[MiniProgram] poller stopped: {}", e);
                }
            });
        }

        let state = ApiState {
            supervisor,
            registry,
            chat_fn,
        };

        println!("[Morn] Daemon started with PID {}", std::process::id());
        println!("[Morn] PID file: {}", pid_file.display());

        tokio::select! {
            result = serve(state) => result,
            result = wait_for_sigterm() => result,
        }
    });

    let cleanup_result = fs::remove_file(&config.daemon.pid_file)
        .or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .map_err(|e| {
            format!(
                "Failed to remove PID file {}: {}",
                config.daemon.pid_file.display(),
                e
            )
        });

    result.and(cleanup_result)
}

#[cfg(unix)]
async fn wait_for_sigterm() -> Result<(), String> {
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| format!("Failed to register SIGTERM handler: {}", e))?;
    signal.recv().await;
    println!("[Morn] SIGTERM received, shutting down daemon");
    Ok(())
}

#[cfg(not(unix))]
async fn wait_for_sigterm() -> Result<(), String> {
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| format!("Failed to wait for shutdown signal: {}", e))?;
    println!("[Morn] Shutdown signal received, stopping daemon");
    Ok(())
}

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

fn run_cli(
    supervisor: Supervisor,
    chat_fn: ChatFn,
    _security: Arc<Mutex<SecurityGuard>>,
    storage: Option<Storage>,
    registry: Arc<Mutex<Registry>>,
    _config: MornConfig,
) -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        let sub = &args[2];
        match args[1].as_str() {
            "app" => match sub.as_str() {
                "launch" if args.len() > 3 => {
                    let result = launch::launch(&args[3]);
                    println!("{}", result.data);
                    return Ok(());
                }
                "close" if args.len() > 3 => {
                    let result = launch::close(&args[3]);
                    println!("{}", result.data);
                    return Ok(());
                }
                "list" => {
                    let result = list::list();
                    println!("{}", result.data);
                    return Ok(());
                }
                _ => {}
            },
            "desktop" => match sub.as_str() {
                "type" if args.len() > 3 => {
                    let result = keyboard::keyboard_type(&args[3]);
                    println!("{}", result.data);
                    return Ok(());
                }
                "click" if args.len() > 3 => {
                    let result = mouse::mouse_click(&args[3]);
                    println!("{}", result.data);
                    return Ok(());
                }
                "move" if args.len() > 4 => {
                    let x: i32 = args[3].parse().map_err(|e| format!("Invalid x: {}", e))?;
                    let y: i32 = args[4].parse().map_err(|e| format!("Invalid y: {}", e))?;
                    let result = mouse::mouse_move(x, y);
                    println!("{}", result.data);
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }
    }

    let _console = ConsoleBackend::new(None, None, None, None, None, None);
    let _cost_center = CostCenter::new(100.0);
    let _assembly = AssemblyBuilder::new();
    let _network = sys_ops::network_status();
    let _studio = StudioManager::new(None, None, None);
    let _computer = ComputerControl;
    let _team_mgr = TeamManager::new(
        storage
            .clone()
            .ok_or_else(|| "Storage required".to_string())?,
    );
    let _a2a = A2ARouter::new();
    let _mcp_tool = adapter::port_to_mcp_tool("test", "test", &[]);
    let mut adapter = ChannelAdapter::new(Some(supervisor));
    let marketplace = storage
        .map(Marketplace::new)
        .ok_or_else(|| "Storage required for marketplace".to_string())?;
    cli::run_repl(&mut adapter, chat_fn, &marketplace, &registry);
    Ok(())
}
