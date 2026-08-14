//! Workspace: the isolation boundary for data, members, policies, artifacts, memory and secrets.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::ids::{PrincipalId, WorkspaceId};
use crate::status::LifecycleStatus;
use crate::time::Timestamp;

/// Kind of workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkspaceKind {
    Personal,
    Project,
    Organization,
    Customer,
    Lab,
    Department,
    Sandbox,
}

/// A workspace is the data/member/policy isolation boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub kind: WorkspaceKind,
    pub owner: PrincipalId,
    pub status: LifecycleStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Workspace {
    pub fn new(name: impl Into<String>, kind: WorkspaceKind, owner: PrincipalId) -> Self {
        let now = Timestamp::now();
        Self {
            id: WorkspaceId::generate_with("ws"),
            name: name.into(),
            kind,
            owner,
            status: LifecycleStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Timestamp::now();
    }

    pub fn suspend(&mut self) -> Result<()> {
        if self.status != LifecycleStatus::Active {
            return Err(Error::invalid_state(
                "only active workspace can be suspended",
            ));
        }
        self.status = LifecycleStatus::Suspended;
        self.touch();
        Ok(())
    }

    pub fn archive(&mut self) -> Result<()> {
        if self.status == LifecycleStatus::Archived {
            return Err(Error::invalid_state("workspace already archived"));
        }
        self.status = LifecycleStatus::Archived;
        self.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_lifecycle() {
        let owner = PrincipalId::generate();
        let mut ws = Workspace::new("Aging Lab", WorkspaceKind::Lab, owner);
        assert_eq!(ws.status, LifecycleStatus::Active);
        ws.suspend().unwrap();
        assert_eq!(ws.status, LifecycleStatus::Suspended);
        assert!(ws.suspend().is_err());
        ws.archive().unwrap();
        assert_eq!(ws.status, LifecycleStatus::Archived);
    }
}
