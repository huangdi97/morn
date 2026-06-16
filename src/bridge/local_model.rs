//! local_model — OpenAI-compatible local LLM client for Ollama and similar endpoints.
use crate::core::error::MornError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalModelConfig {
    pub endpoint: String,
    pub model_name: String,
    pub provider_type: String,
}

pub struct LocalLlmClient {
    config: LocalModelConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
struct LocalChatRequest {
    model: String,
    messages: Vec<LocalMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LocalChatResponse {
    choices: Vec<LocalChoice>,
}

#[derive(Debug, Deserialize)]
struct LocalChoice {
    message: LocalMessage,
}

impl LocalModelConfig {
    pub fn ollama(model_name: &str) -> Self {
        Self {
            endpoint: "http://localhost:11434/v1".to_string(),
            model_name: model_name.to_string(),
            provider_type: "ollama".to_string(),
        }
    }
}

impl LocalLlmClient {
    pub fn new(config: LocalModelConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn chat(&self, prompt: &str, system_prompt: &str) -> Result<String, MornError> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| MornError::Internal(e.to_string()))?;
        runtime.block_on(self.chat_async(prompt, system_prompt))
    }

    pub async fn chat_async(&self, prompt: &str, system_prompt: &str) -> Result<String, MornError> {
        let request = LocalChatRequest {
            model: self.config.model_name.clone(),
            messages: vec![
                LocalMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                LocalMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            stream: false,
        };

        let response = self
            .client
            .post(format!(
                "{}/chat/completions",
                self.config.endpoint.trim_end_matches('/')
            ))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| MornError::Internal(format!("Local LLM request error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MornError::Internal(format!("Local LLM API error {}: {}", status, body)));
        }

        let chat_response: LocalChatResponse = response
            .json()
            .await
            .map_err(|e| MornError::Internal(format!("Local LLM JSON parse error: {}", e)))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| MornError::Internal("Local LLM response contained no choices".to_string()))
    }

    pub fn get_config(&self) -> &LocalModelConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_ollama_defaults() {
        let config = LocalModelConfig::ollama("llama3");

        assert_eq!(config.endpoint, "http://localhost:11434/v1");
        assert_eq!(config.model_name, "llama3");
        assert_eq!(config.provider_type, "ollama");
    }

    #[test]
    fn test_client_get_config() {
        let config = LocalModelConfig {
            endpoint: "http://127.0.0.1:8080/v1".into(),
            model_name: "custom".into(),
            provider_type: "openai-compatible".into(),
        };
        let client = LocalLlmClient::new(config.clone());

        assert_eq!(client.get_config(), &config);
    }
}
