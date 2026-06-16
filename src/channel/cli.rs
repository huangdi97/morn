//! cli — Provides a command-line channel for interactive message handling.
use crate::core::error::MornError;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use crate::core::registry::Registry;
use crate::core::supervisor::Mode;
use crate::market::{Listing, Marketplace};

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, MornError> + Send + Sync>;

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
    println!("  Commands: /exit  /clear  /status  /help  /mode  /market");
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
                if let Some(mode) = adapter.supervisor_mode() {
                    println!("  COO mode: {}", mode.as_str());
                }
                continue;
            }
            "/help" => {
                println!("  Commands:");
                println!("    /exit           - Exit Morn");
                println!("    /clear          - Clear screen");
                println!("    /status         - Show status");
                println!("    /help           - Show this help");
                println!("    /mode <name>    - Set COO mode: proactive, safe, automated");
                println!("    /market list    - List market listings");
                println!("    /market show <id> - Show listing details");
                println!("    /market buy <id> - Purchase a listing");
                println!("    /market install <id> - Install purchased listing");
                println!("    /market search <q> - Search listings");
                println!("    /market my      - Show my licenses");
                println!("    /market publish <id> <type> <price> - Publish a listing");
                continue;
            }
            _ => {}
        }

        if input.starts_with("/mode") {
            handle_mode_command(&input, adapter);
            continue;
        }

        if input.starts_with("/market ") {
            handle_market_command(&input, marketplace, registry);
            continue;
        }

        let msg = ChannelMessage::new(&input, "cli");
        let response = adapter.handle_message(&msg);
        println!("  {}", response);
        println!();
        turn += 1;
    }
}

fn handle_mode_command(input: &str, adapter: &mut ChannelAdapter) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() == 1 {
        match adapter.supervisor_mode() {
            Some(mode) => println!("  COO mode: {}", mode.as_str()),
            None => println!("  Supervisor not initialized. Please set MORN_API_KEY."),
        }
        return;
    }

    match Mode::parse(parts[1]) {
        Some(mode) => match adapter.set_supervisor_mode(mode) {
            Ok(()) => {
                let current = adapter
                    .supervisor_mode()
                    .map(|mode| mode.as_str())
                    .unwrap_or("unknown");
                println!("  COO mode set to {}.", current);
            }
            Err(e) => println!("  {}", e),
        },
        None => println!("  Usage: /mode <proactive|safe|automated>"),
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
                    let mut reg = match registry.lock() {
                        Ok(guard) => guard,
                        Err(e) => {
                            println!("  Error: registry mutex poisoned: {}", e);
                            return;
                        }
                    };
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
    use crate::core::storage::Storage;

    fn market_and_registry() -> (Marketplace, Arc<Mutex<Registry>>) {
        let storage = Storage::new_in_memory()
            .unwrap_or_else(|e| panic!("in-memory storage should initialize: {}", e));
        let registry = Arc::new(Mutex::new(Registry::new(Some(storage.clone()), None)));
        (Marketplace::new(storage), registry)
    }

    #[test]
    fn mode_command_without_arg_is_status_only() {
        let mut adapter = ChannelAdapter::new(None);
        handle_mode_command("/mode", &mut adapter);
        assert!(adapter.supervisor_mode().is_none());
    }

    #[test]
    fn mode_command_rejects_unknown_mode() {
        let mut adapter = ChannelAdapter::new(None);
        handle_mode_command("/mode invalid", &mut adapter);
        assert!(adapter.supervisor_mode().is_none());
    }

    #[test]
    fn market_command_accepts_list_and_show_forms() {
        let (market, registry) = market_and_registry();
        handle_market_command("/market list", &market, &registry);
        handle_market_command("/market show listing-tool-web-search", &market, &registry);
    }

    #[test]
    fn market_command_handles_missing_args() {
        let (market, registry) = market_and_registry();
        handle_market_command("/market", &market, &registry);
        handle_market_command("/market show", &market, &registry);
    }
}
