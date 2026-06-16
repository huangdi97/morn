use crate::core::error::MornError;
use serde::de::DeserializeOwned;

use crate::component::persona::PromptLayers;
use crate::core::registry::Registry;

use super::agent_builder::{
    CapabilitiesStep, DomainStep, KnowledgeStep, PersonaStep, RoleStep, ToolsStep,
};
use crate::core::supervisor::NLAgentDef;

fn clean_json_response(response: &str) -> &str {
    response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
}

pub(super) fn parse_step<T: DeserializeOwned>(step: &str, response: &str) -> Result<T, MornError> {
    let cleaned = clean_json_response(response);
    Ok(serde_json::from_str::<T>(cleaned).map_err(|e| {
        MornError::Internal(format!(
            "Failed to parse {} response as JSON: {}. Raw: {}",
            step, e, cleaned
        ))
    })?)
}

pub(super) fn call_json_step<T: DeserializeOwned>(
    chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
    step: &str,
    prompt: &str,
    system_prompt: &str,
) -> Result<(T, String), MornError> {
    let response = chat_fn(prompt, system_prompt)?;
    let cleaned = clean_json_response(&response).to_string();
    let parsed = parse_step::<T>(step, &response)?;
    Ok((parsed, cleaned))
}

pub(super) fn append_context(context: &mut String, step: &str, response: &str) {
    context.push_str("\n\n");
    context.push_str(step);
    context.push_str(" result:\n");
    context.push_str(response);
}

fn dedup_non_empty(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !deduped.iter().any(|existing| existing == trimmed) {
            deduped.push(trimmed.to_string());
        }
    }
    deduped
}

fn first_non_empty(candidates: &[Option<String>]) -> Option<String> {
    candidates
        .iter()
        .filter_map(|value| value.as_ref())
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn fallback_agent_name(domain: &str, role: &str) -> String {
    let base = if !role.trim().is_empty() {
        role.trim()
    } else if !domain.trim().is_empty() {
        domain.trim()
    } else {
        "Custom"
    };
    format!("{} Agent", base)
}

fn ensure_prompt_layers(mut layers: PromptLayers, role: &str) -> PromptLayers {
    if layers.l1_core_identity.trim().is_empty() {
        let role = if role.trim().is_empty() {
            "helpful AI assistant"
        } else {
            role.trim()
        };
        layers.l1_core_identity = format!("You are a {}.", role);
    }
    layers
}

pub(super) fn list_registry_tools(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut tools: Vec<String> = templates
        .iter()
        .flat_map(|t| t.tools.iter().cloned())
        .collect();
    tools.sort();
    tools.dedup();
    tools
}

pub(super) fn list_registry_personas(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut personas: Vec<String> = templates.iter().map(|t| t.persona.clone()).collect();
    personas.sort();
    personas.dedup();
    personas
}

pub(super) fn list_registry_models(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut models: Vec<String> = templates.iter().map(|t| t.model.clone()).collect();
    models.sort();
    models.dedup();
    models
}

pub(super) fn list_registry_knowledge(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut knowledge: Vec<String> = templates
        .iter()
        .flat_map(|t| t.knowledge.iter().cloned())
        .collect();
    knowledge.sort();
    knowledge.dedup();
    knowledge
}

pub(super) fn list_registry_skills(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut skills: Vec<String> = templates
        .iter()
        .flat_map(|t| t.skills.iter().cloned())
        .collect();
    skills.sort();
    skills.dedup();
    skills
}

pub(super) fn build_default_tool_context() -> String {
    [
        "web_search",
        "read_file",
        "write_file",
        "exec_python",
        "calc",
        "get_time",
        "get_kline",
        "calc_macd",
        "chart",
    ]
    .join(", ")
}

pub(super) fn build_default_persona_context() -> String {
    [
        "assistant",
        "analyst",
        "researcher",
        "writer",
        "coder",
        "translator",
        "reviewer",
    ]
    .join(", ")
}

pub(super) fn build_default_model_context() -> String {
    "deepseek-chat, deepseek-reasoner".to_string()
}

pub(super) fn build_default_knowledge_context() -> String {
    "docs, glossary, data_sources".to_string()
}

pub(super) fn build_default_skill_context() -> String {
    [
        "summarization",
        "translation",
        "code_review",
        "grammar_check",
        "format",
        "style",
        "proofread",
        "report_generation",
        "debug",
        "test",
    ]
    .join(", ")
}

pub(super) fn collect_step_results(
    domain: DomainStep,
    role: RoleStep,
    capabilities: CapabilitiesStep,
    tools: ToolsStep,
    knowledge: KnowledgeStep,
    mut persona: PersonaStep,
    suggestions: Vec<String>,
) -> NLAgentDef {
    let skills = if capabilities.skills.is_empty() {
        capabilities.capabilities
    } else {
        capabilities.skills
    };
    let memory = {
        let selected = dedup_non_empty(knowledge.memory);
        if selected.is_empty() {
            vec!["working_memory".to_string()]
        } else {
            selected
        }
    };

    let name = first_non_empty(&[persona.name.take(), role.name.clone(), domain.name.clone()])
        .unwrap_or_else(|| fallback_agent_name(&domain.domain, &role.role));
    let persona_name = first_non_empty(&[
        persona.persona.take(),
        role.persona.clone(),
        Some(role.role.clone()),
    ])
    .unwrap_or_else(|| "assistant".to_string());
    let model = persona
        .model
        .take()
        .filter(|model| !model.trim().is_empty())
        .unwrap_or_else(|| "deepseek-chat".to_string());
    let communication_style = if persona.communication_style.trim().is_empty() {
        persona.persona_config.parameters.style.clone()
    } else {
        persona.communication_style.trim().to_string()
    };

    persona.persona_config.prompt_layers =
        ensure_prompt_layers(persona.persona_config.prompt_layers, &role.role);

    NLAgentDef {
        name,
        persona: persona_name,
        model,
        tools: dedup_non_empty(tools.tools),
        knowledge: dedup_non_empty(knowledge.knowledge),
        skills: dedup_non_empty(skills),
        memory,
        persona_config: persona.persona_config,
        communication_style,
        suggestions,
    }
}
