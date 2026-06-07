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

    pub(super) fn run_voting(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, String> {
        if members.len() < 3 {
            return Err("Voting mode requires at least 3 members".to_string());
        }
        let mut outputs = Vec::new();
        for member in members {
            let result = self.dispatch_agent(member, &format!("[EVALUATE] {}", input))?;
            outputs.push(result);
        }
        Ok(outputs)
    }
}
