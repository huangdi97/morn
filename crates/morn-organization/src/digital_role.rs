//! Non-canonical Digital Employee / role projection.
//!
//! "Digital Employee" is a product/organization view over existing RoleSlot + MemberBinding.
//! It is not a new source of Work truth and is not 1:1 with an agent session.
//! A role can be filled by an Actor, deterministic worker or external service,
//! while concrete Workcells can still compose any minimum-sufficient executors.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{MemberBindingId, RoleSlotId};
use morn_kernel::status::MemberType;

use crate::role::{MemberBinding, RoleSlot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigitalRoleView {
    pub role_slot_id: RoleSlotId,
    pub member_binding_id: MemberBindingId,
    pub role_name: String,
    pub member_ref: String,
    pub member_type: MemberType,
    pub responsibilities: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub authority_ceiling: Vec<String>,
    pub status: String,
    pub canonicalization: String,
}

impl DigitalRoleView {
    pub fn from_role_binding(slot: &RoleSlot, binding: &MemberBinding) -> Result<Self> {
        if binding.role_slot_id != slot.id {
            return Err(Error::validation(
                "member binding does not belong to the supplied role slot",
            ));
        }
        if !slot.accepted_member_types.contains(&binding.member_type) {
            return Err(Error::validation(
                "member type is not admitted by the role slot",
            ));
        }
        if !matches!(
            binding.member_type,
            MemberType::Actor | MemberType::DeterministicWorker | MemberType::ExternalService
        ) {
            return Err(Error::validation(
                "DigitalRoleView represents a digital role member, not a human/device identity",
            ));
        }

        Ok(Self {
            role_slot_id: slot.id.clone(),
            member_binding_id: binding.id.clone(),
            role_name: slot.name.clone(),
            member_ref: binding.member_ref.clone(),
            member_type: binding.member_type,
            responsibilities: slot.responsibilities.clone(),
            required_capabilities: slot.required_capabilities.clone(),
            authority_ceiling: slot.authority.clone(),
            status: binding.status.clone(),
            canonicalization:
                "projection(RoleSlot + MemberBinding); Work/Workcell/ExecutionBinding remain separate"
                    .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::WorkspaceId;

    #[test]
    fn digital_employee_is_role_projection_not_agent_instance_truth() {
        let mut slot = RoleSlot::new(
            WorkspaceId::generate(),
            "production-exception-coordinator",
            vec!["coordinate exception review".to_string()],
            vec![
                MemberType::Actor,
                MemberType::DeterministicWorker,
                MemberType::Human,
            ],
        );
        slot.required_capabilities = vec!["factory.exception.review".to_string()];
        slot.authority = vec!["historian.read".to_string()];

        let binding =
            MemberBinding::new(slot.id.clone(), MemberType::Actor, "actor://coordinator-17");
        let view = DigitalRoleView::from_role_binding(&slot, &binding).unwrap();

        assert_eq!(view.role_slot_id, slot.id);
        assert_eq!(view.member_ref, "actor://coordinator-17");
        assert!(view.canonicalization.contains("projection"));
    }

    #[test]
    fn human_is_not_relabelled_as_digital_employee() {
        let slot = RoleSlot::new(
            WorkspaceId::generate(),
            "planner",
            vec!["approve schedule".to_string()],
            vec![MemberType::Human],
        );
        let binding = MemberBinding::new(slot.id.clone(), MemberType::Human, "human://planner");
        assert!(DigitalRoleView::from_role_binding(&slot, &binding).is_err());
    }
}
