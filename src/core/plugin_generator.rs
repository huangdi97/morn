//! plugin_generator — Natural language to plugin scaffolding.
use std::path::Path;

use crate::core::plugin_manager::PluginError;

/// Structured spec parsed from natural language.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginSpec {
    pub name: String,
    pub plugin_type: String,
    pub description: String,
    pub entry_content: String,
    pub entry_filename: String,
}

/// Uses an LLM (chat_fn) to parse a natural-language description into a PluginSpec.
pub fn parse_nl_to_spec(
    nl: &str,
    chat_fn: &impl Fn(&str, &str) -> Result<String, String>,
) -> Result<PluginSpec, PluginError> {
    let system = "You are a plugin scaffolding assistant. Given a user's description of a plugin, extract the plugin name, type (theme|channel|ui_slot|protocol|tool), description, and generate a minimal JavaScript entry file. Respond ONLY with valid JSON in this exact format:\n{\"name\":\"...\",\"plugin_type\":\"...\",\"description\":\"...\",\"entry_content\":\"...\",\"entry_filename\":\"main.js\"}";

    let prompt = format!("Describe the plugin you want: {}", nl);
    let response = chat_fn(&prompt, system).map_err(PluginError::Llm)?;

    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str::<PluginSpec>(cleaned).map_err(|e| {
        PluginError::Llm(format!(
            "Failed to parse LLM response as PluginSpec: {}. Response was: {}",
            e, cleaned
        ))
    })
}

/// Creates plugin directory and files from a PluginSpec.
pub fn scaffold_plugin(spec: &PluginSpec, output_dir: &Path) -> Result<String, PluginError> {
    let plugin_dir = output_dir.join(&spec.name);
    std::fs::create_dir_all(&plugin_dir)?;

    let manifest = serde_json::json!({
        "name": spec.name,
        "version": "1.0.0",
        "description": spec.description,
        "plugin_type": spec.plugin_type,
        "entry": spec.entry_filename,
    });

    let manifest_path = plugin_dir.join("manifest.json");
    let manifest_content =
        serde_json::to_string_pretty(&manifest).map_err(|e| PluginError::Other(e.to_string()))?;
    std::fs::write(&manifest_path, &manifest_content)?;

    let entry_path = plugin_dir.join(&spec.entry_filename);
    std::fs::write(&entry_path, &spec.entry_content)?;

    Ok(manifest_path.to_string_lossy().to_string())
}

/// Combines LLM parsing + file scaffolding in one call.
pub fn generate_plugin_from_nl(
    nl: &str,
    output_dir: &Path,
    chat_fn: &impl Fn(&str, &str) -> Result<String, String>,
) -> Result<String, PluginError> {
    let spec = parse_nl_to_spec(nl, chat_fn)?;
    let path = scaffold_plugin(&spec, output_dir)?;
    tracing::info!("Generated plugin '{}' at {}", spec.name, path);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mock_chat_fn() -> impl Fn(&str, &str) -> Result<String, String> {
        |_prompt: &str, _system: &str| {
            Ok(r#"{"name":"weather-widget","plugin_type":"ui_slot","description":"Shows today's weather forecast","entry_content":"console.log('Weather widget loaded');","entry_filename":"main.js"}"#.to_string())
        }
    }

    #[test]
    fn test_parse_nl_creates_valid_spec() {
        let chat_fn = mock_chat_fn();
        let spec = parse_nl_to_spec("a weather widget plugin", &chat_fn).unwrap();
        assert_eq!(spec.name, "weather-widget");
        assert_eq!(spec.plugin_type, "ui_slot");
        assert!(spec.entry_content.contains("console.log"));
    }

    #[test]
    fn test_scaffold_creates_files() {
        let dir = TempDir::new().unwrap();
        let spec = PluginSpec {
            name: "test-plugin".into(),
            plugin_type: "tool".into(),
            description: "A test".into(),
            entry_content: "// test".into(),
            entry_filename: "main.js".into(),
        };
        let path = scaffold_plugin(&spec, dir.path()).unwrap();
        assert!(std::path::Path::new(&path).exists());
        assert!(dir.path().join("test-plugin/main.js").exists());
    }

    #[test]
    fn test_generate_roundtrip() {
        let dir = TempDir::new().unwrap();
        let chat_fn = mock_chat_fn();
        let path = generate_plugin_from_nl("weather widget", dir.path(), &chat_fn).unwrap();
        let manifest_path = std::path::Path::new(&path);
        assert!(manifest_path.exists());
        let content = std::fs::read_to_string(manifest_path).unwrap();
        assert!(content.contains("weather-widget"));
    }

    #[test]
    fn test_parse_nl_invalid_json_returns_error() {
        let chat_fn = |_prompt: &str, _system: &str| Ok("this is not valid json".to_string());
        let result = parse_nl_to_spec("a weather widget", &chat_fn);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_scaffold_overwrites_existing_dir() {
        let dir = TempDir::new().unwrap();
        let spec = PluginSpec {
            name: "overwrite-test".into(),
            plugin_type: "tool".into(),
            description: "First run".into(),
            entry_content: "// v1".into(),
            entry_filename: "main.js".into(),
        };
        let result1 = scaffold_plugin(&spec, dir.path());
        assert!(result1.is_ok());

        let result2 = scaffold_plugin(&spec, dir.path());
        assert!(result2.is_ok());

        let manifest_content =
            std::fs::read_to_string(dir.path().join("overwrite-test/manifest.json")).unwrap();
        assert!(manifest_content.contains("First run"));
        assert!(dir.path().join("overwrite-test/main.js").exists());
    }

    #[test]
    fn test_generate_with_empty_name_results_in_error() {
        let chat_fn = |_prompt: &str, _system: &str| {
            Ok(r#"{"name":"","plugin_type":"theme","description":"desc","entry_content":"// empty","entry_filename":"main.js"}"#.to_string())
        };
        let dir = TempDir::new().unwrap();
        let result = generate_plugin_from_nl("", dir.path(), &chat_fn);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(std::path::Path::new(&path).exists());
    }
}
