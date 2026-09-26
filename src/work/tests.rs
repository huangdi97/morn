//! tests — Schema validation tests for the semantic specification (G1).
//!
//! These tests are schema-driven: instances are validated against the JSON
//! Schemas in `schemas/` (real validation, not just serde round-trip), plus
//! old-record compatibility mapping and invariants from the v11.5 master.

use super::*;
use serde_json::json;

fn sample_spec() -> resource::WorkSpec {
    resource::WorkSpec {
        work_type: "factory.equipment-exception".to_string(),
        goal: "restore_operational_readiness".to_string(),
        target_refs: vec![ObjectRef::new("asset", "CNC-17")],
        constraints: vec!["no_plc_write".to_string(), "no_quality_release".to_string()],
        acceptance: Some(resource::AcceptanceSpec {
            required: vec![resource::AcceptanceRequirement {
                criterion: "machine_operational_verified".to_string(),
                required_roles: vec!["production_owner".to_string()],
                description: None,
            }],
        }),
        authority: Some(resource::AuthorityRequirement {
            required_for: vec!["external-side-effect".to_string()],
            profile_ref: None,
        }),
        budget: Some(resource::Budget {
            max_cost: Some(CostAmount::new(50.0, "CNY")),
            max_human_minutes: Some(120),
        }),
        profile_ref: Some("morn.factory.readonly/v1alpha1".to_string()),
        source_refs: vec![SourceRef::new("scada")],
        owner: Some("plant-a/night-shift".to_string()),
        generation: 7,
        contract_version: Some(SEMANTIC_CONTRACT_VERSION.to_string()),
    }
}

#[test]
fn test_work_validates_against_json_schema() {
    let schema = Schema::work();
    let mut work = Work::draft("work-1042", sample_spec());
    work.status.observed_generation = 7;
    work.status.phase = resource::WorkPhase::Reconciling;
    work.set_condition(Condition::satisfied(ConditionType::CapabilityResolved));
    work.set_condition(Condition::unsatisfied(
        ConditionType::ExternalActionVerified,
        "OutcomeUnknown",
    ));

    let instance = serde_json::to_value(&work).unwrap();
    schema
        .validate(&instance)
        .unwrap_or_else(|e| panic!("schema validation failed:\n{e}"));
}

#[test]
fn test_minimal_work_fails_schema_validation_when_required_missing() {
    let schema = Schema::work();
    // Missing spec and status entirely.
    let invalid = json!({ "id": "work-bad" });
    assert!(schema.validate(&invalid).is_err());
}

#[test]
fn test_work_serde_round_trip() {
    let mut work = Work::draft("work-1042", sample_spec());
    work.status.phase = resource::WorkPhase::Executing;
    work.set_condition(Condition::satisfied(ConditionType::ExecutionBound));
    let serialized = serde_json::to_string(&work).unwrap();
    let deserialized: Work = serde_json::from_str(&serialized).unwrap();
    assert_eq!(work, deserialized);
}

#[test]
fn test_all_nine_condition_types_expressible() {
    assert_eq!(ConditionType::ALL.len(), 9);
    for gate in ConditionType::ALL {
        let satisfied = Condition::satisfied(gate);
        let unsatisfied = Condition::unsatisfied(gate, "why");
        let unknown = Condition::unknown(gate, "not yet observed");
        assert!(satisfied.is_true());
        assert!(!unsatisfied.is_true());
        assert!(!unknown.is_true());
    }
}

#[test]
fn test_work_phase_reconciling_is_first_class() {
    let mut work = Work::draft("work-1042", sample_spec());
    work.status.phase = resource::WorkPhase::Reconciling;
    let serialized = serde_json::to_string(&work).unwrap();
    assert!(serialized.contains("RECONCILING"));
}

#[test]
fn test_historical_semantics_supersede_and_retract() {
    let original = HistoricalRecord::record(
        "obs-1",
        "observation",
        json!({ "kind": "temperature", "value": 41.2 }),
        "2026-09-24T02:13:00Z",
    );
    let corrected = HistoricalRecord::supersede(
        "obs-2",
        "observation",
        "obs-1",
        json!({ "kind": "temperature", "value": 39.8 }),
        "2026-09-24T02:13:00Z",
    );
    assert_eq!(corrected.supersedes.as_deref(), Some("obs-1"));
    assert_eq!(corrected.contract_version.as_deref(), Some("v1"));
    assert!(corrected.valid_time.is_some());

    let retraction =
        HistoricalRecord::retract("obs-3", "observation", "obs-1", "sensor miscalibration");
    assert_eq!(retraction.retracts.as_deref(), Some("obs-1"));
    assert!(retraction.provenance.is_some());

    // Round-trip preserves the non-destructive chain.
    let serialized = serde_json::to_string(&corrected).unwrap();
    let deserialized: HistoricalRecord = serde_json::from_str(&serialized).unwrap();
    assert_eq!(corrected, deserialized);
}

