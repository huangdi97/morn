//! End-to-end reference slice for the first Factory read-only wedge.
//!
//! The scenario exercises resolution, qualification/admission, profile
//! conformance, harness neutrality, pinned bindings, ambiguous external action
//! reconciliation, accepted outcome and durable persistence. All external
//! systems are deterministic fixtures; this is not production evidence.

use std::collections::{BTreeMap, BTreeSet};

use morn_assurance::AdmissionService;
use morn_capability::{
    CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityRequest, CapabilityResolver,
    CapabilityStage, EffectClass, IsolationLevel,
};
use morn_control_plane::{
    ControlPlaneStore, ControllerInputs, ReconciliationController, WorkController,
};
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode};
use morn_harness::{run_harness_neutrality, PiHarnessProvider, PiMode, RuntimeContext};
use morn_kernel::error::Result;
use morn_kernel::ids::{ActorInstanceId, CapabilityId, PrincipalId, WorkspaceId};
use morn_kernel::policy::{Policy, PolicyRule};
use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};
use morn_runtime::{
    enforce_authority, ActionAttempt, AttemptState, AuthorityProvider, AuthorityRequest,
    ExecutionBinding, ExecutionEnvironmentProvider, ExecutionEnvironmentSpec,
    FixtureEnvironmentProvider, IsolationClass, NativePolicyAuthority, OutcomeReconciler,
    ReconciliationObservation,
};
use morn_store::MornStore;
use morn_work::acceptance::AcceptanceSpec;
use morn_work::control::{WorkPhase, WorkResource, WorkSpec};
use morn_work::service::{AcceptanceEvidence, WorkService};
use morn_work::work_package::WorkPackage;
use morn_world::outcome::OutcomeRecord;

struct CmmsCommittedAfterTimeout;

impl OutcomeReconciler for CmmsCommittedAfterTimeout {
    fn reconcile(&self, attempt: &ActionAttempt) -> Result<ReconciliationObservation> {
        Ok(ReconciliationObservation {
            business_key: attempt.business_key.clone(),
            committed: Some(true),
            observed: Some(true),
            external_ref: Some("MO-88273".to_string()),
            evidence_refs: vec!["cmms://orders/MO-88273".to_string()],
        })
    }
}

fn passing_factory_conformance(profile: &DomainProfile) -> morn_profile::ConformanceReport {
    let mut semantics = BTreeSet::new();
    semantics.extend(
        profile
            .requirements
            .iter()
            .filter(|item| item.level == RequirementLevel::Required)
            .map(|item| item.semantic.clone()),
    );
    evaluate_profile(
        profile,
        &ConformanceEvidence {
            satisfied_semantics: semantics,
            isolation: "container".to_string(),
            durable_work_state: true,
            source_of_truth_bound: true,
            provenance_ready: true,
            ..Default::default()
        },
    )
}

