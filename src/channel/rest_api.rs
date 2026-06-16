//! rest_api — Provides a channel adapter backed by REST-style message handling.
use crate::core::error::MornError;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex as AsyncMutex;
use tower_http::cors::CorsLayer;

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};
use crate::component::tool::get_tool_by_name;
use crate::core::component::Data;
use crate::core::registry::Registry;
use crate::core::supervisor::Supervisor;
use crate::core::workflow::WorkflowTemplate;

#[derive(Clone)]
pub struct RestApiState {
    pub adapter: Arc<Mutex<Option<ChannelAdapter>>>,
    pub turn_count: Arc<Mutex<u64>>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub turn_count: u64,
    pub components: Vec<String>,
}

pub struct RestApiServer {
    adapter: Option<ChannelAdapter>,
    turn_count: u64,
}

impl RestApiServer {
    pub fn new(adapter: Option<ChannelAdapter>) -> Self {
        RestApiServer {
            adapter,
            turn_count: 0,
        }
    }

    pub fn chat(&mut self, text: &str) -> Result<String, MornError> {
        self.turn_count += 1;
        if let Some(ref mut adapter) = self.adapter {
            let msg = ChannelMessage {
                content: text.to_string(),
                source: "rest_api".into(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                metadata: serde_json::json!({}),
            };
            Ok(adapter.handle_message(&msg))
        } else {
            Ok("No adapter configured".into())
        }
    }

    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({
            "version": "0.1.0",
            "turn_count": self.turn_count,
            "components": ["chat-agent"],
        })
    }

    pub fn clear(&mut self) -> Result<(), MornError> {
        self.turn_count = 0;
        Ok(())
    }

    pub async fn serve(self) -> Result<(), MornError> {
        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| MornError::Internal(format!("Invalid API_PORT: {}", e)))?;

        let state = RestApiState {
            adapter: Arc::new(Mutex::new(self.adapter)),
            turn_count: Arc::new(Mutex::new(self.turn_count)),
        };

