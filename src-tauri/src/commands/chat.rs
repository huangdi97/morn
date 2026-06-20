use crate::AppState;
use crate::MornError;
use serde::Serialize;
use tauri::State;

const DEFAULT_API_KEY: &str = "sk-zcFNOoh23DWQZxNdmCgXQnomTvc1jmPt";
const SENSENOVA_BASE_URL: &str = "https://token.sensenova.cn/v1";

#[derive(Serialize)]
pub(crate) struct SendMessageResult {
    pub text: String,
    pub execution_events: Vec<serde_json::Value>,
}

#[tauri::command]
pub(crate) fn send_message(
    text: String,
    state: State<AppState>,
) -> Result<SendMessageResult, MornError> {
    if text.trim().starts_with("/clear") {
        let supervisor = state
            .supervisor
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        if let Some(sup) = supervisor.as_ref() {
            sup.lock().unwrap().clear_history();
        }
        let mut turn = state
            .turn_count
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        *turn = 0;
        return Ok(SendMessageResult {
            text: "History cleared.".to_string(),
            execution_events: Vec::new(),
        });
    }

    let api_key = std::env::var("MORN_API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());

    let chat_agent =
        morn::bridge::chat_agent::ChatAgent::new(&api_key, SENSENOVA_BASE_URL, "deepseek-chat");

    let supervisor = state
        .supervisor
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let sup = supervisor
        .as_ref()
        .unwrap()
        .lock()
        .unwrap();

    let chat_fn = |prompt: &str, system: &str| chat_agent.chat(prompt, system);

    let result_text = match sup.execute_chat(&text, &chat_fn) {
        Ok(response) => {
            let mut turn = state
                .turn_count
                .lock()
                .map_err(|e| MornError::Internal(e.to_string()))?;
            *turn = sup.turn_count();
            response
        }
        Err(sup_err) => {
            tracing::warn!("Supervisor execute_chat failed, falling back to direct LLM: {sup_err}");
            match chat_agent.chat(&text, "You are Morn, a helpful AI assistant.") {
                Ok(fallback) => {
                    sup.record_turn("user", &text);
                    sup.record_turn("assistant", &fallback);
                    let mut turn = state
                        .turn_count
                        .lock()
                        .map_err(|e| MornError::Internal(e.to_string()))?;
                    *turn = sup.turn_count();
                    fallback
                }
                Err(fallback_err) => {
                    return Err(format!(
                        "Supervisor failed: {}, fallback also failed: {}",
                        sup_err, fallback_err
                    )
                    .into())
                }
            }
        }
    };

    let execution_events = state
        .storage
        .lock()
        .ok()
        .and_then(|s| {
            s.as_ref()
                .and_then(|storage| storage.list_recent_executions(5).ok())
        })
        .unwrap_or_default()
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "action": e.action,
                "status": e.status,
                "agent_id": e.agent_id,
                "latency_ms": e.latency_ms,
                "error_msg": e.error_msg,
                "created_at": e.created_at,
            })
        })
        .collect();

    Ok(SendMessageResult {
        text: result_text,
        execution_events,
    })
}

#[tauri::command]
pub(crate) fn get_status(state: State<AppState>) -> Result<serde_json::Value, MornError> {
    let turn = state
        .turn_count
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    Ok(serde_json::json!({
        "turn_count": *turn,
        "version": "0.1.0"
    }))
}

#[tauri::command]
pub(crate) fn clear_history(state: State<AppState>) -> Result<(), MornError> {
    let supervisor = state
        .supervisor
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    if let Some(sup) = supervisor.as_ref() {
        sup.lock().unwrap().clear_history();
    }
    let mut turn = state
        .turn_count
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    *turn = 0;
    Ok(())
}
