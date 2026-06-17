use crate::AppState;
use crate::MornError;
use tauri::State;

use morn::core::assembler::AgentDef;
use morn::studio::manager::{CreateComponentDef, StudioManager, UpdateComponentDef};

#[tauri::command]
pub(crate) fn list_components(
    type_filter: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let components = mgr.list_components(type_filter.as_deref());
    Ok(serde_json::to_value(components).map_err(|e| MornError::Internal(e.to_string()))?)
}

#[tauri::command]
pub(crate) fn get_component(
    id: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let detail = mgr.get_component(&id)?;
    Ok(serde_json::to_value(detail).map_err(|e| MornError::Internal(e.to_string()))?)
}

#[tauri::command]
pub(crate) fn create_component(
    name: String,
    component_type: String,
    config_json: Option<String>,
    state: State<AppState>,
) -> Result<String, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let id = mgr.create_component(CreateComponentDef {
        name,
        component_type,
        config_json,
    })?;
    Ok(id)
}

#[tauri::command]
pub(crate) fn update_component(
    id: String,
    name: Option<String>,
    config_json: Option<String>,
    status: Option<String>,
    state: State<AppState>,
) -> Result<(), MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.update_component(
        &id,
        UpdateComponentDef {
            name,
            config_json,
            status,
        },
    )
}

#[tauri::command]
pub(crate) fn delete_component(id: String, state: State<AppState>) -> Result<(), MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    mgr.delete_component(&id)
}

#[tauri::command]
pub(crate) fn assemble_agent(
    name: String,
    persona: String,
    model: String,
    tools: Vec<String>,
    knowledge: Vec<String>,
    skills: Vec<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;

    let persona_obj = match persona.as_str() {
        "researcher" => morn::component::persona::create_researcher_persona(),
        "analyst" => morn::component::persona::create_analyst_persona(),
        "writer" => morn::component::persona::create_writer_persona(),
        "coder" => morn::component::persona::create_coder_persona(),
        "translator" => morn::component::persona::create_translator_persona(),
        "reviewer" => morn::component::persona::create_reviewer_persona(),
        "cs_agent" => morn::component::persona::create_cs_agent_persona(),
        _ => morn::component::persona::create_assistant_persona(),
    };

    let model_obj = morn::component::model::ModelConfig {
        id: format!("model-{}", uuid::Uuid::new_v4()),
        provider: "deepseek".into(),
        model_name: model,
        base_url: "https://api.deepseek.com".into(),
        api_key: std::env::var("MORN_API_KEY").unwrap_or_default(),
        parameters: morn::component::model::ModelParameters::default(),
        fallback: None,
        cost_tier: morn::component::model::CostTier::Low,
    };

    let agent_id = mgr.assemble_agent(AgentDef {
        id: format!("agent-{}", uuid::Uuid::new_v4()),
        name,
        persona: persona_obj,
        model: model_obj,
        tools,
        knowledge,
        skills,
        memory: None,
    })?;

    Ok(serde_json::json!({ "agent_id": agent_id }))
}

#[tauri::command]
pub(crate) fn list_agent_templates(state: State<AppState>) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let templates = mgr.list_templates();
    Ok(serde_json::to_value(templates).map_err(|e| MornError::Internal(e.to_string()))?)
}

#[tauri::command]
pub(crate) fn test_component(
    id: String,
    input: String,
    component_type: Option<String>,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let data = morn::core::component::Data::text(&input);
    let result = mgr.test_component(&id, data, component_type.as_deref())?;
    Ok(serde_json::to_value(result).map_err(|e| MornError::Internal(e.to_string()))?)
}

#[tauri::command]
pub(crate) fn test_component_rerun(
    id: String,
    component_type: String,
    step_index: usize,
    new_input: String,
    state: State<AppState>,
) -> Result<serde_json::Value, MornError> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let mgr = manager
        .as_ref()
        .ok_or_else(|| "StudioManager not initialized".to_string())?;
    let step = mgr.rerun_component_step(&component_type, &id, step_index, &new_input)?;
    Ok(serde_json::to_value(step).map_err(|e| MornError::Internal(e.to_string()))?)
}

pub(crate) fn list_component_types() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"type": "agent", "label": "Agent", "icon": "🤖"}),
        serde_json::json!({"type": "tool", "label": "Tool", "icon": "🔧"}),
        serde_json::json!({"type": "workflow", "label": "Workflow", "icon": "⚙️"}),
        serde_json::json!({"type": "knowledge", "label": "Knowledge", "icon": "📚"}),
        serde_json::json!({"type": "persona", "label": "Persona", "icon": "🧑"}),
    ]
}

#[tauri::command]
pub(crate) fn publish_component(id: String, state: State<AppState>) -> Result<(), MornError> {
    let publisher = state
        .publisher
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let pubr = publisher
        .as_ref()
        .ok_or_else(|| "StudioPublisher not initialized".to_string())?;
    pubr.publish_agent(&id)
}
