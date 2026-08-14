//! Identity model: unified management of Human, Actor, Service, External Agent, Device, Organization.

use serde::{Deserialize, Serialize};

use crate::ids::{IdentityId, PrincipalId, WorkspaceId};
use crate::status::LifecycleStatus;
use crate::time::Timestamp;

/// Principal type of an identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PrincipalType {
    Human,
    Actor,
    Service,
    ExternalAgent,
    Device,
    Organization,
}

/// An identity: who a principal is, independent of any runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub id: IdentityId,
    pub principal_type: PrincipalType,
    pub display_name: String,
    pub owner: Option<PrincipalId>,
    pub sponsor: Option<PrincipalId>,
    pub manager: Option<PrincipalId>,
    pub organization: Option<IdentityId>,
    pub status: LifecycleStatus,
    pub assurance_level: u8,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

impl Identity {
    pub fn new(principal_type: PrincipalType, display_name: impl Into<String>) -> Self {
        Self {
            id: IdentityId::generate_with("idn"),
            principal_type,
            display_name: display_name.into(),
            owner: None,
            sponsor: None,
            manager: None,
            organization: None,
            status: LifecycleStatus::Active,
            assurance_level: 0,
            created_at: Timestamp::now(),
            expires_at: None,
        }
    }

    pub fn with_owner(mut self, owner: PrincipalId) -> Self {
        self.owner = Some(owner);
        self
    }
}

/// A principal is an identity acting within a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    pub id: PrincipalId,
    pub identity_id: IdentityId,
    pub workspace_id: WorkspaceId,
    pub display_name: String,
}

impl Principal {
    pub fn new(
        identity_id: IdentityId,
        workspace_id: WorkspaceId,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: PrincipalId::generate_with("prc"),
            identity_id,
            workspace_id,
            display_name: display_name.into(),
        }
    }
}
