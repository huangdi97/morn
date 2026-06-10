//! local_engine — Scans and manages local GGUF models for on-device inference.

pub struct LocalEngine {
    pub models: Vec<LocalModel>,
}

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
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    let size =
                        std::fs::metadata(&path).map(|m| m.len() / 1_048_576).unwrap_or(0);
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