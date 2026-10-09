//! Application state: one shared backend for Workbench/Studio/Console/Hub.

use std::sync::{Arc, Mutex};

use morn_artifact::service::ArtifactService;
use morn_assurance::certification::CertificationService;
use morn_assurance::evaluation::EvaluationRunner;
use morn_assurance::managed_work::ManagedWorkService;
use morn_assurance::replacement::ReplacementPilot;
use morn_assurance::replay::ReplayRunner;
use morn_assurance::rollback::RollbackService;
use morn_assurance::shadow::ShadowRunner;
use morn_assurance::AdmissionService;
#[cfg(feature = "domain-biolab")]
use morn_biolab_reference::dream_factory::{LoopAResult, LoopCResult};
#[cfg(feature = "domain-biolab")]
use morn_biolab_reference::service::{BioLabService, E2eResult};
use morn_capability::CapabilityRecord;
use morn_evolution::distillation::DistillationService;
use morn_evolution::engine::EvolutionEngine;
use morn_evolution::flywheel::EvolutionFlywheel;
use morn_foundry::compiler::SolutionCompiler;
use morn_foundry::manifest::ManifestService;
use morn_foundry::solution::{ApprovedSolution, ProposedSolution, SolutionPackage};
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode, MornNativeHarness};
use morn_harness::{PiHarnessProvider, PiMode};
use morn_integration::{SourceObservationAttestation, SourceOfTruthBinding};
#[cfg(feature = "domain-biolab")]
use morn_kernel::ids::WorkspaceId;
use morn_kernel::workspace::{Workspace, WorkspaceKind};
use morn_opint::dataset::OutcomeDataset;
use morn_opint::episode::EpisodeAssembler;
use morn_opint::predictor::PredictorRegistry;
use morn_runtime::{AttestedExecutionEnvironmentProvider, ExecutionEnvironmentAttestation};
use morn_store::store::MornStore;
use morn_work::acceptance::{AcceptanceReviewAuthorization, AcceptanceReviewerAttestation};
use morn_work::durable::DurableRuntime;
use morn_work::service::{DurableWorkService, WorkService};
use morn_world::service::WorldService;

/// The shared application backend state.
pub struct AppInner {
    pub store: MornStore,
    pub workspace: Workspace,
    /// Generic (domain-neutral) world/work/artifact services used by the
    /// product surfaces. Domain packs provide their own instances.
    pub world: WorldService,
    pub work: WorkService,
    pub artifacts: ArtifactService,
    #[cfg(feature = "domain-biolab")]
    pub biolab: BioLabService,
    /// Provider runtimes use independent locks. A long model turn must not hold
    /// the global application-state mutex and block unrelated Work/Console reads.
    pub native_harness: Arc<Mutex<MornNativeHarness>>,
    pub dsh_harness: Arc<Mutex<DeepSeekHarnessProvider>>,
    pub pi_harness: Arc<Mutex<PiHarnessProvider>>,
    /// Deployment-owned execution environment attestations. HTTP callers may
    /// select only from this startup-loaded trust set; they cannot self-attest.
    pub execution_environments: AttestedExecutionEnvironmentProvider,
    /// Deployment-owned authoritative read bindings. HTTP callers may attach
    /// these reviewed bindings to Work, but cannot manufacture a new authority.
    pub source_of_truth_catalog: Vec<SourceOfTruthBinding>,
    /// Deployment/connector-originated observations. HTTP callers may select
    /// these immutable attestations but cannot submit world facts themselves.
    pub source_observation_attestations: Vec<SourceObservationAttestation>,
    /// Deployment-attested reviewer identities. UI/API callers may select a
    /// reviewer but cannot self-assert principal identity or reviewer role.
    pub acceptance_reviewers: Vec<AcceptanceReviewerAttestation>,
    /// Deployment-issued, exact Work/Outcome/disposition review authorizations.
    /// IDs are bearer references delivered out-of-band and are never listed by the API.
    pub acceptance_review_authorizations: Vec<AcceptanceReviewAuthorization>,
    pub evolution: EvolutionEngine,
    pub durable: DurableWorkService,
    pub durable_v2: DurableRuntime,
    pub compiler: SolutionCompiler,
    pub manifest: ManifestService,
    pub evaluation: EvaluationRunner,
    pub shadow: ShadowRunner,
    pub replay: ReplayRunner,
    pub certification: CertificationService,
    /// v11.5 capability supply-chain state. These are semantic lifecycle
    /// records, separate from the legacy certification service.
    pub v115_admission: AdmissionService,
    pub v115_capabilities: Vec<CapabilityRecord>,
    pub managed: ManagedWorkService,
    pub replacement: ReplacementPilot,
    pub flywheel: EvolutionFlywheel,
    pub distillation: DistillationService,
    pub rollback: RollbackService,
    pub episodes: EpisodeAssembler,
    pub dataset: OutcomeDataset,
    pub predictors: PredictorRegistry,
    pub last_problem: Option<morn_foundry::problem_spec::ProblemSpec>,
    pub last_proposed: Option<ProposedSolution>,
    pub last_approved: Option<ApprovedSolution>,
    pub last_package: Option<SolutionPackage>,
    #[cfg(feature = "domain-biolab")]
    pub loop_a_result: Option<LoopAResult>,
    #[cfg(feature = "domain-biolab")]
    pub loop_c_result: Option<LoopCResult>,
    #[cfg(feature = "domain-biolab")]
    pub e2e_result: Option<E2eResult>,
}