        let app = Router::new()
            .route("/chat", post(chat_handler))
            .route("/status", get(status_handler))
            .route("/clear", post(clear_handler))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", port);
        println!("[REST API] Server starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| MornError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| MornError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}

async fn chat_handler(
    State(state): State<RestApiState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, Json<Value>> {
    let mut adapter = state
        .adapter
        .lock()
        .map_err(|e| Json(serde_json::json!({"error": format!("Lock error: {}", e)})))?;

    let response = if let Some(ref mut adapter) = *adapter {
        let msg = ChannelMessage {
            content: req.text.clone(),
            source: "rest_api".into(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: serde_json::json!({}),
        };
        adapter.handle_message(&msg)
    } else {
        "No adapter configured".to_string()
    };

    if let Ok(mut turn) = state.turn_count.lock() {
        *turn += 1;
    }

    Ok(Json(ChatResponse { reply: response }))
}

async fn status_handler(State(state): State<RestApiState>) -> Json<StatusResponse> {
    let turn_count = state.turn_count.lock().map(|t| *t).unwrap_or(0);
    Json(StatusResponse {
        version: "0.1.0".to_string(),
        turn_count,
        components: vec!["chat-agent".to_string()],
    })
}

async fn clear_handler(State(state): State<RestApiState>) -> Result<Json<Value>, Json<Value>> {
    if let Ok(mut turn) = state.turn_count.lock() {
        *turn = 0;
    }
    Ok(Json(serde_json::json!({"success": true})))
}

type ChatFn = Arc<dyn Fn(&str, &str) -> Result<String, MornError> + Send + Sync>;

pub struct ApiState {
    pub supervisor: Arc<AsyncMutex<Supervisor>>,
    pub registry: Arc<AsyncMutex<Registry>>,
    pub chat_fn: ChatFn,
}

#[derive(Deserialize)]
pub struct ApiChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiChatResponse {
    pub reply: String,
}

#[derive(Debug, Serialize)]
pub struct ApiHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
}

#[derive(Debug, Serialize)]
pub struct ApiToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct ApiToolExecuteRequest {
    pub input: Value,
}

#[derive(Debug, Serialize)]
pub struct ApiToolExecuteResponse {
    pub output: Value,
}

#[derive(Debug, Serialize)]
pub struct ApiWorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub steps: usize,
    pub estimated_duration_secs: u64,
}

impl From<&WorkflowTemplate> for ApiWorkflowInfo {
    fn from(w: &WorkflowTemplate) -> Self {
        ApiWorkflowInfo {
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

pub async fn serve(state: ApiState) -> Result<(), MornError> {
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .map_err(|e| MornError::Internal(format!("Invalid API_PORT: {}", e)))?;

    let app = Router::new()
        .route("/health", get(api_health_handler))
        .route("/chat", post(api_chat_handler))
        .route("/tools", get(api_tools_list_handler))
        .route("/tools/{name}/execute", post(api_tool_execute_handler))
        .route("/workflows", get(api_workflows_list_handler))
        .route("/workflows/{id}", get(api_workflow_get_handler));

    #[cfg(feature = "channels-full")]
    let app = app.route("/ws", get(crate::channel::browser_ext::ws_handler));

    let app = app
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = format!("0.0.0.0:{}", port);
    println!("[REST API] Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| MornError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| MornError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn api_health_handler() -> Json<ApiHealthResponse> {
    Json(ApiHealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: chrono::Utc::now().to_rfc3339(),
    })
}

async fn api_chat_handler(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ApiChatRequest>,
) -> Result<Json<ApiChatResponse>, Json<Value>> {
    let mut supervisor = state.supervisor.lock().await;
    let chat_fn = state.chat_fn.clone();

    match supervisor.execute_chat(&req.message, &*chat_fn) {
        Ok(reply) => Ok(Json(ApiChatResponse { reply })),
        Err(e) => Err(Json(serde_json::json!({"error": e}))),
    }
}

async fn api_tools_list_handler(State(state): State<Arc<ApiState>>) -> Json<Vec<ApiToolInfo>> {
    let registry = state.registry.lock().await;
    let caps = registry.list_all();

    let tools: Vec<ApiToolInfo> = caps
        .iter()
        .map(|c| ApiToolInfo {
            name: c.id.clone(),
            description: c.description.clone(),
        })
        .collect();

    Json(tools)
}

async fn api_tool_execute_handler(
    Path(name): Path<String>,
    Json(req): Json<ApiToolExecuteRequest>,
) -> Result<Json<ApiToolExecuteResponse>, Json<Value>> {
    let mut tool = get_tool_by_name(&name)
        .ok_or_else(|| Json(serde_json::json!({"error": format!("Unknown tool: {}", name)})))?;

    let input = Data {
        content: req.input,
        mime_type: "application/json".to_string(),
    };

    match tool.execute(input) {
        Ok(output) => Ok(Json(ApiToolExecuteResponse {
            output: output.content,
        })),
        Err(e) => Err(Json(serde_json::json!({"error": e}))),
    }
}

async fn api_workflows_list_handler() -> Json<Vec<ApiWorkflowInfo>> {
    let templates = WorkflowTemplate::list_builtin();
    let workflows: Vec<ApiWorkflowInfo> = templates.iter().map(ApiWorkflowInfo::from).collect();
    Json(workflows)
}

async fn api_workflow_get_handler(
    Path(id): Path<String>,
) -> Result<Json<ApiWorkflowInfo>, Json<Value>> {
    match WorkflowTemplate::get_by_id(&id) {
        Some(template) => Ok(Json(ApiWorkflowInfo::from(&template))),
        None => Err(Json(
            serde_json::json!({"error": format!("Workflow not found: {}", id)}),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_state_can_be_created() {
        let state = ApiState {
            supervisor: Arc::new(tokio::sync::Mutex::new(Supervisor::new(None, None))),
            registry: Arc::new(tokio::sync::Mutex::new(Registry::new(None, None))),
            chat_fn: Arc::new(|_, _| Ok("ok".to_string())),
        };

        assert_eq!(Arc::strong_count(&state.supervisor), 1);
        assert_eq!(Arc::strong_count(&state.registry), 1);
        assert!((state.chat_fn)("hello", "context").is_ok());
    }

    #[test]
    fn router_can_be_created() {
        let _router: Router = Router::new();
    }

    fn api_state() -> Arc<ApiState> {
        Arc::new(ApiState {
            supervisor: Arc::new(tokio::sync::Mutex::new(Supervisor::new(None, None))),
            registry: Arc::new(tokio::sync::Mutex::new(Registry::new(None, None))),
            chat_fn: Arc::new(|_, _| Ok("ok".to_string())),
        })
    }

    #[test]
    fn api_routes_can_be_registered() {
        let state = api_state();

        let _router: Router = Router::new()
            .route("/health", get(api_health_handler))
            .route("/chat", post(api_chat_handler))
            .route("/tools", get(api_tools_list_handler))
            .route("/tools/{name}/execute", post(api_tool_execute_handler))
            .route("/workflows", get(api_workflows_list_handler))
            .route("/workflows/{id}", get(api_workflow_get_handler))
            .with_state(state);
    }

    #[tokio::test]
    async fn workflow_path_parameter_is_parsed() {
        let response = api_workflow_get_handler(Path("workflow-code-delivery".to_string()))
            .await
            .unwrap();

        assert_eq!(response.id, "workflow-code-delivery");
        assert_eq!(response.steps, 7);
    }

    #[tokio::test]
    async fn tool_execute_returns_error_response_for_unknown_tool() {
        let err = api_tool_execute_handler(
            Path("missing-tool".to_string()),
            Json(ApiToolExecuteRequest {
                input: serde_json::json!({}),
            }),
        )
        .await
        .unwrap_err();

        assert!(err["error"].as_str().unwrap().contains("Unknown tool"));
    }

    #[tokio::test]
    async fn health_handler_returns_ok_status() {
        let response = api_health_handler().await;

        assert_eq!(response.status, "ok");
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert!(!response.uptime.is_empty());
    }

    #[tokio::test]
    async fn chat_handler_returns_error_response_when_supervisor_fails() {
        let state = Arc::new(ApiState {
            supervisor: Arc::new(tokio::sync::Mutex::new(Supervisor::new(None, None))),
            registry: Arc::new(tokio::sync::Mutex::new(Registry::new(None, None))),
            chat_fn: Arc::new(|_, _| Err(MornError::Internal("chat failed".to_string()))),
        });

        let err = api_chat_handler(
            State(state),
            Json(ApiChatRequest {
                message: "hello".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(err.0, serde_json::json!({"error": {"Internal": "chat failed"}}));
    }
}
