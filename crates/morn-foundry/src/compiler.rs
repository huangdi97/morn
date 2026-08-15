//! SolutionCompiler: analyze -> propose -> validate -> compile.
//! Compiler only produces reviewable proposals; it never auto-deploys production.

use serde::{Deserialize, Serialize};
use serde_json::json;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    ApprovedSolutionId, CapabilityRequirementId, EvaluationPlanId, HarnessPlanId, MemberTypePlanId,
    ProposedSolutionId, SolutionPackageId,
};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;
use morn_work::acceptance::AcceptanceSpec;
use morn_work::execution_mode::{ExecutionMode, ExecutorType, WorkNature as ExecNature};
use morn_work::work_package::WorkPackage;

use crate::problem_spec::{ProblemSpec, ProblemSpecBuilder, SolutionRequest};
use crate::solution::{
    ApprovedSolution, CapabilityGap, CapabilityResolution, EvaluationPlan, HarnessPlan,
    MemberTypePlan, ProposedSolution, SolutionPackage, ValidationIssue, ValidationReport,
};
use crate::work_graph::{WorkEdgeKind, WorkGraph, WorkNature, WorkNode};

/// Explainability metadata for every planner decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerDecisionSource {
    pub dimension: String,
    pub target: String,
    pub source_facts: String,
    pub rules_or_templates: String,
    pub model_provider: String,
    pub assumptions: Vec<String>,
    pub confidence: String,
    pub alternatives: Vec<String>,
    pub human_review_status: String,
}

/// The organization/solution compiler (rule-based reference planner).
#[derive(Debug, Default)]
pub struct SolutionCompiler {
    pub decision_sources: Vec<CompilerDecisionSource>,
}

impl SolutionCompiler {
    pub fn new() -> Self {
        Self::default()
    }

    fn record_decision(
        &mut self,
        dimension: &str,
        target: &str,
        source_facts: &str,
        rules_or_templates: &str,
        assumptions: Vec<String>,
        alternatives: Vec<String>,
    ) {
        self.decision_sources.push(CompilerDecisionSource {
            dimension: dimension.to_string(),
            target: target.to_string(),
            source_facts: source_facts.to_string(),
            rules_or_templates: rules_or_templates.to_string(),
            model_provider: "rule-based (no LLM)".to_string(),
            assumptions,
            confidence: "high".to_string(),
            alternatives,
            human_review_status: "pending".to_string(),
        });
    }

    // ---- M3: analyze ----

    pub fn analyze(&mut self, request: &SolutionRequest) -> Result<(ProblemSpec, WorkGraph)> {
        let problem = ProblemSpecBuilder::build(request)?;
        let mut graph = WorkGraph::new(problem.id.clone());
        let nodes = self.decompose_nodes(request);
        for node in nodes {
            graph.add_node(node);
        }
        // Temporal/data edges in node order.
        let ids: Vec<morn_kernel::ids::WorkNodeId> =
            graph.nodes.iter().map(|n| n.id.clone()).collect();
        for pair in ids.windows(2) {
            graph.add_edge(crate::work_graph::WorkEdge::new(
                pair[0].clone(),
                pair[1].clone(),
                WorkEdgeKind::Data,
                "output->input",
            ));
        }
        graph.validate()?;
        self.record_decision(
            "work_decomposition",
            &request.goal,
            &format!(
                "domain={}; constraints={}",
                request.domain,
                request.constraints.len()
            ),
            "nature-keyword decomposition template",
            vec!["LLM may propose candidates; structure validated here".to_string()],
            vec!["alternative decomposition shapes".to_string()],
        );
        Ok((problem, graph))
    }

