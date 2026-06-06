use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use crate::core::registry::Registry;
use crate::market::{Listing, Marketplace};

#[derive(Debug, PartialEq)]
pub enum Command {
    Exit,
    Clear,
    Status,
    Help,
    Market(String),
    Message(String),
}

pub fn parse_command(input: &str) -> Command {
    match input.trim() {
        "/exit" | "/quit" => Command::Exit,
        "/clear" => Command::Clear,
        "/status" => Command::Status,
        "/help" => Command::Help,
        s if s.starts_with("/market ") => Command::Market(s.to_string()),
        _ => Command::Message(input.trim().to_string()),
    }
}

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

pub fn run_repl(
    adapter: &mut ChannelAdapter,
    chat_fn: ChatFn,
    marketplace: &Marketplace,
    registry: &Arc<Mutex<Registry>>,
) {
    adapter.set_chat_fn(chat_fn);

    println!();
    println!("  ╔═══════════════════════════════╗");
    println!("  ║         Morn v0.1.0           ║");
    println!("  ║   Your AI Desktop Assistant   ║");
    println!("  ╚═══════════════════════════════╝");
    println!();
    println!("  Commands: /exit  /clear  /status  /help  /market");
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

        match parse_command(&input) {
            Command::Exit => {
                println!("  Goodbye!");
                break;
            }
            Command::Clear => {
                print!("\x1B[2J\x1B[H");
                turn = 1;
                println!("  History cleared.");
                continue;
            }
            Command::Status => {
                println!("  Morn v0.1.0 | Rust | CLI Mode");
                continue;
            }
            Command::Help => {
                println!("  Commands:");
                println!("    /exit           - Exit Morn");
                println!("    /clear          - Clear screen");
                println!("    /status         - Show status");
                println!("    /help           - Show this help");
                println!("    /market list    - List market listings");
                println!("    /market show <id> - Show listing details");
                println!("    /market buy <id> - Purchase a listing");
                println!("    /market install <id> - Install purchased listing");
                println!("    /market search <q> - Search listings");
                println!("    /market my      - Show my licenses");
                println!("    /market publish <id> <type> <price> - Publish a listing");
                continue;
            }
            Command::Market(cmd) => {
                handle_market_command(&cmd, marketplace, registry);
                continue;
            }
            Command::Message(msg) => {
                let msg = ChannelMessage::new(&msg, "cli");
                let response = adapter.handle_message(&msg);
                println!("  {}", response);
                println!();
                turn += 1;
            }
        }
    }
}

