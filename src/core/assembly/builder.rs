//! Builder/construction logic for component assembly.

use crate::core::error::MornError;
use crate::core::assembler::AgentDef;
use crate::core::assembly::graph::{AtomicComponentType, ComponentGraph, ConnectionValidator};
use crate::core::assembly::validator::AssemblyValidator;
use crate::core::component_type::registry::TypeRegistry;

#[derive(Debug, Clone)]
pub struct ComponentSelector {
    pub memory_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub llm_ids: Vec<String>,
    pub channel_ids: Vec<String>,
    pub persona_ids: Vec<String>,
    pub skill_ids: Vec<String>,
}

impl ComponentSelector {
    pub fn new() -> Self {
        ComponentSelector {
            memory_ids: Vec::new(),
            tool_ids: Vec::new(),
            llm_ids: Vec::new(),
            channel_ids: Vec::new(),
            persona_ids: Vec::new(),
            skill_ids: Vec::new(),
        }
    }

    pub fn with_memory(mut self, ids: Vec<String>) -> Self {
        self.memory_ids = ids;
        self
    }

    pub fn with_tools(mut self, ids: Vec<String>) -> Self {
        self.tool_ids = ids;
        self
    }

    pub fn with_llm(mut self, ids: Vec<String>) -> Self {
        self.llm_ids = ids;
        self
    }

    pub fn with_channels(mut self, ids: Vec<String>) -> Self {
        self.channel_ids = ids;
        self
    }

    pub fn with_personas(mut self, ids: Vec<String>) -> Self {
        self.persona_ids = ids;
        self
    }

    pub fn with_skills(mut self, ids: Vec<String>) -> Self {
        self.skill_ids = ids;
        self
    }
}

impl Default for ComponentSelector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DefaultCompleter;

impl DefaultCompleter {
    pub fn complete(selector: &mut ComponentSelector) {
        if selector.memory_ids.is_empty() {
            selector.memory_ids = vec!["working_memory".to_string()];
        }
        if selector.tool_ids.is_empty() {
            selector.tool_ids = vec!["web_search".to_string(), "read_file".to_string()];
        }
        if selector.llm_ids.is_empty() {
            selector.llm_ids = vec!["deepseek-chat".to_string()];
        }
    }
}

pub struct AssemblyBuilder {
    pub registry: TypeRegistry,
}

impl AssemblyBuilder {
    pub fn new() -> Self {
        AssemblyBuilder {
            registry: TypeRegistry::new(),
        }
    }
    pub fn with_registry(registry: TypeRegistry) -> Self {
        AssemblyBuilder { registry }
    }
    pub fn build(selector: &ComponentSelector) -> Result<AgentDef, MornError> {
        AssemblyValidator::validate(selector).map_err(|errs| errs.join("; "))?;

        let persona_id = selector
            .persona_ids
            .first()
            .map(|s| s.as_str())
            .unwrap_or("assistant");
        let persona = match persona_id {
            "researcher" => crate::component::persona::create_researcher_persona(),
            "writer" => crate::component::persona::create_writer_persona(),
            "coder" => crate::component::persona::create_coder_persona(),
            "analyst" => crate::component::persona::create_analyst_persona(),
            _ => crate::component::persona::create_assistant_persona(),
        };

        let llm_id = selector.llm_ids.first().cloned().unwrap_or_default();
        let model = crate::component::model::ModelConfig {
            id: format!("model-{}", llm_id),
            provider: if llm_id.contains("local") {
                "local".to_string()
            } else {
                "deepseek".to_string()
            },
            model_name: llm_id,
            base_url: "https://api.deepseek.com".to_string(),
            api_key: String::new(),
            parameters: crate::component::model::ModelParameters::default(),
            fallback: None,
            cost_tier: crate::component::model::CostTier::Low,
        };

        Ok(AgentDef {
            id: format!("agent-{}", uuid::Uuid::new_v4()),
            name: format!(
                "agent-{}",
                selector.tool_ids.first().cloned().unwrap_or_default()
            ),
            persona,
            model,
            tools: selector.tool_ids.clone(),
            knowledge: vec![],
            skills: selector.skill_ids.clone(),
            memory: selector.memory_ids.first().cloned(),
        })
    }

