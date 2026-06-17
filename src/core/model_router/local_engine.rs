//! local_engine — Scans and manages local GGUF models for on-device inference.

use crate::core::error::MornError;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct LocalEngine {
    pub models: Vec<LocalModel>,
}

#[derive(Debug, Clone)]
pub struct LocalModel {
    pub path: String,
    pub name: String,
    pub size_mb: u64,
}

impl LocalEngine {
    pub fn new() -> Self {
        LocalEngine { models: vec![] }
    }

    pub fn discover(gguf_dir: &str) -> Result<Vec<LocalModel>, MornError> {
        let mut models = vec![];
        if let Ok(entries) = std::fs::read_dir(gguf_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "gguf") {
                    let name = path
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let size = std::fs::metadata(&path)
                        .map(|m| m.len() / 1_048_576)
                        .unwrap_or(0);
                    models.push(LocalModel {
                        path: path.to_string_lossy().to_string(),
                        name,
                        size_mb: size as u64,
                    });
                }
            }
        }
        Ok(models)
    }

    pub fn supports_inference(&self) -> bool {
        !self.models.is_empty()
    }

    pub fn inference(&self, prompt: &str) -> Result<String, MornError> {
        let model = self.models.first().ok_or("No local model found")?;
        let output = std::process::Command::new("llama-cli")
            .args(["-m", &model.path, "-p", prompt, "-n", "256"])
            .output()
            .map_err(|e| MornError::Internal(format!("llama-cli not found: {}", e)))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(MornError::Internal(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    #[cfg(feature = "channels-full")]
    pub fn inference_ollama(&self, prompt: &str, model: &str) -> Result<String, MornError> {
        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        let response = reqwest::blocking::Client::new()
            .post("http://localhost:11434/api/generate")
            .json(&payload)
            .send()
            .map_err(|e| MornError::Internal(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(MornError::Internal(format!(
                "Ollama API error {}: {}",
                status, body
            )));
        }

        let body: OllamaGenerateResponse = response
            .json()
            .map_err(|e| MornError::Internal(format!("Ollama JSON parse error: {}", e)))?;

        if let Some(error) = body.error {
            return Err(MornError::Internal(format!("Ollama API error: {}", error)));
        }

        Ok(body.response.unwrap_or_default())
    }

    #[cfg(not(feature = "channels-full"))]
    pub fn inference_ollama(&self, _prompt: &str, _model: &str) -> Result<String, MornError> {
        Err(MornError::Internal(
            "Ollama HTTP inference requires the channels-full feature".to_string(),
        ))
    }

    #[cfg(feature = "channels-full")]
    pub fn inference_lm_studio(&self, prompt: &str) -> Result<String, MornError> {
        let payload = serde_json::json!({
            "model": "local-model",
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "stream": false,
        });

        let response = reqwest::blocking::Client::new()
            .post("http://localhost:1234/v1/chat/completions")
            .json(&payload)
            .send()
            .map_err(|e| MornError::Internal(format!("LM Studio request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(MornError::Internal(format!(
                "LM Studio API error {}: {}",
                status, body
            )));
        }

        let body: LmStudioChatResponse = response
            .json()
            .map_err(|e| MornError::Internal(format!("LM Studio JSON parse error: {}", e)))?;

        Ok(body
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default())
    }

    #[cfg(not(feature = "channels-full"))]
    pub fn inference_lm_studio(&self, _prompt: &str) -> Result<String, MornError> {
        Err(MornError::Internal(
            "LM Studio HTTP inference requires the channels-full feature".to_string(),
        ))
    }
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LmStudioChatResponse {
    choices: Vec<LmStudioChoice>,
}

#[derive(Debug, Deserialize)]
struct LmStudioChoice {
    message: LmStudioMessage,
}

#[derive(Debug, Deserialize)]
struct LmStudioMessage {
    content: String,
}

impl Default for LocalEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_empty_for_nonexistent_dir() {
        let models = LocalEngine::discover("/nonexistent-gguf-path").unwrap();
        assert!(models.is_empty());
    }

    #[test]
    fn new_engine_has_no_models() {
        let engine = LocalEngine::new();
        assert!(engine.models.is_empty());
    }
}