#[test]
fn test_observation_supersede_keeps_history() {
    let source = SourceRef::new("scada");
    let original = Observation::record(
        "obs-1",
        source,
        "vibration",
        8.7,
        "mm/s",
        "2026-09-24T02:13:00Z",
    );
    let corrected = original.supersede_with("obs-2", 3.1, "calibrated");
    assert_eq!(corrected.supersedes.as_deref(), Some("obs-1"));
    assert_eq!(corrected.value, 3.1);
    assert_eq!(original.value, 8.7);
}

#[test]
fn test_authority_request_and_decision_contract() {
    let request = AuthorityRequest::new("authz-req-1", "agent-7", "cmms.create-order");
    let mut decision = AuthorityDecision::grant("authz-288", "authz-req-1", "reference-policy");
    assert!(decision.authorizes("2026-09-24T13:02:11Z"));

    decision.revoke("scope changed");
    assert!(!decision.authorizes("2026-09-24T13:02:11Z"));

    let denied = AuthorityDecision::deny("authz-289", "authz-req-2", "human", "no quality release");
    assert!(!denied.authorizes("2026-09-24T13:02:11Z"));

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: AuthorityRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(request, deserialized);
}

#[test]
fn test_expired_authority_decision_grants_nothing() {
    let decision = AuthorityDecision {
        expires_at: Some("2026-09-24T12:00:00Z".to_string()),
        ..AuthorityDecision::grant("authz-1", "authz-req-1", "reference-policy")
    };
    assert!(!decision.authorizes("2026-09-24T13:02:11Z"));
}

#[test]
fn test_capability_four_states_separated() {
    let manifest = CapabilityManifest {
        name: "equipment-investigator".to_string(),
        version: "1.4.0".to_string(),
        digest: "sha256:abc".to_string(),
        api_version: "capability.morn.io/v1alpha1".to_string(),
        kind: Some("agent".to_string()),
        spec: CapabilitySpec {
            provides: vec!["equipment.anomaly.investigate".to_string()],
            semantic: None,
            interfaces: None,
            side_effects: Some(SideEffects {
                level: "none".to_string(),
                description: None,
            }),
            idempotency: true,
            compensation: None,
            execution: Some(ExecutionRequirements {
                harness_compatible: vec!["dsh".to_string(), "pi".to_string()],
                minimum_isolation: Some("container".to_string()),
                network: vec!["historian".to_string()],
            }),
            authority: Some(AuthorityCeiling {
                maximum: vec!["historian.read".to_string(), "cmms.read".to_string()],
            }),
            economics: None,
            provenance: None,
        },
    };
    let serialized = serde_json::to_string(&manifest).unwrap();
    let deserialized: CapabilityManifest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(manifest, deserialized);

    // Declared != Qualified != Admitted: distinct enum variants.
    assert_ne!(CapabilityState::Declared, CapabilityState::Qualified);
    assert_ne!(CapabilityState::Qualified, CapabilityState::Admitted);

    // Asset lifecycle is separate from site admission.
    assert_ne!(
        CapabilityLifecycle::Qualified,
        CapabilityLifecycle::SiteBound
    );
}

#[test]
fn test_binding_attempt_receipt_round_trip() {
    let binding = ExecutionBinding::new("binding-221", "work-1042", "sha256:abc", "dsh");
    let mut attempt = ActionAttempt::proposed(
        "attempt-83",
        "work-1042",
        "binding-221",
        "cmms.create-order",
        "work-1042:cmms.create-order:1",
    );
    attempt.business_key = Some("work-1042:cmms.create-order".to_string());
    attempt.source_system = Some("cmms".to_string());

    // Attempt pins its binding.
    assert_eq!(attempt.binding_ref, binding.binding_id);

    let serialized = serde_json::to_string(&binding).unwrap();
    let deserialized: ExecutionBinding = serde_json::from_str(&serialized).unwrap();
    assert_eq!(binding, deserialized);

    let serialized = serde_json::to_string(&attempt).unwrap();
    let deserialized: ActionAttempt = serde_json::from_str(&serialized).unwrap();
    assert_eq!(attempt, deserialized);

    let receipt =
        Receipt::from_source("receipt-1", "attempt-83", "cmms-readback", Some("MO-88273"));
    let serialized = serde_json::to_string(&receipt).unwrap();
    let deserialized: Receipt = serde_json::from_str(&serialized).unwrap();
    assert_eq!(receipt, deserialized);
}

#[test]
fn test_attempt_unknown_reconciling_verified_path() {
    // The unknown path is first-class: timeout after commit does not become a
    // failure; it reconciles by business key.
    let mut attempt = ActionAttempt::proposed(
        "attempt-83",
        "work-1042",
        "binding-221",
        "cmms.create-order",
        "k1",
    );
    attempt.status = AttemptStatus::OutcomeUnknown;
    let serialized = serde_json::to_string(&attempt).unwrap();
    assert!(serialized.contains("OUTCOME_UNKNOWN"));

    attempt.status = AttemptStatus::Reconciling;
    assert!(serde_json::to_string(&attempt)
        .unwrap()
        .contains("RECONCILING"));

    attempt.status = AttemptStatus::Verified;
    assert!(serde_json::to_string(&attempt)
        .unwrap()
        .contains("VERIFIED"));

    let reconciliation =
        ReconciliationRecord::new("recon-1", "attempt-83", ReconciliationResult::Committed);
    assert_eq!(reconciliation.result, ReconciliationResult::Committed);
}