    pub fn from_description(description: &str) -> Result<AgentDef, MornError> {
        match crate::core::assembler::AgentAssembler::natural_language_build(description)? {
            crate::core::assembler::AfterBuildAction::Save(def)
            | crate::core::assembler::AfterBuildAction::Preview(def) => Ok(def),
            crate::core::assembler::AfterBuildAction::Modify(_, _) => {
                Err(MornError::Internal("Natural language build returned a modification request".to_string()))
            }
        }
    }

    pub fn guided_build(steps: GuidedBuildSteps) -> Result<AgentDef, MornError> {
        let mut selector = ComponentSelector::new();

        selector.memory_ids = steps.memory_ids;
        selector.tool_ids = steps.tool_ids;
        selector.llm_ids = steps.llm_ids;
        selector.channel_ids = steps.channel_ids;
        selector.persona_ids = steps.persona_ids;
        selector.skill_ids = steps.skill_ids;

        DefaultCompleter::complete(&mut selector);
        AssemblyBuilder::build(&selector)
    }

    pub fn canvas_build(components: ComponentSelector) -> Result<AgentDef, MornError> {
        let mut selector = components;
        DefaultCompleter::complete(&mut selector);
        AssemblyBuilder::build(&selector)
    }

    pub fn build_from_graph(&self, graph: &ComponentGraph) -> Result<AgentDef, MornError> {
        ConnectionValidator::validate(graph).map_err(|errs| errs.join("; "))?;

        let mut selector = ComponentSelector::new();
        for comp in &graph.components {
            let type_name = if comp.type_name.is_empty() {
                comp.component_type.type_name()
            } else {
                &comp.type_name
            };
            if let Some(def) = self.registry.get(type_name) {
                if def.interfaces.contains(&"store".to_string())
                    || def.interfaces.contains(&"recall".to_string())
                {
                    selector.memory_ids.push(comp.id.clone());
                } else if def.interfaces.contains(&"execute".to_string()) {
                    selector.tool_ids.push(comp.id.clone());
                } else if def.interfaces.contains(&"predict".to_string())
                    || def.interfaces.contains(&"embed".to_string())
                {
                    selector.llm_ids.push(comp.id.clone());
                } else if def.interfaces.contains(&"generate".to_string()) {
                    selector.persona_ids.push(comp.id.clone());
                } else {
                    selector.skill_ids.push(comp.id.clone());
                }
            } else {
                match comp.component_type {
                    AtomicComponentType::Memory => selector.memory_ids.push(comp.id.clone()),
                    AtomicComponentType::Tool => selector.tool_ids.push(comp.id.clone()),
                    AtomicComponentType::LLM => selector.llm_ids.push(comp.id.clone()),
                    AtomicComponentType::Channel => selector.channel_ids.push(comp.id.clone()),
                    AtomicComponentType::Persona => selector.persona_ids.push(comp.id.clone()),
                    AtomicComponentType::Skill => selector.skill_ids.push(comp.id.clone()),
                    AtomicComponentType::Knowledge | AtomicComponentType::SecurityPolicy => {}
                }
            }
        }

        DefaultCompleter::complete(&mut selector);
        AssemblyBuilder::build(&selector)
    }

    pub fn export_graph(graph: &ComponentGraph) -> Result<String, MornError> {
        graph.to_json()
    }

    pub fn import_graph(json: &str) -> Result<ComponentGraph, MornError> {
        let graph = ComponentGraph::from_json(json)?;
        ConnectionValidator::validate(&graph).map_err(|errs| errs.join("; "))?;
        Ok(graph)
    }
}

impl Default for AssemblyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GuidedBuildSteps {
    pub memory_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub llm_ids: Vec<String>,
    pub channel_ids: Vec<String>,
    pub persona_ids: Vec<String>,
    pub skill_ids: Vec<String>,
}

impl GuidedBuildSteps {
    pub fn new() -> Self {
        GuidedBuildSteps {
            memory_ids: Vec::new(),
            tool_ids: Vec::new(),
            llm_ids: Vec::new(),
            channel_ids: Vec::new(),
            persona_ids: Vec::new(),
            skill_ids: Vec::new(),
        }
    }
}

