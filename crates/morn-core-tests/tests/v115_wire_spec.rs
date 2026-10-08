use morn_capability::CapabilityRecord;
use morn_integration::ConnectorReceipt;
use morn_kernel::EventEnvelope;
use morn_profile::DomainProfile;
use morn_runtime::{
    ActionAttempt, BoundAuthorityDecision, ExecutionBinding, ExecutionManifest,
    ReconciliationRecord,
};
use morn_work::{control::WorkResource, AcceptanceDecision};
use morn_world::ObservedOutcome;
use serde_json::Value;

const PROTOCOL: &str = include_str!("../../../spec/v11.5/protocol.json");
const SCHEMA: &str = include_str!("../../../spec/v11.5/morn-protocol.schema.json");
const WORK: &str = include_str!("../../../spec/v11.5/examples/work-resource.json");
const CAPABILITY: &str = include_str!("../../../spec/v11.5/examples/capability-record.json");
const BINDING: &str = include_str!("../../../spec/v11.5/examples/execution-binding.json");
const ATTEMPT: &str = include_str!("../../../spec/v11.5/examples/action-attempt.json");
const PROFILE: &str = include_str!("../../../spec/v11.5/examples/factory-profile.json");
const EVENT: &str = include_str!("../../../spec/v11.5/examples/event-envelope.json");
const AUTHORITY: &str = include_str!("../../../spec/v11.5/examples/authority-decision.json");
const RECEIPT: &str = include_str!("../../../spec/v11.5/examples/connector-receipt.json");
const RECONCILIATION: &str =
    include_str!("../../../spec/v11.5/examples/reconciliation-record.json");
const OUTCOME: &str = include_str!("../../../spec/v11.5/examples/observed-outcome.json");
const ACCEPTANCE: &str = include_str!("../../../spec/v11.5/examples/acceptance-decision.json");
const EXECUTION_MANIFEST: &str =
    include_str!("../../../spec/v11.5/examples/execution-manifest.json");
const SEMANTIC_VECTORS: &str =
    include_str!("../../../spec/v11.5/conformance/semantic-vectors.json");

#[test]
fn published_protocol_manifest_names_all_v115_semantic_slots() {
    let protocol: Value = serde_json::from_str(PROTOCOL).unwrap();
    assert_eq!(protocol["version"], "11.5.0");
    assert_eq!(
        protocol["schemaDialect"],
        "https://json-schema.org/draft/2020-12/schema"
    );

    let slots = protocol["semanticSlots"].as_array().unwrap();
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
        assert!(slots.iter().any(|slot| slot == required), "{required}");
    }
}

#[test]
fn schema_bundle_is_parseable_and_covers_cross_runtime_resources() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    let defs = schema["$defs"].as_object().unwrap();
    for required in [
        "WorkResource",
        "CapabilityRecord",
        "BoundAuthorityDecision",
        "ExecutionBinding",
        "ActionAttempt",
        "ConnectorReceipt",
        "ReconciliationRecord",
        "ObservedOutcome",
        "AcceptanceDecision",
        "ExecutionManifest",
        "DomainProfile",
        "EventEnvelope",
    ] {
        assert!(defs.contains_key(required), "{required}");
    }
}

