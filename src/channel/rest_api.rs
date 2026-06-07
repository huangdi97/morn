//! rest_api — Provides a channel adapter backed by REST-style message handling.
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::channel::adapter::{ChannelAdapter, ChannelMessage};

#[derive(Clone)]
pub struct RestApiState {
    pub adapter: Arc<Mutex<Option<ChannelAdapter>>>,
    pub turn_count: Arc<Mutex<u64>>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Serialize)]
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

    pub fn chat(&mut self, text: &str) -> Result<String, String> {
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

    pub fn clear(&mut self) -> Result<(), String> {
        self.turn_count = 0;
        Ok(())
    }

    pub async fn serve(self) -> Result<(), String> {
        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid API_PORT: {}", e))?;

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
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| format!("Server error: {}", e))?;

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
