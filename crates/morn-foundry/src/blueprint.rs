//! Reusable blueprints are design-time assets, not new canonical runtime Records.
//!
//! A blueprint describes how to instantiate existing WorkPackage / RoleSlot /
//! Workcell objects. Runtime truth continues to live in the canonical crates.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{PrincipalId, WorkspaceId};
use morn_kernel::status::MemberType;
use morn_kernel::version::Version;
use morn_organization::{RoleSlot, Workcell};
use morn_work::acceptance::AcceptanceSpec;
use morn_work::work_package::WorkPackage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkBlueprint {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub objective_template: String,
    pub required_capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub acceptance_requirements: Vec<String>,
    pub forbidden_conditions: Vec<String>,
    pub profile_ref: Option<String>,
}

impl WorkBlueprint {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty()
            || self.name.trim().is_empty()
            || self.objective_template.trim().is_empty()
        {
            return Err(Error::validation(
                "work blueprint requires id, name and objective template",
            ));
        }
        if self.acceptance_requirements.is_empty() {
            return Err(Error::validation(
                "work blueprint requires explicit acceptance requirements",
            ));
        }
        Ok(())
    }

    pub fn instantiate(
        &self,
        workspace_id: WorkspaceId,
        owner: PrincipalId,
        objective: impl Into<String>,
    ) -> Result<(WorkPackage, AcceptanceSpec)> {
        self.validate()?;
        let objective = objective.into();
        if objective.trim().is_empty() {
            return Err(Error::validation(
                "instantiated Work objective must not be empty",
            ));
        }

        let mut acceptance = AcceptanceSpec::new(format!("{} acceptance", self.name));
        acceptance.required_artifacts = self.acceptance_requirements.clone();
        acceptance.forbidden_conditions = self.forbidden_conditions.clone();

        let work = WorkPackage::new(workspace_id, objective, owner)
            .with_acceptance_spec(acceptance.id.clone());

        Ok((work, acceptance))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleBlueprint {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub responsibilities: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub accepted_member_types: Vec<MemberType>,
    pub authority_ceiling: Vec<String>,
    pub memory_scope: String,
    pub escalation_policy: Vec<String>,
    pub supported_work_types: Vec<String>,
}

impl RoleBlueprint {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(Error::validation("role blueprint requires id and name"));
        }
        if self.responsibilities.is_empty() {
            return Err(Error::validation(
                "role blueprint requires at least one responsibility",
            ));
        }
        if self.accepted_member_types.is_empty() {
            return Err(Error::validation(
                "role blueprint requires at least one accepted member type",
            ));
        }
        Ok(())
    }

    pub fn instantiate_slot(&self, workspace_id: WorkspaceId) -> Result<RoleSlot> {
        self.validate()?;
        let mut slot = RoleSlot::new(
            workspace_id,
            self.name.clone(),
            self.responsibilities.clone(),
            self.accepted_member_types.clone(),
        );
        slot.required_capabilities = self.required_capabilities.clone();
        slot.authority = self.authority_ceiling.clone();
        Ok(slot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkcellTemplate {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub role_blueprints: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub minimum_executor_policy: String,
}

impl WorkcellTemplate {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(Error::validation("workcell template requires id and name"));
        }
        if self.minimum_executor_policy.trim().is_empty() {
            return Err(Error::validation(
                "workcell template requires an explicit minimum-executor policy",
            ));
        }
        Ok(())
    }

    pub fn instantiate(
        &self,
        workspace_id: WorkspaceId,
        work_package_id: morn_kernel::ids::WorkPackageId,
        slots: &[RoleSlot],
    ) -> Result<Workcell> {
        self.validate()?;
        let mut workcell = Workcell::new(workspace_id, self.name.clone(), work_package_id);
        for slot in slots {
            workcell.add_role_slot(slot.id.clone());
        }
        workcell.capabilities = self.required_capabilities.clone();
        Ok(workcell)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintBundle {
    pub work: WorkBlueprint,
    pub roles: Vec<RoleBlueprint>,
    pub workcell: WorkcellTemplate,
}

impl BlueprintBundle {
    pub fn validate(&self) -> Result<()> {
        self.work.validate()?;
        self.workcell.validate()?;
        for role in &self.roles {
            role.validate()?;
        }

        for role_ref in &self.workcell.role_blueprints {
            if !self.roles.iter().any(|role| &role.id == role_ref) {
                return Err(Error::validation(format!(
                    "workcell template references unknown role blueprint {role_ref}"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role() -> RoleBlueprint {
        RoleBlueprint {
            id: "role:exception-coordinator".to_string(),
            name: "Exception coordinator".to_string(),
            version: Version::v1(),
            responsibilities: vec!["coordinate outage review".to_string()],
            required_capabilities: vec!["factory.exception.coordinate".to_string()],
            accepted_member_types: vec![MemberType::Human, MemberType::Actor],
            authority_ceiling: vec!["historian.read".to_string()],
            memory_scope: "work".to_string(),
            escalation_policy: vec!["escalate if delivery commitment changes".to_string()],
            supported_work_types: vec!["factory.exception.review".to_string()],
        }
    }

    #[test]
    fn role_blueprint_is_a_stable_role_interface_not_an_agent() {
        let blueprint = role();
        let slot = blueprint.instantiate_slot(WorkspaceId::generate()).unwrap();
        assert!(slot.accepted_member_types.contains(&MemberType::Human));
        assert!(slot.accepted_member_types.contains(&MemberType::Actor));
        assert_eq!(slot.required_capabilities, blueprint.required_capabilities);
    }

    #[test]
    fn blueprint_bundle_instantiates_existing_runtime_objects() {
        let work_blueprint = WorkBlueprint {
            id: "work:factory-exception-review".to_string(),
            name: "Factory exception review".to_string(),
            version: Version::v1(),
            objective_template: "review outage impact".to_string(),
            required_capabilities: vec!["factory.exception.coordinate".to_string()],
            constraints: vec!["no production write".to_string()],
            acceptance_requirements: vec!["delivery-impact-review".to_string()],
            forbidden_conditions: vec!["unapproved schedule commitment".to_string()],
            profile_ref: Some("morn.factory.readonly@1.0.0".to_string()),
        };
        let role = role();
        let workcell = WorkcellTemplate {
            id: "workcell:factory-exception".to_string(),
            name: "Factory exception workcell".to_string(),
            version: Version::v1(),
            role_blueprints: vec![role.id.clone()],
            required_capabilities: vec!["factory.exception.coordinate".to_string()],
            minimum_executor_policy: "minimum-sufficient".to_string(),
        };
        let bundle = BlueprintBundle {
            work: work_blueprint,
            roles: vec![role],
            workcell,
        };
        bundle.validate().unwrap();

        let ws = WorkspaceId::generate();
        let (work, acceptance) = bundle
            .work
            .instantiate(
                ws.clone(),
                PrincipalId::generate_with("owner"),
                "CNC-17 outage delivery-impact review",
            )
            .unwrap();
        assert_eq!(work.acceptance_spec_id, Some(acceptance.id.clone()));

        let slots: Vec<RoleSlot> = bundle
            .roles
            .iter()
            .map(|role| role.instantiate_slot(ws.clone()).unwrap())
            .collect();
        let workcell = bundle
            .workcell
            .instantiate(ws, work.id.clone(), &slots)
            .unwrap();
        assert_eq!(workcell.role_slots.len(), 1);
        assert_eq!(workcell.capabilities, bundle.workcell.required_capabilities);
    }

    #[test]
    fn missing_role_reference_is_rejected() {
        let bundle = BlueprintBundle {
            work: WorkBlueprint {
                id: "work:x".to_string(),
                name: "x".to_string(),
                version: Version::v1(),
                objective_template: "x".to_string(),
                required_capabilities: vec![],
                constraints: vec![],
                acceptance_requirements: vec!["artifact".to_string()],
                forbidden_conditions: vec![],
                profile_ref: None,
            },
            roles: vec![],
            workcell: WorkcellTemplate {
                id: "wc:x".to_string(),
                name: "x".to_string(),
                version: Version::v1(),
                role_blueprints: vec!["role:missing".to_string()],
                required_capabilities: vec![],
                minimum_executor_policy: "minimum-sufficient".to_string(),
            },
        };
        assert!(bundle.validate().is_err());
    }
}
