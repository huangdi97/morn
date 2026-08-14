//! CapabilityScope: a business scope tree with add/override/restrict/isolate.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ScopeId, WorkspaceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ScopeKind {
    Organization,
    Workspace,
    Solution,
    Workcell,
    Actor,
    ExecutionRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityScope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub restrictions: Vec<String>,
}

impl CapabilityScope {
    pub fn new(
        kind: ScopeKind,
        parent: Option<ScopeId>,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: ScopeId::generate_with("scope"),
            kind,
            parent,
            workspace_id,
            name: name.into(),
            restrictions: Vec::new(),
        }
    }

    pub fn with_restriction(mut self, restriction: impl Into<String>) -> Self {
        self.restrictions.push(restriction.into());
        self
    }
}