impl Default for GuidedBuildSteps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_selector_new_empty() {
        let sel = ComponentSelector::new();
        assert!(sel.memory_ids.is_empty());
        assert!(sel.tool_ids.is_empty());
        assert!(sel.llm_ids.is_empty());
        assert!(sel.channel_ids.is_empty());
        assert!(sel.persona_ids.is_empty());
        assert!(sel.skill_ids.is_empty());
    }

    #[test]
    fn test_component_selector_with_memory() {
        let sel = ComponentSelector::new().with_memory(vec!["mem1".into()]);
        assert_eq!(sel.memory_ids, vec!["mem1"]);
    }

    #[test]
    fn test_component_selector_with_tools() {
        let sel = ComponentSelector::new().with_tools(vec!["tool1".into(), "tool2".into()]);
        assert_eq!(sel.tool_ids.len(), 2);
    }

    #[test]
    fn test_component_selector_with_llm() {
        let sel = ComponentSelector::new().with_llm(vec!["gpt-4".into()]);
        assert_eq!(sel.llm_ids, vec!["gpt-4"]);
    }

    #[test]
    fn test_component_selector_with_channels() {
        let sel = ComponentSelector::new().with_channels(vec!["ch1".into()]);
        assert_eq!(sel.channel_ids, vec!["ch1"]);
    }

    #[test]
    fn test_component_selector_with_personas() {
        let sel = ComponentSelector::new().with_personas(vec!["researcher".into()]);
        assert_eq!(sel.persona_ids, vec!["researcher"]);
    }

    #[test]
    fn test_component_selector_with_skills() {
        let sel = ComponentSelector::new().with_skills(vec!["skill1".into()]);
        assert_eq!(sel.skill_ids, vec!["skill1"]);
    }

    #[test]
    fn test_component_selector_chaining() {
        let sel = ComponentSelector::new()
            .with_memory(vec!["m1".into()])
            .with_tools(vec!["t1".into()])
            .with_llm(vec!["l1".into()]);
        assert_eq!(sel.memory_ids, vec!["m1"]);
        assert_eq!(sel.tool_ids, vec!["t1"]);
        assert_eq!(sel.llm_ids, vec!["l1"]);
    }

    #[test]
    fn test_default_completer_fills_empty() {
        let mut sel = ComponentSelector::new();
        DefaultCompleter::complete(&mut sel);
        assert_eq!(sel.memory_ids, vec!["working_memory"]);
        assert_eq!(sel.tool_ids, vec!["web_search", "read_file"]);
        assert_eq!(sel.llm_ids, vec!["deepseek-chat"]);
    }

    #[test]
    fn test_default_completer_does_not_override() {
        let mut sel = ComponentSelector::new()
            .with_memory(vec!["custom_mem".into()])
            .with_tools(vec!["custom_tool".into()])
            .with_llm(vec!["custom_llm".into()]);
        DefaultCompleter::complete(&mut sel);
        assert_eq!(sel.memory_ids, vec!["custom_mem"]);
    }

    #[test]
    fn test_assembly_builder_build_default() {
        let mut sel = ComponentSelector::new();
        DefaultCompleter::complete(&mut sel);
        let result = AssemblyBuilder::build(&sel);
        assert!(result.is_ok());
        let def = result.unwrap();
        assert!(def.id.starts_with("agent-"));
    }

    #[test]
    fn test_guided_build_steps_new() {
        let steps = GuidedBuildSteps::new();
        assert!(steps.memory_ids.is_empty());
        assert!(steps.tool_ids.is_empty());
    }

    #[test]
    fn test_assembly_builder_export_import_graph() {
        let graph = ComponentGraph {
            components: vec![],
            connections: vec![],
        };
        let json = AssemblyBuilder::export_graph(&graph).unwrap();
        assert!(json.contains("\"components\""));
        let imported = AssemblyBuilder::import_graph(&json).unwrap();
        assert!(imported.components.is_empty());
    }

    #[test]
    fn test_assembly_builder_new() {
        let _builder = AssemblyBuilder::new();
        // just verify it constructs
    }
}
