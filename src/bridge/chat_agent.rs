//! chat_agent — Adapts chat model calls into the bridge agent interface.
use crate::core::error::MornError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing;

use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE};
use crate::core::model_router::{ModelRouter, ModelType, RoutedModel};

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: u32,
}

pub struct ChatAgent {
    client: Client,
    api_key: String,
    api_key_header: String,
    api_url: String,
    model: String,
    event_bus: Option<SimpleEventBus>,
}

impl ChatAgent {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        ChatAgent {
            client: Client::new(),
            api_key: api_key.to_string(),
            api_key_header: "Authorization".to_string(),
            api_url: format!("{}/chat/completions", base_url.trim_end_matches('/')),
            model: model.to_string(),
            event_bus: None,
        }
    }

    pub fn from_router(router: &ModelRouter, request: &str) -> Result<Self, MornError> {
        let route = router.route(request)?;
        Self::from_route(&route)
    }

    pub fn from_route(route: &RoutedModel) -> Result<Self, MornError> {
        if route.base_url.trim().is_empty() {
            return Err(MornError::Internal(format!(
                "Routed model '{}' from provider '{}' has no chat endpoint",
                route.name, route.provider
            )));
        }

        let api_key = route.api_key.clone().unwrap_or_default();
        if route.model_type == ModelType::Cloud && api_key.trim().is_empty() {
            return Err(MornError::Internal(format!(
                "Routed cloud model '{}' from provider '{}' is missing an API key",
                route.name, route.provider
            )));
        }

        Ok(ChatAgent {
            client: Client::new(),
            api_key,
            api_key_header: route.api_key_header.clone(),
            api_url: format!("{}/chat/completions", route.base_url.trim_end_matches('/')),
            model: route.name.clone(),
            event_bus: None,
        })
    }

    pub fn with_event_bus(mut self, event_bus: SimpleEventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn chat(&self, prompt: &str, system_prompt: &str) -> Result<String, MornError> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| MornError::Internal(e.to_string()))?;
        runtime.block_on(self.chat_async(prompt, system_prompt))
    }

    pub async fn chat_async(&self, prompt: &str, system_prompt: &str) -> Result<String, MornError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            stream: false,
        };

        let mut last_error = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            let mut builder = self
                .client
                .post(&self.api_url)
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(std::time::Duration::from_secs(30));

            if !self.api_key.trim().is_empty() {
                builder = if self.api_key_header == "Authorization" {
                    builder.header("Authorization", format!("Bearer {}", self.api_key))
                } else {
                    builder.header(&self.api_key_header, self.api_key.clone())
                };
            }

            match builder.send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        last_error = format!("API error {}: {}", status, body);
                        continue;
                    }

                    let chat_response: ChatResponse = response
                        .json()
                        .await
                        .map_err(|e| MornError::Internal(format!("JSON parse error: {}", e)))?;

                    let content = chat_response
                        .choices
                        .into_iter()
                        .next()
                        .map(|c| c.message.content)
                        .unwrap_or_default();

                    let total_tokens = chat_response.usage.map(|u| u.total_tokens).unwrap_or(0);

                    if let Some(ref bus) = self.event_bus {
                        bus.publish_event(
                            EVENT_CHAT_AGENT_RESPONSE,
                            "chat_agent",
                            serde_json::json!({
                                "model": self.model,
                                "tokens": total_tokens,
                                "response_length": content.len(),
                            }),
                        );
                    }

                    return Ok(content);
                }
                Err(e) => {
                    last_error = format!("Request error: {}", e);
                    if attempt < 2 {
                        tracing::info!("[ChatAgent] Retry {}/3 after error: {}", attempt + 1, e);
                    }
                }
            }
        }

        Err(MornError::Internal(format!("ChatAgent failed after 3 retries: {}", last_error)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_agent_new() {
        let agent = ChatAgent::new("test-key", "https://api.deepseek.com", "deepseek-chat");
        assert_eq!(agent.model, "deepseek-chat");
        assert_eq!(agent.api_url, "https://api.deepseek.com/chat/completions");
    }

    #[test]
    fn test_chat_agent_from_router_uses_configured_model() {
        let router = ModelRouter::with_default_model(
            "deepseek",
            "deepseek-reasoner",
            "https://api.deepseek.com",
            Some("test-key".to_string()),
        );

        let agent = ChatAgent::from_router(&router, "hello").unwrap();

        assert_eq!(agent.model, "deepseek-reasoner");
        assert_eq!(agent.api_url, "https://api.deepseek.com/chat/completions");
    }
}
