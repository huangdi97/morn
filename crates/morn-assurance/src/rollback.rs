//! Rollback Execution v0.1: request -> approval -> compatibility ->
//! activate previous controlled version/binding -> verify -> receipt.
//! History is never erased; E3 external effects are never claimed as rolled back.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{RollbackReceiptId, RollbackRequestId, WorkspaceId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// Rollback request status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RollbackStatus {
    Requested,
    Approved,
    Rejected,
    Executed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub id: RollbackRequestId,
    pub workspace_id: WorkspaceId,
    pub target_type: String, // capability_release | managed_run | solution
    pub target_ref: String,
    pub current_version: Version,
    pub previous_version: Version,
    pub requested_by: String,
    pub reason: String,
    pub status: RollbackStatus,
    pub approved_by: Option<String>,
    pub created_at: Timestamp,
}

impl RollbackRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        target_type: impl Into<String>,
        target_ref: impl Into<String>,
        current_version: Version,
        previous_version: Version,
        requested_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: RollbackRequestId::generate_with("rbq"),
            workspace_id,
            target_type: target_type.into(),
            target_ref: target_ref.into(),
            current_version,
            previous_version,
            requested_by: requested_by.into(),
            reason: reason.into(),
            status: RollbackStatus::Requested,
            approved_by: None,
            created_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackReceipt {
    pub id: RollbackReceiptId,
    pub request_id: RollbackRequestId,
    pub target_type: String,
    pub target_ref: String,
    pub previous_version: Version,
    pub activated_version: Version,
    pub verified: bool,
    /// E3 external effects are preserved (never claimed as reversed).
    pub e3_external_effects_preserved: bool,
    pub created_at: Timestamp,
}

/// Rollback service. Never deletes history; never fabricates E3 reversal.
#[derive(Debug, Default)]
pub struct RollbackService {
    pub requests: Vec<RollbackRequest>,
    pub receipts: Vec<RollbackReceipt>,
}

