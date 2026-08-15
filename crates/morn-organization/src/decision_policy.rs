//! DecisionPolicyAsset: a versioned decision framework (criteria, priorities,
//! trade-offs, risk preference, review rubric), not a persona copy.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{DecisionPolicyAssetId, PrincipalId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// One entry in the decision criteria list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionCriterion {
    pub name: String,
    pub weight: f64,
    pub description: String,
}

/// A versioned decision-policy asset for an expert/role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionPolicyAsset {
    pub id: DecisionPolicyAssetId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub version: Version,
    pub owner: PrincipalId,
    pub criteria: Vec<DecisionCriterion>,
    pub priorities: Vec<String>,
    pub tradeoff_rules: Vec<String>,
    pub risk_preference: String,
    pub review_rubric: Vec<String>,
    pub escalation_threshold: Option<String>,
    pub known_exceptions: Vec<String>,
    pub historical_decisions: Vec<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DecisionPolicyAsset {
    pub fn new(workspace_id: WorkspaceId, name: impl Into<String>, owner: PrincipalId) -> Self {
        let now = Timestamp::now();
        Self {
            id: DecisionPolicyAssetId::generate_with("dpa"),
            workspace_id,
            name: name.into(),
            version: Version::v1(),
            owner,
            criteria: Vec::new(),
            priorities: Vec::new(),
            tradeoff_rules: Vec::new(),
            risk_preference: "balanced".to_string(),
            review_rubric: Vec::new(),
            escalation_threshold: None,
            known_exceptions: Vec::new(),
            historical_decisions: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Immutable-by-convention update: returns a NEW version instead of mutating.
    pub fn updated(&self) -> DecisionPolicyAsset {
        let mut next = self.clone();
        next.version = self.version.next_patch();
        next.updated_at = Timestamp::now();
        next
    }

    pub fn add_criterion(mut self, name: &str, weight: f64, description: &str) -> Self {
        self.criteria.push(DecisionCriterion {
            name: name.to_string(),
            weight,
            description: description.to_string(),
        });
        self
    }

    pub fn add_exception(mut self, exception: &str) -> Self {
        self.known_exceptions.push(exception.to_string());
        self
    }

    pub fn criterion(&self, name: &str) -> Option<&DecisionCriterion> {
        self.criteria.iter().find(|c| c.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_policy_asset_is_versioned() {
        let ws = WorkspaceId::generate();
        let owner = PrincipalId::generate();
        let v1 = DecisionPolicyAsset::new(ws, "pi-review", owner)
            .add_criterion("reproducibility", 0.4, "must be reproducible")
            .add_exception("single-cell pilot");
        assert_eq!(v1.version, Version::v1());
        let v2 = v1.updated().add_criterion("cost", 0.1, "budget impact");
        assert_eq!(v2.version, Version::new(1, 0, 1));
        assert!(v2.criterion("cost").is_some());
        // v1 remains readable
        assert!(v1.criterion("cost").is_none());
        assert_eq!(v1.known_exceptions.len(), 1);
    }
}
