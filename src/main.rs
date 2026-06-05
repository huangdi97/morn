use std::env;
use std::sync::{Arc, Mutex};

use morn::channel::adapter::ChannelAdapter;
use morn::channel::cli;
use morn::core::assembler::AgentAssembler;
use morn::core::event_bus::SimpleEventBus;
use morn::core::registry::Registry;
use morn::core::security::SecurityGuard;
use morn::core::storage::Storage;
use morn::core::supervisor::Supervisor;

fn main() {
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
            let _assembler = AgentAssembler::new(Some(registry.lock().unwrap().clone()));

            let chat_fn = Arc::new(move |prompt: &str, system: &str| -> Result<String, String> {
                chat_agent.chat(prompt, system)
            });

            run_cli(supervisor, chat_fn, security);
        }
        Err(_) => {
            println!("[Morn] MORN_API_KEY not set.");
            println!("[Morn] Usage: MORN_API_KEY=your_key cargo run -- cli");
            println!("");
            println!("Available commands:");
            println!("  /exit     - Exit Morn");
            println!("  /clear    - Clear conversation history");
            println!("  /status   - Show system status");
            println!("  /help     - Show this help");
            println!("  /mode     - Set COO mode (active/safe/auto)");
        }
    }
}

fn run_cli(supervisor: Supervisor, chat_fn: Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>, _security: Arc<Mutex<SecurityGuard>>) {
    let mut adapter = ChannelAdapter::new(Some(supervisor));
    cli::run_repl(&mut adapter, chat_fn);
}