//! SolutionPackage -> Work instantiation.
//!
//! "Blueprint" is a Studio/product metaphor, not a second canonical object
//! model. A reusable approved SolutionPackage is instantiated as a normal
//! v11.5 WorkResource whose source_solution_ref preserves package provenance.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{WorkPackageId, WorkspaceId};
use morn_work::control::{WorkResource, WorkSpec};

use crate::solution::SolutionPackage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionInstantiationRequest {
    pub workspace_id: WorkspaceId,
    pub goal: String,
    pub profile_ref: String,
    pub site_ref: Option<String>,
    pub constraints: Vec<String>,
    pub acceptance_ref: Option<String>,
    pub required_conditions: Vec<String>,
}

impl SolutionInstantiationRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        goal: impl Into<String>,
        profile_ref: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id,
            goal: goal.into(),
            profile_ref: profile_ref.into(),
            site_ref: None,
            constraints: Vec::new(),
            acceptance_ref: None,
            required_conditions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstantiationPlan {
    pub solution_package_ref: String,
    pub site_ref: Option<String>,
    pub work: WorkResource,
    pub unresolved_gates: Vec<String>,
}

pub fn solution_package_ref(package: &SolutionPackage) -> String {
    format!(
        "solution://{}@{}.{}.{}",
        package.id, package.version.major, package.version.minor, package.version.patch
    )
}

/// Turn an approved reusable solution package into canonical Work.
///
/// This does not resolve capabilities, admit them to a site, grant authority or
/// start a harness. Those remain explicit control-plane gates.
pub fn instantiate_approved_solution(
    package: &SolutionPackage,
    request: SolutionInstantiationRequest,
) -> Result<InstantiationPlan> {
    if package.approved_solution_id.is_none() {
        return Err(Error::invalid_state(
            "SolutionPackage must be approved before instantiation",
        ));
    }
    if request.goal.trim().is_empty() || request.profile_ref.trim().is_empty() {
        return Err(Error::validation(
            "solution instantiation requires non-empty goal and profile",
        ));
    }

    let package_ref = solution_package_ref(package);
    let mut spec = WorkSpec::new(
        WorkPackageId::generate_with("work"),
        request.goal,
        request.profile_ref,
    );
    spec.source_solution_ref = Some(package_ref.clone());
    spec.constraints = request.constraints;
    if let Some(site) = &request.site_ref {
        spec.constraints.push(format!("site={site}"));
    }
    spec.acceptance_ref = request.acceptance_ref;

    let mut required = vec![
        "CapabilityResolved".to_string(),
        "ProfileVersionPinned".to_string(),
        "ProvenanceReady".to_string(),
    ];
    required.extend(request.required_conditions);
    required.sort();
    required.dedup();
    spec.required_conditions = required.clone();

    let work = WorkResource::new(request.workspace_id, spec);
    Ok(InstantiationPlan {
        solution_package_ref: package_ref,
        site_ref: request.site_ref,
        work,
        unresolved_gates: required,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{ProposedSolutionId, SolutionPackageId};
    use morn_kernel::time::Timestamp;
    use morn_kernel::version::Version;
    use serde_json::json;

    fn package(approved: bool) -> SolutionPackage {
        SolutionPackage {
            id: SolutionPackageId::generate_with("solution-package"),
            name: "factory-review".to_string(),
            version: Version::new(1, 2, 0),
            manifest: json!({"kind":"reference"}),
            proposed_solution_id: ProposedSolutionId::generate_with("proposed"),
            approved_solution_id: approved.then(|| {
                morn_kernel::ids::ApprovedSolutionId::generate_with("approved")
            }),
            created_at: Timestamp::now(),
        }
    }

    #[test]
    fn approved_package_becomes_work_not_parallel_instance_truth() {
        let pkg = package(true);
        let plan = instantiate_approved_solution(
            &pkg,
            SolutionInstantiationRequest {
                workspace_id: WorkspaceId::generate(),
                goal: "review outage impact".to_string(),
                profile_ref: "morn.factory.readonly@1.0.0".to_string(),
                site_ref: Some("plant-a".to_string()),
                constraints: vec!["budget<=100".to_string()],
                acceptance_ref: Some("acceptance://delivery-review".to_string()),
                required_conditions: vec![
                    "CapabilityQualified".to_string(),
                    "SourceOfTruthBound".to_string(),
                ],
            },
        )
        .unwrap();

        assert_eq!(
            plan.work.spec.source_solution_ref.as_deref(),
            Some(plan.solution_package_ref.as_str())
        );
        assert!(plan
            .work
            .spec
            .required_conditions
            .contains(&"CapabilityResolved".to_string()));
        assert!(plan
            .work
            .spec
            .required_conditions
            .contains(&"SourceOfTruthBound".to_string()));
        assert_eq!(plan.work.status.phase, morn_work::control::WorkPhase::Proposed);
    }

    #[test]
    fn unapproved_package_cannot_instantiate_work() {
        let pkg = package(false);
        let request = SolutionInstantiationRequest::new(
            WorkspaceId::generate(),
            "do work",
            "morn.lite@1.0.0",
        );
        assert!(instantiate_approved_solution(&pkg, request).is_err());
    }
}
