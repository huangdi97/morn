//! Morn CLI entry point — REPL, ASCII banner, and protocol selection.
//! Use --cli flag to force CLI mode; otherwise defaults to CLI.
//! Tauri mode is activated when launched via src-tauri.

use std::env;
use std::sync::{Arc, Mutex};

use morn::channel::adapter::ChannelAdapter;
use morn::channel::cli;
use morn::computer::sys_ops;
use morn::console::cost::CostCenter;
use morn::console::ConsoleBackend;
use morn::core::assembler::AgentAssembler;
use morn::core::assembly::AssemblyBuilder;
use morn::core::computer_control::ComputerControl;
use morn::core::event_bus::SimpleEventBus;
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
    let is_cli_mode = args.iter().any(|a| a == "--cli");

    if !is_cli_mode {
        // Tauri mode is launched via src-tauri binary, not from here
    }

    let api_key = env::var("MORN_API_KEY");

    let event_bus = SimpleEventBus::new();
    let storage = Storage::new_in_memory().ok();
    let security = Arc::new(Mutex::new(SecurityGuard::new()));
    let registry = Arc::new(Mutex::new(Registry::new(storage.clone(), Some(event_bus))));

    let version = env!("CARGO_PKG_VERSION");

    println!("╔══════════════════════════════════╗");
    println!("║         Morn Desktop v{}         ║", version);
    println!("║    Single Agent · Desktop · CLI   ║");
    println!("╚══════════════════════════════════╝");

    match &api_key {
        Ok(key) => {
            let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
                key,
                "https://api.deepseek.com",
                "deepseek-chat",
            );

            println!("[Morn] ChatAgent initialized (DeepSeek)");

            let supervisor = Supervisor::new(storage.clone(), None);
            let _assembler = AgentAssembler::new(Some(
                registry
                    .lock()
                    .map_err(|e| e.to_string())?
                    .clone(),
            ));

            let chat_fn = Arc::new(
                move |prompt: &str, system: &str| -> Result<String, String> {
                    chat_agent.chat(prompt, system)
                },
            );

            run_cli(supervisor, chat_fn, security, storage, registry)?;
        }
        Err(_) => {
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

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

fn run_cli(
    supervisor: Supervisor,
    chat_fn: ChatFn,
    _security: Arc<Mutex<SecurityGuard>>,
    storage: Option<Storage>,
    registry: Arc<Mutex<Registry>>,
) -> Result<(), String> {
    let _console = ConsoleBackend::new(None, None, None, None, None, None);
    let _cost_center = CostCenter::new(100.0);
    let _assembly = AssemblyBuilder::new();
    let _network = sys_ops::network_status();
    let _studio = StudioManager::new(None, None, None);
    let _computer = ComputerControl;
    let _team_mgr = TeamManager::new(storage.clone().ok_or_else(|| "Storage required".to_string())?);
    let _a2a = A2ARouter::new();
    let _mcp_tool = adapter::port_to_mcp_tool("test", "test", &[]);
    let mut adapter = ChannelAdapter::new(Some(supervisor));
    let marketplace = storage
        .map(Marketplace::new)
        .ok_or_else(|| "Storage required for marketplace".to_string())?;
    cli::run_repl(&mut adapter, chat_fn, &marketplace, &registry);
    Ok(())
}
