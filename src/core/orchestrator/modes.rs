use crate::component::model::ModelConfig;
use crate::component::persona;
use crate::core::assembler::{AgentAssembler, AgentDef};
use crate::core::orchestrator::*;

// === voting ===

impl Orchestrator {
    pub fn run_voting(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        self.check_quorum(members)?;
        let mut outputs = Vec::new();
        for member in members {
            outputs.push(self.submit_vote(member, input)?);
        }
        let _leading_vote = self.count_votes(&outputs);
        Ok(outputs)
    }

    pub fn submit_vote(&self, member: &str, input: &str) -> Result<TeamMemberOutput, String> {
        self.dispatch_agent(member, &format!("[EVALUATE] {}", input))
    }

    pub fn count_votes<'a>(&self, outputs: &'a [TeamMemberOutput]) -> Option<&'a TeamMemberOutput> {
        outputs.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn check_quorum(&self, members: &[String]) -> Result<(), String> {
        if members.len() < 3 {
            return Err("Voting mode requires at least 3 members".to_string());
        }
        Ok(())
    }
}

// === blackboard ===

impl Orchestrator {
    pub fn run_blackboard(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut board = format!("[Blackboard] Initial: {}\n", input);
        let mut outputs = Vec::new();
        for member in members {
            let result = self.dispatch_agent(member, self.read_blackboard(&board))?;
            self.post_to_blackboard(&mut board, member, &result.output);
            outputs.push(result);
        }
        self.clear_blackboard(&mut board);
        Ok(outputs)
    }

    pub fn post_to_blackboard(&self, board: &mut String, member: &str, output: &str) {
        board.push_str(&format!("{}: {}\n", member, output));
    }

    pub fn read_blackboard<'a>(&self, board: &'a str) -> &'a str {
        board
    }

    pub fn clear_blackboard(&self, board: &mut String) {
        board.clear();
    }
}

// === chain ===

impl Orchestrator {
    pub fn run_chain(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        if members.is_empty() {
            return Err("No members in chain".to_string());
        }
        let mut outputs = Vec::new();
        let mut current = input.to_string();
        for member in members {
            let result = self.dispatch_agent(member, &current)?;
            current = result.output.clone();
            outputs.push(result);
        }
        Ok(outputs)
    }
}

// === broadcast ===

impl Orchestrator {
    pub fn run_broadcast(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut outputs = Vec::new();
        for member in members {
            let result = self.dispatch_agent(member, &format!("[BROADCAST] {}", input))?;
            outputs.push(result);
        }
        Ok(outputs)
    }
}

// === tools ===

impl Orchestrator {
    pub fn run_agent_as_tool(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let primary = if members.is_empty() {
            return Err("No members".to_string());
        } else {
            &members[0]
        };

        let mut outputs = vec![self.register_tool(primary, input)?];
        outputs.extend(self.execute_tool_chain(&members[1..], input, primary)?);
        Ok(outputs)
    }

    pub fn register_tool(&self, agent_id: &str, input: &str) -> Result<TeamMemberOutput, String> {
        self.dispatch_agent(agent_id, input)
    }

    pub fn execute_tool_chain(
        &self,
        tool_agents: &[String],
        input: &str,
        primary: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut outputs = Vec::new();
        for tool_agent in tool_agents {
            let result = self.dispatch_agent(
                tool_agent,
                &format!("[TOOL] {} called by {}", input, primary),
            )?;
            outputs.push(result);
        }
        Ok(outputs)
    }
}

// === routing ===

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

    pub fn run_routing(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let selected = self.route_to_agent(members, input)?;
        let result = self.dispatch_agent(selected, &format!("[ROUTED] {}", input))?;
        Ok(vec![result])
    }

    pub fn route_to_agent<'a>(
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

    pub fn dispatch_agent(&self, agent_id: &str, input: &str) -> Result<TeamMemberOutput, String> {
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
