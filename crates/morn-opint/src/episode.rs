//! OperationalEpisode: the six-record mapping (world/work/organization/
//! execution/evidence/outcome) aggregated into one versioned episode with
//! economics and authoritative labels.

use serde::{Deserialize, Serialize};

use morn_assurance::managed_work::{AcceptanceDecision, DeliveryStatus};
use morn_kernel::ids::{OperationalEpisodeId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeWorld {
    pub before_ref: Option<String>,
    pub after_ref: Option<String>,
    pub state_diff_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeWork {
    pub package_ref: Option<String>,
    pub contract_ref: Option<String>,
    pub graph_ref: Option<String>,
    pub acceptance_ref: Option<String>,
    pub outcome_contract_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeOrganization {
    pub workcell_ref: Option<String>,
    pub member_bindings: Vec<String>,
    pub delegation_refs: Vec<String>,
    pub representation_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeExecution {
    pub execution_mode: Option<String>,
    pub harness_refs: Vec<String>,
    pub runtime_refs: Vec<String>,
    pub actions: Vec<String>,
    pub retries: u32,
    pub compensations: Vec<String>,
    pub approvals: Vec<String>,
    pub attention_items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeEvidence {
    pub artifacts: Vec<String>,
    pub decisions: Vec<String>,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeOutcome {
    pub delivery_receipt_ref: Option<String>,
    pub acceptance_decision_ref: Option<String>,
    pub outcome_record_ref: Option<String>,
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EpisodeEconomics {
    pub latency_ms: u64,
    pub compute: f64,
    pub model_usage: u32,
    pub cost: f64,
    pub human_minutes: f64,
}

/// Authoritative labels derived from independent records (never actor/LLM self-eval).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EpisodeLabels {
    pub success: bool,
    pub accepted: Option<bool>,
    pub failure_type: Option<String>,
    pub escalation: bool,
    pub rework: bool,
}

/// A versioned operational episode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalEpisode {
    pub id: OperationalEpisodeId,
    pub workspace_id: WorkspaceId,
    pub domain: String,
    pub context_of_use: String,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub world: EpisodeWorld,
    pub work: EpisodeWork,
    pub organization: EpisodeOrganization,
    pub execution: EpisodeExecution,
    pub evidence: EpisodeEvidence,
    pub outcome: EpisodeOutcome,
    pub economics: EpisodeEconomics,
    pub labels: EpisodeLabels,
}

impl OperationalEpisode {
    pub fn new(
        workspace_id: WorkspaceId,
        domain: impl Into<String>,
        context_of_use: impl Into<String>,
    ) -> Self {
        Self {
            id: OperationalEpisodeId::generate_with("ep"),
            workspace_id,
            domain: domain.into(),
            context_of_use: context_of_use.into(),
            started_at: Timestamp::now(),
            ended_at: None,
            world: EpisodeWorld::default(),
            work: EpisodeWork::default(),
            organization: EpisodeOrganization::default(),
            execution: EpisodeExecution::default(),
            evidence: EpisodeEvidence::default(),
            outcome: EpisodeOutcome::default(),
            economics: EpisodeEconomics::default(),
            labels: EpisodeLabels::default(),
        }
    }
}

/// Assembles episodes from authoritative Morn records. Labels come from
/// independent AcceptanceDecision and terminal DeliveryStatus — never from the
/// executor.
#[derive(Debug, Default)]
pub struct EpisodeAssembler {
    pub episodes: Vec<OperationalEpisode>,
}

impl EpisodeAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Finalize an episode with the terminal delivery status and an independent
    /// acceptance decision (if any), deriving authoritative labels.
    pub fn finalize(
        &mut self,
        mut episode: OperationalEpisode,
        terminal: DeliveryStatus,
        acceptance: Option<&AcceptanceDecision>,
    ) -> OperationalEpisode {
        episode.ended_at = Some(Timestamp::now());
        episode.labels.success = terminal == DeliveryStatus::Closed;
        episode.labels.accepted = acceptance.map(|a| a.decision == "accepted");
        episode.labels.escalation = terminal == DeliveryStatus::Escalated;
        episode.labels.rework =
            episode.execution.retries > 0 || !episode.execution.compensations.is_empty();
        episode.labels.failure_type = if terminal == DeliveryStatus::Rejected {
            Some("rejected_delivery".to_string())
        } else if terminal == DeliveryStatus::Escalated {
            Some("escalated".to_string())
        } else if !episode.labels.success {
            Some("not_completed".to_string())
        } else {
            None
        };
        self.episodes.push(episode.clone());
        episode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_assurance::managed_work::{AcceptanceDecision, AcceptanceSource};
    use morn_kernel::ids::ManagedWorkRunId;

    #[test]
    fn episode_labels_come_from_independent_acceptance() {
        let ws = WorkspaceId::generate();
        let mut assembler = EpisodeAssembler::new();
        let mut episode = OperationalEpisode::new(ws, "biolab", "dataset-to-claim");
        episode.execution.retries = 1;
        let acceptance = AcceptanceDecision {
            id: morn_kernel::ids::AcceptanceDecisionId::generate(),
            run_id: ManagedWorkRunId::generate(),
            decision: "accepted".to_string(),
            source: AcceptanceSource::IndependentReviewer,
            decided_by: "pi".to_string(),
            note: "ok".to_string(),
            created_at: Timestamp::now(),
        };
        let final_ep = assembler.finalize(episode, DeliveryStatus::Closed, Some(&acceptance));
        assert!(final_ep.labels.success);
        assert_eq!(final_ep.labels.accepted, Some(true));
        assert!(final_ep.labels.rework, "retry implies rework");
        assert_eq!(assembler.episodes.len(), 1);
    }

    #[test]
    fn rejected_delivery_labeled_failure() {
        let ws = WorkspaceId::generate();
        let mut assembler = EpisodeAssembler::new();
        let episode = OperationalEpisode::new(ws, "biolab", "dataset-to-claim");
        let final_ep = assembler.finalize(episode, DeliveryStatus::Rejected, None);
        assert!(!final_ep.labels.success);
        assert_eq!(
            final_ep.labels.failure_type.as_deref(),
            Some("rejected_delivery")
        );
    }
}
