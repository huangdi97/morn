use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) fn send_message(text: String, state: State<AppState>) -> Result<String, String> {
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

    let api_key = std::env::var("MORN_API_KEY").map_err(|_| "MORN_API_KEY not set".to_string())?;

    let chat_agent = morn::bridge::chat_agent::ChatAgent::new(
        &api_key,
        "https://api.deepseek.com",
        "deepseek-chat",
    );

    let mut supervisor = state.supervisor.lock().map_err(|e| e.to_string())?;
    let sup = supervisor
        .as_mut()
        .ok_or_else(|| "Supervisor not initialized.".to_string())?;

    let response = runtime.block_on(async {
        chat_agent
            .chat_async(&text, "You are Morn, a helpful AI assistant.")
            .await
    })?;

    sup.record_turn("user", &text);
    sup.record_turn("assistant", &response);

    let mut turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    *turn = sup.turn_count();

    Ok(response)
}

#[tauri::command]
pub(crate) fn get_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "turn_count": *turn,
        "version": "0.1.0"
    }))
}

#[tauri::command]
pub(crate) fn clear_history(state: State<AppState>) -> Result<(), String> {
    let mut supervisor = state.supervisor.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut sup) = *supervisor {
        sup.clear_history();
    }
    let mut turn = state.turn_count.lock().map_err(|e| e.to_string())?;
    *turn = 0;
    Ok(())
}
