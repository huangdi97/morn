//! RuntimeContext: the minimal, temporary, auditable context handed to a harness.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ActorInstanceId, WorkPackageId, WorkspaceId};

/// The context a harness receives for one execution. Morn retains provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub workspace_id: WorkspaceId,
    pub actor_id: ActorInstanceId,
    pub work_package_id: WorkPackageId,
    pub policy_snapshot_version: Option<String>,
    pub provenance_refs: Vec<String>,
    pub scope_id: Option<String>,
}

impl RuntimeContext {
    pub fn new(
        workspace_id: WorkspaceId,
        actor_id: ActorInstanceId,
        work_package_id: WorkPackageId,
    ) -> Self {
        Self {
            workspace_id,
            actor_id,
            work_package_id,
            policy_snapshot_version: None,
            provenance_refs: Vec::new(),
            scope_id: None,
        }
    }
}