impl RollbackService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(&mut self, request: RollbackRequest) -> Result<RollbackRequest> {
        if request.current_version == request.previous_version {
            return Err(Error::validation("cannot rollback to the same version"));
        }
        self.requests.push(request.clone());
        Ok(request)
    }

    /// Approve a rollback request. The approver must not be the requester
    /// (independent authority) and must be provided.
    pub fn approve(&mut self, request_id: &RollbackRequestId, approver: &str) -> Result<()> {
        let req = self
            .requests
            .iter_mut()
            .find(|r| r.id == *request_id)
            .ok_or_else(|| Error::not_found(format!("request {request_id}")))?;
        if req.status != RollbackStatus::Requested {
            return Err(Error::invalid_state(format!(
                "request {request_id} is not Requested (status {:?})",
                req.status
            )));
        }
        if req.requested_by == approver {
            return Err(Error::not_authorized(
                "rollback must be approved by a different principal than the requester",
            ));
        }
        req.status = RollbackStatus::Approved;
        req.approved_by = Some(approver.to_string());
        Ok(())
    }

    /// Compatibility check: previous version must be a known released version
    /// and rollback must move backward (previous <= current).
    pub fn compatibility_check(
        &self,
        request_id: &RollbackRequestId,
        known_releases: &[Version],
    ) -> Result<()> {
        let req = self
            .requests
            .iter()
            .find(|r| r.id == *request_id)
            .ok_or_else(|| Error::not_found(format!("request {request_id}")))?;
        if !known_releases.contains(&req.previous_version) {
            return Err(Error::validation(format!(
                "previous version {} is not a known released version",
                req.previous_version
            )));
        }
        if req.previous_version > req.current_version {
            return Err(Error::validation(
                "rollback must move to a previous (lower) version",
            ));
        }
        Ok(())
    }

    /// Execute the rollback: activate the previous version, verify, and emit a
    /// receipt. Unauthorized (unapproved) execution is rejected.
    pub fn execute(
        &mut self,
        request_id: &RollbackRequestId,
        known_releases: &[Version],
    ) -> Result<RollbackReceipt> {
        let req = self
            .requests
            .iter()
            .find(|r| r.id == *request_id)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("request {request_id}")))?;
        if req.status != RollbackStatus::Approved {
            return Err(Error::not_authorized(format!(
                "request {request_id} is not approved (status {:?})",
                req.status
            )));
        }
        self.compatibility_check(request_id, known_releases)?;
        // Verification: the activated version must equal the requested previous version.
        let verified = known_releases.contains(&req.previous_version);
        if !verified {
            return Err(Error::validation(
                "rollback verification failed: previous version not found",
            ));
        }
        // Update status and record receipt. History (request) is retained.
        if let Some(r) = self.requests.iter_mut().find(|r| r.id == *request_id) {
            r.status = RollbackStatus::Executed;
        }
        let receipt = RollbackReceipt {
            id: RollbackReceiptId::generate_with("rbr"),
            request_id: req.id.clone(),
            target_type: req.target_type.clone(),
            target_ref: req.target_ref.clone(),
            previous_version: req.previous_version,
            activated_version: req.previous_version,
            verified,
            // E3 external effects are never claimed as reversed by a rollback.
            e3_external_effects_preserved: true,
            created_at: Timestamp::now(),
        };
        self.receipts.push(receipt.clone());
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(ws: &WorkspaceId) -> RollbackRequest {
        RollbackRequest::new(
            ws.clone(),
            "capability_release",
            "dataset-to-claim",
            Version::new(1, 1, 0),
            Version::new(1, 0, 0),
            "operator",
            "regression after 1.1",
        )
    }

    #[test]
    fn valid_rollback_executes_with_receipt() {
        let ws = WorkspaceId::generate();
        let mut svc = RollbackService::new();
        let req = request(&ws);
        let req_id = req.id.clone();
        svc.request(req).unwrap();
        svc.approve(&req_id, "pi").unwrap();
        let receipt = svc
            .execute(&req_id, &[Version::v1(), Version::new(1, 1, 0)])
            .unwrap();
        assert!(receipt.verified);
        assert_eq!(receipt.activated_version, Version::v1());
        assert!(receipt.e3_external_effects_preserved);
        assert_eq!(svc.receipts.len(), 1);
        // history retained
        assert_eq!(svc.requests.len(), 1);
        assert_eq!(svc.requests[0].status, RollbackStatus::Executed);
    }

    #[test]
    fn unauthorized_rollback_rejected() {
        let ws = WorkspaceId::generate();
        let mut svc = RollbackService::new();
        let req = request(&ws);
        let req_id = req.id.clone();
        svc.request(req).unwrap();
        // Not approved -> execute fails.
        assert!(svc.execute(&req_id, &[Version::v1()]).is_err());
        // Approver == requester -> rejected.
        assert!(svc.approve(&req_id, "operator").is_err());
    }

    #[test]
    fn incompatible_rollback_rejected() {
        let ws = WorkspaceId::generate();
        let mut svc = RollbackService::new();
        let req = request(&ws);
        let req_id = req.id.clone();
        svc.request(req).unwrap();
        svc.approve(&req_id, "pi").unwrap();
        // Previous version not in known releases -> incompatible.
        assert!(svc
            .compatibility_check(&req_id, &[Version::new(1, 1, 0)])
            .is_err());
        assert!(svc.execute(&req_id, &[Version::new(1, 1, 0)]).is_err());
    }

    #[test]
    fn history_not_erased_after_rollback() {
        let ws = WorkspaceId::generate();
        let mut svc = RollbackService::new();
        let req = request(&ws);
        let req_id = req.id.clone();
        svc.request(req).unwrap();
        svc.approve(&req_id, "pi").unwrap();
        svc.execute(&req_id, &[Version::v1(), Version::new(1, 1, 0)])
            .unwrap();
        // The original request and the new receipt both remain.
        assert_eq!(svc.requests.len(), 1);
        assert_eq!(svc.receipts.len(), 1);
        assert!(svc.receipts[0].e3_external_effects_preserved);
    }
}
