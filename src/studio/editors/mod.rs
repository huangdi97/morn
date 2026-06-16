//! editors — Studio editor types for agent, knowledge, memory, model, persona, pipeline, skill, and tool editing.

use crate::core::error::MornError;
pub trait Editor {
    fn name(&self) -> &str;
}

pub mod editor_base;
pub mod knowledge;
pub mod make_editors;
pub mod memory;
pub mod model;
pub mod pipeline;
pub mod tool;

pub use editor_base::{EditorPosition, NodeEditorFields};
pub use knowledge::{KnowledgeEditor, KnowledgeSource};
pub use make_editors::AgentEditor;
pub use make_editors::PersonaEditor;
pub use make_editors::SkillEditor;
pub use memory::MemoryEditor;
pub use model::{CostTier, ModelEditor, ModelParameters};
pub use pipeline::{PipelineEditor, PipelineStage};
pub use tool::{PortDef, ToolEditor};

impl Editor for ToolEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for KnowledgeEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for MemoryEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for ModelEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for PersonaEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for SkillEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for AgentEditor {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Editor for PipelineEditor {
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_editor_default() {
        let editor = ToolEditor::new("test-tool");
        assert_eq!(editor.name, "test-tool");
        assert_eq!(editor.ports.len(), 2);
    }

    #[test]
    fn tool_editor_to_config() {
        let editor = ToolEditor::new("calc");
        let config = editor.to_config();
        assert_eq!(config["type"], "tool");
        assert_eq!(config["name"], "calc");
    }

    #[test]
    fn knowledge_editor_default() {
        let editor = KnowledgeEditor::new("docs");
        assert_eq!(editor.name, "docs");
        assert_eq!(editor.update_strategy, "manual");
    }

    #[test]
    fn knowledge_editor_to_config() {
        let mut editor = KnowledgeEditor::new("knowledge-base");
        editor.data_sources.push(KnowledgeSource {
            name: "wiki".into(),
            source_type: "web".into(),
        });
        let config = editor.to_config();
        assert_eq!(config["data_sources"][0]["name"], "wiki");
    }

    #[test]
    fn memory_editor_default() {
        let editor = MemoryEditor::new("short-term");
        assert_eq!(editor.capacity, 1000);
        assert_eq!(editor.retrieval_method, "semantic");
    }

    #[test]
    fn memory_editor_to_config() {
        let editor = MemoryEditor::new("cache");
        let config = editor.to_config();
        assert_eq!(config["name"], "cache");
        assert_eq!(config["capacity"], 1000);
    }

    #[test]
    fn model_editor_default() {
        let editor = ModelEditor::new("main-model");
        assert_eq!(editor.provider, "deepseek");
        assert_eq!(editor.model_name, "deepseek-chat");
    }

    #[test]
    fn model_editor_to_config() {
        let editor = ModelEditor::new("gpt-model");
        let config = editor.to_config();
        assert_eq!(config["name"], "gpt-model");
        assert_eq!(config["cost_tier"], "low");
    }

    #[test]
    fn model_editor_with_fallback() {
        let mut editor = ModelEditor::new("primary");
        editor.fallback = Some("backup-model".to_string());
        let config = editor.to_config();
        assert_eq!(config["fallback"], "backup-model");
    }

    #[test]
    fn tool_editor_port_editing() {
        let mut editor = ToolEditor::new("custom-tool");
        editor.ports.push(PortDef {
            name: "debug".into(),
            direction: "out".into(),
            data_type: "string".into(),
        });
        assert_eq!(editor.ports.len(), 3);
        assert_eq!(editor.ports[2].name, "debug");
    }

