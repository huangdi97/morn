//! Approval requests and decisions (human-in-the-loop gates).

use serde::{Deserialize, Serialize};

use crate::ids::{ApprovalRequestId, PrincipalId, WorkspaceId};
use crate::status::ApprovalStatus;
use crate::time::Timestamp;

/// Decision taken on an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    ChangesRequested,
}

/// An approval request that must be satisfied before gated actions proceed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: ApprovalRequestId,
    pub workspace_id: WorkspaceId,
    pub subject: String,
    pub description: String,
    /// Roles (or principal ids) that are allowed to decide.
    pub required_approvers: Vec<String>,
    pub status: ApprovalStatus,
    pub decision: Option<ApprovalDecision>,
    pub decided_by: Option<PrincipalId>,
    pub decision_note: Option<String>,
    pub created_at: Timestamp,
    pub decided_at: Option<Timestamp>,
}

impl ApprovalRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        subject: impl Into<String>,
        description: impl Into<String>,
        required_approvers: Vec<String>,
    ) -> Self {
        Self {
            id: ApprovalRequestId::generate_with("apr"),
            workspace_id,
            subject: subject.into(),
            description: description.into(),
            required_approvers,
            status: ApprovalStatus::Pending,
            decision: None,
            decided_by: None,
            decision_note: None,
            created_at: Timestamp::now(),
            decided_at: None,
        }
    }

    pub fn is_pending(&self) -> bool {
        self.status == ApprovalStatus::Pending
    }

    /// Decide the request. `approver` must be listed among required approvers.
    pub fn decide(
        &mut self,
        approver: &PrincipalId,
        decision: ApprovalDecision,
        note: Option<String>,
    ) -> crate::Result<()> {
        if !self.is_pending() {
            return Err(crate::Error::invalid_state(format!(
                "approval {} already decided",
                self.id
            )));
        }
        if !self
            .required_approvers
            .iter()
            .any(|r| r == approver.as_str())
        {
            return Err(crate::Error::not_authorized(format!(
                "principal {approver} is not an allowed approver"
            )));
        }
        self.status = match decision {
            ApprovalDecision::Approved => ApprovalStatus::Approved,
            ApprovalDecision::Rejected => ApprovalStatus::Rejected,
            ApprovalDecision::ChangesRequested => ApprovalStatus::ChangesRequested,
        };
        self.decision = Some(decision);
        self.decided_by = Some(approver.clone());
        self.decision_note = note;
        self.decided_at = Some(Timestamp::now());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_requires_allowed_approver() {
        let ws = WorkspaceId::generate();
        let pi = PrincipalId::generate_with("pi");
        let analyst = PrincipalId::generate_with("analyst");
        let mut req = ApprovalRequest::new(
            ws,
            "release claim",
            "claim-1",
            vec![pi.as_str().to_string()],
        );
        assert!(req
            .decide(&analyst, ApprovalDecision::Approved, None)
            .is_err());
        assert!(req.is_pending());
        req.decide(&pi, ApprovalDecision::Approved, None).unwrap();
        assert!(!req.is_pending());
        assert_eq!(req.decision, Some(ApprovalDecision::Approved));
        // second decide must fail
        assert!(req.decide(&pi, ApprovalDecision::Rejected, None).is_err());
    }
}