fn configured_dsh_harness() -> morn_kernel::Result<DeepSeekHarnessProvider> {
    match std::env::var("MORN_DSH_MODE") {
        Err(std::env::VarError::NotPresent) => Ok(DeepSeekHarnessProvider::new(DshMode::Fixture)),
        Ok(mode) if mode.eq_ignore_ascii_case("fixture") => {
            Ok(DeepSeekHarnessProvider::new(DshMode::Fixture))
        }
        Ok(mode) if mode.eq_ignore_ascii_case("real") => DeepSeekHarnessProvider::from_real_env(),
        Ok(mode) => Err(morn_kernel::error::Error::validation(format!(
            "unsupported MORN_DSH_MODE {mode:?}; expected fixture or real"
        ))),
        Err(error) => Err(morn_kernel::error::Error::validation(format!(
            "cannot read MORN_DSH_MODE: {error}"
        ))),
    }
}

fn configured_execution_environments() -> morn_kernel::Result<AttestedExecutionEnvironmentProvider>
{
    let provider_name = std::env::var("MORN_EXECUTION_ATTESTOR")
        .unwrap_or_else(|_| "deployment-attestor".to_string());
    let mut provider = AttestedExecutionEnvironmentProvider::new(provider_name)?;
    let Ok(path) = std::env::var("MORN_EXECUTION_ATTESTATION_FILE") else {
        return Ok(provider);
    };
    if path.trim().is_empty() {
        return Err(morn_kernel::error::Error::validation(
            "MORN_EXECUTION_ATTESTATION_FILE must not be empty when set",
        ));
    }
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        morn_kernel::error::Error::external(format!(
            "cannot read execution attestation file {path:?}: {error}"
        ))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        morn_kernel::error::Error::validation(format!(
            "invalid execution attestation JSON in {path:?}: {error}"
        ))
    })?;
    let attestations: Vec<ExecutionEnvironmentAttestation> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| {
                morn_kernel::error::Error::validation(format!(
                    "invalid execution attestation entry: {error}"
                ))
            })?,
        other => vec![serde_json::from_value(other).map_err(|error| {
            morn_kernel::error::Error::validation(format!(
                "invalid execution attestation entry: {error}"
            ))
        })?],
    };
    for attestation in attestations {
        provider.register_attestation(attestation)?;
    }
    Ok(provider)
}

fn configured_source_of_truth_bindings() -> morn_kernel::Result<Vec<SourceOfTruthBinding>> {
    let Ok(path) = std::env::var("MORN_SOURCE_OF_TRUTH_BINDINGS_FILE") else {
        return Ok(Vec::new());
    };
    if path.trim().is_empty() {
        return Err(morn_kernel::error::Error::validation(
            "MORN_SOURCE_OF_TRUTH_BINDINGS_FILE must not be empty when set",
        ));
    }
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        morn_kernel::error::Error::external(format!(
            "cannot read source-of-truth binding file {path:?}: {error}"
        ))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        morn_kernel::error::Error::validation(format!(
            "invalid source-of-truth binding JSON in {path:?}: {error}"
        ))
    })?;
    let bindings: Vec<SourceOfTruthBinding> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| {
                morn_kernel::error::Error::validation(format!(
                    "invalid source-of-truth binding entry: {error}"
                ))
            })?,
        other => vec![serde_json::from_value(other).map_err(|error| {
            morn_kernel::error::Error::validation(format!(
                "invalid source-of-truth binding entry: {error}"
            ))
        })?],
    };

    let mut ids = std::collections::BTreeSet::new();
    for binding in &bindings {
        binding.validate()?;
        if !ids.insert(binding.id.to_string()) {
            return Err(morn_kernel::error::Error::validation(
                "source-of-truth binding ids must be unique",
            ));
        }
    }
    Ok(bindings)
}

