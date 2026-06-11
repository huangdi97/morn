//! Agent builder — constructs agents from intent.
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

fn step1_analyze_domain(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(DomainStep, String), String> {
    let prompt = format!(
        r#"{}

Step 1: Analyze the domain: what field is this agent for?
Return exactly:
{{
  "domain": "short domain label",
  "name": "short agent name, 2-5 words"
}}"#,
        context
    );
    call_json_step::<DomainStep>(chat_fn, "Step 1", &prompt, system_prompt)
}

fn step2_determine_role(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(RoleStep, String), String> {
    let prompt = format!(
        r#"{}

Step 2: Determine the role: what role should this agent play?
Use the previous domain result as context. Select persona from available personas when possible.
Return exactly:
{{
  "role": "role this agent should play",
  "persona": "persona id or name from the available personas",
  "name": "optional refined short agent name"
}}"#,
        context
    );
    call_json_step::<RoleStep>(chat_fn, "Step 2", &prompt, system_prompt)
}

fn step3_infer_capabilities(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(CapabilitiesStep, String), String> {
    let prompt = format!(
        r#"{}

Step 3: Infer capabilities needed.
Use the previous domain and role results as context. Map capabilities to available skills when possible.
Return exactly:
{{
  "capabilities": ["capability names needed by the agent"],
  "skills": ["skill names selected from available skills"]
}}"#,
        context
    );
    call_json_step::<CapabilitiesStep>(chat_fn, "Step 3", &prompt, system_prompt)
}

fn step4_infer_tools(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(ToolsStep, String), String> {
    let prompt = format!(
        r#"{}

Step 4: Infer tools needed.
Use all previous results as context. Select tool names from available tools when possible.
Return exactly:
{{
  "tools": ["tool names"]
}}"#,
        context
    );
    call_json_step::<ToolsStep>(chat_fn, "Step 4", &prompt, system_prompt)
}

fn step5_infer_knowledge(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(KnowledgeStep, String), String> {
    let prompt = format!(
        r#"{}

Step 5: Infer knowledge needed.
Use all previous results as context. Select knowledge sources from available knowledge when possible, and choose memory layers needed by this agent.
Return exactly:
{{
  "knowledge": ["knowledge source names"],
  "memory": ["memory layer names"]
}}"#,
        context
    );
    call_json_step::<KnowledgeStep>(chat_fn, "Step 5", &prompt, system_prompt)
}

fn step6_infer_persona(
    context: &str,
    chat_fn: &dyn Fn(&str, &str) -> Result<String, String>,
    system_prompt: &str,
) -> Result<(PersonaStep, String), String> {
    let prompt = format!(
        r#"{}

Step 6: Infer persona configuration.
Use all previous results as context. Generate persona parameters and all 5 prompt layers for the final agent definition.
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
        context
    );
    call_json_step::<PersonaStep>(chat_fn, "Step 6", &prompt, system_prompt)
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

        let (persona_list, model_list, tool_list, knowledge_list, skill_list) =
            if let Some(reg) = registry {
                let templates = reg.list_templates();
                let personas: Vec<&str> = templates.iter().map(|t| t.persona.as_str()).collect();
                let models: Vec<&str> = templates.iter().map(|t| t.model.as_str()).collect();
                let tools: Vec<&str> = templates
                    .iter()
                    .flat_map(|t| t.tools.iter().map(|s| s.as_str()))
                    .collect();
                let knowledge: Vec<&str> = templates
                    .iter()
                    .flat_map(|t| t.knowledge.iter().map(|s| s.as_str()))
                    .collect();
                let skills: Vec<&str> = templates
                    .iter()
                    .flat_map(|t| t.skills.iter().map(|s| s.as_str()))
                    .collect();
                let mut unique_personas: Vec<&str> = personas.clone();
                unique_personas.sort();
                unique_personas.dedup();
                let mut unique_models: Vec<&str> = models.clone();
                unique_models.sort();
                unique_models.dedup();
                let mut unique_tools: Vec<&str> = tools.clone();
                unique_tools.sort();
                unique_tools.dedup();
                let mut unique_knowledge: Vec<&str> = knowledge.clone();
                unique_knowledge.sort();
                unique_knowledge.dedup();
                let mut unique_skills: Vec<&str> = skills.clone();
                unique_skills.sort();
                unique_skills.dedup();
                (
                    unique_personas.join(", "),
                    unique_models.join(", "),
                    unique_tools.join(", "),
                    unique_knowledge.join(", "),
                    unique_skills.join(", "),
                )
            } else {
                (
                    [
                        "assistant",
                        "analyst",
                        "researcher",
                        "writer",
                        "coder",
                        "translator",
                        "reviewer",
                    ]
                    .join(", "),
                    ["deepseek-chat", "deepseek-reasoner"].join(", "),
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
                    .join(", "),
                    ["docs", "glossary", "data_sources"].join(", "),
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
                    .join(", "),
                )
            };

        let mut context = format!(
            r#"User wants to create an agent.
Description:
{}

Available personas: {}
Available models: {}
Available tools: {}
Available knowledge: {}
Available skills: {}
Available memory: working_memory, episodic_memory, semantic_memory, graph_memory, flash_memory"#,
            nl, persona_list, model_list, tool_list, knowledge_list, skill_list
        );

        let (domain, domain_json) = step1_analyze_domain(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 1", &domain_json);

        let (role, role_json) = step2_determine_role(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 2", &role_json);

        let (capabilities, capabilities_json) =
            step3_infer_capabilities(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 3", &capabilities_json);

        let (tools, tools_json) = step4_infer_tools(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 4", &tools_json);

        let (knowledge, knowledge_json) = step5_infer_knowledge(&context, chat_fn, system_prompt)?;
        append_context(&mut context, "Step 5", &knowledge_json);

        let (persona, _persona_json) = step6_infer_persona(&context, chat_fn, system_prompt)?;

        Ok(collect_step_results(
            domain, role, capabilities, tools, knowledge, persona, suggestions,
        ))
    }
}