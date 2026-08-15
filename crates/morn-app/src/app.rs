//! Application state: one shared backend for Workbench/Studio/Console/Hub.

use std::sync::{Arc, Mutex};

use morn_assurance::certification::CertificationService;
use morn_assurance::evaluation::EvaluationRunner;
use morn_assurance::managed_work::ManagedWorkService;
use morn_assurance::replacement::ReplacementPilot;
use morn_assurance::replay::ReplayRunner;
use morn_assurance::shadow::ShadowRunner;
use morn_biolab::dream_factory::{LoopAResult, LoopCResult};
use morn_biolab::service::{BioLabService, E2eResult};
use morn_evolution::distillation::DistillationService;
use morn_evolution::engine::EvolutionEngine;
use morn_evolution::flywheel::EvolutionFlywheel;
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
    pub certification: CertificationService,
    pub managed: ManagedWorkService,
    pub replacement: ReplacementPilot,
    pub flywheel: EvolutionFlywheel,
    pub distillation: DistillationService,
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
        // Reuse the persisted workspace on restart so workspace-scoped records hydrate.
        let workspace = store
            .list_workspaces()?
            .into_iter()
            .next()
            .unwrap_or_else(|| {
                Workspace::new(
                    "Aging Lab Pilot",
                    WorkspaceKind::Lab,
                    morn_kernel::ids::PrincipalId::generate_with("lab-owner"),
                )
            });
        let workspace_id: WorkspaceId = workspace.id.clone();
        store.save_workspace(&workspace)?;
        let biolab = BioLabService::new(workspace_id);
        let mut inner = AppInner {
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
            certification: CertificationService::new(),
            managed: ManagedWorkService::new(),
            replacement: ReplacementPilot::new(),
            flywheel: EvolutionFlywheel::new(),
            distillation: DistillationService::new(),
            last_problem: None,
            last_proposed: None,
            last_approved: None,
            last_package: None,
            loop_a_result: None,
            loop_c_result: None,
            e2e_result: None,
        };
        inner.hydrate_all()?;
        Ok(Self(Arc::new(Mutex::new(inner))))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, AppInner> {
        self.0.lock().expect("app state poisoned")
    }
}

impl AppInner {
    /// Write-through persistence: persist all Goal 3/4 canonical records to the
    /// one canonical store. Called after every mutating operation.
    pub fn persist_all(&self) -> morn_kernel::Result<()> {
        let store = &self.store;
        for s in &self.certification.runs {
            store.save_certification_run(s)?;
        }
        for d in &self.certification.decisions {
            persist_immutable(
                store,
                "certification_decision",
                d.id.as_str(),
                d.created_at.millis(),
                d,
            )?;
        }
        for c in &self.certification.capabilities {
            store.save_certified_capability(c)?;
        }
        for r in &self.certification.releases {
            persist_immutable(
                store,
                "capability_release",
                r.id.as_str(),
                r.released_at.millis(),
                r,
            )?;
        }
        for r in &self.managed.runs {
            store.save_managed_run(r)?;
        }
        for r in &self.managed.receipts {
            persist_immutable(store, "delivery_receipt", r.id.as_str(), 0, r)?;
        }
        for a in &self.managed.acceptances {
            persist_immutable(
                store,
                "acceptance_decision",
                a.id.as_str(),
                a.created_at.millis(),
                a,
            )?;
        }
        for c in &self.replacement.comparisons {
            store.save_replacement_comparison(c)?;
        }
        for r in &self.replacement.records {
            persist_immutable(
                store,
                "replacement_record",
                r.id.as_str(),
                r.created_at.millis(),
                r,
            )?;
        }
        for c in &self.replacement.candidates {
            store.save_r4_candidate(c)?;
        }
        for p in &self.flywheel.patterns {
            store.save_flywheel_pattern(p)?;
        }
        for c in &self.flywheel.candidates {
            store.save_flywheel_candidate(c)?;
        }
        for t in &self.flywheel.traces {
            store.save_trace_record(t)?;
        }
        for c in &self.distillation.candidates {
            store.save_distillation_candidate(c)?;
        }
        Ok(())
    }

    /// Restart hydration: rebuild Goal 3 service state from the canonical store.
    pub fn hydrate_all(&mut self) -> morn_kernel::Result<()> {
        let store = &self.store;
        self.certification.runs = store.load_certification_runs()?;
        self.certification.decisions = store.load_certification_decisions()?;
        self.certification.capabilities = store.load_certified_capabilities()?;
        self.certification.releases = store.load_capability_releases()?;
        self.managed.runs = store.load_managed_runs(&self.workspace.id)?;
        self.managed.receipts = store.load_delivery_receipts()?;
        self.managed.acceptances = store.load_acceptance_decisions()?;
        self.replacement.comparisons = store.load_replacement_comparisons()?;
        self.replacement.records = store.load_replacement_records()?;
        self.replacement.candidates = store.load_r4_candidates()?;
        self.flywheel.patterns = store.load_flywheel_patterns()?;
        self.flywheel.candidates = store.load_flywheel_candidates(&self.workspace.id)?;
        self.flywheel.traces = store.load_trace_records(&self.workspace.id)?;
        self.distillation.candidates = store.load_distillation_candidates()?;
        Ok(())
    }
}
/// Persist an immutable record, treating "already exists" as idempotent no-op.
fn persist_immutable<T: serde::Serialize>(
    store: &morn_store::store::MornStore,
    kind: &str,
    id: &str,
    created_at: i64,
    record: &T,
) -> morn_kernel::Result<()> {
    match store.save_record_immutable(kind, id, "", created_at, record) {
        Ok(()) => Ok(()),
        Err(morn_kernel::error::Error::Conflict(_)) => Ok(()),
        Err(e) => Err(e),
    }
}
