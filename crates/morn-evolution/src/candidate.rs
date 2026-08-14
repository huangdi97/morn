//! Evolution candidates: proposals to change production, never direct writes.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{EvolutionCandidateId, PrincipalId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CandidateType {
    Capability,
    Workflow,
    HarnessPatch,
    Workcell,
    RoleSuggestion,
    SoftwareReplacementSuggestion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CandidateStatus {
    Proposed,
    Branched,
    Evaluated,
    Certified,
    Promoted,
    Rejected,
    Retired,
}

/// A candidate to evolve some production capability. A candidate holds no
/// production write authority; only a certified promotion may change production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionCandidate {
    pub id: EvolutionCandidateId,
    pub workspace_id: WorkspaceId,
    pub candidate_type: CandidateType,
    pub source_window: String,
    pub evidence_refs: Vec<String>,
    pub current_version: Version,
    pub proposed_change: String,
    pub expected_benefit: String,
    pub risk: String,
    pub required_evaluations: Vec<String>,
    pub status: CandidateStatus,
    pub created_by: PrincipalId,
    pub created_at: Timestamp,
}

impl EvolutionCandidate {
    pub fn new(
        workspace_id: WorkspaceId,
        candidate_type: CandidateType,
        current_version: Version,
        proposed_change: impl Into<String>,
        created_by: PrincipalId,
    ) -> Self {
        Self {
            id: EvolutionCandidateId::generate_with("evc"),
            workspace_id,
            candidate_type,
            source_window: String::new(),
            evidence_refs: Vec::new(),
            current_version,
            proposed_change: proposed_change.into(),
            expected_benefit: String::new(),
            risk: String::new(),
            required_evaluations: Vec::new(),
            status: CandidateStatus::Proposed,
            created_by,
            created_at: Timestamp::now(),
        }
    }
}
