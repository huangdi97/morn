//! Outcome records: did the work actually achieve its objective.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    OutcomeRecordId, StateSnapshotId, VerificationReportId, WorkPackageId, WorkspaceId,
};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub id: OutcomeRecordId,
    pub workspace_id: WorkspaceId,
    pub objective: String,
    pub acceptance_met: bool,
    pub state_snapshot_ids: Vec<StateSnapshotId>,
    pub verification_report_id: Option<VerificationReportId>,
    pub work_package_id: Option<WorkPackageId>,
    pub related_artifacts: Vec<String>,
    pub created_at: Timestamp,
}

impl OutcomeRecord {
    pub fn new(workspace_id: WorkspaceId, objective: impl Into<String>, acceptance_met: bool) -> Self {
        Self {
            id: OutcomeRecordId::generate_with("out"),
            workspace_id,
            objective: objective.into(),
            acceptance_met,
            state_snapshot_ids: Vec::new(),
            verification_report_id: None,
            work_package_id: None,
            related_artifacts: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}
