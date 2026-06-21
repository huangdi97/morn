use crate::commands::errors::CommandError;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct DailyCallStat {
    pub date: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct DailyTokenStat {
    pub date: String,
    pub tokens: i64,
    pub cost: f64,
}

#[derive(Serialize)]
pub struct AgentStat {
    pub agent_id: String,
    pub calls: i64,
    pub avg_latency: f64,
}

#[derive(Serialize)]
pub struct ErrorRateStat {
    pub date: String,
    pub errors: i64,
    pub total: i64,
}

#[derive(Serialize)]
pub struct LatencyStat {
    pub date: String,
    pub avg_latency: f64,
}

#[derive(Serialize)]
pub struct AnalyticsData {
    pub daily_calls: Vec<DailyCallStat>,
    pub daily_tokens: Vec<DailyTokenStat>,
    pub top_agents: Vec<AgentStat>,
    pub error_rates: Vec<ErrorRateStat>,
    pub avg_latency: Vec<LatencyStat>,
    pub active_users: u64,
    pub total_executions: u64,
}

#[tauri::command]
pub(crate) fn get_analytics_data(
    days: u64,
    state: State<AppState>,
) -> Result<AnalyticsData, CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not initialized".to_string()))?;
    let conn = s.conn().map_err(|e| CommandError::Internal(e.to_string()))?;

    let days_param = format!("-{} days", days);

    let mut stmt = conn
        .prepare(
            "SELECT DATE(created_at) as date, COUNT(*) as count FROM executions \
             WHERE DATE(created_at) >= DATE('now', ?1) GROUP BY date ORDER BY date",
        )
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let daily_calls = stmt
        .query_map([&days_param], |row| {
            Ok(DailyCallStat {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    let mut stmt = conn
        .prepare(
            "SELECT date, SUM(token_count) as tokens, SUM(cost_usd) as cost \
             FROM daily_costs WHERE date >= DATE('now', ?1) GROUP BY date ORDER BY date",
        )
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let daily_tokens = stmt
        .query_map([&days_param], |row| {
            Ok(DailyTokenStat {
                date: row.get(0)?,
                tokens: row.get(1)?,
                cost: row.get(2)?,
            })
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    let mut stmt = conn
        .prepare(
            "SELECT agent_id, COUNT(*) as calls, COALESCE(AVG(latency_ms), 0) as avg_latency \
             FROM executions GROUP BY agent_id ORDER BY calls DESC LIMIT 10",
        )
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let top_agents = stmt
        .query_map([], |row| {
            Ok(AgentStat {
                agent_id: row.get(0)?,
                calls: row.get(1)?,
                avg_latency: row.get(2)?,
            })
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    let mut stmt = conn
        .prepare(
            "SELECT DATE(created_at) as date, \
             SUM(CASE WHEN status = 'error' OR status = 'failed' THEN 1 ELSE 0 END) as errors, \
             COUNT(*) as total \
             FROM executions WHERE DATE(created_at) >= DATE('now', ?1) \
             GROUP BY date ORDER BY date",
        )
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let error_rates = stmt
        .query_map([&days_param], |row| {
            Ok(ErrorRateStat {
                date: row.get(0)?,
                errors: row.get(1)?,
                total: row.get(2)?,
            })
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    let mut stmt = conn
        .prepare(
            "SELECT DATE(created_at) as date, COALESCE(AVG(latency_ms), 0) as avg_latency \
             FROM executions WHERE DATE(created_at) >= DATE('now', ?1) AND latency_ms IS NOT NULL \
             GROUP BY date ORDER BY date",
        )
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let avg_latency = stmt
        .query_map([&days_param], |row| {
            Ok(LatencyStat {
                date: row.get(0)?,
                avg_latency: row.get(1)?,
            })
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    let active_users: u64 = conn
        .query_row("SELECT COUNT(DISTINCT agent_id) FROM executions", [], |row| {
            row.get(0)
        })
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    let total_executions: u64 = conn
        .query_row("SELECT COUNT(*) FROM executions", [], |row| row.get(0))
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    Ok(AnalyticsData {
        daily_calls,
        daily_tokens,
        top_agents,
        error_rates,
        avg_latency,
        active_users,
        total_executions,
    })
}