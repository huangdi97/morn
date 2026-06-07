//! voting — Coordinates voting-based team decisions across member outputs.
use super::{Orchestrator, TeamMemberOutput};

impl Orchestrator {
    pub(super) fn run_voting(
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

    pub(super) fn submit_vote(
        &self,
        member: &str,
        input: &str,
    ) -> Result<TeamMemberOutput, String> {
        self.dispatch_agent(member, &format!("[EVALUATE] {}", input))
    }

    pub(super) fn count_votes<'a>(
        &self,
        outputs: &'a [TeamMemberOutput],
    ) -> Option<&'a TeamMemberOutput> {
        outputs.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub(super) fn check_quorum(&self, members: &[String]) -> Result<(), String> {
        if members.len() < 3 {
            return Err("Voting mode requires at least 3 members".to_string());
        }
        Ok(())
    }
}
