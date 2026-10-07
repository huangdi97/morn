//! Morn Foundry: ProblemSpec, WorkGraph and the Organization/Solution Compiler.

pub mod artifact2capability;
pub mod compiler;
pub mod instantiate;
pub mod manifest;
pub mod problem_spec;
pub mod solution;
pub mod work_graph;

pub use artifact2capability::{
    ArtifactCompiler, ArtifactKind, ArtifactSource, CandidateCapability, CompilationReport,
    OpenApiJsonCompiler, ProcedureJsonCompiler, RepositoryManifestCompiler,
    ReviewedPaperManifestCompiler,
};
pub use compiler::{CompilerDecisionSource, SolutionCompiler};
pub use instantiate::{
    instantiate_approved_solution, solution_package_ref, InstantiationPlan,
    SolutionInstantiationRequest,
};
pub use manifest::ManifestService;
pub use problem_spec::{Assumption, Constraint, ProblemSpec, ProblemSpecBuilder, SolutionRequest};
pub use solution::{
    ApprovedSolution, ProposedSolution, SolutionPackage, ValidationIssue, ValidationReport,
};
pub use work_graph::{WorkEdge, WorkEdgeKind, WorkGraph, WorkGraphError, WorkNature, WorkNode};
