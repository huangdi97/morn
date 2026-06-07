use super::{Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub(super) fn run_chain(
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

    pub(super) fn run_routing(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        if members.is_empty() {
            return Err("No members for routing".to_string());
        }
        let idx = input.len() % members.len();
        let selected = &members[idx];
        let result = self.dispatch_agent(selected, &format!("[ROUTED] {}", input))?;
        Ok(vec![result])
    }

    pub(super) fn run_agent_as_tool(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut outputs = Vec::new();
        let primary = if members.is_empty() {
            return Err("No members".to_string());
        } else {
            &members[0]
        };
        let primary_result = self.dispatch_agent(primary, input)?;
        outputs.push(primary_result);

        for tool_agent in &members[1..] {
            let result = self.dispatch_agent(
                tool_agent,
                &format!("[TOOL] {} called by {}", input, primary),
            )?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    pub(super) fn run_blackboard(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        let mut board = format!("[Blackboard] Initial: {}\n", input);
        let mut outputs = Vec::new();
        for member in members {
            let result = self.dispatch_agent(member, &board)?;
            board.push_str(&format!("{}: {}\n", member, result.output));
            outputs.push(result);
        }
        Ok(outputs)
    }
}
