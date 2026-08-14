//! WorkPackage: the core delegable work unit.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{AcceptanceSpecId, PrincipalId, WorkPackageId, WorkspaceId};
use morn_kernel::status::WorkStatus;
use morn_kernel::time::Timestamp;

use crate::execution_mode::ExecutionMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: WorkPackageId,
    pub workspace_id: WorkspaceId,
    pub objective: String,
    pub inputs: Vec<String>,
    pub required_outputs: Vec<String>,
    pub acceptance_spec_id: Option<AcceptanceSpecId>,
    pub execution_mode: Option<ExecutionMode>,
    pub allowed_actions: Vec<String>,
    pub prohibited_actions: Vec<String>,
    pub budget: Option<String>,
    pub deadline: Option<Timestamp>,
    pub accountable_owner: PrincipalId,
    pub status: WorkStatus,
    pub created_at: Timestamp,
}

impl WorkPackage {
    pub fn new(
        workspace_id: WorkspaceId,
        objective: impl Into<String>,
        accountable_owner: PrincipalId,
    ) -> Self {
        Self {
            id: WorkPackageId::generate_with("wp"),
            workspace_id,
            objective: objective.into(),
            inputs: Vec::new(),
            required_outputs: Vec::new(),
            acceptance_spec_id: None,
            execution_mode: None,
            allowed_actions: Vec::new(),
            prohibited_actions: Vec::new(),
            budget: None,
            deadline: None,
            accountable_owner,
            status: WorkStatus::Proposed,
            created_at: Timestamp::now(),
        }
    }

    pub fn with_acceptance_spec(mut self, acceptance_spec_id: AcceptanceSpecId) -> Self {
        self.acceptance_spec_id = Some(acceptance_spec_id);
        self
    }

    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = Some(mode);
        self
    }

    pub fn has_acceptance_spec(&self) -> bool {
        self.acceptance_spec_id.is_some()
    }
}
