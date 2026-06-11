use crate::AppState;
use tauri::State;

use morn::org::audit::AuditLogger;
use morn::org::permissions::PermissionChecker;
use morn::org::team::{TeamManager, UserManager};

#[tauri::command]
pub(crate) fn create_user(
    username: String,
    display_name: String,
    role: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let um = UserManager::new(s.clone());
    let user = um.register(&username, &display_name, &role)?;
    Ok(serde_json::to_string(&user).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn list_users(state: State<AppState>) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let um = UserManager::new(s.clone());
    let users = um.list_users()?;
    Ok(serde_json::to_value(users).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn create_team(
    name: String,
    description: String,
    owner_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    let team = tm.create_team(&name, &description, &owner_id)?;
    Ok(serde_json::to_string(&team).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn list_teams(state: State<AppState>) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let teams = s.list_teams()?;
    Ok(serde_json::to_value(teams).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn add_member(
    team_id: String,
    user_id: String,
    role: String,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    let member = tm.add_member(&team_id, &user_id, &role)?;
    Ok(serde_json::to_string(&member).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn remove_member(
    team_id: String,
    user_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let tm = TeamManager::new(s.clone());
    tm.remove_member(&team_id, &user_id)
}

#[tauri::command]
pub(crate) fn grant_permission(
    user_id: String,
    agent_id: String,
    permission: String,
    team_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let pc = PermissionChecker::new(s.clone());
    let perm = pc.grant(&user_id, &agent_id, &permission, team_id.as_deref())?;
    Ok(serde_json::to_string(&perm).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub(crate) fn revoke_permission(
    user_id: String,
    agent_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let pc = PermissionChecker::new(s.clone());
    pc.revoke(&user_id, &agent_id)
}

#[tauri::command]
pub(crate) fn get_audit_log(
    user_id: Option<String>,
    action_type: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let s = storage
        .as_ref()
        .ok_or_else(|| "Storage not initialized".to_string())?;
    let audit = AuditLogger::new(s.clone());
    let logs = audit.query(
        user_id.as_deref(),
        action_type.as_deref(),
        limit.unwrap_or(50),
    )?;
    Ok(serde_json::to_value(logs).map_err(|e| e.to_string())?)
}