    fn decompose_nodes(&self, request: &SolutionRequest) -> Vec<WorkNode> {
        let goal = request.goal.to_lowercase();
        let mut nodes = Vec::new();
        let mut push = |name: &str,
                        objective: &str,
                        nature: WorkNature,
                        inputs: Vec<&str>,
                        outputs: Vec<&str>,
                        acceptance: Vec<&str>| {
            nodes.push(
                WorkNode::new(name, objective, nature)
                    .with_inputs(inputs.into_iter().map(String::from).collect())
                    .with_outputs(outputs.into_iter().map(String::from).collect())
                    .with_acceptance(acceptance.into_iter().map(String::from).collect()),
            );
        };
        // Domain-aware template: biolab-like flows get evidence/review/approval nodes.
        if goal.contains("claim")
            || goal.contains("hypothesis")
            || request.domain.contains("biolab")
        {
            push(
                "evidence",
                "collect and lock evidence",
                WorkNature::Deterministic,
                vec!["dataset"],
                vec!["evidence"],
                vec!["evidence locked/versioned"],
            );
            push(
                "analysis",
                "run analysis and produce artifact",
                WorkNature::Probabilistic,
                vec!["evidence"],
                vec!["analysis artifact"],
                vec!["artifact version exists"],
            );
            push(
                "review",
                "independent review",
                WorkNature::Regulated,
                vec!["analysis artifact"],
                vec!["review decision"],
                vec!["reviewer decision recorded"],
            );
            push(
                "approval",
                "human approval",
                WorkNature::Regulated,
                vec!["review decision"],
                vec!["approval"],
                vec!["human approval recorded"],
            );
            push(
                "release",
                "release claim/result",
                WorkNature::Regulated,
                vec!["approval"],
                vec!["released outcome"],
                vec!["claim linked to evidence/artifact/decision"],
            );
            return nodes;
        }
        if goal.contains("report") || goal.contains("transform") || goal.contains("convert") {
            push(
                "transform",
                "transform input deterministically",
                WorkNature::Deterministic,
                vec!["input"],
                vec!["output"],
                vec!["output schema valid"],
            );
            push(
                "verify",
                "verify output",
                WorkNature::Deterministic,
                vec!["output"],
                vec!["verification"],
                vec!["verification passed"],
            );
            return nodes;
        }
        // Generic fallback.
        push(
            "collect",
            "collect inputs",
            WorkNature::Deterministic,
            vec!["input"],
            vec!["locked inputs"],
            vec!["inputs locked"],
        );
        push(
            "analyze",
            "analyze inputs",
            WorkNature::Probabilistic,
            vec!["locked inputs"],
            vec!["draft output"],
            vec!["draft output exists"],
        );
        push(
            "review",
            "review output",
            WorkNature::Regulated,
            vec!["draft output"],
            vec!["reviewed output"],
            vec!["review recorded"],
        );
        nodes
    }

    // ---- M4: propose ----

