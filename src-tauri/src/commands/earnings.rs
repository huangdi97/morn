use crate::MornError;
use crate::AppState;
use tauri::State;
use morn::market::revenue::CreatorEarnings;

#[tauri::command]
pub(crate) fn get_creator_earnings(state: State<AppState>) -> Result<CreatorEarnings, MornError> {
    let console = state.console.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let _con = console
        .as_ref()
        .ok_or_else(|| MornError::Internal("ConsoleBackend not initialized".to_string()))?;
    Ok(CreatorEarnings {
        creator_id: "creator-1".to_string(),
        total_earnings: 1250.00,
        pending_payout: 340.00,
        sale_count: 18,
    })
}