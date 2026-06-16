use crate::MornError;
use crate::AppState;
use morn::core::scheduler::ScheduleType;
use tauri::State;

#[tauri::command]
pub(crate) fn schedule_task(
    state: State<'_, AppState>,
    agent_id: String,
    input: String,
    schedule_type: String,
    max_runs: Option<u32>,
) -> Result<String, MornError> {
    let st = match schedule_type.split_once(':') {
        Some(("once", secs)) => ScheduleType::Once {
            delay_seconds: secs.parse().map_err(|_| MornError::Internal("invalid delay".into()))?,
        },
        Some(("interval", secs)) => ScheduleType::Interval {
            interval_seconds: secs.parse().map_err(|_| MornError::Internal("invalid interval".into()))?,
        },
        Some(("cron", expr)) => ScheduleType::Cron { expression: expr.to_string() },
        _ => return Err(MornError::Internal("use once:<secs>, interval:<secs>, or cron:<expr>".into())),
    };
    let mut guard = state.scheduler.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let sched = guard.as_mut().ok_or_else(|| MornError::Internal("scheduler not init".into()))?;
    let rt = tokio::runtime::Handle::current();
    let id = rt.block_on(sched.add_task(agent_id, input, st, max_runs));
    Ok(id)
}

#[tauri::command]
pub(crate) fn list_scheduled_tasks(state: State<'_, AppState>) -> Result<Vec<String>, MornError> {
    let guard = state.scheduler.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let sched = guard.as_ref().ok_or_else(|| MornError::Internal("scheduler not init".into()))?;
    let rt = tokio::runtime::Handle::current();
    let tasks = rt.block_on(sched.list_tasks());
    let ids: Vec<String> = tasks.into_iter().map(|t| t.id).collect();
    Ok(ids)
}

#[tauri::command]
pub(crate) fn cancel_task(state: State<'_, AppState>, task_id: String) -> Result<bool, MornError> {
    let mut guard = state.scheduler.lock().map_err(|e| MornError::Internal(e.to_string()))?;
    let sched = guard.as_mut().ok_or_else(|| MornError::Internal("scheduler not init".into()))?;
    let rt = tokio::runtime::Handle::current();
    Ok(rt.block_on(sched.remove_task(&task_id)))
}