fn handle_market_command(input: &str, market: &Marketplace, registry: &Arc<Mutex<Registry>>) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 2 {
        println!("  Usage: /market <list|show|buy|install|search|my|publish> [args]");
        return;
    }

    match parts[1] {
        "list" => {
            let filter = parts.get(2).copied();
            let listings = market.list(filter);
            if listings.is_empty() {
                println!("  No listings found.");
                return;
            }
            for listing in &listings {
                println!(
                    "  [{}] {} ({} | {} MORN | ★ {})",
                    listing.id, listing.name, listing.item_type, listing.price, listing.rating
                );
            }
        }
        "show" => {
            let id = match parts.get(2) {
                Some(id) => id,
                None => {
                    println!("  Usage: /market show <id>");
                    return;
                }
            };
            match market.get(id) {
                Some(listing) => {
                    println!("  ID: {}", listing.id);
                    println!("  Name: {}", listing.name);
                    println!("  Type: {}", listing.item_type);
                    println!("  Description: {}", listing.description);
                    println!("  Price: {} MORN", listing.price);
                    println!("  Author: {}", listing.author);
                    println!("  Rating: {} / 5.0", listing.rating);
                    println!("  Downloads: {}", listing.downloads);
                    println!("  Created: {}", listing.created_at);
                }
                None => println!("  Listing '{}' not found.", id),
            }
        }
        "buy" => {
            let id = match parts.get(2) {
                Some(id) => id,
                None => {
                    println!("  Usage: /market buy <id>");
                    return;
                }
            };
            match market.purchase(id, "cli-user") {
                Ok(license) => println!(
                    "  Purchased! License: {} (expires: {:?})",
                    license.id, license.expires_at
                ),
                Err(e) => println!("  Purchase failed: {}", e),
            }
        }
        "install" => {
            let id = match parts.get(2) {
                Some(id) => id,
                None => {
                    println!("  Usage: /market install <id>");
                    return;
                }
            };
            match market.install(id, "cli-user") {
                Ok(()) => {
                    let mut reg = registry.lock().unwrap();
                    match market.install_to_registry(id, &mut reg) {
                        Ok(()) => println!("  Installed and registered successfully."),
                        Err(e) => println!("  Install failed: {}", e),
                    }
                }
                Err(e) => println!("  Install check failed: {}", e),
            }
        }
        "search" => {
            let query = match parts.get(2) {
                Some(q) => *q,
                None => {
                    println!("  Usage: /market search <query>");
                    return;
                }
            };
            let results = market.search(query);
            if results.is_empty() {
                println!("  No results for '{}'.", query);
                return;
            }
            for listing in &results {
                println!(
                    "  [{}] {} ({} | {} MORN | ★ {})",
                    listing.id, listing.name, listing.item_type, listing.price, listing.rating
                );
            }
        }
        "my" => {
            let licenses = market.user_licenses("cli-user");
            if licenses.is_empty() {
                println!("  No licenses found.");
                return;
            }
            for lic in &licenses {
                let listing = market.get(&lic.listing_id);
                let name = listing
                    .map(|l| l.name)
                    .unwrap_or_else(|| lic.listing_id.clone());
                println!(
                    "  {} - {} (granted: {}, expires: {:?})",
                    lic.id, name, lic.granted_at, lic.expires_at
                );
            }
        }
        "publish" => {
            let name = match parts.get(2) {
                Some(n) => n,
                None => {
                    println!("  Usage: /market publish <id> <type> <price>");
                    return;
                }
            };
            let item_type = parts.get(3).unwrap_or(&"tool");
            let price: f64 = parts.get(4).unwrap_or(&"0.0").parse().unwrap_or(0.0);
            let listing = Listing {
                id: format!("listing-{}-{}", item_type, uuid::Uuid::new_v4()),
                item_type: item_type.to_string(),
                name: name.to_string(),
                description: format!("Published from CLI: {}", name),
                price,
                author: "cli-user".to_string(),
                rating: 0.0,
                downloads: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            match market.publish(listing) {
                Ok(()) => println!("  Published successfully."),
                Err(e) => println!("  Publish failed: {}", e),
            }
        }
        _ => {
            println!("  Unknown market command: {}", parts[1]);
            println!("  Usage: /market <list|show|buy|install|search|my|publish> [args]");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_exit() {
        assert_eq!(parse_command("/exit"), Command::Exit);
        assert_eq!(parse_command("/quit"), Command::Exit);
        assert_eq!(parse_command("  /exit  "), Command::Exit);
    }

    #[test]
    fn test_parse_command_clear() {
        assert_eq!(parse_command("/clear"), Command::Clear);
    }

    #[test]
    fn test_parse_command_status() {
        assert_eq!(parse_command("/status"), Command::Status);
    }

    #[test]
    fn test_parse_command_help() {
        assert_eq!(parse_command("/help"), Command::Help);
    }

    #[test]
    fn test_parse_command_market() {
        let cmd = parse_command("/market list");
        assert!(matches!(cmd, Command::Market(_)));
        if let Command::Market(s) = cmd {
            assert_eq!(s, "/market list");
        }
    }

    #[test]
    fn test_parse_command_message() {
        assert_eq!(
            parse_command("hello world"),
            Command::Message("hello world".to_string())
        );
        assert_eq!(
            parse_command("/notacommand"),
            Command::Message("/notacommand".to_string())
        );
    }

    #[test]
    fn test_parse_command_empty() {
        assert_eq!(parse_command(""), Command::Message("".to_string()));
    }

    #[test]
    fn test_handle_market_command_list() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let market = Marketplace::new(storage);
        let registry = Arc::new(Mutex::new(Registry::new(None, None)));
        handle_market_command("/market list", &market, &registry);
    }

    #[test]
    fn test_handle_market_command_show() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let market = Marketplace::new(storage);
        let registry = Arc::new(Mutex::new(Registry::new(None, None)));
        handle_market_command("/market show listing-tool-web-search", &market, &registry);
    }

    #[test]
    fn test_handle_market_command_short_args() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let market = Marketplace::new(storage);
        let registry = Arc::new(Mutex::new(Registry::new(None, None)));
        handle_market_command("/market show", &market, &registry);
        handle_market_command("/market buy", &market, &registry);
        handle_market_command("/market search", &market, &registry);
        handle_market_command("/market install", &market, &registry);
        handle_market_command("/market publish", &market, &registry);
    }

    #[test]
    fn test_handle_market_command_unknown() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let market = Marketplace::new(storage);
        let registry = Arc::new(Mutex::new(Registry::new(None, None)));
        handle_market_command("/market unknown", &market, &registry);
    }

    #[test]
    fn test_handle_market_command_no_subcommand() {
        let storage = crate::core::storage::Storage::new_in_memory().unwrap();
        let market = Marketplace::new(storage);
        let registry = Arc::new(Mutex::new(Registry::new(None, None)));
        handle_market_command("/market", &market, &registry);
    }
}
