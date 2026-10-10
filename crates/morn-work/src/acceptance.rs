//! AcceptanceSpec: the definition of done for a WorkPackage.

use serde::{Deserialize, Serialize};

use crate::acceptance_decision::AcceptanceDisposition;
use morn_kernel::ids::{AcceptanceSpecId, OutcomeRecordId, PrincipalId, WorkPackageId};
use morn_kernel::time::Timestamp;

/// A WorkPackage can only be accepted when its AcceptanceSpec is satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceSpec {
    pub id: AcceptanceSpecId,
    pub name: String,
    pub required_artifacts: Vec<String>,
    pub schema_checks: Vec<String>,
    pub domain_rules: Vec<String>,
    pub reproducibility: bool,
    pub reviewer_requirements: Vec<String>,
    pub metric_thresholds: Vec<String>,
    pub forbidden_conditions: Vec<String>,
    pub human_approval_required: bool,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceReviewerAttestation {
    pub principal_id: PrincipalId,
    pub acting_roles: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub observed_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceReviewAuthorization {
    pub authorization_id: String,
    pub principal_id: PrincipalId,
    pub acting_role: String,
    pub work_package_id: WorkPackageId,
    pub work_generation: u64,
    pub outcome_id: OutcomeRecordId,
    pub disposition: AcceptanceDisposition,
    pub evidence_refs: Vec<String>,
    pub issued_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}

impl AcceptanceReviewAuthorization {
    pub fn validate(&self) -> morn_kernel::error::Result<()> {
        if self.authorization_id.trim().is_empty()
            || self.acting_role.trim().is_empty()
            || self.work_generation == 0
            || self.evidence_refs.is_empty()
        {
            return Err(morn_kernel::error::Error::validation(
                "review authorization requires id, role and deployment evidence",
            ));
        }
        if self.valid_until.is_some_and(|until| until < self.issued_at) {
            return Err(morn_kernel::error::Error::validation(
                "review authorization validity cannot end before issue time",
            ));
        }
        Ok(())
    }

    pub fn authorizes(
        &self,
        principal_id: &PrincipalId,
        role: &str,
        work_identity: (&WorkPackageId, u64),
        outcome_id: &OutcomeRecordId,
        disposition: AcceptanceDisposition,
        now: Timestamp,
    ) -> bool {
        self.validate().is_ok()
            && &self.principal_id == principal_id
            && self.acting_role == role
            && &self.work_package_id == work_identity.0
            && self.work_generation == work_identity.1
            && &self.outcome_id == outcome_id
            && self.disposition == disposition
            && self.issued_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
}

impl AcceptanceReviewerAttestation {
    pub fn validate(&self) -> morn_kernel::error::Result<()> {
        if self.acting_roles.is_empty()
            || self.acting_roles.iter().any(|role| role.trim().is_empty())
            || self.evidence_refs.is_empty()
        {
            return Err(morn_kernel::error::Error::validation(
                "acceptance reviewer attestation requires roles and identity evidence",
            ));
        }
        if self
            .valid_until
            .is_some_and(|until| until < self.observed_at)
        {
            return Err(morn_kernel::error::Error::validation(
                "reviewer attestation validity cannot end before observation",
            ));
        }
        Ok(())
    }

    pub fn active_for(&self, role: &str, now: Timestamp) -> bool {
        self.validate().is_ok()
            && self.acting_roles.iter().any(|allowed| allowed == role)
            && self.observed_at <= now
            && self.valid_until.is_none_or(|until| now <= until)
    }
}

impl AcceptanceSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AcceptanceSpecId::generate_with("acc"),
            name: name.into(),
            required_artifacts: Vec::new(),
            schema_checks: Vec::new(),
            domain_rules: Vec::new(),
            reproducibility: true,
            reviewer_requirements: Vec::new(),
            metric_thresholds: Vec::new(),
            forbidden_conditions: Vec::new(),
            human_approval_required: false,
            created_at: Timestamp::now(),
        }
    }

    pub fn with_required_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.required_artifacts = artifacts;
        self
    }

    pub fn with_human_approval(mut self, required: bool) -> Self {
        self.human_approval_required = required;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_authorization_is_exact_and_time_bounded() {
        let principal = PrincipalId::generate_with("reviewer");
        let work = WorkPackageId::generate_with("work");
        let outcome = OutcomeRecordId::generate_with("outcome");
        let authorization = AcceptanceReviewAuthorization {
            authorization_id: "review-auth-1".to_string(),
            principal_id: principal.clone(),
            acting_role: "independent-reviewer".to_string(),
            work_package_id: work.clone(),
            work_generation: 2,
            outcome_id: outcome.clone(),
            disposition: AcceptanceDisposition::Accept,
            evidence_refs: vec!["iam://signed-review/review-auth-1".to_string()],
            issued_at: Timestamp::from_millis(10),
            valid_until: Some(Timestamp::from_millis(20)),
        };
        authorization.validate().unwrap();
        assert!(authorization.authorizes(
            &principal,
            "independent-reviewer",
            (&work, 2),
            &outcome,
            AcceptanceDisposition::Accept,
            Timestamp::from_millis(15),
        ));
        assert!(!authorization.authorizes(
            &principal,
            "independent-reviewer",
            (&work, 2),
            &outcome,
            AcceptanceDisposition::Reject,
            Timestamp::from_millis(15),
        ));
        assert!(!authorization.authorizes(
            &principal,
            "independent-reviewer",
            (&work, 2),
            &outcome,
            AcceptanceDisposition::Accept,
            Timestamp::from_millis(21),
        ));
    }

    #[test]
    fn reviewer_attestation_requires_identity_evidence_and_active_role() {
        let reviewer = AcceptanceReviewerAttestation {
            principal_id: PrincipalId::generate_with("reviewer"),
            acting_roles: vec!["independent-reviewer".to_string()],
            evidence_refs: vec!["iam://reviewers/alice".to_string()],
            observed_at: Timestamp::from_millis(10),
            valid_until: Some(Timestamp::from_millis(20)),
        };
        reviewer.validate().unwrap();
        assert!(reviewer.active_for("independent-reviewer", Timestamp::from_millis(15)));
        assert!(!reviewer.active_for("approver", Timestamp::from_millis(15)));
        assert!(!reviewer.active_for("independent-reviewer", Timestamp::from_millis(21)));
    }
}
