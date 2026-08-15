//! Workcell: a dynamic execution unit assembled around a WorkPackage.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{RoleSlotId, WorkPackageId, WorkcellId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// Lifecycle of a workcell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkcellStatus {
    Assembling,
    Active,
    Suspended,
    Completed,
    Archived,
}

impl WorkcellStatus {
    pub fn can_transition_to(self, next: WorkcellStatus) -> bool {
        use WorkcellStatus::*;
        matches!(
            (self, next),
            (Assembling, Active)
                | (Assembling, Archived)
                | (Active, Suspended)
                | (Active, Completed)
                | (Suspended, Active)
                | (Suspended, Completed)
                | (Suspended, Archived)
                | (Completed, Archived)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workcell {
    pub id: WorkcellId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub work_package_id: WorkPackageId,
    pub role_slots: Vec<RoleSlotId>,
    pub capabilities: Vec<String>,
    pub status: WorkcellStatus,
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
            status: WorkcellStatus::Assembling,
            created_at: Timestamp::now(),
        }
    }

    pub fn add_role_slot(&mut self, role_slot_id: RoleSlotId) {
        self.role_slots.push(role_slot_id);
    }

    pub fn transition(&mut self, next: WorkcellStatus) -> Result<()> {
        if !self.status.can_transition_to(next) {
            return Err(Error::invalid_state(format!(
                "workcell {} cannot transition from {:?} to {:?}",
                self.id, self.status, next
            )));
        }
        self.status = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workcell_lifecycle() {
        let ws = WorkspaceId::generate();
        let wp = WorkPackageId::generate();
        let mut wc = Workcell::new(ws, "analysis", wp);
        assert_eq!(wc.status, WorkcellStatus::Assembling);
        wc.transition(WorkcellStatus::Active).unwrap();
        assert!(wc.transition(WorkcellStatus::Completed).is_ok());
        assert!(wc.transition(WorkcellStatus::Archived).is_ok());
        // invalid transition from Archived
        assert!(wc.transition(WorkcellStatus::Active).is_err());
    }

    #[test]
    fn workcell_cannot_jump_to_completed_from_assembling() {
        let ws = WorkspaceId::generate();
        let wp = WorkPackageId::generate();
        let mut wc = Workcell::new(ws, "analysis", wp);
        assert!(wc.transition(WorkcellStatus::Completed).is_err());
    }
}
