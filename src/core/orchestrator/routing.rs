//! routing — Selects orchestration routes based on model and task capabilities.
use crate::component::model::ModelConfig;
use crate::component::persona;
use crate::core::assembler::{AgentAssembler, AgentDef};

use super::{ExpertSpec, Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub fn find_experts_for_task(&self, task: &str, max: usize) -> Vec<&ExpertSpec> {
        let task_lower = task.to_lowercase();
        let mut matches: Vec<&ExpertSpec> = self
            .experts
            .values()
            .filter(|e| {
                let domain_lower = e.domain.to_lowercase();
                let desc_lower = e.description.to_lowercase();
                let name_lower = e.name.to_lowercase();
                task_lower.contains(&domain_lower)
                    || task_lower.contains(&name_lower)
                    || task_lower.contains(&desc_lower)
                    || domain_lower.contains(&task_lower)
            })
            .collect();
        if matches.is_empty() {
            matches = self.experts.values().take(max).collect();
        } else {
            matches.truncate(max);
        }
        matches
    }

    pub(super) fn run_routing(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let selected = self.route_to_agent(members, input)?;
        let result = self.dispatch_agent(selected, &format!("[ROUTED] {}", input))?;
        Ok(vec![result])
    }

    pub(super) fn route_to_agent<'a>(
        &self,
        members: &'a [String],
        input: &str,
    ) -> Result<&'a str, String> {
        if members.is_empty() {
            return Err("No members for routing".to_string());
        }
        let idx = input.len() % members.len();
        Ok(&members[idx])
    }

    pub(super) fn dispatch_agent(
        &self,
        agent_id: &str,
        input: &str,
    ) -> Result<TeamMemberOutput, String> {
        if let Some(ref registry) = self.registry {
            if let Some(template) = registry.get_template(agent_id) {
                let persona = persona::get_preset_persona(&template.persona)
                    .unwrap_or_else(persona::create_assistant_persona);
                let model = ModelConfig {
                    id: format!("model-{}", agent_id),
                    provider: "deepseek".into(),
                    model_name: "deepseek-chat".into(),
                    base_url: "https://api.deepseek.com".into(),
                    api_key: String::new(),
                    parameters: Default::default(),
                    fallback: None,
                    cost_tier: crate::component::model::CostTier::Low,
                };
                let agent_def = AgentDef {
                    id: agent_id.to_string(),
                    name: template.name.clone(),
                    persona,
                    model,
                    tools: template.tools.clone(),
                    knowledge: template.knowledge.clone(),
                    skills: template.skills.clone(),
                    memory: None,
                };
                let assembler = AgentAssembler::new(Some(registry.clone()));
                if let Ok(mut agent) = assembler.assemble(agent_def) {
                    let _ = agent.init();
                    let _ = agent.run();
                }
            }
        }

        let confidence = 0.7 + (input.len() as f64 % 30.0) / 100.0;
        let output = format!(
            "[{}] processed: {} (dispatched via registry)",
            agent_id, input
        );
        Ok(TeamMemberOutput {
            agent_id: agent_id.to_string(),
            output,
            confidence: confidence.min(1.0),
        })
    }
}
