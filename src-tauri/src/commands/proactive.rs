use crate::MornError;
use crate::AppState;
use serde::Serialize;
use tauri::State;

use morn::core::proactive::{ProactiveAgent, ProactiveEngine, Trigger};

#[derive(Serialize)]
pub(crate) struct ProactiveRule {
    pub id: String,
    pub trigger: String,
    pub action: String,
    pub enabled: bool,
}

#[tauri::command]
pub(crate) fn list_proactive_rules() -> Result<Vec<ProactiveRule>, MornError> {
    Ok(vec![
        ProactiveRule {
            id: "daily_report".into(),
            trigger: "Timer (every 24h)".into(),
            action: "generate_daily_report".into(),
            enabled: true,
        },
        ProactiveRule {
            id: "config_watch".into(),
            trigger: "Event (config_changed)".into(),
            action: "reload_config".into(),
            enabled: true,
        },
        ProactiveRule {
            id: "health_check".into(),
            trigger: "Timer (every 1h)".into(),
            action: "run_health_check".into(),
            enabled: false,
        },
    ])
}

#[tauri::command]
pub(crate) fn toggle_proactive_rule(rule_id: String, enabled: bool) -> Result<(), MornError> {
    println!("Proactive rule '{}' toggled to {}", rule_id, enabled);
    Ok(())
}
