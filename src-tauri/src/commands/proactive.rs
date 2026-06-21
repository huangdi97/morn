use crate::commands::errors::CommandError;
use crate::AppState;
use tauri::State;

use morn::core::proactive::{ProactiveAgent, ProactiveEngine, Trigger};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct ProactiveRule {
    pub id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_config: String,
    pub action: String,
    pub enabled: bool,
    pub last_triggered_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<morn::core::storage::ProactiveRule> for ProactiveRule {
    fn from(r: morn::core::storage::ProactiveRule) -> Self {
        ProactiveRule {
            id: r.id,
            name: r.name,
            trigger_type: r.trigger_type,
            trigger_config: r.trigger_config,
            action: r.action,
            enabled: r.enabled,
            last_triggered_at: r.last_triggered_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[tauri::command]
pub(crate) fn list_proactive_rules(
    state: State<AppState>,
) -> Result<Vec<ProactiveRule>, CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let storage = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    let rules = storage
        .list_proactive_rules()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    Ok(rules.into_iter().map(|r| r.into()).collect())
}

#[tauri::command]
pub(crate) fn create_proactive_rule(
    state: State<AppState>,
    name: String,
    trigger_type: String,
    trigger_config: String,
    action: String,
) -> Result<ProactiveRule, CommandError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    if name.trim().is_empty() {
        return Err(CommandError::InvalidInput("Name cannot be empty".into()));
    }
    if trigger_type != "timer" && trigger_type != "event" {
        return Err(CommandError::InvalidInput(
            "trigger_type must be 'timer' or 'event'".into(),
        ));
    }
    if trigger_config.trim().is_empty() {
        return Err(CommandError::InvalidInput(
            "trigger_config cannot be empty".into(),
        ));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let id = format!("rule_{}", now);

    let rule = morn::core::storage::ProactiveRule {
        id: id.clone(),
        name,
        trigger_type,
        trigger_config,
        action,
        enabled: true,
        last_triggered_at: None,
        created_at: now,
        updated_at: now,
    };

    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let storage = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    storage
        .create_proactive_rule(&rule)
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    // Also register in engine
    if let Ok(mut engine) = state.proactive_engine.lock() {
        let trigger = match rule.trigger_type.as_str() {
            "timer" => {
                let interval = rule.trigger_config.parse::<u64>().unwrap_or(60);
                Trigger::Timer(interval)
            }
            "event" => Trigger::Event(rule.trigger_config.clone()),
            _ => unreachable!(),
        };
        engine.register(ProactiveAgent {
            id: rule.id.clone(),
            trigger,
            action: rule.action.clone(),
            counter: 0,
        });
    }

    Ok(rule.into())
}

#[tauri::command]
pub(crate) fn toggle_proactive_rule(
    state: State<AppState>,
    rule_id: String,
    enabled: bool,
) -> Result<(), CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let storage = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    storage
        .toggle_proactive_rule(&rule_id, enabled)
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    // Sync engine
    if let Ok(mut engine) = state.proactive_engine.lock() {
        if enabled {
            if let Ok(Some(rule)) = storage.get_proactive_rule(&rule_id) {
                let trigger = match rule.trigger_type.as_str() {
                    "timer" => {
                        let interval = rule.trigger_config.parse::<u64>().unwrap_or(60);
                        Trigger::Timer(interval)
                    }
                    "event" => Trigger::Event(rule.trigger_config.clone()),
                    _ => return Ok(()),
                };
                engine.register(ProactiveAgent {
                    id: rule.id,
                    trigger,
                    action: rule.action,
                    counter: 0,
                });
            }
        } else {
            engine.remove(&rule_id);
        }
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn delete_proactive_rule(
    state: State<AppState>,
    rule_id: String,
) -> Result<(), CommandError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let storage = storage
        .as_ref()
        .ok_or_else(|| CommandError::Internal("Storage not available".to_string()))?;
    storage
        .delete_proactive_rule(&rule_id)
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    if let Ok(mut engine) = state.proactive_engine.lock() {
        engine.remove(&rule_id);
    }

    Ok(())
}
