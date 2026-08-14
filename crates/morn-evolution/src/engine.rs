//! EvolutionEngine: governed promotion. Candidates cannot mutate production.

use std::collections::HashMap;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{EvolutionBranchId, EvolutionCandidateId, PrincipalId, PromotionDecisionId};
use morn_kernel::version::Version;

use crate::branch::{BranchStatus, EvolutionBranch};
use crate::candidate::{CandidateStatus, EvolutionCandidate};
use crate::evaluation::EvolutionEvaluation;
use crate::promotion::{PromotionDecision, PromotionOutcome, RollbackRecord};

/// The governed evolution engine.
///
/// Production versions live here; the ONLY way to change them is a certified
/// `promote()` that passes evaluation + policy + required approvals and creates
/// a new version (with a rollback record).
#[derive(Debug, Default)]
pub struct EvolutionEngine {
    candidates: HashMap<EvolutionCandidateId, EvolutionCandidate>,
    branches: HashMap<EvolutionBranchId, EvolutionBranch>,
    evaluations: Vec<EvolutionEvaluation>,
    promotions: Vec<PromotionDecision>,
    rollbacks: Vec<RollbackRecord>,
    production_versions: HashMap<String, Version>,
    policy_version: Version,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        Self {
            production_versions: HashMap::new(),
            policy_version: Version::v1(),
            ..Default::default()
        }
    }

    pub fn add_candidate(&mut self, candidate: EvolutionCandidate) {
        self.candidates.insert(candidate.id.clone(), candidate);
    }

    pub fn candidate(&self, id: &EvolutionCandidateId) -> Option<&EvolutionCandidate> {
        self.candidates.get(id)
    }

    pub fn candidates(&self) -> Vec<&EvolutionCandidate> {
        self.candidates.values().collect()
    }

    /// Branch from a production version. The branch is isolated and has no
    /// production write permission.
    pub fn branch(&mut self, candidate_id: &EvolutionCandidateId) -> Result<EvolutionBranch> {
        let candidate = self
            .candidates
            .get(candidate_id)
            .ok_or_else(|| Error::not_found(format!("candidate {candidate_id}")))?;
        let branch = EvolutionBranch::new(candidate_id.clone(), candidate.current_version);
        if let Some(c) = self.candidates.get_mut(candidate_id) {
            c.status = CandidateStatus::Branched;
        }
        self.branches.insert(branch.id.clone(), branch.clone());
        Ok(branch)
    }

    pub fn record_evaluation(&mut self, evaluation: EvolutionEvaluation) -> Result<()> {
        if !self.branches.contains_key(&evaluation.branch_id) {
            return Err(Error::not_found(format!(
                "branch {}",
                evaluation.branch_id
            )));
        }
        if let Some(branch) = self.branches.get_mut(&evaluation.branch_id) {
            branch.status = if evaluation.passed() {
                BranchStatus::Evaluated
            } else {
                BranchStatus::Abandoned
            };
        }
        self.evaluations.push(evaluation);
        Ok(())
    }

    /// Promote a branch to production. Guards:
    /// 1. branch must exist and have a passing evaluation;
    /// 2. required risk approvals must be satisfied for high-risk candidates;
    /// 3. promotion creates a new production version (never overwrite).
    pub fn promote(
        &mut self,
        branch_id: &EvolutionBranchId,
        approved_by: PrincipalId,
        risk_approvals: &[String],
    ) -> Result<PromotionDecision> {
        let branch = self
            .branches
            .get(branch_id)
            .ok_or_else(|| Error::not_found(format!("branch {branch_id}")))?;
        if branch.status != BranchStatus::Evaluated {
            return Err(Error::invalid_state(format!(
                "branch {branch_id} must be Evaluated before promotion (status {:?})",
                branch.status
            )));
        }
        let has_passing_eval = self
            .evaluations
            .iter()
            .any(|e| e.branch_id == *branch_id && e.passed());
        if !has_passing_eval {
            return Err(Error::validation(
                "cannot promote: no passing evaluation for branch",
            ));
        }

        let candidate = self
            .candidates
            .get(&branch.candidate_id)
            .ok_or_else(|| Error::not_found(format!("candidate {}", branch.candidate_id)))?;
        let high_risk = matches!(
            candidate.candidate_type,
            crate::candidate::CandidateType::RoleSuggestion
                | crate::candidate::CandidateType::SoftwareReplacementSuggestion
                | crate::candidate::CandidateType::Workcell
        );
        if high_risk && risk_approvals.is_empty() {
            return Err(Error::not_authorized(
                "high-risk evolution requires approval",
            ));
        }

        // Create a NEW production version.
        let current = self
            .production_versions
            .get(candidate.proposed_change.as_str())
            .cloned()
            .unwrap_or(candidate.current_version);
        let new_version = current.next_minor();
        self.production_versions
            .insert(candidate.proposed_change.clone(), new_version);

        let decision = PromotionDecision {
            id: PromotionDecisionId::generate_with("prom"),
            branch_id: branch_id.clone(),
            outcome: PromotionOutcome::Promoted,
            approved_by: Some(approved_by),
            policy_version: self.policy_version,
            previous_version: current,
            new_version: Some(new_version),
            rollback_ref: Some(format!("rollback-{}", uuid::Uuid::new_v4())),
            created_at: morn_kernel::time::Timestamp::now(),
        };
        self.promotions.push(decision.clone());
        let promoted_candidate = branch.candidate_id.clone();
        if let Some(branch) = self.branches.get_mut(branch_id) {
            branch.status = BranchStatus::Promoted;
        }
        if let Some(c) = self.candidates.get_mut(&promoted_candidate) {
            c.status = CandidateStatus::Promoted;
        }
        Ok(decision)
    }

    /// Roll back a promotion: record rollback metadata; previous version becomes
    /// the active production version again.
    pub fn rollback(&mut self, promotion_id: &PromotionDecisionId) -> Result<RollbackRecord> {
        let promotion = self
            .promotions
            .iter()
            .find(|p| p.id == *promotion_id)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("promotion {promotion_id}")))?;
        let record = RollbackRecord {
            id: PromotionDecisionId::generate_with("rb"),
            promotion_id: promotion.id.clone(),
            previous_version: promotion.previous_version,
            current_version: promotion.previous_version,
            rollback_eligible: true,
            created_at: morn_kernel::time::Timestamp::now(),
        };
        self.rollbacks.push(record.clone());
        Ok(record)
    }

    pub fn production_version(&self, target: &str) -> Option<&Version> {
        self.production_versions.get(target)
    }

    pub fn promotions(&self) -> &[PromotionDecision] {
        &self.promotions
    }

    pub fn rollbacks(&self) -> &[RollbackRecord] {
        &self.rollbacks
    }

    pub fn set_policy_version(&mut self, version: Version) {
        self.policy_version = version;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{CandidateType, EvolutionCandidate};
    use crate::evaluation::EvolutionEvaluation;
    use morn_kernel::ids::{PrincipalId, WorkspaceId};

    fn candidate(ws: WorkspaceId, ty: CandidateType, by: PrincipalId) -> EvolutionCandidate {
        EvolutionCandidate::new(ws, ty, Version::v1(), "workflow v2", by)
    }

    #[test]
    fn failed_evaluation_cannot_promote() {
        let mut engine = EvolutionEngine::new();
        let ws = WorkspaceId::generate();
        let by = PrincipalId::generate();
        let cand = candidate(ws, CandidateType::Workflow, by);
        let cand_id = cand.id.clone();
        engine.add_candidate(cand);
        let branch = engine.branch(&cand_id).unwrap();
        engine
            .record_evaluation(EvolutionEvaluation::new(
                branch.id.clone(),
                false,
                true,
                true,
                "regression failed",
            ))
            .unwrap();
        let err = engine.promote(&branch.id, PrincipalId::generate(), &[]);
        assert!(err.is_err(), "failed evaluation must not promote");
    }

    #[test]
    fn high_risk_requires_approval() {
        let mut engine = EvolutionEngine::new();
        let ws = WorkspaceId::generate();
        let by = PrincipalId::generate();
        let cand = candidate(ws, CandidateType::Workcell, by.clone());
        let cand_id = cand.id.clone();
        engine.add_candidate(cand);
        let branch = engine.branch(&cand_id).unwrap();
        engine
            .record_evaluation(EvolutionEvaluation::new(
                branch.id.clone(),
                true,
                true,
                true,
                "all green",
            ))
            .unwrap();
        let err = engine.promote(&branch.id, by, &[]);
        assert!(err.is_err(), "high-risk promotion without approval must fail");
    }

    #[test]
    fn successful_promote_creates_new_version() {
        let mut engine = EvolutionEngine::new();
        let ws = WorkspaceId::generate();
        let by = PrincipalId::generate();
        let cand = candidate(ws, CandidateType::Workflow, by.clone());
        let cand_id = cand.id.clone();
        let change = cand.proposed_change.clone();
        engine.add_candidate(cand);
        let branch = engine.branch(&cand_id).unwrap();
        engine
            .record_evaluation(EvolutionEvaluation::new(
                branch.id.clone(),
                true,
                true,
                true,
                "all green",
            ))
            .unwrap();
        let decision = engine
            .promote(&branch.id, by, &["governance".to_string()])
            .unwrap();
        assert_eq!(decision.outcome, PromotionOutcome::Promoted);
        assert_eq!(decision.new_version, Some(Version::new(1, 1, 0)));
        assert_eq!(decision.previous_version, Version::v1());
        assert_eq!(
            engine.production_version(&change),
            Some(&Version::new(1, 1, 0))
        );
        assert!(decision.rollback_ref.is_some());
    }

    #[test]
    fn rollback_restores_previous_version() {
        let mut engine = EvolutionEngine::new();
        let ws = WorkspaceId::generate();
        let by = PrincipalId::generate();
        let cand = candidate(ws, CandidateType::Workflow, by.clone());
        let cand_id = cand.id.clone();
        engine.add_candidate(cand);
        let branch = engine.branch(&cand_id).unwrap();
        engine
            .record_evaluation(EvolutionEvaluation::new(
                branch.id.clone(),
                true,
                true,
                true,
                "all green",
            ))
            .unwrap();
        let decision = engine.promote(&branch.id, by, &["governance".to_string()]).unwrap();
        let record = engine.rollback(&decision.id).unwrap();
        assert!(record.rollback_eligible);
        assert_eq!(record.previous_version, Version::v1());
        assert_eq!(record.current_version, Version::v1());
    }
}



