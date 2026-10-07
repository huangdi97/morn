//! Solution objects: ProposedSolution, ValidationReport, ApprovedSolution, SolutionPackage.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use morn_kernel::ids::{
    ApprovedSolutionId, CapabilityRequirementId, EvaluationPlanId, HarnessPlanId, MemberTypePlanId,
    ProblemSpecId, ProposedSolutionId, RuntimeProfileId, SolutionPackageId, WorkGraphId,
    WorkPackageId,
};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityResolution {
    pub requirement_id: CapabilityRequirementId,
    pub capability: String,
    pub status: String, // Resolved | PartiallyResolved | Missing | Incompatible | Restricted
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGap {
    pub requirement: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberTypePlan {
    pub id: MemberTypePlanId,
    pub role_slot: String,
    pub accepted_member_types: Vec<String>,
    pub recommendation: String,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessPlan {
    pub id: HarnessPlanId,
    pub work_package: String,
    pub harness_spec: String,
    pub runtime_profile: RuntimeProfileId,
    pub fallbacks: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationPlan {
    pub id: EvaluationPlanId,
    pub work_package: String,
    pub acceptance: Vec<String>,
    pub policy_checks: Vec<String>,
    pub failure_modes: Vec<String>,
    pub reviewers: Vec<String>,
    pub regression_baseline: Option<String>,
    pub evidence_requirements: Vec<String>,
}

/// A proposed solution: fully explainable, reviewable, not deployed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedSolution {
    pub id: ProposedSolutionId,
    pub problem_spec_id: ProblemSpecId,
    pub work_graph_id: WorkGraphId,
    pub work_packages: Vec<WorkPackageId>,
    pub member_type_plans: Vec<MemberTypePlan>,
    pub capability_resolutions: Vec<CapabilityResolution>,
    pub capability_gaps: Vec<CapabilityGap>,
    pub harness_plans: Vec<HarnessPlan>,
    pub evaluation_plans: Vec<EvaluationPlan>,
    pub assumptions: Vec<String>,
    pub decision_sources: Vec<String>,
    pub unresolved_gaps: Vec<String>,
    pub risk_summary: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub passed: bool,
}

impl ValidationReport {
    pub fn ok() -> Self {
        Self {
            issues: Vec::new(),
            passed: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedSolution {
    pub id: ApprovedSolutionId,
    pub proposed_solution_id: ProposedSolutionId,
    pub approved_by: String,
    pub approved_at: Timestamp,
}

/// A compiled, machine-readable SolutionPackage (Work System as Code).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionPackage {
    pub id: SolutionPackageId,
    pub name: String,
    pub version: Version,
    pub manifest: Value,
    pub proposed_solution_id: ProposedSolutionId,
    pub approved_solution_id: Option<ApprovedSolutionId>,
    pub created_at: Timestamp,
}


/// Typed view over the v11.5 fields embedded in SolutionPackage.manifest.
///
/// The persisted SolutionPackage keeps its historical JSON manifest for backward
/// compatibility. New control-plane code uses this typed view to prevent a
/// reviewed package from being silently instantiated under a different profile
/// or site scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionPackagePolicyV115 {
    pub schema: String,
    pub profile_ref: Option<String>,
    pub site_ref: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub harness_policy: String,
    pub production_write_allowed: bool,
}

impl SolutionPackage {
    pub fn policy_v115(&self) -> Option<SolutionPackagePolicyV115> {
        let schema = self.manifest.get("schema")?.as_str()?.to_string();
        if schema != "morn.solution-package/v11.5" {
            return None;
        }
        let profile_ref = self
            .manifest
            .get("profile_ref")
            .and_then(Value::as_str)
            .map(str::to_string);
        let site_ref = self
            .manifest
            .get("site_ref")
            .and_then(Value::as_str)
            .map(str::to_string);
        let acceptance_criteria = self
            .manifest
            .get("acceptance_criteria")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let required_capabilities = self
            .manifest
            .get("required_capabilities")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let harness_policy = self
            .manifest
            .get("harness_policy")
            .and_then(Value::as_str)
            .unwrap_or("provider-neutral")
            .to_string();
        let production_write_allowed = self
            .manifest
            .get("production_write_allowed")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        Some(SolutionPackagePolicyV115 {
            schema,
            profile_ref,
            site_ref,
            acceptance_criteria,
            required_capabilities,
            harness_policy,
            production_write_allowed,
        })
    }
}
