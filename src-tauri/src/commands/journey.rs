use crate::commands::errors::CommandError;
use crate::MornError;

#[tauri::command]
pub(crate) fn get_user_journey() -> Result<serde_json::Value, CommandError> {
    let (day, message, next) = morn::core::onboarding::get_user_journey();
    Ok(serde_json::json!({
        "day": day,
        "message": message,
        "next": next
    }))
}
