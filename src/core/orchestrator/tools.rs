//! tools — Runs tool-enabled team orchestration and collects member outputs.
use super::{Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub(super) fn run_agent_as_tool(
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

    pub(super) fn register_tool(
        &self,
        agent_id: &str,
        input: &str,
    ) -> Result<TeamMemberOutput, String> {
        self.dispatch_agent(agent_id, input)
    }

    pub(super) fn execute_tool_chain(
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
