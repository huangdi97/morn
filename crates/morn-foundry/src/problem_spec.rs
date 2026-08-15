//! ProblemSpec: the structured definition of the problem to solve.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ProblemSpecId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// Input to the compiler: a goal/request with constraints and available assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionRequest {
    pub workspace_id: WorkspaceId,
    pub goal: String,
    pub domain: String,
    pub constraints: Vec<String>,
    pub available_members: Vec<String>,
    pub available_capabilities: Vec<String>,
    pub available_harnesses: Vec<String>,
    pub budget: Option<String>,
    pub deadline: Option<String>,
    pub risk_profile: String,
}

impl SolutionRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        goal: impl Into<String>,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id,
            goal: goal.into(),
            domain: domain.into(),
            constraints: Vec::new(),
            available_members: Vec::new(),
            available_capabilities: Vec::new(),
            available_harnesses: Vec::new(),
            budget: None,
            deadline: None,
            risk_profile: "balanced".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    pub name: String,
    pub value: String,
    pub confidence: String,
}

/// Structured problem definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemSpec {
    pub id: ProblemSpecId,
    pub workspace_id: WorkspaceId,
    pub objective: String,
    pub success_definition: String,
    pub domain: String,
    pub world_scope: Vec<String>,
    pub constraints: Vec<Constraint>,
    pub assumptions: Vec<Assumption>,
    pub risks: Vec<String>,
    pub unknowns: Vec<String>,
    pub required_evidence: Vec<String>,
    pub prohibited_conditions: Vec<String>,
    pub created_at: Timestamp,
}

impl ProblemSpec {
    pub fn new(workspace_id: WorkspaceId, objective: impl Into<String>) -> Self {
        Self {
            id: ProblemSpecId::generate_with("pspec"),
            workspace_id,
            objective: objective.into(),
            success_definition: String::new(),
            domain: String::new(),
            world_scope: Vec::new(),
            constraints: Vec::new(),
            assumptions: Vec::new(),
            risks: Vec::new(),
            unknowns: Vec::new(),
            required_evidence: Vec::new(),
            prohibited_conditions: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}

/// Structured, rule-based builder for a ProblemSpec from a SolutionRequest.
/// LLM may only act as a candidate planner; this builder validates structure.
pub struct ProblemSpecBuilder;

impl ProblemSpecBuilder {
    pub fn build(request: &SolutionRequest) -> Result<ProblemSpec> {
        if request.goal.trim().is_empty() {
            return Err(Error::validation("solution request goal must not be empty"));
        }
        if request.domain.trim().is_empty() {
            return Err(Error::validation(
                "solution request domain must not be empty",
            ));
        }
        let mut spec = ProblemSpec::new(request.workspace_id.clone(), request.goal.clone());
        spec.success_definition = format!(
            "goal achieved when all required outputs exist, acceptance specs are satisfied, and prohibited conditions are absent (domain: {})",
            request.domain
        );
        spec.domain = request.domain.clone();
        spec.world_scope = vec!["workspace".to_string(), request.domain.clone()];
        spec.constraints = request
            .constraints
            .iter()
            .map(|c| Constraint {
                name: "user_constraint".to_string(),
                value: c.clone(),
                source: "user_request".to_string(),
            })
            .collect();
        spec.assumptions.push(Assumption {
            name: "available_members".to_string(),
            value: request.available_members.join(","),
            confidence: "fact".to_string(),
        });
        spec.assumptions.push(Assumption {
            name: "available_capabilities".to_string(),
            value: request.available_capabilities.join(","),
            confidence: "fact".to_string(),
        });
        spec.risks
            .push("risk_profile: ".to_string() + &request.risk_profile);
        spec.required_evidence = vec![
            "execution_receipt".to_string(),
            "artifact_version".to_string(),
            "review_or_approval".to_string(),
        ];
        spec.prohibited_conditions = vec![
            "unapproved_irreversible_action".to_string(),
            "artifact_overwrite".to_string(),
        ];
        Ok(spec)
    }
}
