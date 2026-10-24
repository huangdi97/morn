//! Morn Foundry: ProblemSpec, WorkGraph and the Organization/Solution Compiler.

pub mod artifact2capability;
pub mod blueprint;
pub mod compiler;
pub mod creator;
pub mod instantiate;
pub mod manifest;
pub mod problem_spec;
pub mod solution;
pub mod work_graph;
pub mod workcell_plan;

pub use artifact2capability::{
    ArtifactCompiler, ArtifactKind, ArtifactSource, CandidateCapability, CompilationReport,
    ModelManifestCompiler, OpenApiJsonCompiler, ProcedureJsonCompiler, RepositoryManifestCompiler,
    ReviewedPaperManifestCompiler, WorkflowManifestCompiler,
};
pub use blueprint::{BlueprintBundle, RoleBlueprint, WorkBlueprint, WorkcellTemplate};
pub use compiler::{CompilerDecisionSource, SolutionCompiler};
pub use creator::{draft_creator_solution, CreatorAutonomy, CreatorDraft, CreatorRequest};
pub use instantiate::{
    instantiate_approved_solution, solution_package_ref, InstantiationPlan,
    SolutionInstantiationRequest,
};
pub use manifest::ManifestService;
pub use problem_spec::{Assumption, Constraint, ProblemSpec, ProblemSpecBuilder, SolutionRequest};
pub use solution::{
    ApprovedSolution, ProposedSolution, SolutionPackage, SolutionPackagePolicyV115,
    ValidationIssue, ValidationReport,
};
pub use work_graph::{WorkEdge, WorkEdgeKind, WorkGraph, WorkGraphError, WorkNature, WorkNode};

pub use workcell_plan::materialize_workcell_plan;
