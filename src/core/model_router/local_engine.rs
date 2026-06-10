//! local_engine — Scans and manages local GGUF models for on-device inference.

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

    pub fn discover(gguf_dir: &str) -> Result<Vec<LocalModel>, String> {
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

    pub fn inference(&self, prompt: &str) -> Result<String, String> {
        let model = self.models.first().ok_or("No local model found")?;
        let output = std::process::Command::new("llama-cli")
            .args(["-m", &model.path, "-p", prompt, "-n", "256"])
            .output()
            .map_err(|e| format!("llama-cli not found: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
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