    #[test]
    fn tool_editor_permissions() {
        let mut editor = ToolEditor::new("admin-tool");
        editor.permissions.push("write".to_string());
        let config = editor.to_config();
        assert!(config["permissions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("write")));
    }

    #[test]
    fn knowledge_editor_data_source_config() {
        let mut editor = KnowledgeEditor::new("wiki-knowledge");
        editor.data_sources.push(KnowledgeSource {
            name: "wikipedia".into(),
            source_type: "web".into(),
        });
        editor.data_sources.push(KnowledgeSource {
            name: "internal-docs".into(),
            source_type: "file".into(),
        });
        assert_eq!(editor.data_sources.len(), 2);
        let config = editor.to_config();
        assert_eq!(config["data_sources"][1]["name"], "internal-docs");
    }

    #[test]
    fn knowledge_editor_process_method() {
        let mut editor = KnowledgeEditor::new("hybrid-knowledge");
        editor.process_method = "hybrid".to_string();
        editor.update_strategy = "scheduled".to_string();
        let config = editor.to_config();
        assert_eq!(config["process_method"], "hybrid");
        assert_eq!(config["update_strategy"], "scheduled");
    }

    #[test]
    fn memory_editor_capacity_config() {
        let mut editor = MemoryEditor::new("large-memory");
        editor.capacity = 5000;
        editor.ttl_secs = Some(7200);
        assert_eq!(editor.capacity, 5000);
        assert_eq!(editor.ttl_secs, Some(7200));
    }

    #[test]
    fn memory_editor_no_ttl() {
        let mut editor = MemoryEditor::new("persistent-memory");
        editor.ttl_secs = None;
        let config = editor.to_config();
        assert_eq!(config["ttl_secs"], serde_json::Value::Null);
    }

    #[test]
    fn model_editor_parameter_config() {
        let mut editor = ModelEditor::new("custom-model");
        editor.parameters =
            serde_json::json!({"temperature": 0.3, "max_tokens": 4096, "top_p": 0.9});
        editor.cost_tier = CostTier("high".to_string());
        let config = editor.to_config();
        assert_eq!(config["parameters"]["temperature"], 0.3);
        assert_eq!(config["cost_tier"], "high");
    }

    #[test]
    fn persona_editor_new() {
        let editor = PersonaEditor::new("persona-1", "Analyst");
        assert_eq!(editor.id, "persona-1");
        assert_eq!(editor.name, "Analyst");
    }

    #[test]
    fn persona_editor_load() {
        let editor = PersonaEditor::load();
        assert_eq!(editor.id, "default");
        assert_eq!(editor.name, "Default Persona");
    }

    #[test]
    fn persona_editor_save() {
        let editor = PersonaEditor::new("persona-1", "Analyst");
        assert!(editor.save().is_ok());
    }

    #[test]
    fn skill_editor_new() {
        let editor = SkillEditor::new("skill-1", "Data Analysis");
        assert_eq!(editor.id, "skill-1");
        assert_eq!(editor.name, "Data Analysis");
    }

    #[test]
    fn skill_editor_load() {
        let editor = SkillEditor::load();
        assert_eq!(editor.id, "default");
    }

    #[test]
    fn skill_editor_save() {
        let editor = SkillEditor::new("skill-1", "Skill");
        assert!(editor.save().is_ok());
    }

    #[test]
    fn agent_editor_new() {
        let editor = AgentEditor::new("agent-1", "Research Agent");
        assert_eq!(editor.id, "agent-1");
        assert_eq!(editor.name, "Research Agent");
    }

    #[test]
    fn agent_editor_load() {
        let editor = AgentEditor::load();
        assert_eq!(editor.id, "default");
    }

    #[test]
    fn agent_editor_save() {
        let editor = AgentEditor::new("agent-1", "Agent");
        assert!(editor.save().is_ok());
    }

    #[test]
    fn pipeline_editor_new() {
        let editor = PipelineEditor::new("pipe-1", "Data Pipeline");
        assert_eq!(editor.id, "pipe-1");
        assert_eq!(editor.name, "Data Pipeline");
    }

    #[test]
    fn pipeline_editor_load() {
        let editor = PipelineEditor::load();
        assert_eq!(editor.id, "default");
    }

    #[test]
    fn pipeline_editor_save() {
        let editor = PipelineEditor::new("pipe-1", "Pipeline");
        assert!(editor.save().is_ok());
    }
}