#[test]
fn canonical_wire_examples_deserialize_into_reference_implementation() {
    let work: WorkResource = serde_json::from_str(WORK).unwrap();
    assert_eq!(work.spec.profile_ref, "morn.factory.readonly@1.0.0");
    assert_eq!(work.spec.protocol_version.major, 11);

    let capability: CapabilityRecord = serde_json::from_str(CAPABILITY).unwrap();
    capability.manifest.validate_governance().unwrap();
    assert_eq!(capability.admission_refs.len(), 1);

    let binding: ExecutionBinding = serde_json::from_str(BINDING).unwrap();
    assert_eq!(binding.work_id, work.id);
    assert_eq!(binding.work_generation, work.generation);

    let attempt: ActionAttempt = serde_json::from_str(ATTEMPT).unwrap();
    assert_eq!(attempt.binding_id.as_str(), "binding-1042-cmms");
    assert_eq!(format!("{:?}", attempt.state), "OutcomeUnknown");

    let profile: DomainProfile = serde_json::from_str(PROFILE).unwrap();
    profile.validate().unwrap();
    assert!(profile.forbids("ProductionWrite"));

    let authority: BoundAuthorityDecision = serde_json::from_str(AUTHORITY).unwrap();
    assert!(authority.decision.allowed);
    assert_eq!(authority.request.work_ref.as_deref(), Some("work-1042"));

    let receipt: ConnectorReceipt = serde_json::from_str(RECEIPT).unwrap();
    assert!(receipt.ok);
    assert_eq!(receipt.external_id.as_deref(), Some("MO-88273"));

    let reconciliation: ReconciliationRecord = serde_json::from_str(RECONCILIATION).unwrap();
    assert_eq!(reconciliation.after, morn_runtime::AttemptState::Observed);
    assert_eq!(
        reconciliation.observation.external_ref.as_deref(),
        Some("MO-88273")
    );

    let outcome: ObservedOutcome = serde_json::from_str(OUTCOME).unwrap();
    assert!(outcome.is_source_grounded());
    assert_eq!(outcome.work_package_id, work.id);

    let acceptance: AcceptanceDecision = serde_json::from_str(ACCEPTANCE).unwrap();
    assert!(acceptance.is_final_acceptance());
    assert_eq!(acceptance.work_package_id, work.id);

    let manifest: ExecutionManifest = serde_json::from_str(EXECUTION_MANIFEST).unwrap();
    assert_eq!(manifest.protocol_version, "11.5.0");
    assert_eq!(manifest.work_ref, work.id.to_string());
    assert_eq!(manifest.execution_binding_ref, binding.id.to_string());

    let event: EventEnvelope = serde_json::from_str(EVENT).unwrap();
    assert_eq!(event.specversion, "1.0");
    assert_eq!(event.event_type, "io.morn.action.receipt.v1");
    assert_eq!(
        event.extensions.get("mornworkid").map(String::as_str),
        Some("work-1042")
    );
}

#[test]
fn published_examples_do_not_smuggle_executor_success_into_acceptance() {
    let work: WorkResource = serde_json::from_str(WORK).unwrap();
    let event: EventEnvelope = serde_json::from_str(EVENT).unwrap();

    assert_ne!(work.status.phase, morn_work::control::WorkPhase::Accepted);
    assert!(event.data.get("accepted").is_none());
}

#[test]
fn executor_receipt_external_receipt_outcome_and_acceptance_are_distinct() {
    use morn_harness::ExecutionReceipt;
    use morn_kernel::ids::WorkspaceId;

    let executor_receipt = ExecutionReceipt::new(WorkspaceId::generate(), "dsh-session-1");
    let external_receipt: ConnectorReceipt = serde_json::from_str(RECEIPT).unwrap();
    let outcome: ObservedOutcome = serde_json::from_str(OUTCOME).unwrap();
    let acceptance: AcceptanceDecision = serde_json::from_str(ACCEPTANCE).unwrap();

    assert_eq!(executor_receipt.outcome, "running");
    assert_eq!(external_receipt.external_id.as_deref(), Some("MO-88273"));
    assert!(outcome.is_source_grounded());
    assert!(acceptance.is_final_acceptance());

    // Distinct records deliberately use different identities and evidence roles.
    assert_ne!(
        executor_receipt.id.to_string(),
        external_receipt.id.to_string()
    );
    assert_ne!(external_receipt.id.to_string(), outcome.id.to_string());
    assert_ne!(outcome.id.to_string(), acceptance.id.to_string());
}

#[test]
fn published_black_box_vectors_cover_every_protocol_invariant() {
    let snapshot = morn_kernel::protocol::ProtocolSnapshot::v11_5();
    let vectors: Value = serde_json::from_str(SEMANTIC_VECTORS).unwrap();
    assert_eq!(vectors["protocol"], "11.5.0");

    let published: std::collections::BTreeSet<String> = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|vector| vector["required"].as_bool() == Some(true))
        .filter_map(|vector| vector["id"].as_str())
        .map(str::to_string)
        .collect();
    let required: std::collections::BTreeSet<String> =
        snapshot.invariant_ids().into_iter().collect();

    assert_eq!(published, required);
}
