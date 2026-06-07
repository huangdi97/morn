//! chat_agent — Adapts chat model calls into the bridge agent interface.
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE};

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
    api_url: String,
    model: String,
    event_bus: Option<SimpleEventBus>,
}

impl ChatAgent {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        ChatAgent {
            client: Client::new(),
            api_key: api_key.to_string(),
            api_url: format!("{}/chat/completions", base_url.trim_end_matches('/')),
            model: model.to_string(),
            event_bus: None,
        }
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

    pub fn chat(&self, prompt: &str, system_prompt: &str) -> Result<String, String> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        runtime.block_on(self.chat_async(prompt, system_prompt))
    }

    pub async fn chat_async(&self, prompt: &str, system_prompt: &str) -> Result<String, String> {
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

            match self
                .client
                .post(&self.api_url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
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
                        .map_err(|e| format!("JSON parse error: {}", e))?;

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
                        eprintln!("[ChatAgent] Retry {}/3 after error: {}", attempt + 1, e);
                    }
                }
            }
        }

        Err(format!("ChatAgent failed after 3 retries: {}", last_error))
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
}
