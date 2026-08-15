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
