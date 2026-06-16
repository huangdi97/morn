//! manager_worker — Runs manager-worker orchestration and aggregates team results.
use crate::core::error::MornError;
use super::{Orchestrator, TeamMemberOutput, TeamResult};

impl Orchestrator {
    pub(super) fn run_manager_worker(
        &self,
        members: &[String],
        input: &str,
    ) -> Result<Vec<TeamMemberOutput>, MornError> {
        if members.is_empty() {
            return Err(MornError::Internal("No members".to_string()))
        }
        let mut outputs = Vec::new();
        let manager = &members[0];
        let mgr = self.dispatch_agent(manager, input)?;
        outputs.push(mgr);

        for worker in &members[1..] {
            let result = self.dispatch_agent(worker, &format!("{} (from {})", input, manager))?;
            outputs.push(result);
        }
        Ok(outputs)
    }

    pub fn run_manager_expert(&self, manager_id: &str, task: &str) -> Result<TeamResult, MornError> {
        let experts = self.find_experts_for_task(task, 5);
        if experts.is_empty() {
            return Err(MornError::Internal("No suitable experts found for task".to_string()))
        }

        let mut outputs = Vec::new();
        let mgr_output = self.dispatch_agent(
            manager_id,
            &format!(
                "[MANAGER] Task: {}. Delegate to {} experts.",
                task,
                experts.len()
            ),
        )?;
        outputs.push(mgr_output);

        for expert in &experts {
            let result = self.dispatch_agent(
                &expert.id,
                &format!(
                    "[EXPERT:{}] Task: {} (delegated by {})",
                    expert.domain, task, manager_id
                ),
            )?;
            outputs.push(result);
        }

        let synthesis = format!(
            "[Manager Synthesis of {} expert outputs]\n{}",
            experts.len(),
            outputs
                .iter()
                .map(|o| format!("{}: {}", o.agent_id, o.output))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(TeamResult {
            team_id: format!("manager-expert-{}", manager_id),
            outputs,
            consensus_output: synthesis,
            mode: "manager_expert".to_string(),
        })
    }
}
