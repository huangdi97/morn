//! Agent builder — constructs agents from intent using DESIGN.md §3.1 6-step inference chain.
use crate::core::error::MornError;
use serde::Deserialize;

use crate::core::registry::Registry;

use crate::core::supervisor::Supervisor;
use crate::core::supervisor::{NLAgentDef, NLPersonaConfig};

use super::helpers::*;

#[derive(Debug, Deserialize)]
pub(super) struct DomainStep {
    pub(super) domain: String,
    #[serde(default)]
    pub(super) name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RoleStep {
    pub(super) role: String,
    #[serde(default)]
    pub(super) persona: Option<String>,
    #[serde(default)]
    pub(super) name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CapabilitiesStep {
    #[serde(default)]
    pub(super) capabilities: Vec<String>,
    #[serde(default)]
    pub(super) skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ToolsStep {
    #[serde(default)]
    pub(super) tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct KnowledgeStep {
    #[serde(default)]
    pub(super) knowledge: Vec<String>,
    #[serde(default)]
    pub(super) memory: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PersonaStep {
    #[serde(default)]
    pub(super) persona_config: NLPersonaConfig,
    #[serde(default)]
    pub(super) communication_style: String,
    #[serde(default)]
    pub(super) model: Option<String>,
    #[serde(default)]
    pub(super) persona: Option<String>,
    #[serde(default)]
    pub(super) name: Option<String>,
}

impl Supervisor {
    /// Step 1 — 领域识别: extract the professional domain from natural language.
    fn domain_recognition(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
    ) -> Result<(DomainStep, String), MornError> {
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
        call_json_step::<DomainStep>(
            chat_fn,
            "Step 1 — Domain Recognition",
            &prompt,
            system_prompt,
        )
    }

    /// Step 2 — 角色推断: infer the role type needed.
    fn role_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
    ) -> Result<(RoleStep, String), MornError> {
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
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
    ) -> Result<(CapabilitiesStep, String), MornError> {
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
        call_json_step::<CapabilitiesStep>(
            chat_fn,
            "Step 3 — Capability Inference",
            &prompt,
            system_prompt,
        )
    }

    /// Step 4 — 工具推断: infer required tools, query Registry for recommendations.
    fn tool_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(ToolsStep, String), MornError> {
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
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(KnowledgeStep, String), MornError> {
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
        call_json_step::<KnowledgeStep>(
            chat_fn,
            "Step 5 — Knowledge Inference",
            &prompt,
            system_prompt,
        )
    }

    /// Step 6 — 人格推断: infer suitable persona, match against available personas.
    fn persona_inference(
        &self,
        context: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        system_prompt: &str,
        registry: Option<&Registry>,
    ) -> Result<(PersonaStep, String), MornError> {
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
        call_json_step::<PersonaStep>(
            chat_fn,
            "Step 6 — Persona Inference",
            &prompt,
            system_prompt,
        )
    }

    pub fn create_agent_from_nl(
        &self,
        nl: &str,
        chat_fn: &dyn Fn(&str, &str) -> Result<String, MornError>,
        registry: Option<&Registry>,
    ) -> Result<NLAgentDef, MornError> {
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
        append_context(
            &mut context,
            "Step 3 — Capability Inference",
            &capabilities_json,
        );

        let (tools, tools_json) =
            self.tool_inference(&context, chat_fn, system_prompt, registry)?;
        append_context(&mut context, "Step 4 — Tool Inference", &tools_json);

        let (knowledge, knowledge_json) =
            self.knowledge_inference(&context, chat_fn, system_prompt, registry)?;
        append_context(
            &mut context,
            "Step 5 — Knowledge Inference",
            &knowledge_json,
        );

        let (persona, _persona_json) =
            self.persona_inference(&context, chat_fn, system_prompt, registry)?;

        Ok(collect_step_results(
            domain,
            role,
            capabilities,
            tools,
            knowledge,
            persona,
            suggestions,
        ))
    }
}