#[test]
fn factory_readonly_wedge_closes_without_agent_becoming_business_truth() {
    let workspace = WorkspaceId::generate();
    let owner = PrincipalId::generate();

    // Acceptance exists before execution and remains independent from harness output.
    let acceptance = AcceptanceSpec::new("factory delivery-impact review")
        .with_required_artifacts(vec!["delivery-impact-review".to_string()])
        .with_human_approval(true);
    let acceptance_id = acceptance.id.clone();
    let work_package = WorkPackage::new(
        workspace.clone(),
        "CNC-17 outage -> capacity -> delivery impact review",
        owner,
    )
    .with_acceptance_spec(acceptance_id.clone());
    let work_package_id = work_package.id.clone();

    // Capability begins only as a declaration.
    let mut manifest = CapabilityManifest::new(
        CapabilityId::generate_with("cap"),
        "equipment-investigator",
        "harness://dsh",
        CapabilityKind::Llm,
        EffectClass::E0LifecycleReversible,
    );
    manifest.provides = vec!["equipment.anomaly.investigate".to_string()];
    manifest.authority.allow = vec!["historian.read".to_string()];
    manifest.execution.minimum_isolation = IsolationLevel::Container;
    manifest.provenance.source_ref = "repo://factory/equipment-investigator".to_string();
    manifest.provenance.source_digest = Some("sha256:capability-fixture".to_string());
    let mut capability = CapabilityRecord::new(manifest);
    assert_eq!(capability.stage, CapabilityStage::Declared);

    // Qualification and site admission are separate gates.
    let profile = DomainProfile::factory_readonly_v1();
    let conformance = passing_factory_conformance(&profile);
    assert!(conformance.passed);
    let mut admission = AdmissionService::default();
    let qualification = admission
        .qualify(
            &mut capability,
            "release:equipment-investigator@1",
            "certification-decision:factory-fixture",
            vec!["evaluation:factory-fixture".to_string()],
            vec!["factory-readonly".to_string()],
        )
        .unwrap();
    admission
        .admit(
            &mut capability,
            &qualification,
            "plant-a",
            conformance.profile_ref.clone(),
            &conformance,
            "site-owner",
        )
        .unwrap();
    assert_eq!(capability.stage, CapabilityStage::Admitted);

    // Resolver selects only a capability admitted for the target site.
    let resolved = CapabilityResolver.resolve(
        &CapabilityRequest {
            required_provides: vec!["equipment.anomaly.investigate".to_string()],
            allowed_kinds: vec![CapabilityKind::Llm],
            minimum_isolation: Some(IsolationLevel::Container),
            required_authority: vec!["historian.read".to_string()],
            site_ref: Some("plant-a".to_string()),
            ..Default::default()
        },
        &[capability.clone()],
    );
    assert_eq!(resolved.len(), 1);

    // Authority decision is provider-neutral and enforced before side effects.
    let policy = Policy::new(
        workspace.clone(),
        "factory-readonly",
        vec![PolicyRule::allow("historian.read")],
    );
    let authority = NativePolicyAuthority::new(policy);
    let authority_decision = authority
        .decide(&{
            let mut request =
                AuthorityRequest::new("equipment-investigator", "historian.read", "CNC-17");
            request.work_ref = Some(work_package_id.to_string());
            request.site_ref = Some("plant-a".to_string());
            request.scope = vec!["historian.read".to_string()];
            request
        })
        .unwrap();
    enforce_authority(&authority_decision).unwrap();

    // Execution environment is selected by requirements rather than hard-coded.
    let mut environment_provider = FixtureEnvironmentProvider::default();
    let environment = environment_provider
        .provision(&ExecutionEnvironmentSpec {
            minimum_isolation: IsolationClass::Container,
            ..Default::default()
        })
        .unwrap();

    // Work desired/observed state is canonical; harness state is not.
    let mut work_spec = WorkSpec::new(
        work_package_id.clone(),
        "review CNC-17 outage capacity and delivery impact",
        conformance.profile_ref.clone(),
    );
    work_spec.acceptance_ref = Some(acceptance_id.to_string());
    work_spec.required_conditions = vec![
        "CapabilityResolved".to_string(),
        "CapabilityQualified".to_string(),
        "AuthoritySatisfied".to_string(),
        "SourceOfTruthBound".to_string(),
        "ProvenanceReady".to_string(),
    ];
    let mut work = WorkResource::new(workspace.clone(), work_spec);
    WorkController.reconcile(
        &mut work,
        &profile,
        &ControllerInputs {
            capability_resolved: true,
            capability_qualified: true,
            authority_satisfied: authority_decision.allowed,
            source_of_truth_bound: true,
            provenance_ready: true,
        },
    );
    assert_eq!(work.status.phase, WorkPhase::Ready);

    // DSH and Pi both satisfy the same Morn harness boundary.
    let ctx = RuntimeContext {
        workspace_id: workspace.clone(),
        work_package_id: work_package_id.clone(),
        actor_id: ActorInstanceId::generate_with("actor"),
        correlation_id: "factory-wedge".to_string(),
        trace_id: "trace-factory-wedge".to_string(),
    };
    let mut dsh = DeepSeekHarnessProvider::new(DshMode::Fixture);
    let mut pi = PiHarnessProvider::new(PiMode::Fixture);
    assert!(run_harness_neutrality(&mut dsh, &mut pi, &ctx)
        .unwrap()
        .all_passed());

    // The active attempt pins the exact provider/runtime selection.
    let mut binding = ExecutionBinding::for_work(
        &work,
        capability.manifest.id.to_string(),
        resolved[0].provider_ref.clone(),
        "fixture-v1",
    );
    binding.provider_digest = Some("sha256:dsh-fixture".to_string());
    binding.runtime_ref = Some(environment.runtime_ref.clone());
    binding.authority_decision_ref = Some(authority_decision.id.to_string());
    work.status.active_binding = Some(binding.id.clone());

    // Simulate the classic timeout-after-remote-commit ambiguity.
    let mut attempt = ActionAttempt::new(
        binding.id.clone(),
        format!("{}:maintenance-order", work.id),
        "create-maintenance-order-fixture",
    );
    attempt.transition(AttemptState::Authorized).unwrap();
    attempt.transition(AttemptState::Dispatched).unwrap();
    attempt
        .mark_outcome_unknown("client timeout after external commit")
        .unwrap();
    assert!(attempt.transition(AttemptState::Dispatched).is_err());

    let reconciliation = ReconciliationController
        .reconcile(&mut attempt, &CmmsCommittedAfterTimeout)
        .unwrap();
    assert_eq!(attempt.state, AttemptState::Observed);
    assert_eq!(attempt.external_ref.as_deref(), Some("MO-88273"));
    attempt.transition(AttemptState::Verified).unwrap();

    // Provider migration never edits the binding used by the already-started attempt.
    let migrated = binding.migrate_to_provider("pi", "fixture-v1");
    assert_eq!(binding.provider_ref, "harness://dsh");
    assert_eq!(attempt.binding_id, binding.id);
    assert_eq!(migrated.migration_from, Some(binding.id.clone()));
    assert_eq!(migrated.provider_ref, "pi");

    // Independent acceptance closes the business outcome.
    let mut work_service = WorkService::new();
    work_service.add_acceptance_spec(acceptance);
    work_service.add_work_package(work_package);
    work_service
        .attempt_accept(
            &work_package_id,
            &AcceptanceEvidence {
                produced_artifacts: vec!["delivery-impact-review".to_string()],
                verification_passed: true,
                required_approvals: vec!["production-planner".to_string()],
                forbidden_condition_hit: None,
            },
        )
        .unwrap();
    let mut outcome = OutcomeRecord::new(
        workspace.clone(),
        "delivery impact reviewed with reconciled maintenance reference",
        true,
    );
    outcome.work_package_id = Some(work_package_id.clone());
    outcome.related_artifacts = vec!["delivery-impact-review".to_string()];
    assert!(outcome.acceptance_met);

    // Durable control-plane state survives serialization independently of harness sessions.
    let store = MornStore::open_in_memory().unwrap();
    store.save_work_resource(&work).unwrap();
    store.save_execution_binding(&work, &binding).unwrap();
    store.save_action_attempt(&work, &attempt).unwrap();
    store.save_reconciliation(&work, &reconciliation).unwrap();
    let restored: WorkResource = store
        .load_record("work_resource_v115", work.id.as_str())
        .unwrap()
        .unwrap();
    assert_eq!(restored.status.phase, WorkPhase::Ready);
    assert_eq!(restored.status.active_binding, Some(binding.id.clone()));

    environment_provider.release(&environment).unwrap();
}