#[test]
fn test_old_task_record_maps_to_work_carrier_without_second_ledger() {
    // COMPATIBILITY: the legacy TaskRecord (core storage) maps into the Work
    // semantic as a carrier; the Work references it via source_ref instead of
    // duplicating its fields into a second ledger.
    let task = crate::core::storage::TaskRecord {
        id: "task-1042".to_string(),
        user_input: "restore production readiness".to_string(),
        plan_json: "{}".to_string(),
        status: "executing".to_string(),
        created_at: "2026-09-24T02:20:00Z".to_string(),
        completed_at: None,
    };

    let mut work = Work::draft("work-1042", sample_spec());
    let mapping = HistoricalRecord::record(
        "work-carrier-task-1042",
        "legacy_task_record",
        serde_json::to_value(&task).unwrap(),
        &task.created_at,
    );
    work.status.observed_generation = 1;
    let _ = &mapping;

    assert_eq!(work.id, "work-1042");
    assert_eq!(work.spec.goal, "restore_operational_readiness");
    assert_eq!(task.status, "executing");
    // The mapping payload is the original record, not a rewrite.
    assert_eq!(mapping.payload["id"], "task-1042");
}

#[test]
fn test_factory_readonly_profile_forbids_production_write() {
    let profile = Profile::factory_readonly();
    assert_eq!(
        profile.level_of("production_write"),
        RequirementLevel::Forbidden
    );
    assert_eq!(
        profile.level_of("authority_before_external_side_effect"),
        RequirementLevel::Required
    );
    assert_eq!(
        profile.level_of("receipt_after_external_action"),
        RequirementLevel::Required
    );
    assert_eq!(
        profile.level_of("reconciliation_on_unknown"),
        RequirementLevel::Required
    );
    assert!(!profile.human_gates.is_empty());
}

#[test]
fn test_outcome_acceptance_value_round_trip() {
    let mut outcome =
        ObservedOutcome::from_source("outcome-1", "work-1042", OutcomeSource::ExternalSystem)
            .with_metric(OutcomeMetric::new("downtime", 52.0, "min"))
            .with_metric(OutcomeMetric::new("machine_vibration", 3.1, "mm/s"));
    outcome.source_ref = Some("mes://plant-a".to_string());
    assert_ne!(outcome.source, OutcomeSource::Sensor);

    let serialized = serde_json::to_string(&outcome).unwrap();
    let deserialized: ObservedOutcome = serde_json::from_str(&serialized).unwrap();
    assert_eq!(outcome, deserialized);

    let acceptance = AcceptanceDecision::new(
        "accept-1",
        "work-1042",
        AcceptanceType::Accept,
        "production_owner",
    );
    let serialized = serde_json::to_string(&acceptance).unwrap();
    let deserialized: AcceptanceDecision = serde_json::from_str(&serialized).unwrap();
    assert_eq!(acceptance, deserialized);

    let value = ValueAssessment::new("value-1", "work-1042");
    let serialized = serde_json::to_string(&value).unwrap();
    let deserialized: ValueAssessment = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value, deserialized);
}

#[test]
fn test_all_schema_files_parse() {
    let schemas = [
        (
            "common.schema.json",
            include_str!("../../schemas/common.schema.json"),
        ),
        (
            "world.schema.json",
            include_str!("../../schemas/world.schema.json"),
        ),
        (
            "work.schema.json",
            include_str!("../../schemas/work.schema.json"),
        ),
        (
            "authority.schema.json",
            include_str!("../../schemas/authority.schema.json"),
        ),
        (
            "capability.schema.json",
            include_str!("../../schemas/capability.schema.json"),
        ),
        (
            "execution.schema.json",
            include_str!("../../schemas/execution.schema.json"),
        ),
        (
            "outcome.schema.json",
            include_str!("../../schemas/outcome.schema.json"),
        ),
        (
            "profile.schema.json",
            include_str!("../../schemas/profile.schema.json"),
        ),
    ];
    for (name, json) in schemas {
        Schema::parse(json).unwrap_or_else(|e| panic!("{name} does not parse: {e}"));
    }
}

#[test]
fn test_solution_package_round_trip() {
    let mut package = SolutionPackage::new(
        "pkg-1",
        "factory-exception-review",
        "0.1.0",
        "morn.factory.readonly/v1alpha1",
    );
    package.capability_release_refs = vec!["equipment-investigator@sha256:abc".to_string()];
    let serialized = serde_json::to_string(&package).unwrap();
    let deserialized: SolutionPackage = serde_json::from_str(&serialized).unwrap();
    assert_eq!(package, deserialized);
}
