//! Component assembly module — types for building, validating, and wiring agent components.

mod builder;
mod graph;
pub mod rules;
mod validator;

pub use builder::{AssemblyBuilder, ComponentSelector, DefaultCompleter, GuidedBuildSteps};
pub use graph::{
    AtomicComponentDef, AtomicComponentType, ComponentConnection, ComponentGraph,
    ComponentRegistry, ConnectionValidator,
};
pub use validator::AssemblyValidator;

pub use crate::core::component_type::registry::TypeRegistry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_accepts_valid_selector() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        assert!(AssemblyValidator::validate(&selector).is_ok());
    }

    #[test]
    fn test_validator_rejects_no_memory() {
        let selector = ComponentSelector::new()
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        assert!(AssemblyValidator::validate(&selector).is_err());
    }

    #[test]
    fn test_validator_rejects_no_tools() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        assert!(AssemblyValidator::validate(&selector).is_err());
    }

    #[test]
    fn test_validator_rejects_no_llm() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_tools(vec!["web_search".to_string()]);
        assert!(AssemblyValidator::validate(&selector).is_err());
    }

    #[test]
    fn test_validator_rejects_too_many_memory() {
        let selector = ComponentSelector::new()
            .with_memory(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ])
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        assert!(AssemblyValidator::validate(&selector).is_err());
    }

    #[test]
    fn test_default_completer_fills_missing() {
        let mut selector = ComponentSelector::new();
        DefaultCompleter::complete(&mut selector);
        assert!(!selector.memory_ids.is_empty());
        assert!(!selector.tool_ids.is_empty());
        assert!(!selector.llm_ids.is_empty());
    }

    #[test]
    fn test_build_from_valid_selector() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        let agent = AssemblyBuilder::build(&selector);
        assert!(agent.is_ok());
        assert!(agent.unwrap().id.starts_with("agent-"));
    }

    #[test]
    fn test_build_from_description() {
        let agent = AssemblyBuilder::from_description("build a research agent for biology");
        assert!(agent.is_ok());
        let def = agent.unwrap();
        assert!(def.id.starts_with("agent-"));
    }

    #[test]
    fn test_guided_build() {
        let steps = GuidedBuildSteps {
            memory_ids: vec!["working".to_string()],
            tool_ids: vec!["web_search".to_string(), "file_read".to_string()],
            llm_ids: vec!["gpt-4".to_string()],
            ..GuidedBuildSteps::new()
        };
        let agent = AssemblyBuilder::guided_build(steps);
        assert!(agent.is_ok());
    }

    #[test]
    fn test_canvas_build_returns_ok() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        let result = AssemblyBuilder::canvas_build(selector);
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_registry_has_components() {
        let components = ComponentRegistry::available_components();
        assert!(components.len() >= 12);
        assert!(components
            .iter()
            .any(|c| c.component_type == AtomicComponentType::Memory));
        assert!(components
            .iter()
            .any(|c| c.component_type == AtomicComponentType::Tool));
        assert!(components
            .iter()
            .any(|c| c.component_type == AtomicComponentType::LLM));
    }

    #[test]
    fn test_connection_validator_accepts_valid() {
        let graph = ComponentGraph {
            components: vec![
                ComponentRegistry::get_component("tool_web_search").unwrap(),
                ComponentRegistry::get_component("llm_deepseek").unwrap(),
            ],
            connections: vec![ComponentConnection {
                source_id: "tool_web_search".into(),
                source_output: "results".into(),
                target_id: "llm_deepseek".into(),
                target_input: "prompt".into(),
            }],
        };
        assert!(ConnectionValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_connection_validator_rejects_bad_source() {
        let graph = ComponentGraph {
            components: vec![ComponentRegistry::get_component("tool_web_search").unwrap()],
            connections: vec![ComponentConnection {
                source_id: "nonexistent".into(),
                source_output: "out".into(),
                target_id: "tool_web_search".into(),
                target_input: "query".into(),
            }],
        };
        assert!(ConnectionValidator::validate(&graph).is_err());
    }

    #[test]
    fn test_component_graph_export_import() {
        let graph = ComponentGraph {
            components: vec![
                ComponentRegistry::get_component("tool_web_search").unwrap(),
                ComponentRegistry::get_component("llm_deepseek").unwrap(),
            ],
            connections: vec![ComponentConnection {
                source_id: "tool_web_search".into(),
                source_output: "results".into(),
                target_id: "llm_deepseek".into(),
                target_input: "prompt".into(),
            }],
        };
        let json = ComponentGraph::to_json(&graph).unwrap();
        let parsed = ComponentGraph::from_json(&json).unwrap();
        assert_eq!(parsed.components.len(), 2);
        assert_eq!(parsed.connections.len(), 1);
    }

    #[test]
    fn test_constraint_check() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["working".to_string()])
            .with_tools(vec!["web_search".to_string()])
            .with_llm(vec!["deepseek-chat".to_string()]);
        assert!(AssemblyValidator::check_constraints(&selector, 5).is_ok());
        assert!(AssemblyValidator::check_constraints(&selector, 6).is_err());
    }

    #[test]
    fn test_selector_builder_methods() {
        let selector = ComponentSelector::new()
            .with_memory(vec!["flash".to_string()])
            .with_tools(vec!["search".to_string(), "calc".to_string()])
            .with_llm(vec!["local-gguf".to_string()])
            .with_channels(vec!["desktop".to_string()])
            .with_personas(vec!["analyst".to_string()])
            .with_skills(vec!["analysis".to_string()]);
        assert_eq!(selector.memory_ids.len(), 1);
        assert_eq!(selector.tool_ids.len(), 2);
        assert_eq!(selector.llm_ids.len(), 1);
    }
}
