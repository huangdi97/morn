use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::component::tool::get_tool_by_name;
use crate::core::component::Data;
use crate::core::registry::Registry;
use crate::core::supervisor::Supervisor;
use crate::core::workflow::WorkflowTemplate;

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

pub struct ApiState {
    pub supervisor: Arc<Mutex<Supervisor>>,
    pub registry: Arc<Mutex<Registry>>,
    pub chat_fn: ChatFn,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
}

#[derive(Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct ToolExecuteRequest {
    pub input: Value,
}

#[derive(Serialize)]
pub struct ToolExecuteResponse {
    pub output: Value,
}

#[derive(Serialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub steps: usize,
    pub estimated_duration_secs: u64,
}

impl From<&WorkflowTemplate> for WorkflowInfo {
    fn from(w: &WorkflowTemplate) -> Self {
        WorkflowInfo {
            id: w.id.clone(),
            name: w.name.clone(),
            description: w.description.clone(),
            category: w.category.clone(),
            tags: w.tags.clone(),
            steps: w.steps.len(),
            estimated_duration_secs: w.estimated_duration_secs,
        }
    }
}

pub async fn serve(state: ApiState) -> Result<(), String> {
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|e| format!("Invalid API_PORT: {}", e))?;

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/chat", post(chat_handler))
        .route("/tools", get(tools_list_handler))
        .route("/tools/{name}/execute", post(tool_execute_handler))
        .route("/workflows", get(workflows_list_handler))
        .route("/workflows/{id}", get(workflow_get_handler))
        .with_state(Arc::new(state));

    let addr = format!("0.0.0.0:{}", port);
    println!("[REST API] Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: chrono::Utc::now().to_rfc3339(),
    })
}

async fn chat_handler(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, Json<Value>> {
    let mut supervisor = state.supervisor.lock().await;
    let chat_fn = state.chat_fn.clone();

    match supervisor.execute_chat(&req.message, &*chat_fn) {
        Ok(reply) => Ok(Json(ChatResponse { reply })),
        Err(e) => Err(Json(serde_json::json!({"error": e}))),
    }
}

async fn tools_list_handler(
    State(state): State<Arc<ApiState>>,
) -> Json<Vec<ToolInfo>> {
    let registry = state.registry.lock().await;
    let caps = registry.list_all();

    let tools: Vec<ToolInfo> = caps
        .iter()
        .map(|c| ToolInfo {
            name: c.id.clone(),
            description: c.description.clone(),
        })
        .collect();

    Json(tools)
}

async fn tool_execute_handler(
    Path(name): Path<String>,
    Json(req): Json<ToolExecuteRequest>,
) -> Result<Json<ToolExecuteResponse>, Json<Value>> {
    let mut tool = get_tool_by_name(&name).ok_or_else(|| {
        Json(serde_json::json!({"error": format!("Unknown tool: {}", name)}))
    })?;

    let input = Data {
        content: req.input,
        mime_type: "application/json".to_string(),
    };

    match tool.execute(input) {
        Ok(output) => Ok(Json(ToolExecuteResponse {
            output: output.content,
        })),
        Err(e) => Err(Json(serde_json::json!({"error": e}))),
    }
}

async fn workflows_list_handler() -> Json<Vec<WorkflowInfo>> {
    let templates = WorkflowTemplate::list_builtin();
    let workflows: Vec<WorkflowInfo> = templates.iter().map(|w| WorkflowInfo::from(w)).collect();
    Json(workflows)
}

async fn workflow_get_handler(
    Path(id): Path<String>,
) -> Result<Json<WorkflowInfo>, Json<Value>> {
    match WorkflowTemplate::get_by_id(&id) {
        Some(template) => Ok(Json(WorkflowInfo::from(&template))),
        None => Err(Json(serde_json::json!({"error": format!("Workflow not found: {}", id)}))),
    }
}