    pub fn propose(
        &mut self,
        problem: &ProblemSpec,
        graph: &WorkGraph,
        request: &SolutionRequest,
    ) -> Result<ProposedSolution> {
        let mut work_packages = Vec::new();
        let mut member_plans = Vec::new();
        let mut capability_resolutions = Vec::new();
        let mut capability_gaps = Vec::new();
        let mut harness_plans = Vec::new();
        let mut evaluation_plans = Vec::new();
        let mut assumptions = Vec::new();

        for node in &graph.nodes {
            let (exec_nature, executor, rationale) = plan_execution(node.nature);
            let wp = WorkPackage::new(
                request.workspace_id.clone(),
                node.objective.clone(),
                morn_kernel::ids::PrincipalId::generate_with("owner"),
            )
            .with_acceptance_spec(AcceptanceSpec::new(&node.name).id.clone())
            .with_execution_mode(ExecutionMode::hybrid(vec![exec_nature], vec![executor]));
            let wp_id = wp.id.clone();
            work_packages.push(wp_id.clone());
            assumptions.push(format!(
                "work package {} accepts: {}",
                node.name,
                node.acceptance.join("; ")
            ));

            self.record_decision(
                "execution_mode",
                &node.name,
                &format!("nature={:?}", node.nature),
                "execution mode planner: deterministic->program, probabilistic->actor, regulated/physical->human/device",
                vec![],
                vec!["alternate executor".to_string()],
            );

            let member_plan = MemberTypePlan {
                id: MemberTypePlanId::generate_with("mtp"),
                role_slot: format!("slot_{}", node.name),
                accepted_member_types: match node.nature {
                    WorkNature::Deterministic => {
                        vec!["program".to_string(), "deterministic_worker".to_string()]
                    }
                    WorkNature::Physical => vec!["human".to_string(), "device".to_string()],
                    WorkNature::Regulated => vec!["human".to_string(), "actor".to_string()],
                    _ => vec!["actor".to_string(), "human".to_string()],
                },
                recommendation: format!("{:?}", node.nature),
                alternatives: vec!["human".to_string()],
            };
            member_plans.push(member_plan);

            // Capability resolution against request.available_capabilities.
            for output in &node.outputs {
                let available = request
                    .available_capabilities
                    .iter()
                    .any(|c| c.to_lowercase().contains(&output.to_lowercase()) || c == "*");
                if !available {
                    capability_gaps.push(CapabilityGap {
                        requirement: format!("capability for output {output}"),
                        detail: format!(
                            "no available capability provides {output}; do not invent a tool"
                        ),
                    });
                }
                capability_resolutions.push(CapabilityResolution {
                    requirement_id: CapabilityRequirementId::generate_with("req"),
                    capability: output.clone(),
                    status: if available {
                        "Resolved".to_string()
                    } else {
                        "Missing".to_string()
                    },
                });
            }

            let harness = if request.available_harnesses.is_empty() {
                "morn-native".to_string()
            } else {
                request.available_harnesses[0].clone()
            };
            harness_plans.push(HarnessPlan {
                id: HarnessPlanId::generate_with("hp"),
                work_package: wp_id.to_string(),
                harness_spec: harness,
                runtime_profile: morn_kernel::ids::RuntimeProfileId::generate_with("rp"),
                fallbacks: vec!["morn-native".to_string()],
                rationale,
            });

            evaluation_plans.push(EvaluationPlan {
                id: EvaluationPlanId::generate_with("evp"),
                work_package: wp_id.to_string(),
                acceptance: node.acceptance.clone(),
                policy_checks: vec!["irreversible action requires approval".to_string()],
                failure_modes: vec!["tool failure".to_string(), "approval missing".to_string()],
                reviewers: vec!["independent_reviewer".to_string()],
                regression_baseline: None,
                evidence_requirements: vec![
                    "execution receipt".to_string(),
                    "artifact version".to_string(),
                ],
            });
        }

        let proposed = ProposedSolution {
            id: ProposedSolutionId::generate_with("sol"),
            problem_spec_id: problem.id.clone(),
            work_graph_id: graph.id.clone(),
            work_packages,
            member_type_plans: member_plans,
            capability_resolutions,
            capability_gaps: capability_gaps.clone(),
            harness_plans,
            evaluation_plans,
            assumptions,
            decision_sources: self
                .decision_sources
                .iter()
                .map(|d| d.target.clone())
                .collect(),
            unresolved_gaps: capability_gaps
                .iter()
                .map(|g| g.requirement.clone())
                .collect(),
            risk_summary: format!("risk_profile: {}", request.risk_profile),
            created_at: Timestamp::now(),
        };
        Ok(proposed)
    }

    // ---- M4: validate ----

