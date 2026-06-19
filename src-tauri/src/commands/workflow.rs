use crate::AppState;
use crate::MornError;
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::State;
use morn::core::workflow::{StepResult, WorkflowTemplateDef, WorkflowStepDef};

fn get_storage(state: &State<AppState>) -> Result<std::sync::MutexGuard<Option<morn::core::storage::Storage>>, MornError> {
    state.storage.lock().map_err(|e| MornError::Internal(e.to_string()))
}

#[tauri::command]
pub(crate) fn list_workflow_templates(state: State<AppState>) -> Result<Vec<WorkflowTemplateDef>, MornError> {
    let storage = get_storage(&state)?;
    match storage.as_ref() {
        Some(s) => {
            let templates: Vec<WorkflowTemplateDef> = s.get_setting("workflow_templates")
                .ok()
                .flatten()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();
            Ok(templates)
        }
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub(crate) fn list_workflow_node_types() -> Result<serde_json::Value, MornError> {
    let json = serde_json::json!([
        {"type": "llm_call", "label": "LLM 调用", "category": "🤖 AI 处理"},
        {"type": "agent_call", "label": "Agent 调用", "category": "🤖 AI 处理"},
        {"type": "analyze", "label": "分析", "category": "🤖 AI 处理"},
        {"type": "tool_exec", "label": "工具执行", "category": "🔧 工具"},
        {"type": "api_request", "label": "API 请求", "category": "🔧 工具"},
        {"type": "web_search", "label": "网页搜索", "category": "🔍 搜索/获取"},
        {"type": "kb_query", "label": "知识库查询", "category": "🔍 搜索/获取"},
        {"type": "read_file", "label": "文件读取", "category": "🔍 搜索/获取"},
        {"type": "generate_report", "label": "生成报告", "category": "📤 输出"},
        {"type": "notify", "label": "推送通知", "category": "📤 输出"},
        {"type": "write_file", "label": "写文件", "category": "📤 输出"},
    ]);
    Ok(json)
}

#[tauri::command]
pub(crate) fn execute_workflow(state: State<AppState>, template: WorkflowTemplateDef) -> Result<Vec<StepResult>, MornError> {
    let step_results = Mutex::new(Vec::new());

    let mut executed = HashSet::new();
    let steps = template.steps.clone();

    let mut made_progress = true;
    while executed.len() < steps.len() && made_progress {
        made_progress = false;
        for step in &steps {
            if executed.contains(&step.id) { continue; }

            let deps_met = step.depends_on.iter().all(|d| executed.contains(d));
            if !deps_met { continue; }

            let start = std::time::Instant::now();
            let result = match step.action_type.as_str() {
                "llm_call" => {
                    Ok(serde_json::json!({"status": "completed", "output": "LLM call simulation"}))
                }
                "tool_exec" => {
                    Ok(serde_json::json!({"status": "completed", "output": "Tool execution simulation"}))
                }
                "api_request" => {
                    Ok(serde_json::json!({"status": "completed", "output": "API request simulation"}))
                }
                "web_search" => {
                    Ok(serde_json::json!({"status": "completed", "output": "Search results simulation"}))
                }
                _ => {
                    Ok(serde_json::json!({"status": "completed", "output": format!("Executed: {}", step.action_type)}))
                }
            };

            let duration = start.elapsed().as_millis() as u64;
            executed.insert(step.id.clone());

            step_results.lock().map_err(|e| MornError::Internal(e.to_string()))?.push(
                StepResult {
                    step_id: step.id.clone(),
                    status: "success".to_string(),
                    output: result.unwrap_or(serde_json::json!({"error": "execution failed"})),
                    duration_ms: duration,
                }
            );
            made_progress = true;
        }
    }

    Ok(step_results.into_inner().map_err(|e| MornError::Internal(e.to_string()))?)
}

#[tauri::command]
pub(crate) fn save_workflow_template(state: State<AppState>, template: WorkflowTemplateDef) -> Result<(), MornError> {
    let storage = get_storage(&state)?;
    match storage.as_ref() {
        Some(s) => {
            let mut templates: Vec<WorkflowTemplateDef> = s.get_setting("workflow_templates")
                .ok()
                .flatten()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            if let Some(pos) = templates.iter().position(|t| t.id == template.id) {
                templates[pos] = template;
            } else {
                templates.push(template);
            }

            let json = serde_json::to_string(&templates)
                .map_err(|e| MornError::Internal(e.to_string()))?;
            s.set_setting("workflow_templates", &json)
                .map_err(|e| MornError::Internal(e.to_string()))?;
            Ok(())
        }
        None => Err(MornError::Internal("Storage not available".to_string())),
    }
}

#[tauri::command]
pub(crate) fn delete_workflow_template(state: State<AppState>, id: String) -> Result<(), MornError> {
    let storage = get_storage(&state)?;
    match storage.as_ref() {
        Some(s) => {
            let mut templates: Vec<WorkflowTemplateDef> = s.get_setting("workflow_templates")
                .ok()
                .flatten()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();
            templates.retain(|t| t.id != id);
            let json = serde_json::to_string(&templates)
                .map_err(|e| MornError::Internal(e.to_string()))?;
            s.set_setting("workflow_templates", &json)
                .map_err(|e| MornError::Internal(e.to_string()))?;
            Ok(())
        }
        None => Err(MornError::Internal("Storage not available".to_string())),
    }
}

#[tauri::command]
pub(crate) fn list_workflow_node_types() -> Result<serde_json::Value, MornError> {
    Ok(serde_json::json!([
        {"type": "llm_call", "label": "LLM 调用", "category": "🤖 AI 处理"},
        {"type": "agent_call", "label": "Agent 调用", "category": "🤖 AI 处理"},
        {"type": "analyze", "label": "分析", "category": "🤖 AI 处理"},
        {"type": "tool_exec", "label": "工具执行", "category": "🔧 工具"},
        {"type": "api_request", "label": "API 请求", "category": "🔧 工具"},
        {"type": "web_search", "label": "网页搜索", "category": "🔍 搜索/获取"},
        {"type": "kb_query", "label": "知识库查询", "category": "🔍 搜索/获取"},
        {"type": "read_file", "label": "文件读取", "category": "🔍 搜索/获取"},
        {"type": "generate_report", "label": "生成报告", "category": "📤 输出"},
        {"type": "notify", "label": "推送通知", "category": "📤 输出"},
        {"type": "write_file", "label": "写文件", "category": "📤 输出"},
    ]))
}
