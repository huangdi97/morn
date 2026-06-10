//! Validation logic for component assembly.

pub struct AssemblyValidator;

impl AssemblyValidator {
    pub fn validate(selector: &crate::core::assembly::builder::ComponentSelector) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if selector.memory_ids.is_empty() {
            errors.push("at least 1 memory component required".to_string());
        }
        if selector.memory_ids.len() > 3 {
            errors.push("at most 3 memory components allowed".to_string());
        }

        if selector.tool_ids.is_empty() {
            errors.push("at least 1 tool component required".to_string());
        }
        if selector.tool_ids.len() > 15 {
            errors.push("at most 15 tool components allowed per session".to_string());
        }

        if selector.llm_ids.is_empty() {
            errors.push("at least 1 LLM component required".to_string());
        }
        if selector.llm_ids.len() > 3 {
            errors.push("at most 3 LLM components allowed".to_string());
        }

        if selector.persona_ids.len() > 1 {
            errors.push("at most 1 persona allowed (use CompositePersona for multiple)".to_string());
        }

        let has_local_llm = selector
            .llm_ids
            .iter()
            .any(|id| id.contains("local") || id.contains("gguf"));
        let has_cloud_tool = selector
            .tool_ids
            .iter()
            .any(|id| id.contains("web") || id.contains("api") || id.contains("search"));
        if has_local_llm && has_cloud_tool {
            errors.push(
                "incompatible: local LLM with cloud-dependent tools requires network".to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn check_constraints(selector: &crate::core::assembly::builder::ComponentSelector, active_agents: usize) -> Result<(), String> {
        let memory_layers = selector.memory_ids.len();
        if memory_layers * active_agents > 5 {
            return Err(format!(
                "constraint violation: memory layers ({}) x active agents ({}) exceeds 5",
                memory_layers, active_agents
            ));
        }
        if selector.tool_ids.len() > 15 {
            return Err(format!(
                "constraint violation: tool count ({}) exceeds 15 per session",
                selector.tool_ids.len()
            ));
        }
        Ok(())
    }
}