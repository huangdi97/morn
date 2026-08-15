//! Application state: one shared backend for Workbench/Studio/Console/Hub.

use std::sync::{Arc, Mutex};

use morn_assurance::evaluation::EvaluationRunner;
use morn_assurance::replay::ReplayRunner;
use morn_assurance::shadow::ShadowRunner;
use morn_biolab::dream_factory::{LoopAResult, LoopCResult};
use morn_biolab::service::{BioLabService, E2eResult};
use morn_evolution::engine::EvolutionEngine;
use morn_foundry::compiler::SolutionCompiler;
use morn_foundry::manifest::ManifestService;
use morn_foundry::solution::{ApprovedSolution, ProposedSolution, SolutionPackage};
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode, MornNativeHarness};
use morn_kernel::ids::WorkspaceId;
use morn_kernel::workspace::{Workspace, WorkspaceKind};
use morn_store::store::MornStore;
use morn_work::durable::DurableRuntime;
use morn_work::service::DurableWorkService;

/// The shared application backend state.
pub struct AppInner {
    pub store: MornStore,
    pub workspace: Workspace,
    pub biolab: BioLabService,
    pub native_harness: MornNativeHarness,
    pub dsh_harness: DeepSeekHarnessProvider,
    pub evolution: EvolutionEngine,
    pub durable: DurableWorkService,
    pub durable_v2: DurableRuntime,
    pub compiler: SolutionCompiler,
    pub manifest: ManifestService,
    pub evaluation: EvaluationRunner,
    pub shadow: ShadowRunner,
    pub replay: ReplayRunner,
    pub last_problem: Option<morn_foundry::problem_spec::ProblemSpec>,
    pub last_proposed: Option<ProposedSolution>,
    pub last_approved: Option<ApprovedSolution>,
    pub last_package: Option<SolutionPackage>,
    pub loop_a_result: Option<LoopAResult>,
    pub loop_c_result: Option<LoopCResult>,
    pub e2e_result: Option<E2eResult>,
}

/// Thread-safe shared state for HTTP handlers.
#[derive(Clone)]
pub struct AppState(pub Arc<Mutex<AppInner>>);

impl AppState {
    /// Build a fresh application with a demo BioLab workspace and harness providers.
    pub fn new(db_path: &str) -> morn_kernel::Result<Self> {
        let store = MornStore::open(db_path)?;
        let workspace = Workspace::new(
            "Aging Lab Pilot",
            WorkspaceKind::Lab,
            morn_kernel::ids::PrincipalId::generate_with("lab-owner"),
        );
        let workspace_id: WorkspaceId = workspace.id.clone();
        store.save_workspace(&workspace)?;
        let biolab = BioLabService::new(workspace_id);
        let inner = AppInner {
            store,
            workspace,
            biolab,
            native_harness: MornNativeHarness::new(),
            dsh_harness: DeepSeekHarnessProvider::new(DshMode::Fixture),
            evolution: EvolutionEngine::new(),
            durable: DurableWorkService::new(),
            durable_v2: DurableRuntime::new(),
            compiler: SolutionCompiler::new(),
            manifest: ManifestService::new(),
            evaluation: EvaluationRunner::new(),
            shadow: ShadowRunner::new(),
            replay: ReplayRunner::new(),
            last_problem: None,
            last_proposed: None,
            last_approved: None,
            last_package: None,
            loop_a_result: None,
            loop_c_result: None,
            e2e_result: None,
        };
        Ok(Self(Arc::new(Mutex::new(inner))))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, AppInner> {
        self.0.lock().expect("app state poisoned")
    }
}
