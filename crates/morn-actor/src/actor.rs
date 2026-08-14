//! Actors: persistent software principals with identity, capabilities and lifecycle.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    ActorInstanceId, ActorTemplateId, HarnessBindingId, IdentityId, PrincipalId,
    RepresentationContractId, RoleSlotId, RuntimeBindingId, WorkspaceId,
};
use morn_kernel::status::LifecycleStatus;
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// Where a digital actor comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ActorOrigin {
    /// Created from scratch, owned by a person or organization.
    Independent,
    /// Instantiated from a role template + organization rules + workspace.
    RoleDerived,
    /// A controlled digital delegate of a real human.
    HumanDelegated,
}

/// A reusable actor template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorTemplate {
    pub id: ActorTemplateId,
    pub name: String,
    pub version: Version,
    pub origin: ActorOrigin,
    pub description: String,
    pub capabilities: Vec<String>,
    pub model_policy: String,
}

impl ActorTemplate {
    pub fn new(
        name: impl Into<String>,
        origin: ActorOrigin,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: ActorTemplateId::generate_with("actt"),
            name: name.into(),
            version: Version::v1(),
            origin,
            description: description.into(),
            capabilities: Vec::new(),
            model_policy: "default".to_string(),
        }
    }
}

/// An instantiated actor bound to identity, workspace, harness and roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorInstance {
    pub id: ActorInstanceId,
    pub template_id: ActorTemplateId,
    pub identity_id: IdentityId,
    pub actor_origin: ActorOrigin,
    pub owner: Option<PrincipalId>,
    pub sponsor: Option<PrincipalId>,
    pub harness_binding: Option<HarnessBindingId>,
    pub runtime_binding: Option<RuntimeBindingId>,
    pub representation_contract: Option<RepresentationContractId>,
    pub capabilities: Vec<String>,
    pub workspace_bindings: Vec<WorkspaceId>,
    pub role_bindings: Vec<RoleSlotId>,
    pub status: LifecycleStatus,
    pub created_at: Timestamp,
}

impl ActorInstance {
    pub fn new(
        template_id: ActorTemplateId,
        identity_id: IdentityId,
        actor_origin: ActorOrigin,
    ) -> Self {
        Self {
            id: ActorInstanceId::generate_with("acti"),
            template_id,
            identity_id,
            actor_origin,
            owner: None,
            sponsor: None,
            harness_binding: None,
            runtime_binding: None,
            representation_contract: None,
            capabilities: Vec::new(),
            workspace_bindings: Vec::new(),
            role_bindings: Vec::new(),
            status: LifecycleStatus::Active,
            created_at: Timestamp::now(),
        }
    }

    pub fn bind_workspace(mut self, workspace_id: WorkspaceId) -> Self {
        self.workspace_bindings.push(workspace_id);
        self
    }

    pub fn bind_role(mut self, role_slot_id: RoleSlotId) -> Self {
        self.role_bindings.push(role_slot_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_is_distinct_from_role_and_identity() {
        let tmpl = ActorTemplate::new(
            "analyst",
            ActorOrigin::RoleDerived,
            "bioinformatics analyst",
        );
        let identity_id = IdentityId::generate();
        let instance = ActorInstance::new(
            tmpl.id.clone(),
            identity_id.clone(),
            ActorOrigin::RoleDerived,
        );
        // An actor instance has its own id, distinct from its identity.
        assert_ne!(instance.id.as_str(), identity_id.as_str());
        assert_eq!(instance.template_id, tmpl.id);
        assert_eq!(instance.actor_origin, ActorOrigin::RoleDerived);
    }
}
