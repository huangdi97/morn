//! Evolution branches: isolated sandboxes based on a production version.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{EvolutionBranchId, EvolutionCandidateId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BranchStatus {
    Open,
    Evaluated,
    Certified,
    Promoted,
    Abandoned,
}

/// A branch is based on a production version and does not share production
/// write permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvolutionBranch {
    pub id: EvolutionBranchId,
    pub candidate_id: EvolutionCandidateId,
    pub base_version: Version,
    pub status: BranchStatus,
    pub isolated: bool,
    pub created_at: Timestamp,
}

impl EvolutionBranch {
    pub fn new(candidate_id: EvolutionCandidateId, base_version: Version) -> Self {
        Self {
            id: EvolutionBranchId::generate_with("evb"),
            candidate_id,
            base_version,
            status: BranchStatus::Open,
            isolated: true,
            created_at: Timestamp::now(),
        }
    }
}
