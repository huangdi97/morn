//! Application state: one shared backend for Workbench/Studio/Console/Hub.

use std::sync::{Arc, Mutex};

use morn_biolab::service::{BioLabService, E2eResult};
use morn_evolution::engine::EvolutionEngine;
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode, MornNativeHarness};
use morn_kernel::ids::WorkspaceId;
use morn_kernel::workspace::{Workspace, WorkspaceKind};
use morn_store::store::MornStore;
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
            e2e_result: None,
        };
        Ok(Self(Arc::new(Mutex::new(inner))))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, AppInner> {
        self.0.lock().expect("app state poisoned")
    }
}
