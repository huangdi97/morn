//! Workcell: a dynamic execution unit assembled around a WorkPackage.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{RoleSlotId, WorkPackageId, WorkcellId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workcell {
    pub id: WorkcellId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub work_package_id: WorkPackageId,
    pub role_slots: Vec<RoleSlotId>,
    pub capabilities: Vec<String>,
    pub status: String,
    pub created_at: Timestamp,
}

impl Workcell {
    pub fn new(
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        work_package_id: WorkPackageId,
    ) -> Self {
        Self {
            id: WorkcellId::generate_with("wc"),
            workspace_id,
            name: name.into(),
            work_package_id,
            role_slots: Vec::new(),
            capabilities: Vec::new(),
            status: "assembling".to_string(),
            created_at: Timestamp::now(),
        }
    }

    pub fn add_role_slot(&mut self, role_slot_id: RoleSlotId) {
        self.role_slots.push(role_slot_id);
    }
}
