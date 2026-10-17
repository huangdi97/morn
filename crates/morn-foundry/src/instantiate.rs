//! SolutionPackage -> Work instantiation.
//!
//! "Blueprint" is a Studio/product metaphor, not a second canonical object
//! model. A reusable approved SolutionPackage is instantiated as a normal
//! v11.5 WorkResource whose source_solution_ref preserves package provenance.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{WorkPackageId, WorkspaceId};
use morn_profile::DomainProfile;
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

    let v115_policy = package.policy_v115();
    if let Some(policy) = &v115_policy {
        if policy.acceptance_criteria.is_empty() {
            return Err(Error::validation(
                "v11.5 SolutionPackage must preserve at least one reviewed acceptance criterion",
            ));
        }
        if let Some(package_profile) = policy.profile_ref.as_deref() {
            if package_profile != request.profile_ref {
                return Err(Error::validation(format!(
                    "SolutionPackage was reviewed for profile {package_profile}; requested profile {} requires explicit revalidation/migration",
                    request.profile_ref
                )));
            }
        }
        if let Some(package_site) = policy.site_ref.as_deref() {
            if request.site_ref.as_deref() != Some(package_site) {
                return Err(Error::validation(format!(
                    "SolutionPackage is site-scoped to {package_site}; cross-site instantiation requires explicit revalidation"
                )));
            }
        }
        if !policy.production_write_allowed
            && request
                .constraints
                .iter()
                .any(|constraint| constraint == "allow=ProductionWrite")
        {
            return Err(Error::validation(
                "SolutionPackage forbids ProductionWrite; instantiation cannot silently widen effect authority",
            ));
        }
    }

    let package_ref = solution_package_ref(package);
    let canonical_acceptance_ref = v115_policy
        .as_ref()
        .map(|_| format!("{package_ref}#acceptance"));
    if let (Some(expected), Some(requested)) = (
        canonical_acceptance_ref.as_deref(),
        request.acceptance_ref.as_deref(),
    ) {
        if expected != requested {
            return Err(Error::validation(
                "v11.5 Work acceptance_ref must remain bound to the reviewed SolutionPackage acceptance contract",
            ));
        }
    }
    let profile_ref = request.profile_ref.clone();
    let mut spec = WorkSpec::new(
        WorkPackageId::generate_with("work"),
        request.goal,
        profile_ref.clone(),
    );
    spec.source_solution_ref = Some(package_ref.clone());
    spec.site_ref = request.site_ref.clone();
    spec.constraints = request.constraints;
    spec.acceptance_ref = canonical_acceptance_ref.or(request.acceptance_ref);

    // Only pre-execution readiness gates belong here. Binding/receipt/outcome/
    // acceptance conditions are reconciled after Work becomes executable.
    let mut required = DomainProfile::from_ref(&profile_ref)
        .map(|profile| profile.pre_execution_work_conditions())
        .unwrap_or_else(|| vec!["CapabilityResolved".to_string()]);
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
            approved_solution_id: approved
                .then(|| morn_kernel::ids::ApprovedSolutionId::generate_with("approved")),
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
        assert_eq!(plan.work.spec.site_ref.as_deref(), Some("plant-a"));
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
        assert!(!plan
            .work
            .spec
            .required_conditions
            .contains(&"ProfileVersionPinned".to_string()));
        assert_eq!(
            plan.work.status.phase,
            morn_work::control::WorkPhase::Proposed
        );
    }

    #[test]
    fn known_profile_injects_non_bypassable_pre_execution_gates() {
        let pkg = package(true);
        let profile = morn_profile::DomainProfile::factory_readonly_v1();
        let plan = instantiate_approved_solution(
            &pkg,
            SolutionInstantiationRequest {
                workspace_id: WorkspaceId::generate(),
                goal: "review outage".to_string(),
                profile_ref: profile.canonical_ref(),
                site_ref: Some("plant-a".to_string()),
                constraints: vec![],
                acceptance_ref: None,
                required_conditions: vec![],
            },
        )
        .unwrap();

        for required in profile.pre_execution_work_conditions() {
            assert!(
                plan.work.spec.required_conditions.contains(&required),
                "missing profile readiness gate {required}"
            );
        }
    }

    #[test]
    fn v115_package_cannot_be_silently_reprofiled_or_cross_site_instantiated() {
        let mut pkg = package(true);
        pkg.manifest = json!({
            "schema":"morn.solution-package/v11.5",
            "profile_ref":"morn.factory.readonly@1.0.0",
            "site_ref":"plant-a",
            "acceptance_criteria":["delivery-impact-review"],
            "required_capabilities":["capacity.optimize"],
            "harness_policy":"provider-neutral",
            "production_write_allowed":false
        });

        let wrong_profile = instantiate_approved_solution(
            &pkg,
            SolutionInstantiationRequest {
                workspace_id: WorkspaceId::generate(),
                goal: "review outage".to_string(),
                profile_ref: "morn.enterprise@1.0.0".to_string(),
                site_ref: Some("plant-a".to_string()),
                constraints: vec![],
                acceptance_ref: None,
                required_conditions: vec![],
            },
        );
        assert!(wrong_profile.is_err());

        let wrong_site = instantiate_approved_solution(
            &pkg,
            SolutionInstantiationRequest {
                workspace_id: WorkspaceId::generate(),
                goal: "review outage".to_string(),
                profile_ref: "morn.factory.readonly@1.0.0".to_string(),
                site_ref: Some("plant-b".to_string()),
                constraints: vec![],
                acceptance_ref: None,
                required_conditions: vec![],
            },
        );
        assert!(wrong_site.is_err());

        let widened = instantiate_approved_solution(
            &pkg,
            SolutionInstantiationRequest {
                workspace_id: WorkspaceId::generate(),
                goal: "review outage".to_string(),
                profile_ref: "morn.factory.readonly@1.0.0".to_string(),
                site_ref: Some("plant-a".to_string()),
                constraints: vec!["allow=ProductionWrite".to_string()],
                acceptance_ref: None,
                required_conditions: vec![],
            },
        );
        assert!(widened.is_err());
    }

    #[test]
    fn v115_instantiation_pins_acceptance_to_reviewed_package_contract() {
        let mut pkg = package(true);
        pkg.manifest = json!({
            "schema":"morn.solution-package/v11.5",
            "profile_ref":"morn.lite@1.0.0",
            "site_ref":null,
            "acceptance_criteria":["reviewed result exists"],
            "required_capabilities":["draft.generate"],
            "harness_policy":"provider-neutral",
            "production_write_allowed":false
        });
        let package_ref = solution_package_ref(&pkg);
        let request = SolutionInstantiationRequest::new(
            WorkspaceId::generate(),
            "produce reviewed result",
            "morn.lite@1.0.0",
        );
        let plan = instantiate_approved_solution(&pkg, request).unwrap();
        let expected_acceptance_ref = format!("{package_ref}#acceptance");
        assert_eq!(
            plan.work.spec.acceptance_ref.as_deref(),
            Some(expected_acceptance_ref.as_str())
        );

        let mut forged = SolutionInstantiationRequest::new(
            WorkspaceId::generate(),
            "produce reviewed result",
            "morn.lite@1.0.0",
        );
        forged.acceptance_ref = Some("acceptance://different-contract".to_string());
        assert!(instantiate_approved_solution(&pkg, forged).is_err());

        let mut missing = pkg.clone();
        missing.manifest["acceptance_criteria"] = json!([]);
        assert!(instantiate_approved_solution(
            &missing,
            SolutionInstantiationRequest::new(
                WorkspaceId::generate(),
                "produce reviewed result",
                "morn.lite@1.0.0",
            ),
        )
        .is_err());
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
