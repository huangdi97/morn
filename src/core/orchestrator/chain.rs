//! chain — Executes team members sequentially with chained intermediate context.
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
}
