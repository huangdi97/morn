use crate::AppState;
use crate::MornError;
use serde::Serialize;
use sysinfo::System;
use tauri::State;

#[derive(Debug, Serialize)]
pub(crate) struct CheckResult {
    pub label: String,
    pub status: String,
    pub value: Option<String>,
}

#[tauri::command]
pub(crate) fn run_system_check(state: State<AppState>) -> Result<Vec<CheckResult>, MornError> {
    let mut results = Vec::new();

    // Storage Status — verify SQLite connection
    {
        let storage = state
            .storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        match storage.as_ref() {
            Some(s) => {
                let healthy = s.check_health().map(|_| true).unwrap_or(false);
                results.push(CheckResult {
                    label: "Storage Status".into(),
                    status: if healthy { "ok" } else { "fail" }.into(),
                    value: Some(if healthy {
                        "SQLite connected".into()
                    } else {
                        "Health check failed".into()
                    }),
                });
            }
            None => {
                results.push(CheckResult {
                    label: "Storage Status".into(),
                    status: "fail".into(),
                    value: Some("Not initialized".into()),
                });
            }
        }
    }

    // API Connection — check supervisor model router has a default model
    {
        let supervisor = state
            .supervisor
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        match supervisor.as_ref() {
            Some(sup) => {
                let router = sup.model_router();
                let configured = router.default_model().is_some();
                results.push(CheckResult {
                    label: "API Connection".into(),
                    status: if configured { "ok" } else { "fail" }.into(),
                    value: Some(if configured {
                        "LLM endpoint configured".into()
                    } else {
                        "No default model set".into()
                    }),
                });
            }
            None => {
                results.push(CheckResult {
                    label: "API Connection".into(),
                    status: "fail".into(),
                    value: Some("Supervisor not initialized".into()),
                });
            }
        }
    }

    // Memory Usage — sysinfo RAM
    {
        let mut sys = System::new();
        sys.refresh_memory();
        let total_gb = sys.total_memory() as f64 / 1_073_741_824.0;
        let used_gb = sys.used_memory() as f64 / 1_073_741_824.0;
        let usage_pct = (used_gb / total_gb * 100.0) as u32;
        let status = if usage_pct > 90 { "fail" } else { "ok" };
        results.push(CheckResult {
            label: "Memory Usage".into(),
            status: status.into(),
            value: Some(format!(
                "{:.1} GB / {:.1} GB ({usage_pct}%)",
                used_gb, total_gb
            )),
        });
    }

    // Plugin Count
    {
        let mgr = state
            .plugin_manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let count = mgr.as_ref().map(|m| m.list().len()).unwrap_or(0);
        results.push(CheckResult {
            label: "Plugin Count".into(),
            status: "ok".into(),
            value: Some(format!("{count} active")),
        });
    }

    // Agent Count
    {
        let storage = state
            .storage
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let count = match storage.as_ref() {
            Some(s) => s.list_agents().map(|a| a.len()).unwrap_or(0),
            None => 0,
        };
        results.push(CheckResult {
            label: "Agent Count".into(),
            status: "ok".into(),
            value: Some(format!("{count} registered")),
        });
    }

    // Workflow Templates
    {
        let mgr = state
            .manager
            .lock()
            .map_err(|e| MornError::Internal(e.to_string()))?;
        let count = mgr.as_ref().map(|m| m.list_templates().len()).unwrap_or(0);
        results.push(CheckResult {
            label: "Workflow Templates".into(),
            status: "ok".into(),
            value: Some(format!("{count} available")),
        });
    }

    Ok(results)
}
