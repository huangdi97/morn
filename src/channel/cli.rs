use std::io::{self, Write};
use std::sync::Arc;

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

pub fn run_repl(
    adapter: &mut ChannelAdapter,
    chat_fn: Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>,
) {
    adapter.set_chat_fn(chat_fn);

    println!();
    println!("  ╔═══════════════════════════════╗");
    println!("  ║         Morn v0.1.0           ║");
    println!("  ║   Your AI Desktop Assistant   ║");
    println!("  ╚═══════════════════════════════╝");
    println!();
    println!("  Commands: /exit  /clear  /status  /help");
    println!();

    let mut turn: u64 = 1;

    loop {
        print!("  [{}] morn > ", turn);
        io::stdout().flush().ok();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(_) => break,
        }

        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        match input.as_str() {
            "/exit" | "/quit" => {
                println!("  Goodbye!");
                break;
            }
            "/clear" => {
                print!("\x1B[2J\x1B[H");
                turn = 1;
                println!("  History cleared.");
                continue;
            }
            "/status" => {
                println!("  Morn v0.1.0 | Rust | CLI Mode");
                continue;
            }
            "/help" => {
                println!("  Commands:");
                println!("    /exit    - Exit Morn");
                println!("    /clear   - Clear screen");
                println!("    /status  - Show status");
                println!("    /help    - Show this help");
                continue;
            }
            _ => {}
        }

        let msg = ChannelMessage::new(&input, "cli");
        let response = adapter.handle_message(&msg);
        println!("  {}", response);
        println!();
        turn += 1;
    }
}