fn configured_source_observation_attestations(
) -> morn_kernel::Result<Vec<SourceObservationAttestation>> {
    let Ok(path) = std::env::var("MORN_SOURCE_OBSERVATION_ATTESTATIONS_FILE") else {
        return Ok(Vec::new());
    };
    if path.trim().is_empty() {
        return Err(morn_kernel::error::Error::validation(
            "MORN_SOURCE_OBSERVATION_ATTESTATIONS_FILE must not be empty when set",
        ));
    }
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        morn_kernel::error::Error::external(format!(
            "cannot read source observation attestation file {path:?}: {error}"
        ))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        morn_kernel::error::Error::validation(format!(
            "invalid source observation attestation JSON in {path:?}: {error}"
        ))
    })?;
    let attestations: Vec<SourceObservationAttestation> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| {
                morn_kernel::error::Error::validation(format!(
                    "invalid source observation attestation entry: {error}"
                ))
            })?,
        other => vec![serde_json::from_value(other).map_err(|error| {
            morn_kernel::error::Error::validation(format!(
                "invalid source observation attestation entry: {error}"
            ))
        })?],
    };
    let mut ids = std::collections::BTreeSet::new();
    for attestation in &attestations {
        attestation.validate()?;
        if !ids.insert(attestation.attestation_id.clone()) {
            return Err(morn_kernel::error::Error::validation(
                "source observation attestation ids must be unique",
            ));
        }
    }
    Ok(attestations)
}

fn configured_acceptance_reviewers() -> morn_kernel::Result<Vec<AcceptanceReviewerAttestation>> {
    let Ok(path) = std::env::var("MORN_ACCEPTANCE_REVIEWERS_FILE") else {
        return Ok(Vec::new());
    };
    if path.trim().is_empty() {
        return Err(morn_kernel::error::Error::validation(
            "MORN_ACCEPTANCE_REVIEWERS_FILE must not be empty when set",
        ));
    }
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        morn_kernel::error::Error::external(format!(
            "cannot read acceptance reviewer file {path:?}: {error}"
        ))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        morn_kernel::error::Error::validation(format!(
            "invalid acceptance reviewer JSON in {path:?}: {error}"
        ))
    })?;
    let reviewers: Vec<AcceptanceReviewerAttestation> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| {
                morn_kernel::error::Error::validation(format!(
                    "invalid acceptance reviewer entry: {error}"
                ))
            })?,
        other => vec![serde_json::from_value(other).map_err(|error| {
            morn_kernel::error::Error::validation(format!(
                "invalid acceptance reviewer entry: {error}"
            ))
        })?],
    };
    let mut principals = std::collections::BTreeSet::new();
    for reviewer in &reviewers {
        reviewer.validate()?;
        if !principals.insert(reviewer.principal_id.to_string()) {
            return Err(morn_kernel::error::Error::validation(
                "acceptance reviewer principal ids must be unique",
            ));
        }
    }
    Ok(reviewers)
}

fn configured_acceptance_review_authorizations(
) -> morn_kernel::Result<Vec<AcceptanceReviewAuthorization>> {
    let Ok(path) = std::env::var("MORN_ACCEPTANCE_REVIEW_AUTHORIZATIONS_FILE") else {
        return Ok(Vec::new());
    };
    if path.trim().is_empty() {
        return Err(morn_kernel::error::Error::validation(
            "MORN_ACCEPTANCE_REVIEW_AUTHORIZATIONS_FILE must not be empty when set",
        ));
    }
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        morn_kernel::error::Error::external(format!(
            "cannot read acceptance review authorization file {path:?}: {error}"
        ))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        morn_kernel::error::Error::validation(format!(
            "invalid acceptance review authorization JSON in {path:?}: {error}"
        ))
    })?;
    let authorizations: Vec<AcceptanceReviewAuthorization> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| {
                morn_kernel::error::Error::validation(format!(
                    "invalid acceptance review authorization entry: {error}"
                ))
            })?,
        other => vec![serde_json::from_value(other).map_err(|error| {
            morn_kernel::error::Error::validation(format!(
                "invalid acceptance review authorization entry: {error}"
            ))
        })?],
    };
    let mut ids = std::collections::BTreeSet::new();
    for authorization in &authorizations {
        authorization.validate()?;
        if !ids.insert(authorization.authorization_id.clone()) {
            return Err(morn_kernel::error::Error::validation(
                "acceptance review authorization ids must be unique",
            ));
        }
    }
    Ok(authorizations)
}

