//! custom — Loads and saves custom workflow template catalog entries.
use std::path::Path;

use super::{WorkflowTemplateEntry, WorkflowTemplateStore};

impl WorkflowTemplateStore {
    pub fn install(&mut self, template: WorkflowTemplateEntry) -> Result<(), String> {
        if self.templates.contains_key(&template.workflow_id) {
            return Err(format!(
                "Template '{}' already registered",
                template.workflow_id
            ));
        }
        self.register(template);
        Ok(())
    }

    pub fn from_json_file(path: &Path) -> Result<WorkflowTemplateEntry, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))
    }

    pub fn from_yaml_file(path: &Path) -> Result<WorkflowTemplateEntry, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&content).map_err(|e| format!("YAML parse error: {}", e))
    }

    pub fn from_json(path: &str) -> Result<WorkflowTemplateEntry, String> {
        Self::from_json_file(Path::new(path))
    }

    pub fn from_yaml(path: &str) -> Result<WorkflowTemplateEntry, String> {
        Self::from_yaml_file(Path::new(path))
    }

    pub fn load_json_to_store(&mut self, path: &Path) -> Result<(), String> {
        let template = Self::from_json_file(path)?;
        self.install(template)
    }

    pub fn load_yaml_to_store(&mut self, path: &Path) -> Result<(), String> {
        let template = Self::from_yaml_file(path)?;
        self.install(template)
    }

    pub fn unregister(&mut self, workflow_id: &str) -> Option<WorkflowTemplateEntry> {
        self.templates.remove(workflow_id)
    }
}
