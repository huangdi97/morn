//! Roles, member bindings and responsibility bindings.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{
    MemberBindingId, PrincipalId, ResponsibilityBindingId, RoleSlotId, WorkPackageId, WorkspaceId,
};
use morn_kernel::time::Timestamp;

/// Re-export of kernel member type for convenience.
pub use morn_kernel::status::MemberType;

/// A role slot defines what an organization needs, before choosing a member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleSlot {
    pub id: RoleSlotId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub responsibilities: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub accepted_member_types: Vec<MemberType>,
    pub authority: Vec<String>,
    pub accountable_to: Option<PrincipalId>,
}

impl RoleSlot {
    pub fn new(
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        responsibilities: Vec<String>,
        accepted_member_types: Vec<MemberType>,
    ) -> Self {
        Self {
            id: RoleSlotId::generate_with("role"),
            workspace_id,
            name: name.into(),
            responsibilities,
            required_capabilities: Vec::new(),
            accepted_member_types,
            authority: Vec::new(),
            accountable_to: None,
        }
    }
}

/// A binding of a concrete member (human/actor/worker/service/device) to a role slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberBinding {
    pub id: MemberBindingId,
    pub role_slot_id: RoleSlotId,
    pub member_type: MemberType,
    pub member_ref: String,
    pub status: String,
    pub created_at: Timestamp,
}

impl MemberBinding {
    pub fn new(
        role_slot_id: RoleSlotId,
        member_type: MemberType,
        member_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: MemberBindingId::generate_with("mb"),
            role_slot_id,
            member_type,
            member_ref: member_ref.into(),
            status: "active".to_string(),
            created_at: Timestamp::now(),
        }
    }
}

/// Assign different responsibilities of one work package to different members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponsibilityBinding {
    pub id: ResponsibilityBindingId,
    pub work_package_id: WorkPackageId,
    pub responsibility: String,
    pub member_type: MemberType,
    pub member_ref: String,
}

impl ResponsibilityBinding {
    pub fn new(
        work_package_id: WorkPackageId,
        responsibility: impl Into<String>,
        member_type: MemberType,
        member_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: ResponsibilityBindingId::generate_with("rb"),
            work_package_id,
            responsibility: responsibility.into(),
            member_type,
            member_ref: member_ref.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_slot_and_member_binding_are_separate() {
        let ws = WorkspaceId::generate();
        let slot = RoleSlot::new(
            ws,
            "statistical-reviewer",
            vec!["independent review".to_string()],
            vec![MemberType::Actor, MemberType::Human],
        );
        // A role slot exists before any member is bound.
        assert!(slot.accepted_member_types.contains(&MemberType::Human));
        let binding = MemberBinding::new(slot.id.clone(), MemberType::Human, "pi-1");
        assert_eq!(binding.role_slot_id, slot.id);
        assert_eq!(binding.member_type, MemberType::Human);
    }
}
