//! End-to-end reference slice for the first Factory read-only wedge.
//!
//! The scenario exercises resolution, qualification/admission, profile
//! conformance, harness neutrality, pinned bindings, ambiguous external action
//! reconciliation, accepted outcome and durable persistence. All external
//! systems are deterministic fixtures; this is not production evidence.

use std::collections::BTreeSet;

use morn_assurance::{AdmissionService, QualificationEvidence, StrictQualificationRequest};
use morn_capability::{
    CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityRequest, CapabilityResolver,
    CapabilityStage, EffectClass, IsolationLevel,
};
use morn_control_plane::{
    enforce_profile_action, evaluate_profile_action, ControlPlaneStore, ControllerInputs,
    ExternalActionMode, ReconciliationController, WorkController, WorkProgressController,
    WorkProgressInputs,
};
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode};
use morn_harness::{run_harness_neutrality, PiHarnessProvider, PiMode, RuntimeContext};
use morn_integration::{
    ConflictPolicy, SourceOfTruthBinding, SourceOfTruthBindingId, TruthAuthorityKind,
};
use morn_kernel::error::Result;
use morn_kernel::ids::{ActorInstanceId, CapabilityId, PrincipalId, WorkspaceId};
use morn_kernel::policy::{Policy, PolicyRule};
use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};
use morn_runtime::{
    enforce_authority, ActionAttempt, AttemptState, AuthorityProvider, AuthorityRequest,
    BindingMigrationReason, ExecutionBinding, ExecutionEnvironmentOffer,
    ExecutionEnvironmentProvider, ExecutionEnvironmentResolver, ExecutionEnvironmentSpec,
    FixtureEnvironmentProvider, NativePolicyAuthority, OutcomeReconciler,
    ReconciliationObservation,
};
use morn_store::MornStore;
use morn_work::acceptance::AcceptanceSpec;
use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
use morn_work::control::{WorkPhase, WorkResource, WorkSpec};
use morn_work::service::{AcceptanceEvidence, WorkService};
use morn_work::value::{ValueAssessment, ValueEvidenceClass};
use morn_work::work_package::WorkPackage;
use morn_world::{ObservedOutcome, OutcomeSourceKind};

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
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .collect(),
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

    // Factory profile drives capability/environment requirements from the start.
    let profile = DomainProfile::factory_readonly_v1();

    // Capability begins only as a declaration.
    let mut manifest = CapabilityManifest::new(
        CapabilityId::generate_with("cap"),
        "equipment-investigator",
        "harness://dsh",
        CapabilityKind::Agent,
        EffectClass::E0LifecycleReversible,
    );
    manifest.provides = vec!["equipment.anomaly.investigate".to_string()];
    manifest.authority.allow = vec!["historian.read".to_string()];
    manifest.execution.minimum_isolation = IsolationLevel::Container;
    // Capability declares only its intrinsic execution needs. Profile
    // requirements are merged later by the resolver; declarations are not
    // environment evidence.
    manifest.execution.required_guarantees =
        vec![morn_kernel::ExecutionGuarantee::FilesystemReadPolicy];
    manifest.provenance.source_ref = "repo://factory/equipment-investigator".to_string();
    manifest.provenance.source_digest = Some("sha256:capability-fixture".to_string());
    let mut capability = CapabilityRecord::new(manifest);
    assert_eq!(capability.stage, CapabilityStage::Declared);

    // Qualification and site admission are separate gates.
    let conformance = passing_factory_conformance(&profile);
    assert!(conformance.passed);
    let mut admission = AdmissionService::default();
    admission
        .observe(
            &mut capability,
            vec!["evaluation-observation:factory-fixture".to_string()],
            "morn-conformance-suite",
        )
        .unwrap();
    assert_eq!(capability.stage, CapabilityStage::Observed);
    let qualification = admission
        .qualify_with_evidence(
            &mut capability,
            StrictQualificationRequest {
                candidate_ref: "release:equipment-investigator@1".to_string(),
                decision_ref: "certification-decision:factory-fixture".to_string(),
                evidence_refs: vec!["evaluation:factory-fixture".to_string()],
                qualification_evidence: QualificationEvidence {
                    test_suite_refs: vec!["suite:factory-timeout-reconcile".to_string()],
                    environment_digest: Some("sha256:fixture-environment".to_string()),
                    input_scope: vec!["synthetic:CNC-17".to_string()],
                    expected_properties: vec![
                        "no-blind-retry".to_string(),
                        "binding-pinned".to_string(),
                    ],
                    known_failure_modes: vec!["timeout-after-commit".to_string()],
                    cost_evidence_ref: Some("fixture://metrics/cost".to_string()),
                    latency_evidence_ref: Some("fixture://metrics/latency".to_string()),
                    evaluator_identity: Some("morn-conformance-suite".to_string()),
                },
                context_of_use: vec!["factory-readonly".to_string()],
                valid_until: None,
            },
        )
        .unwrap();
    let release = admission
        .record_release(
            &mut capability,
            &qualification,
            format!(
                "oci://fixture/morn/equipment-investigator@sha256:{}",
                "a".repeat(64)
            ),
            format!("sha256:{}", "a".repeat(64)),
            Some("sigstore://fixture/equipment-investigator".to_string()),
            Some("slsa://fixture/equipment-investigator".to_string()),
        )
        .unwrap();
    assert!(capability
        .release_refs
        .iter()
        .any(|reference| reference == &release.id.to_string()));
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
            allowed_kinds: vec![CapabilityKind::Agent],
            minimum_isolation: Some(IsolationLevel::Container),
            required_authority: vec!["historian.read".to_string()],
            required_execution_guarantees: profile.required_execution_guarantees.clone(),
            site_ref: Some("plant-a".to_string()),
            profile_ref: Some(conformance.profile_ref.clone()),
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

    // Execution environment is selected from provider offers by requirements.
    let mut environment_provider = FixtureEnvironmentProvider::default();
    let environment_spec = ExecutionEnvironmentSpec {
        minimum_isolation: resolved[0].required_execution_class,
        required_guarantees: resolved[0].required_execution_guarantees.clone(),
        ..Default::default()
    };
    let offers = ExecutionEnvironmentOffer::from_provider(&environment_provider, Some(1));
    let environment_selection = ExecutionEnvironmentResolver
        .resolve(&environment_spec, &offers)
        .expect("Factory slice requires a conformant execution environment");
    let environment = environment_provider
        .provision(&ExecutionEnvironmentSpec {
            minimum_isolation: environment_selection.isolation,
            ..environment_spec.clone()
        })
        .unwrap();
    for required in &resolved[0].required_execution_guarantees {
        assert!(
            environment.guarantees.contains(required),
            "required Factory execution guarantees must be provided"
        );
    }

    let source_binding = SourceOfTruthBinding {
        id: SourceOfTruthBindingId::generate_with("sot"),
        site_ref: Some("plant-a".to_string()),
        source_ref: "cmms://plant-a".to_string(),
        authority_kind: TruthAuthorityKind::SystemOfRecord,
        authoritative_fact_types: vec!["maintenance.order".to_string()],
        key_mapping_ref: "mapping://cmms-order-key@1".to_string(),
        query_capability_ref: "capability://cmms.read-order@1".to_string(),
        freshness_sla_ms: Some(30_000),
        conflict_policy: ConflictPolicy::ReconcileBeforeUse,
        version_ref: "binding:v1".to_string(),
        created_at: morn_kernel::time::Timestamp::now(),
    };
    source_binding.validate().unwrap();
    assert!(source_binding.authoritative_for("maintenance.order"));

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
            source_of_truth_bound: source_binding.validate().is_ok(),
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
    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            binding: Some(&binding),
            attempt: Some(&attempt),
            ..Default::default()
        },
    );
    assert_eq!(work.status.phase, WorkPhase::Reconciling);

    let reconciliation = ReconciliationController
        .reconcile(&mut attempt, &CmmsCommittedAfterTimeout)
        .unwrap();
    assert_eq!(attempt.state, AttemptState::Observed);
    assert_eq!(attempt.external_ref.as_deref(), Some("MO-88273"));
    attempt.transition(AttemptState::Verified).unwrap();
    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            binding: Some(&binding),
            attempt: Some(&attempt),
            ..Default::default()
        },
    );
    assert!(work.condition_is_true("ProfileVersionPinned"));
    assert!(work.condition_is_true("ReceiptAfterExternalAction"));
    assert!(work.condition_is_true("ReconciliationOnUnknown"));

    // Provider migration never edits the binding used by the already-started attempt.
    let (migrated, migration_decision) = binding.rebind_for_work(
        &work,
        capability.manifest.id.to_string(),
        "pi",
        "fixture-v1",
        BindingMigrationReason::ProviderReplacement,
        "factory-slice-controller",
        vec!["harness-neutrality:passed".to_string()],
    );
    assert_eq!(binding.provider_ref, "harness://dsh");
    assert_eq!(attempt.binding_id, binding.id);
    assert_eq!(migrated.migration_from, Some(binding.id.clone()));
    assert_eq!(migrated.provider_ref, "pi");
    assert_eq!(migration_decision.from_binding, binding.id);
    assert_eq!(migration_decision.to_binding, migrated.id);

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
    let mut outcome = ObservedOutcome::new(
        workspace.clone(),
        work_package_id.clone(),
        "delivery impact reviewed with reconciled maintenance reference",
        OutcomeSourceKind::ExternalSystem,
        "cmms://orders/MO-88273",
        serde_json::json!({
            "maintenance_order": "MO-88273",
            "delivery_impact_review": "produced"
        }),
    );
    outcome
        .evidence_refs
        .push("cmms://orders/MO-88273/receipt".to_string());
    outcome
        .evidence_refs
        .push("artifact://delivery-impact-review".to_string());
    assert!(outcome.is_source_grounded());
    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            binding: Some(&binding),
            outcome: Some(&outcome),
            ..Default::default()
        },
    );
    assert_eq!(work.status.phase, WorkPhase::Delivered);
    assert!(work.condition_is_true("OutcomeObservation"));

    let mut acceptance_decision = AcceptanceDecision::new(
        work_package_id.clone(),
        acceptance_id,
        AcceptanceDisposition::Accept,
        PrincipalId::generate_with("production-owner"),
        "production-owner",
        "source-grounded outcome and delivery-impact review accepted",
    );
    acceptance_decision.outcome_refs.push(outcome.id.clone());
    acceptance_decision
        .evidence_refs
        .extend(outcome.evidence_refs.clone());
    assert!(acceptance_decision.is_final_acceptance());
    WorkProgressController.reconcile(
        &mut work,
        &WorkProgressInputs {
            binding: Some(&binding),
            outcome: Some(&outcome),
            acceptance: Some(&acceptance_decision),
            ..Default::default()
        },
    );
    assert_eq!(work.status.phase, WorkPhase::Accepted);
    assert!(work.condition_is_true("IndependentAcceptance"));
    assert!(work.condition_is_true("AcceptedOutcomeSemantics"));

    let mut value = ValueAssessment::new(
        work_package_id.clone(),
        outcome.id.clone(),
        ValueEvidenceClass::Fixture,
    );
    value.acceptance_ref = Some(acceptance_decision.id.clone());
    value.unknown_outcome_count = 1;
    value.retry_count = 0;
    value.evidence_refs = vec!["fixture://factory-readonly-slice".to_string()];
    assert!(!value.is_customer_value_claim());

    // Durable control-plane state survives serialization independently of harness sessions.
    let store = MornStore::open_in_memory().unwrap();
    store.save_work_resource(&work).unwrap();
    store
        .save_source_of_truth_binding(&work, &source_binding)
        .unwrap();
    store.save_execution_binding(&work, &binding).unwrap();
    store
        .save_binding_migration(&work, &migration_decision)
        .unwrap();
    store.save_action_attempt(&work, &attempt).unwrap();
    store.save_reconciliation(&work, &reconciliation).unwrap();
    store.save_observed_outcome(&work, &outcome).unwrap();
    store
        .save_acceptance_decision(&work, &acceptance_decision)
        .unwrap();
    store.save_value_assessment(&work, &value).unwrap();
    let restored: WorkResource = store
        .load_record("work_resource_v115", work.id.as_str())
        .unwrap()
        .unwrap();
    assert_eq!(restored.status.phase, WorkPhase::Accepted);
    assert_eq!(restored.status.active_binding, Some(binding.id.clone()));

    let restored_outcome: ObservedOutcome = store
        .load_record("observed_outcome_v115", outcome.id.as_str())
        .unwrap()
        .unwrap();
    let restored_acceptance: AcceptanceDecision = store
        .load_record("acceptance_decision_v115", acceptance_decision.id.as_str())
        .unwrap()
        .unwrap();
    let restored_value: ValueAssessment = store
        .load_record("value_assessment_v115", value.id.as_str())
        .unwrap()
        .unwrap();
    assert!(restored_outcome.is_source_grounded());
    assert!(restored_acceptance.is_final_acceptance());
    assert!(!restored_value.is_customer_value_claim());

    environment_provider.release(&environment).unwrap();
}
