//! browser_ext — WebSocket channel for browser extension communication.
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::channel::rest_api::ApiState;

/// Incoming message from the browser extension.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BrowserExtMessage {
    #[serde(rename = "page_context")]
    PageContext {
        url: String,
        title: String,
        content: String,
        selection: Option<String>,
    },
    #[serde(rename = "chat")]
    Chat { text: String },
}

/// Outgoing response to the browser extension.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrowserExtResponse {
    #[serde(rename = "ack")]
    Ack { message: String },
    #[serde(rename = "reply")]
    Reply { text: String },
    #[serde(rename = "suggestion")]
    Suggestion { text: String, actions: Vec<String> },
    #[serde(rename = "error")]
    Error { message: String },
}

/// WebSocket upgrade handler — called when client connects to /ws.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ApiState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<ApiState>) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(message)) = receiver.next().await {
        let response = match message {
            Message::Text(text) => handle_text_message(text.as_str(), &state).await,
            Message::Close(_) => break,
            _ => continue,
        };

        let response_text = match serde_json::to_string(&response) {
            Ok(text) => text,
            Err(e) => {
                format!(
                    r#"{{"type":"error","message":"serialization failed: {}"}}"#,
                    e
                )
            }
        };

        if sender
            .send(Message::Text(response_text.into()))
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn handle_text_message(text: &str, state: &Arc<ApiState>) -> BrowserExtResponse {
    match serde_json::from_str::<BrowserExtMessage>(text) {
        Ok(BrowserExtMessage::PageContext {
            url,
            title,
            content,
            selection,
        }) => BrowserExtResponse::Ack {
            message: format!(
                "Context received for '{}' at {} ({} chars, selection: {})",
                title,
                url,
                content.len(),
                selection.is_some()
            ),
        },
        Ok(BrowserExtMessage::Chat { text }) => {
            let mut supervisor = state.supervisor.lock().await;
            let chat_fn = state.chat_fn.clone();

            match supervisor.execute_chat(&text, &*chat_fn) {
                Ok(reply) => BrowserExtResponse::Reply { text: reply },
                Err(e) => BrowserExtResponse::Error { message: e },
            }
        }
        Err(e) => BrowserExtResponse::Error {
            message: format!("Invalid message: {}", e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::registry::Registry;
    use crate::core::supervisor::Supervisor;
    use tokio::sync::Mutex as AsyncMutex;

    fn test_state() -> Arc<ApiState> {
        Arc::new(ApiState {
            supervisor: Arc::new(AsyncMutex::new(Supervisor::new(None, None))),
            registry: Arc::new(AsyncMutex::new(Registry::new(None, None))),
            chat_fn: Arc::new(|prompt, _system| Ok(format!("reply: {}", prompt))),
        })
    }

    #[tokio::test]
    async fn parses_page_context_as_ack() {
        let response = handle_text_message(
            r#"{"type":"page_context","url":"https://example.com","title":"Example","content":"body","selection":null}"#,
            &test_state(),
        )
        .await;
        assert!(matches!(response, BrowserExtResponse::Ack { .. }));
    }

    #[tokio::test]
    async fn parses_chat_as_reply() {
        let response = handle_text_message(r#"{"type":"chat","text":"hello"}"#, &test_state()).await;
        match response {
            BrowserExtResponse::Reply { text } => assert!(text.contains("reply")),
            other => panic!("unexpected response: {:?}", serde_json::to_string(&other)),
        }
    }

    #[tokio::test]
    async fn invalid_json_returns_error() {
        let response = handle_text_message("not-json", &test_state()).await;
        assert!(matches!(response, BrowserExtResponse::Error { .. }));
    }
}
