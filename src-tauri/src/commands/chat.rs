use crate::MornError;
use crate::AppState;
use tauri::State;

const DEFAULT_API_KEY: &str = "sk-zcFNOoh23DWQZxNdmCgXQnomTvc1jmPt";
const SENSENOVA_BASE_URL: &str = "https://token.sensenova.cn/v1";

#[tauri::command]
pub(crate) fn send_message(text: String, state: State<AppState>) -> Result<String, MornError> {
    if text.trim().starts_with("/clear") {
        let mut supervisor = state.supervisor.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(ref mut sup) = *supervisor {
            sup.clear_history();
        }
        let mut turn = state.turn_count.lock().map_err(|e| MornError::Internal(e.to_string()))?;
        *turn = 0;
        return Ok("History cleared.".to_string());
    }

    let api_key = std::env::var("MORN_API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());

    let chat_agent =
        morn::bridge::chat_agent::ChatAgent::new(&api_key, SENSENOVA_BASE_URL, "deepseek-chat");

    let mut supervisor = state.supervisor.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let sup = supervisor
        .as_mut()
        .ok_or_else(|| "Supervisor not initialized.".to_string())?;

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);

    match sup.execute_chat(&text, &chat_fn) {
        Ok(response) => {
            let mut turn = state.turn_count.lock().map_err(|e| MornError::Internal(e.to_string()))?;
            *turn = sup.turn_count();
            Ok(response)
        }
        Err(sup_err) => {
            tracing::warn!("Supervisor execute_chat failed, falling back to direct LLM: {sup_err}");
            match chat_agent.chat(&text, "You are Morn, a helpful AI assistant.") {
                Ok(fallback) => {
                    sup.record_turn("user", &text);
                    sup.record_turn("assistant", &fallback);
                    let mut turn = state.turn_count.lock().map_err(|e| MornError::Internal(e.to_string()))?;
                    *turn = sup.turn_count();
                    Ok(fallback)
                }
                Err(fallback_err) => Err(format!(
                    "Supervisor failed: {}, fallback also failed: {}",
                    sup_err, fallback_err
                )),
            }
        }
    }
}

#[tauri::command]
pub(crate) fn get_status(state: State<AppState>) -> Result<serde_json::Value, MornError> {
    let turn = state.turn_count.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    Ok(serde_json::json!({
        "turn_count": *turn,
        "version": "0.1.0"
    }))
}

#[tauri::command]
pub(crate) fn clear_history(state: State<AppState>) -> Result<(), MornError> {
    let mut supervisor = state.supervisor.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    if let Some(ref mut sup) = *supervisor {
        sup.clear_history();
    }
    let mut turn = state.turn_count.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    *turn = 0;
    Ok(())
}