    pub fn validate(
        &self,
        proposed: &ProposedSolution,
        graph: &WorkGraph,
        request: &SolutionRequest,
    ) -> Result<ValidationReport> {
        let mut issues = Vec::new();
        if proposed.work_packages.is_empty() {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                message: "proposed solution has no work packages".to_string(),
            });
        }
        for gap in &proposed.capability_gaps {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                message: format!("capability gap: {}", gap.detail),
            });
        }
        // Harness compatibility: a required harness must be available or a fallback.
        for constraint in &request.constraints {
            if let Some(required) = constraint.strip_prefix("harness=") {
                let ok = request.available_harnesses.iter().any(|h| h == required)
                    || proposed
                        .harness_plans
                        .iter()
                        .any(|p| p.fallbacks.iter().any(|f| f == required));
                if !ok {
                    issues.push(ValidationIssue {
                        severity: "error".to_string(),
                        message: format!(
                            "required harness {required} is not available and not a fallback"
                        ),
                    });
                }
            }
        }
        // Regulated/physical nodes must have human approval in their evaluation plan.
        for node in &graph.nodes {
            if matches!(node.nature, WorkNature::Regulated | WorkNature::Physical) {
                let has_approval_check = proposed
                    .evaluation_plans
                    .iter()
                    .any(|e| e.policy_checks.iter().any(|c| c.contains("approval")));
                if !has_approval_check {
                    issues.push(ValidationIssue {
                        severity: "error".to_string(),
                        message: format!(
                            "node {} is regulated/physical but no approval gate in evaluation plan",
                            node.name
                        ),
                    });
                }
            }
        }
        Ok(ValidationReport {
            passed: issues.iter().all(|i| i.severity != "error"),
            issues,
        })
    }

    pub fn approve(
        &self,
        proposed_solution_id: ProposedSolutionId,
        approver: &str,
    ) -> ApprovedSolution {
        ApprovedSolution {
            id: ApprovedSolutionId::generate_with("apprsol"),
            proposed_solution_id,
            approved_by: approver.to_string(),
            approved_at: Timestamp::now(),
        }
    }

    // ---- M4/M5: compile ----

    pub fn compile(
        &self,
        approved: &ApprovedSolution,
        proposed: &ProposedSolution,
        problem: &ProblemSpec,
    ) -> Result<SolutionPackage> {
        if approved.proposed_solution_id != proposed.id {
            return Err(Error::validation(
                "approved solution does not match proposed solution",
            ));
        }
        if !proposed.capability_gaps.is_empty() {
            return Err(Error::validation(
                "cannot compile: unresolved capability gaps exist; resolve or explicitly accept them",
            ));
        }
        let manifest = json!({
            "morn": { "goal2": "solution-factory", "domain": problem.domain, "version": "1.0" },
            "problem": {
                "objective": problem.objective,
                "success_definition": problem.success_definition,
                "constraints": problem.constraints.iter().map(|c| c.value.clone()).collect::<Vec<_>>(),
            },
            "work_packages": proposed.work_packages.iter().map(|w| w.to_string()).collect::<Vec<_>>(),
            "member_type_plans": proposed.member_type_plans.iter().map(|m| json!({
                "role_slot": m.role_slot,
                "recommendation": m.recommendation,
                "accepted_member_types": m.accepted_member_types,
            })).collect::<Vec<_>>(),
            "capability_resolutions": proposed.capability_resolutions.iter().map(|c| json!({
                "capability": c.capability,
                "status": c.status,
            })).collect::<Vec<_>>(),
            "harness_plans": proposed.harness_plans.iter().map(|h| json!({
                "work_package": h.work_package,
                "harness_spec": h.harness_spec,
                "fallbacks": h.fallbacks,
            })).collect::<Vec<_>>(),
            "evaluation_plans": proposed.evaluation_plans.iter().map(|e| json!({
                "work_package": e.work_package,
                "acceptance": e.acceptance,
            })).collect::<Vec<_>>(),
            "decision_sources": proposed.decision_sources,
            "gaps": proposed.capability_gaps.iter().map(|g| g.requirement.clone()).collect::<Vec<_>>(),
            "risk_summary": proposed.risk_summary,
        });
        Ok(SolutionPackage {
            id: SolutionPackageId::generate_with("solpkg"),
            name: problem.objective.clone(),
            version: Version::v1(),
            manifest,
            proposed_solution_id: proposed.id.clone(),
            approved_solution_id: Some(approved.id.clone()),
            created_at: Timestamp::now(),
        })
    }
}

