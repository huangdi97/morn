//! helpers — Shared helper methods for orchestration types.
use super::*;

impl CollaborationMode {
    /// Returns the stable string identifier for this collaboration mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            CollaborationMode::Chain => "chain",
            CollaborationMode::ManagerWorker => "manager_worker",
            CollaborationMode::Broadcast => "broadcast",
            CollaborationMode::Voting => "voting",
            CollaborationMode::Routing => "routing",
            CollaborationMode::AgentAsTool => "agent_as_tool",
            CollaborationMode::Blackboard => "blackboard",
        }
    }
}

impl Orchestrator {
    pub(super) fn compute_consensus(
        &self,
        outputs: &[TeamMemberOutput],
        mechanism: &ConsensusMechanism,
    ) -> String {
        match mechanism {
            ConsensusMechanism::CeoDecides => outputs
                .first()
                .map(|o| o.output.clone())
                .unwrap_or_default(),
            ConsensusMechanism::Vote => {
                let best = outputs.iter().max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                best.map(|o| o.output.clone()).unwrap_or_default()
            }
            ConsensusMechanism::MungerVeto => {
                let worst = outputs.iter().min_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                worst
                    .map(|o| format!("[VETO] {}", o.output))
                    .unwrap_or_default()
            }
            ConsensusMechanism::AutoSynthesis => {
                let combined: Vec<String> = outputs.iter().map(|o| o.output.clone()).collect();
                format!(
                    "[Synthesis of {} opinions] {}",
                    outputs.len(),
                    combined.join(" | ")
                )
            }
        }
    }
}
