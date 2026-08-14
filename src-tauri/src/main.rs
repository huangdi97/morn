//! Morn desktop shell (Tauri v2). Loads the built Workbench/Studio/Console/Hub
//! frontend and talks to the same `morn-app` backend. No business logic here —
//! the Tauri layer is a boundary shell only.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Report the desktop shell and backend API base (used by the UI status bar).
#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "shell": "morn-desktop",
        "version": env!("CARGO_PKG_VERSION"),
        "api_base": std::env::var("MORN_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8090".to_string()),
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .setup(|app| {
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Morn desktop shell");
}
