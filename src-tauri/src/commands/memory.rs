use crate::commands::errors::CommandError;
use crate::MornError;

#[tauri::command]
pub(crate) fn list_memories() -> Result<Vec<String>, CommandError> {
    Ok(Vec::new())
}

#[tauri::command]
pub(crate) fn search_memories(q: String) -> Result<Vec<String>, CommandError> {
    if q.is_empty() {
        return Err(CommandError::NotFound("no results found".to_string()));
    }
    Ok(Vec::new())
}

#[tauri::command]
pub(crate) fn delete_memory(_id: String) -> Result<String, CommandError> {
    Ok("deleted".to_string())
}
