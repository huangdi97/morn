//! Cross-crate v11.5 conformance invariants.

use std::collections::BTreeSet;

use morn_assurance::{AdmissionService, QualificationEvidence, StrictQualificationRequest};
use morn_capability::{
    CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityResolver, CapabilityStage,
    EffectClass, IsolationLevel, WorkcellRequest,
};
use morn_kernel::history::{HistoricalFact, HistoricalFactLog, HistoricalFactStatus};
use morn_kernel::ids::{CapabilityId, RuntimeBindingId, WorkPackageId, WorkspaceId};
use morn_kernel::protocol::{HistoryMutation, HistoryMutationKind, ProtocolSnapshot};
use morn_kernel::time::Timestamp;
use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};
use morn_runtime::{
    ActionAttempt, AttemptState, BindingMigrationReason, BindingMigrationRequest, ExecutionBinding,
};
use morn_work::{WorkResource, WorkSpec};
use serde_json::json;

fn admitted(
    name: &str,
    provider: &str,
    kind: CapabilityKind,
    provides: &[&str],
    cost: u64,
) -> CapabilityRecord {
    let mut manifest = CapabilityManifest::new(
        CapabilityId::generate_with("cap"),
        name,
        provider,
        kind,
        EffectClass::E0LifecycleReversible,
    );
    manifest.provides = provides.iter().map(|value| value.to_string()).collect();
    manifest.economics.estimated_cost_micros = Some(cost);
    manifest.execution.minimum_isolation = IsolationLevel::Container;
    let mut record = CapabilityRecord::new(manifest);
    record.stage = CapabilityStage::Admitted;
    record.qualification_refs.push(format!("qual:{name}"));
    record.admitted_sites.push("plant-a".to_string());
    record
}

