//! Promotion decisions and rollback metadata.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{EvolutionBranchId, PromotionDecisionId, PrincipalId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PromotionOutcome {
    Promoted,
    Rejected,
    Pending,
}

/// A promotion creates a NEW production version; it never overwrites in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub id: PromotionDecisionId,
    pub branch_id: EvolutionBranchId,
    pub outcome: PromotionOutcome,
    pub approved_by: Option<PrincipalId>,
    pub policy_version: Version,
    pub previous_version: Version,
    pub new_version: Option<Version>,
    pub rollback_ref: Option<String>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackRecord {
    pub id: PromotionDecisionId,
    pub promotion_id: PromotionDecisionId,
    pub previous_version: Version,
    pub current_version: Version,
    pub rollback_eligible: bool,
    pub created_at: Timestamp,
}
