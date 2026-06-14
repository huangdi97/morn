use crate::commands::errors::CommandError;

#[tauri::command]
pub(crate) fn send_notification(title: String, body: String) -> Result<String, CommandError> {
    tracing::info!("Notification: {} - {}", title, body);
    Ok(format!("Notification sent: {}", title))
}

#[tauri::command]
pub(crate) fn list_notifications() -> Result<Vec<String>, CommandError> {
    Ok(Vec::new())
}