fn configured_pi_harness() -> morn_kernel::Result<PiHarnessProvider> {
    match std::env::var("MORN_PI_MODE") {
        Err(std::env::VarError::NotPresent) => Ok(PiHarnessProvider::new(PiMode::Fixture)),
        Ok(mode) if mode.eq_ignore_ascii_case("fixture") => {
            Ok(PiHarnessProvider::new(PiMode::Fixture))
        }
        Ok(mode) if mode.eq_ignore_ascii_case("real") => PiHarnessProvider::from_real_env(),
        Ok(mode) => Err(morn_kernel::error::Error::validation(format!(
            "unsupported MORN_PI_MODE {mode:?}; expected fixture or real"
        ))),
        Err(error) => Err(morn_kernel::error::Error::validation(format!(
            "cannot read MORN_PI_MODE: {error}"
        ))),
    }
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
        store.save_workspace(&workspace)?;
        #[cfg(feature = "domain-biolab")]
        let biolab = {
            let workspace_id: WorkspaceId = workspace.id.clone();
            BioLabService::new(workspace_id)
        };
        let execution_environments = configured_execution_environments()?;
        let source_of_truth_catalog = configured_source_of_truth_bindings()?;
        let source_observation_attestations = configured_source_observation_attestations()?;
        let acceptance_reviewers = configured_acceptance_reviewers()?;
        let acceptance_review_authorizations = configured_acceptance_review_authorizations()?;
        let mut inner = AppInner {
            store,
            workspace,
            world: WorldService::new(),
            work: WorkService::new(),
            artifacts: ArtifactService::new(),
            #[cfg(feature = "domain-biolab")]
            biolab,
            native_harness: Arc::new(Mutex::new(MornNativeHarness::new())),
            dsh_harness: Arc::new(Mutex::new(configured_dsh_harness()?)),
            pi_harness: Arc::new(Mutex::new(configured_pi_harness()?)),
            execution_environments,
            source_of_truth_catalog,
            source_observation_attestations,
            acceptance_reviewers,
            acceptance_review_authorizations,
            evolution: EvolutionEngine::new(),
            durable: DurableWorkService::new(),
            durable_v2: DurableRuntime::new(),
            compiler: SolutionCompiler::new(),
            manifest: ManifestService::new(),
            evaluation: EvaluationRunner::new(),
            shadow: ShadowRunner::new(),
            replay: ReplayRunner::new(),
            certification: CertificationService::new(),
            v115_admission: AdmissionService::default(),
            v115_capabilities: Vec::new(),
            managed: ManagedWorkService::new(),
            replacement: ReplacementPilot::new(),
            flywheel: EvolutionFlywheel::new(),
            distillation: DistillationService::new(),
            rollback: RollbackService::new(),
            episodes: EpisodeAssembler::new(),
            dataset: OutcomeDataset::new(),
            predictors: PredictorRegistry::new(),
            last_problem: None,
            last_proposed: None,
            last_approved: None,
            last_package: None,
            #[cfg(feature = "domain-biolab")]
            loop_a_result: None,
            #[cfg(feature = "domain-biolab")]
            loop_c_result: None,
            #[cfg(feature = "domain-biolab")]
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
        for capability in &self.v115_capabilities {
            store.save_record(
                "capability_record_v115",
                capability.manifest.id.as_str(),
                self.workspace.id.as_str(),
                capability.manifest.declared_at.millis(),
                capability,
            )?;
        }
        for observation in &self.v115_admission.observations {
            persist_immutable(
                store,
                "capability_observation_v115",
                observation.id.as_str(),
                observation.created_at.millis(),
                observation,
            )?;
        }
        for qualification in &self.v115_admission.qualifications {
            persist_immutable(
                store,
                "qualification_record_v115",
                qualification.id.as_str(),
                qualification.created_at.millis(),
                qualification,
            )?;
        }
        for release in &self.v115_admission.releases {
            // Release bytes/digest are immutable, while lifecycle status is a
            // mutable projection. Revocation history is append-only below.
            store.save_record(
                "capability_distribution_release_v115",
                release.id.as_str(),
                self.workspace.id.as_str(),
                release.created_at.millis(),
                release,
            )?;
        }
        for admission in &self.v115_admission.admissions {
            // Admission status is a current projection; lifecycle events retain
            // the non-destructive history of admission/suspension.
            store.save_record(
                "site_admission_v115",
                admission.id.as_str(),
                self.workspace.id.as_str(),
                admission.created_at.millis(),
                admission,
            )?;
        }
        for event in &self.v115_admission.events {
            persist_immutable(
                store,
                "capability_lifecycle_event_v115",
                event.id.as_str(),
                event.created_at.millis(),
                event,
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
        for r in &self.rollback.requests {
            store.save_rollback_request(r)?;
        }
        for r in &self.rollback.receipts {
            persist_immutable(
                store,
                "rollback_receipt",
                r.id.as_str(),
                r.created_at.millis(),
                r,
            )?;
        }
        for o in self.world.objects() {
            store.save_object(o)?;
        }
        for wp in self.work.work_packages() {
            store.save_work_package(wp)?;
        }
        for a in self.artifacts.all_artifacts() {
            store.save_artifact(a)?;
        }
        let mut artifact_versions: Vec<morn_artifact::artifact::ArtifactVersion> = Vec::new();
        for a in self.artifacts.all_artifacts() {
            artifact_versions.extend(self.artifacts.versions_of(&a.id).into_iter().cloned());
        }
        for v in artifact_versions {
            store.save_artifact_version(&v)?;
        }
        for o in self.world.outcomes() {
            store.save_record(
                "world_outcome",
                o.id.as_str(),
                self.workspace.id.as_str(),
                o.created_at.millis(),
                o,
            )?;
        }
        for e in &self.episodes.episodes {
            store.save_opint_episode(e)?;
        }
        store.save_predictor_states(&self.predictors.snapshot_all())?;
        for p in &self.predictors.predictors {
            for pred in &p.predictions {
                store.save_prediction(pred)?;
            }
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
        self.v115_capabilities = store
            .load_records_in_workspace("capability_record_v115", self.workspace.id.as_str())?;
        let manifest_ids: std::collections::HashSet<String> = self
            .v115_capabilities
            .iter()
            .map(|cap| cap.manifest.id.to_string())
            .collect();
        // Historical v11.5 observations, qualification and lifecycle events
        // were stored as immutable rows with an empty workspace field. Restore
        // them by their owning capability manifest, never by a global read
        // directly into the active tenant's runtime state.
        self.v115_admission.observations = store
            .load_records::<morn_assurance::CapabilityObservation>("capability_observation_v115")?
            .into_iter()
            .filter(|item| manifest_ids.contains(item.manifest_id.as_str()))
            .collect();
        self.v115_admission.qualifications = store
            .load_records::<morn_assurance::QualificationRecord>("qualification_record_v115")?
            .into_iter()
            .filter(|item| manifest_ids.contains(item.manifest_id.as_str()))
            .collect();
        self.v115_admission.releases = store
            .load_records_in_workspace::<morn_assurance::CapabilityDistributionRelease>(
                "capability_distribution_release_v115",
                self.workspace.id.as_str(),
            )?
            .into_iter()
            .filter(|item| manifest_ids.contains(item.manifest_id.as_str()))
            .collect();
        self.v115_admission.admissions = store
            .load_records_in_workspace::<morn_assurance::SiteAdmission>(
                "site_admission_v115",
                self.workspace.id.as_str(),
            )?
            .into_iter()
            .filter(|item| manifest_ids.contains(item.manifest_id.as_str()))
            .collect();
        self.v115_admission.events = store
            .load_records::<morn_assurance::CapabilityLifecycleEvent>(
                "capability_lifecycle_event_v115",
            )?
            .into_iter()
            .filter(|item| manifest_ids.contains(item.manifest_id.as_str()))
            .collect();
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
        self.rollback.requests = store.load_rollback_requests(&self.workspace.id)?;
        self.rollback.receipts = store.load_rollback_receipts()?;
        let objects = store.list_objects(&self.workspace.id)?;
        for o in objects {
            self.world.register_object(o);
        }
        let work_packages = store.list_work_packages(&self.workspace.id)?;
        for wp in work_packages {
            self.work.add_work_package(wp);
        }
        let artifacts = store.load_records::<morn_artifact::artifact::Artifact>("artifact")?;
        self.artifacts.restore_artifacts(artifacts);
        let versions =
            store.load_records::<morn_artifact::artifact::ArtifactVersion>("artifact_version")?;
        self.artifacts.restore_versions(versions);
        let outcomes = store.load_records::<morn_world::outcome::OutcomeRecord>("world_outcome")?;
        self.world.restore_outcomes(outcomes);
        let episodes = store.load_opint_episodes(&self.workspace.id)?;
        self.episodes.episodes = episodes;
        let states = store.load_predictor_states()?;
        self.predictors.restore_all(states);
        let predictions = store.load_predictions()?;
        for state in &mut self.predictors.predictors {
            state.predictions = predictions
                .iter()
                .filter(|p| p.predictor_id == state.spec.id)
                .cloned()
                .collect();
        }
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
