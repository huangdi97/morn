use crate::commands::errors::CommandError;
use std::path::PathBuf;

fn get_models_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    path.push(".morn");
    path.push("models");
    path
}

fn get_model_hf_repo(name: &str) -> &str {
    match () {
        _ if name.starts_with("llama") => "meta-llama/Llama-3.2-3B-Instruct",
        _ if name.starts_with("qwen") => "Qwen/Qwen2.5-7B-Instruct",
        _ if name.starts_with("mistral") => "mistralai/Mistral-7B-Instruct-v0.3",
        _ if name.starts_with("phi") => "microsoft/Phi-3-mini-4k-instruct",
        _ if name.starts_with("deepseek") => "deepseek-ai/DeepSeek-R1-Distill-Qwen-7B",
        _ => "meta-llama/Llama-3.2-3B-Instruct",
    }
}

fn fetch_ollama_models() -> Vec<String> {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    runtime.block_on(async {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let resp = match client.get("http://localhost:11434/api/tags").send().await {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };

        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let mut models = Vec::new();
        if let Some(arr) = body["models"].as_array() {
            for m in arr {
                if let Some(name) = m["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }
        models
    })
}

fn fetch_lm_studio_models() -> Vec<String> {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    runtime.block_on(async {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let resp = match client.get("http://localhost:1234/v1/models").send().await {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };

        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };

        let mut models = Vec::new();
        if let Some(arr) = body["data"].as_array() {
            for m in arr {
                if let Some(id) = m["id"].as_str() {
                    models.push(id.to_string());
                }
            }
        }
        models
    })
}

#[tauri::command]
pub(crate) fn list_local_models() -> Result<Vec<String>, CommandError> {
    let models_dir = get_models_dir();
    let mut models = Vec::new();

    if models_dir.exists() {
        for entry in
            std::fs::read_dir(&models_dir).map_err(|e| CommandError::Internal(e.to_string()))?
        {
            let entry = entry.map_err(|e| CommandError::Internal(e.to_string()))?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "gguf") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    models.push(name.to_string());
                }
            }
        }
    }

    for ollama_model in fetch_ollama_models() {
        models.push(format!("ollama/{}", ollama_model));
    }

    for ls_model in fetch_lm_studio_models() {
        models.push(format!("lmstudio/{}", ls_model));
    }

    models.sort();
    Ok(models)
}

#[tauri::command]
pub(crate) async fn download_model(
    name: String,
    url: Option<String>,
) -> Result<String, CommandError> {
    if name.is_empty() {
        return Err(CommandError::NotFound("model name is empty".to_string()));
    }

    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir).map_err(|e| CommandError::Internal(e.to_string()))?;

    let model_path = models_dir.join(format!("{}.gguf", name));
    if model_path.exists() {
        return Ok(format!("already exists: {}", model_path.display()));
    }

    let download_url = url.unwrap_or_else(|| {
        format!(
            "https://huggingface.co/{}/resolve/main/{}.gguf",
            get_model_hf_repo(&name),
            name
        )
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| CommandError::Network(format!("Download failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(CommandError::Network(format!(
            "Download failed with status {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| CommandError::Network(format!("Failed to read response data: {}", e)))?;

    tokio::fs::write(&model_path, &bytes)
        .await
        .map_err(|e| CommandError::Internal(format!("Failed to write model file: {}", e)))?;

    Ok(format!("downloaded to {}", model_path.display()))
}

#[tauri::command]
pub(crate) fn delete_local_model(name: String) -> Result<String, CommandError> {
    if name.is_empty() {
        return Err(CommandError::NotFound("model name is empty".to_string()));
    }

    let model_path = get_models_dir().join(format!("{}.gguf", name));
    if !model_path.exists() {
        return Err(CommandError::NotFound(format!(
            "Model '{}' not found",
            name
        )));
    }

    std::fs::remove_file(&model_path)
        .map_err(|e| CommandError::Internal(format!("Failed to delete model file: {}", e)))?;

    Ok(format!("deleted {}", model_path.display()))
}
