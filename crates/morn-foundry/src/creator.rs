//! Product-facing Creator input.
//!
//! Creator is intentionally a translation layer into the existing
//! ProblemSpec -> WorkGraph -> ProposedSolution -> SolutionPackage pipeline.
//! It is not a new canonical object model and does not create an "agent
//! instance" source of truth.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::WorkspaceId;
use morn_profile::DomainProfile;

use crate::{
    ProblemSpec, ProposedSolution, SolutionCompiler, SolutionRequest, ValidationReport, WorkGraph,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CreatorAutonomy {
    Assist,
    Governed,
    AutonomousWithinPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorRequest {
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub goal: String,
    pub profile_ref: String,
    pub site_ref: Option<String>,
    pub required_capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub acceptance: Vec<String>,
    pub autonomy: CreatorAutonomy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorDraft {
    pub name: String,
    pub profile_ref: String,
    pub site_ref: Option<String>,
    pub autonomy: CreatorAutonomy,
    pub problem: ProblemSpec,
    pub work_graph: WorkGraph,
    pub proposed: ProposedSolution,
    pub validation: ValidationReport,
    pub readiness_gates: Vec<String>,
    pub unresolved: Vec<String>,
    pub execution_started: bool,
    pub canonicalization: String,
}

pub fn draft_creator_solution(request: CreatorRequest) -> Result<CreatorDraft> {
    if request.name.trim().is_empty() || request.goal.trim().is_empty() {
        return Err(Error::validation("creator requires non-empty name and goal"));
    }
    if request.acceptance.is_empty() {
        return Err(Error::validation(
            "creator requires at least one explicit acceptance criterion",
        ));
    }

    let profile = DomainProfile::from_ref(&request.profile_ref)
        .ok_or_else(|| Error::validation("unknown guarantee profile_ref"))?;

    let mut constraints = request.constraints.clone();
    constraints.push(format!("profile={}", profile.canonical_ref()));
    if let Some(site) = &request.site_ref {
        if site.trim().is_empty() {
            return Err(Error::validation("site_ref cannot be blank when supplied"));
        }
        constraints.push(format!("site={site}"));
    }
    if profile.forbids("ProductionWrite") {
        constraints.push("forbid=ProductionWrite".to_string());
    }
    constraints.extend(
        request
            .acceptance
            .iter()
            .map(|criterion| format!("acceptance={criterion}")),
    );

    let mut solution_request =
        SolutionRequest::new(request.workspace_id.clone(), request.goal.clone(), "generic");
    solution_request.constraints = constraints;
    solution_request.available_capabilities = request.required_capabilities.clone();
    solution_request.available_harnesses = vec!["morn-native".to_string()];
    solution_request.risk_profile = match request.autonomy {
        CreatorAutonomy::Assist => "human-led",
        CreatorAutonomy::Governed => "balanced",
        CreatorAutonomy::AutonomousWithinPolicy => "policy-bounded",
    }
    .to_string();

    let mut compiler = SolutionCompiler::new();
    let (problem, work_graph) = compiler.analyze(&solution_request)?;
    let proposed = compiler.propose(&problem, &work_graph, &solution_request)?;
    let validation = compiler.validate(&proposed, &work_graph, &solution_request)?;

    let mut unresolved = proposed.unresolved_gaps.clone();
    unresolved.extend(profile.pre_execution_work_conditions());
    if profile.source_of_truth_binding_required && request.site_ref.is_none() {
        unresolved.push("SiteRefRequiredForSourceOfTruthBinding".to_string());
    }
    unresolved.sort();
    unresolved.dedup();

    Ok(CreatorDraft {
        name: request.name,
        profile_ref: profile.canonical_ref(),
        site_ref: request.site_ref,
        autonomy: request.autonomy,
        problem,
        work_graph,
        proposed,
        validation,
        readiness_gates: profile.pre_execution_work_conditions(),
        unresolved,
        execution_started: false,
        canonicalization:
            "CreatorDraft -> reviewed/approved SolutionPackage -> canonical WorkResource"
                .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creator_is_translation_not_parallel_instance_truth() {
        let draft = draft_creator_solution(CreatorRequest {
            workspace_id: WorkspaceId::generate(),
            name: "factory exception reviewer".to_string(),
            goal: "review an outage and delivery impact".to_string(),
            profile_ref: "morn.factory.readonly@1.0.0".to_string(),
            site_ref: Some("plant-a".to_string()),
            required_capabilities: vec!["*".to_string()],
            constraints: vec![],
            acceptance: vec!["delivery impact review exists".to_string()],
            autonomy: CreatorAutonomy::Governed,
        })
        .unwrap();

        assert!(!draft.execution_started);
        assert!(draft
            .readiness_gates
            .contains(&"CapabilityResolved".to_string()));
        assert!(draft
            .canonicalization
            .contains("SolutionPackage -> canonical WorkResource"));
        assert!(draft
            .problem
            .constraints
            .iter()
            .any(|c| c.value == "forbid=ProductionWrite"));
    }

    #[test]
    fn creator_rejects_unknown_profile_or_missing_acceptance() {
        let base = CreatorRequest {
            workspace_id: WorkspaceId::generate(),
            name: "x".to_string(),
            goal: "do x".to_string(),
            profile_ref: "morn.unknown@1.0.0".to_string(),
            site_ref: None,
            required_capabilities: vec![],
            constraints: vec![],
            acceptance: vec!["x done".to_string()],
            autonomy: CreatorAutonomy::Assist,
        };
        assert!(draft_creator_solution(base.clone()).is_err());

        let mut missing = base;
        missing.profile_ref = "morn.lite@1.0.0".to_string();
        missing.acceptance.clear();
        assert!(draft_creator_solution(missing).is_err());
    }
}
