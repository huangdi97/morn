use crate::AppState;
use crate::MornError;
use morn::market::revenue::CreatorEarnings;
use tauri::State;

#[tauri::command]
pub(crate) fn get_creator_earnings(state: State<AppState>) -> Result<CreatorEarnings, MornError> {
    let console = state
        .console
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let _con = console
        .as_ref()
        .unwrap()
        .lock()
        .unwrap();
    Ok(CreatorEarnings {
        creator_id: "creator-1".to_string(),
        total_earnings: 1250.00,
        pending_payout: 340.00,
        sale_count: 18,
    })
}
