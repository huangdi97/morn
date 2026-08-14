//! Verification reports.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{VerificationReportId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub id: VerificationReportId,
    pub workspace_id: WorkspaceId,
    pub target: String,
    pub suite: String,
    pub checks: Vec<VerificationCheck>,
    pub passed: bool,
    pub created_at: Timestamp,
}

impl VerificationReport {
    pub fn new(
        workspace_id: WorkspaceId,
        target: impl Into<String>,
        suite: impl Into<String>,
        checks: Vec<VerificationCheck>,
    ) -> Self {
        let passed = !checks.is_empty() && checks.iter().all(|c| c.passed);
        Self {
            id: VerificationReportId::generate_with("ver"),
            workspace_id,
            target: target.into(),
            suite: suite.into(),
            checks,
            passed,
            created_at: Timestamp::now(),
        }
    }
}
