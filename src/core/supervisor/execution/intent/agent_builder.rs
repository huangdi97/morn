//! Agent builder — constructs agents from intent using DESIGN.md §3.1 6-step inference chain.
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::component::persona::PromptLayers;
use crate::core::registry::Registry;

use crate::core::supervisor::Supervisor;
use crate::core::supervisor::{NLAgentDef, NLPersonaConfig};

#[derive(Debug, Deserialize)]
struct DomainStep {
    domain: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoleStep {
    role: String,
    #[serde(default)]
    persona: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CapabilitiesStep {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ToolsStep {
    #[serde(default)]
    tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeStep {
    #[serde(default)]
    knowledge: Vec<String>,
    #[serde(default)]
    memory: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PersonaStep {
    #[serde(default)]
    persona_config: NLPersonaConfig,
    #[serde(default)]
    communication_style: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    persona: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

fn clean_json_response(response: &str) -> &str {
    response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
}

fn parse_step<T: DeserializeOwned>(step: &str, response: &str) -> Result<T, String> {
    let cleaned = clean_json_response(response);
    serde_json::from_str::<T>(cleaned).map_err(|e| {
        format!(
            "Failed to parse {} response as JSON: {}. Raw: {}",
            step, e, cleaned
        )
    })
}

fn call_json_step<T: DeserializeOwned>(
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    step: &str,
    prompt: &str,
    system_prompt: &str,
) -> Result<(T, String), String> {
    let response = chat_fn(prompt, system_prompt)?;
    let cleaned = clean_json_response(&response).to_string();
    let parsed = parse_step::<T>(step, &response)?;
    Ok((parsed, cleaned))
}

fn append_context(context: &mut String, step: &str, response: &str) {
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

fn list_registry_tools(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut tools: Vec<String> = templates
        .iter()
        .flat_map(|t| t.tools.iter().cloned())
        .collect();
    tools.sort();
    tools.dedup();
    tools
}

fn list_registry_personas(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut personas: Vec<String> = templates.iter().map(|t| t.persona.clone()).collect();
    personas.sort();
    personas.dedup();
    personas
}

fn list_registry_models(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut models: Vec<String> = templates.iter().map(|t| t.model.clone()).collect();
    models.sort();
    models.dedup();
    models
}

fn list_registry_knowledge(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut knowledge: Vec<String> = templates
        .iter()
        .flat_map(|t| t.knowledge.iter().cloned())
        .collect();
    knowledge.sort();
    knowledge.dedup();
    knowledge
}

fn list_registry_skills(registry: &Registry) -> Vec<String> {
    let templates = registry.list_templates();
    let mut skills: Vec<String> = templates
        .iter()
        .flat_map(|t| t.skills.iter().cloned())
        .collect();
    skills.sort();
    skills.dedup();
    skills
}

fn build_default_tool_context() -> String {
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

fn build_default_persona_context() -> String {
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

fn build_default_model_context() -> String {
    "deepseek-chat, deepseek-reasoner".to_string()
}

fn build_default_knowledge_context() -> String {
    "docs, glossary, data_sources".to_string()
}

fn build_default_skill_context() -> String {
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

fn collect_step_results(
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

impl Supervisor {
    /// Step 1 — 领域识别: extract the professional domain from natural language.
    fn domain_recognition(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
    ) -> Result<(DomainStep, String), String> {
        let prompt = format!(
            r#"{}

Step 1 — Domain Recognition: extract the professional domain from the user's description.
Return exactly:
{{
  "domain": "short domain label",
  "name": "short agent name, 2-5 words"
}}"#,
            context
        );
        call_json_step::<DomainStep>(chat_fn, "Step 1 — Domain Recognition", &prompt, system_prompt)
    }

    /// Step 2 — 角色推断: infer the role type needed.
    fn role_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
    ) -> Result<(RoleStep, String), String> {
        let prompt = format!(
            r#"{}

Step 2 — Role Inference: determine what role this agent should play.
Use the previous domain result as context. Select persona from available personas when possible.
Return exactly:
{{
  "role": "role this agent should play",
  "persona": "persona id or name from the available personas",
  "name": "optional refined short agent name"
}}"#,
            context
        );
        call_json_step::<RoleStep>(chat_fn, "Step 2 — Role Inference", &prompt, system_prompt)
    }

    /// Step 3 — 能力推断: infer the required capability set.
    fn capability_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
    ) -> Result<(CapabilitiesStep, String), String> {
        let prompt = format!(
            r#"{}

Step 3 — Capability Inference: infer capabilities needed.
Use the previous domain and role results as context. Map capabilities to available skills when possible.
Return exactly:
{{
  "capabilities": ["capability names needed by the agent"],
  "skills": ["skill names selected from available skills"]
}}"#,
            context
        );
        call_json_step::<CapabilitiesStep>(chat_fn, "Step 3 — Capability Inference", &prompt, system_prompt)
    }

    /// Step 4 — 工具推断: infer required tools, query Registry for recommendations.
    fn tool_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(ToolsStep, String), String> {
        let tool_list = if let Some(reg) = registry {
            list_registry_tools(reg).join(", ")
        } else {
            build_default_tool_context()
        };

        let prompt = format!(
            r#"{}

Step 4 — Tool Inference: infer tools needed.
Use all previous results as context. Select tool names from available tools when possible.
Available tools: {}

Return exactly:
{{
  "tools": ["tool names"]
}}"#,
            context, tool_list
        );
        call_json_step::<ToolsStep>(chat_fn, "Step 4 — Tool Inference", &prompt, system_prompt)
    }

    /// Step 5 — 知识推断: infer required knowledge bases and data sources.
    fn knowledge_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(KnowledgeStep, String), String> {
        let knowledge_list = if let Some(reg) = registry {
            list_registry_knowledge(reg).join(", ")
        } else {
            build_default_knowledge_context()
        };

        let prompt = format!(
            r#"{}

Step 5 — Knowledge Inference: infer knowledge needed.
Use all previous results as context. Select knowledge sources from available knowledge when possible, and choose memory layers needed by this agent.
Available knowledge: {}
Available memory: working_memory, episodic_memory, semantic_memory, graph_memory, flash_memory

Return exactly:
{{
  "knowledge": ["knowledge source names"],
  "memory": ["memory layer names"]
}}"#,
            context, knowledge_list
        );
        call_json_step::<KnowledgeStep>(chat_fn, "Step 5 — Knowledge Inference", &prompt, system_prompt)
    }

    /// Step 6 — 人格推断: infer suitable persona, match against available personas.
    fn persona_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(PersonaStep, String), String> {
        let persona_list = if let Some(reg) = registry {
            list_registry_personas(reg).join(", ")
        } else {
            build_default_persona_context()
        };
        let model_list = if let Some(reg) = registry {
            list_registry_models(reg).join(", ")
        } else {
            build_default_model_context()
        };

        let prompt = format!(
            r#"{}

Step 6 — Persona Inference: infer persona configuration.
Use all previous results as context. Generate persona parameters and all 5 prompt layers for the final agent definition.
Match against available personas when possible.
Available personas: {}
Available models: {}

Return exactly:
{{
  "model": "model name from available models",
  "persona": "optional refined persona",
  "name": "optional refined short agent name",
  "communication_style": "professional|friendly|detailed|creative|technical or similar",
  "persona_config": {{
    "parameters": {{
      "temperature": 0.6,
      "style": "professional",
      "verbosity": 0.5,
      "proactiveness": 0.3
    }},
    "prompt_layers": {{
      "l1_core_identity": "core identity prompt",
      "l2_skill_instructions": "skill instruction prompt or null",
      "l3_format_template": "format template prompt or null",
      "l4_constraints": "constraints prompt or null",
      "l5_conversation_style": "conversation style prompt or null"
    }}
  }}
}}"#,
            context, persona_list, model_list
        );
        call_json_step::<PersonaStep>(chat_fn, "Step 6 — Persona Inference", &prompt, system_prompt)
    }

    pub fn create_agent_from_nl(
        &self,
        nl: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
        registry: Option<&Registry>,
    ) -> Result<NLAgentDef, String> {
        let system_prompt = "You are a COO agent configuration planner. Complete exactly the requested step. Only return valid JSON, no markdown, no explanation.";

        let mut suggestions: Vec<String> = Vec::new();
        suggestions.extend(self.infer_from_registry(nl));
        suggestions.extend(self.suggest_from_market(nl));
        suggestions.sort();
        suggestions.dedup();

        let persona_list = if let Some(reg) = registry {
            list_registry_personas(reg).join(", ")
        } else {
            build_default_persona_context()
        };
        let model_list = if let Some(reg) = registry {
            list_registry_models(reg).join(", ")
        } else {
            build_default_model_context()
        };
        let skill_list = if let Some(reg) = registry {
            list_registry_skills(reg).join(", ")
        } else {
            build_default_skill_context()
        };

        let mut context = format!(
            r#"User wants to create an agent.
Description:
{}

Available personas: {}
Available models: {}
Available skills: {}"#,
            nl, persona_list, model_list, skill_list
        );

        let (domain, domain_json) = self.domain_recognition(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 1 — Domain Recognition", &domain_json);

        let (role, role_json) = self.role_inference(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 2 — Role Inference", &role_json);

        let (capabilities, capabilities_json) =
            self.capability_inference(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 3 — Capability Inference", &capabilities_json);

        let (tools, tools_json) = self.tool_inference(&context, chat_fn, system_prompt, registry)?;
        append_context(&mut context, "Step 4 — Tool Inference", &tools_json);

        let (knowledge, knowledge_json) =
            self.knowledge_inference(&context, chat_fn, system_prompt, registry)?;
        append_context(&mut context, "Step 5 — Knowledge Inference", &knowledge_json);

        let (persona, _persona_json) =
            self.persona_inference(&context, chat_fn, system_prompt, registry)?;

        Ok(collect_step_results(
            domain, role, capabilities, tools, knowledge, persona, suggestions,
        ))
    }
}