fn plan_execution(nature: WorkNature) -> (ExecNature, ExecutorType, String) {
    match nature {
        WorkNature::Deterministic => (
            ExecNature::Deterministic,
            ExecutorType::Program,
            "deterministic work prefers program execution".to_string(),
        ),
        WorkNature::Physical => (
            ExecNature::Physical,
            ExecutorType::Human,
            "physical work requires human/device execution".to_string(),
        ),
        WorkNature::Regulated => (
            ExecNature::Regulated,
            ExecutorType::Human,
            "regulated work requires human approval/execution".to_string(),
        ),
        WorkNature::Social => (
            ExecNature::Social,
            ExecutorType::Human,
            "social work requires human interaction".to_string(),
        ),
        WorkNature::Mixed => (
            ExecNature::Mixed,
            ExecutorType::Hybrid,
            "mixed work uses hybrid execution".to_string(),
        ),
        WorkNature::Probabilistic => (
            ExecNature::Probabilistic,
            ExecutorType::Actor,
            "probabilistic analysis uses actor execution".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bio_request(ws: &morn_kernel::ids::WorkspaceId) -> SolutionRequest {
        let mut req =
            SolutionRequest::new(ws.clone(), "Dataset to Reviewed Scientific Claim", "biolab");
        req.constraints
            .push("no unapproved irreversible release".to_string());
        req.available_capabilities.push("analysis".to_string());
        req.available_harnesses.push("morn-native".to_string());
        req
    }

    #[test]
    fn analyze_builds_valid_graph() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let request = bio_request(&ws);
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&request).unwrap();
        assert!(!problem.objective.is_empty());
        assert_eq!(graph.nodes.len(), 5);
        assert!(graph.validate().is_ok());
        assert!(!compiler.decision_sources.is_empty());
    }

    #[test]
    fn deterministic_csv_report_prefers_program_not_actor() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut req = SolutionRequest::new(ws, "Convert CSV to standard report", "generic");
        req.available_capabilities.push("*".to_string());
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&req).unwrap();
        let proposed = compiler.propose(&problem, &graph, &req).unwrap();
        let plan = &proposed.member_type_plans[0];
        assert!(
            plan.accepted_member_types.iter().any(|m| m == "program"),
            "deterministic work must prefer program, not auto-create actor"
        );
    }

    #[test]
    fn capability_gap_is_reported_not_invented() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let request = bio_request(&ws); // only "analysis" capability available
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&request).unwrap();
        let proposed = compiler.propose(&problem, &graph, &request).unwrap();
        let report = compiler.validate(&proposed, &graph, &request).unwrap();
        assert!(report.passed, "warnings only: {:?}", report.issues);
        assert!(
            !proposed.capability_gaps.is_empty(),
            "missing capabilities must produce explicit gaps, not invented tools"
        );
        // compile must fail while gaps exist
        let approved = compiler.approve(proposed.id.clone(), "human");
        assert!(compiler.compile(&approved, &proposed, &problem).is_err());
    }

    #[test]
    fn incompatible_harness_fails_validation() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut req = bio_request(&ws);
        req.available_capabilities.push("*".to_string());
        req.constraints.push("harness=deepseek-harness".to_string());
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&req).unwrap();
        let proposed = compiler.propose(&problem, &graph, &req).unwrap();
        let report = compiler.validate(&proposed, &graph, &req).unwrap();
        assert!(
            !report.passed,
            "incompatible required harness must fail validation"
        );
    }

    #[test]
    fn regulated_node_requires_approval_gate() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut req = bio_request(&ws);
        req.available_capabilities.push("*".to_string());
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&req).unwrap();
        let proposed = compiler.propose(&problem, &graph, &req).unwrap();
        let report = compiler.validate(&proposed, &graph, &req).unwrap();
        assert!(
            report.passed,
            "biolab approval nodes must carry approval gates: {:?}",
            report.issues
        );
    }
    #[test]
    fn compile_requires_approval_and_produces_manifest() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut req = bio_request(&ws);
        req.available_capabilities.push("*".to_string());
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(&req).unwrap();
        let proposed = compiler.propose(&problem, &graph, &req).unwrap();
        let approved = compiler.approve(proposed.id.clone(), "pi");
        let pkg = compiler.compile(&approved, &proposed, &problem).unwrap();
        assert_eq!(pkg.version, Version::v1());
        assert!(pkg.manifest.get("work_packages").is_some());
        // compile without approval for a different solution must fail
        let other = ProposedSolution {
            id: ProposedSolutionId::generate(),
            ..proposed.clone()
        };
        assert!(compiler.compile(&approved, &other, &problem).is_err());
    }
}