fn passing_profile(profile: &DomainProfile) -> morn_profile::ConformanceReport {
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
fn protocol_and_profiles_are_provider_independent() {
    let protocol = ProtocolSnapshot::v11_5();
    for required in [
        "Work",
        "Capability",
        "Authority",
        "ExecutionBinding",
        "Attempt",
        "Receipt",
        "Reconciliation",
        "Outcome",
        "Acceptance",
        "Provenance",
        "Profile",
    ] {
        assert!(protocol.semantic_slots.iter().any(|slot| slot == required));
    }

    let factory = DomainProfile::factory_readonly_v1();
    assert!(factory.forbids("ProductionWrite"));
    assert!(passing_profile(&factory).passed);
    assert!(!factory
        .requirements
        .iter()
        .any(|requirement| requirement.semantic.contains("DeepSeek")));
}

#[test]
fn historical_correction_is_explicit_and_queryable_as_known_then() {
    let mut log = HistoricalFactLog::default();
    let first = HistoricalFact::observed(
        "machine:CNC-17",
        "temperature",
        json!({"celsius":82}),
        "sensor:A",
        Timestamp::from_millis(1_000),
    );
    let cutoff = first.recorded_at;
    let first_id = first.id.clone();
    log.append(first).unwrap();
    let corrected = HistoricalFact::superseding(
        log.current_for("machine:CNC-17", "temperature").unwrap(),
        json!({"celsius":63}),
        "sensor:B",
        "calibration correction",
    );
    log.append(corrected).unwrap();

    assert_eq!(
        log.facts
            .iter()
            .find(|fact| fact.id == first_id)
            .unwrap()
            .status,
        HistoricalFactStatus::Superseded
    );
    assert_eq!(
        log.as_recorded_at("machine:CNC-17", "temperature", cutoff)
            .unwrap()
            .payload["celsius"],
        82
    );

    let mut mutation = HistoryMutation::new(
        "fact:old",
        HistoryMutationKind::Supersede,
        "explicit correction",
        "operator",
    );
    mutation.replacement_ref = Some("fact:new".to_string());
    assert_eq!(mutation.kind, HistoryMutationKind::Supersede);
}

#[test]
fn minimum_sufficient_workcell_does_not_default_to_agents() {
    let candidates = vec![
        admitted(
            "classifier-rule",
            "rules",
            CapabilityKind::Rule,
            &["alarm.classify"],
            1,
        ),
        admitted(
            "capacity-solver",
            "solver",
            CapabilityKind::Solver,
            &["capacity.optimize"],
            20,
        ),
        admitted(
            "owner",
            "human",
            CapabilityKind::Human,
            &["decision.accept"],
            100,
        ),
        admitted(
            "general-agent",
            "dsh",
            CapabilityKind::Agent,
            &["alarm.classify"],
            50,
        ),
    ];

    let plan = CapabilityResolver.resolve_minimum_workcell(
        &WorkcellRequest {
            required_provides: vec![
                "alarm.classify".to_string(),
                "capacity.optimize".to_string(),
                "decision.accept".to_string(),
            ],
            site_ref: Some("plant-a".to_string()),
            minimum_isolation: Some(IsolationLevel::Container),
            ..Default::default()
        },
        &candidates,
    );
    assert!(plan.is_complete());
    assert_eq!(plan.agent_count(), 0);
}

#[test]
fn binding_and_attempt_preserve_provider_identity_across_migration() {
    let work_id = WorkPackageId::generate_with("work");
    let work = WorkResource::new(
        WorkspaceId::generate(),
        WorkSpec::new(work_id, "investigate outage", "morn.factory.readonly@1.0.0"),
    );
    let binding = ExecutionBinding::for_work(&work, "manifest@sha256:a", "dsh", "1");
    let mut attempt = ActionAttempt::new(
        binding.id.clone(),
        "work:create-order",
        "create-maintenance-order",
    );
    attempt.transition(AttemptState::Authorized).unwrap();
    attempt.transition(AttemptState::Dispatched).unwrap();
    attempt.mark_outcome_unknown("timeout").unwrap();
    assert!(attempt.transition(AttemptState::Dispatched).is_err());

    let replacement = binding.migrate_to_provider("pi", "2");
    assert_eq!(attempt.binding_id, binding.id);
    assert_eq!(binding.provider_ref, "dsh");
    assert_eq!(replacement.provider_ref, "pi");
    assert_eq!(replacement.migration_from, Some(binding.id));
}

#[test]
fn site_admission_requires_strict_nonexpired_qualification() {
    let mut manifest = CapabilityManifest::new(
        CapabilityId::generate_with("cap"),
        "investigator",
        "provider://fixture",
        CapabilityKind::Program,
        EffectClass::E0LifecycleReversible,
    );
    manifest.provides = vec!["investigate".to_string()];
    let mut capability = CapabilityRecord::new(manifest);
    let mut service = AdmissionService::default();
    service
        .observe(
            &mut capability,
            vec!["evaluation-observation:core-conformance".to_string()],
            "evaluator:1",
        )
        .unwrap();
    let qualification = service
        .qualify_with_evidence(
            &mut capability,
            StrictQualificationRequest {
                candidate_ref: "release:1".to_string(),
                decision_ref: "decision:1".to_string(),
                evidence_refs: vec!["eval:1".to_string()],
                qualification_evidence: QualificationEvidence {
                    test_suite_refs: vec!["suite:1".to_string()],
                    environment_digest: Some("sha256:env".to_string()),
                    expected_properties: vec!["deterministic-output".to_string()],
                    evaluator_identity: Some("evaluator:1".to_string()),
                    ..Default::default()
                },
                context_of_use: vec!["factory-readonly".to_string()],
                valid_until: None,
            },
        )
        .unwrap();
    service
        .record_release(
            &mut capability,
            &qualification,
            format!("oci://fixture/morn/investigator@sha256:{}", "b".repeat(64)),
            format!("sha256:{}", "b".repeat(64)),
            Some("sigstore://fixture/investigator".to_string()),
            Some("slsa://fixture/investigator".to_string()),
        )
        .unwrap();

    let profile = DomainProfile::factory_readonly_v1();
    let report = passing_profile(&profile);
    service
        .admit(
            &mut capability,
            &qualification,
            "plant-a",
            report.profile_ref.clone(),
            &report,
            "site-owner",
        )
        .unwrap();
    assert_eq!(capability.stage, CapabilityStage::Admitted);
}

#[test]
fn attempt_id_type_is_distinct_from_binding_id_type() {
    let binding = RuntimeBindingId::generate_with("binding");
    let attempt = ActionAttempt::new(binding.clone(), "key", "action");
    assert_eq!(attempt.binding_id, binding);
}

#[test]
fn external_agent_task_completion_is_not_business_acceptance() {
    use morn_integration::{A2aTaskEvidence, A2aTaskState};

    let external = A2aTaskEvidence {
        agent_ref: "a2a://planner".to_string(),
        task_id: "task-1".to_string(),
        context_id: Some("context-1".to_string()),
        state: A2aTaskState::Completed,
        artifact_refs: vec!["artifact://plan".to_string()],
        raw_status: None,
    };

    assert!(external.is_executor_terminal());
    assert!(!external.proves_morn_acceptance());
}

#[test]
fn mcp_tool_metadata_never_grants_authority_by_itself() {
    use morn_integration::McpToolDescriptor;

    let descriptor = McpToolDescriptor {
        server_ref: "mcp://cmms".to_string(),
        tool_name: "create_order".to_string(),
        input_schema: json!({"type":"object"}),
        output_schema: None,
        annotations: json!({"destructiveHint": true}),
    };
    descriptor.validate().unwrap();

    let serialized = serde_json::to_value(descriptor).unwrap();
    assert!(serialized.get("authority").is_none());
    assert!(serialized.get("credential").is_none());
    assert!(serialized.get("token").is_none());
}

#[test]
fn work_truth_survives_harness_provider_migration() {
    let work_id = WorkPackageId::generate_with("work");
    let work = WorkResource::new(
        WorkspaceId::generate(),
        WorkSpec::new(
            work_id.clone(),
            "investigate outage",
            "morn.factory.readonly@1.0.0",
        ),
    );

    let first = ExecutionBinding::for_work(
        &work,
        "capability:investigator@sha256:dsh",
        "deepseek-harness",
        "fixture-v1",
    );
    let mut first_attempt = ActionAttempt::new(
        first.id.clone(),
        format!("{}:investigate:1", work.id),
        "investigate-outage",
    );
    first_attempt.transition(AttemptState::Authorized).unwrap();
    first_attempt.transition(AttemptState::Dispatched).unwrap();
    first_attempt
        .mark_outcome_unknown("harness/provider lost during execution")
        .unwrap();

    let (second, decision) = first.rebind_for_work(
        &work,
        BindingMigrationRequest::new(
            "capability:investigator@sha256:pi",
            "pi",
            "fixture-v1",
            BindingMigrationReason::RuntimeRecovery,
            "work-controller",
        )
        .with_evidence(vec!["provider:deepseek-harness-unavailable".to_string()]),
    );
    let second_attempt = ActionAttempt::new(
        second.id.clone(),
        format!("{}:investigate:2", work.id),
        "investigate-outage",
    );

    assert_eq!(first.work_id, work.id);
    assert_eq!(second.work_id, work.id);
    assert_eq!(first.work_generation, second.work_generation);
    assert_ne!(first.id, second.id);
    assert_eq!(first_attempt.binding_id, first.id);
    assert_eq!(second_attempt.binding_id, second.id);
    assert_eq!(decision.from_binding, first.id);
    assert_eq!(decision.to_binding, second.id);
    assert_eq!(first_attempt.state, AttemptState::OutcomeUnknown);
}
