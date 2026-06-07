//! broadcast — Runs team members in broadcast mode and gathers their outputs.
use super::{Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub(super) fn run_broadcast(
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
