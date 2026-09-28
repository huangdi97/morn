//! capability_supply_chain — Capability Supply Chain (Phase 10, G8 Qualification Gate).
//!
//! Proves (Goal §14 / master §16):
//! - the four capability states are separated: Declared != Observed !=
//!   Qualified != Admitted — a self-description is not a qualification and a
//!   qualification is not a site admission;
//! - the asset lifecycle is separate from site admission (SITE_BOUND is an
//!   admission stage, not a build stage);
//! - the strict-profile gate: a capability enters a strict-profile critical
//!   binding ONLY with a PASSED, non-expired qualification;
//! - expired / failed / absent qualifications block new bindings.

use morn::control::controllers::resolve_loop::QualificationController;
use morn::control::store::ControlPlaneStore;
use morn::work::{
    CapabilityLifecycle, CapabilityManifest, CapabilitySpec, CapabilityState, QualificationRecord,
    QualificationStatus,
};

fn manifest(name: &str, version: &str, digest: &str) -> CapabilityManifest {
    CapabilityManifest {
        name: name.to_string(),
        version: version.to_string(),
        digest: digest.to_string(),
        api_version: "capability.morn.io/v1alpha1".to_string(),
        kind: Some("agent".to_string()),
        spec: CapabilitySpec {
            provides: vec!["equipment.anomaly.investigate".to_string()],
            semantic: None,
            interfaces: None,
            side_effects: None,
            idempotency: false,
            compensation: None,
            execution: None,
            authority: None,
            economics: None,
            provenance: None,
        },
    }
}

fn qualification(id: &str, digest: &str, status: QualificationStatus) -> QualificationRecord {
    let mut record = QualificationRecord {
        id: id.to_string(),
        capability_digest: digest.to_string(),
        runtime_provider: Some("container".to_string()),
        execution_environment: Some("env-1".to_string()),
        environment_digest: Some("sha256:env".to_string()),
        input_scope: Some("plant-a/CNC-17".to_string()),
        expected_outputs: vec!["hypothesis".to_string()],
        failure_modes: vec!["no_data".to_string()],
        cost_evidence: None,
        evaluator_identity: Some("qualification-runner".to_string()),
        status,
        version: Some("1".to_string()),
        qualified_at: Some("2026-09-28T00:00:00Z".to_string()),
        limitations: None,
    };
    let _ = &mut record;
    record
}

#[test]
fn g8_four_states_are_separated() {
    // The four states are distinct enum values — a self-description (Declared)
    // can never equal a qualification (Qualified) or admission (Admitted).
    let states = [
        CapabilityState::Declared,
        CapabilityState::Observed,
        CapabilityState::Qualified,
        CapabilityState::Admitted,
    ];
    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "capability states must stay separated");
            }
        }
    }

    // A manifest is a DECLARATION only: it carries no qualification or
    // admission state — those are separate records (QualificationRecord /
    // SiteAdmission).
    let m = manifest("equipment-investigator", "1.4.0", "sha256:cap");
    assert_eq!(m.name, "equipment-investigator");
    assert!(!m.digest.is_empty());
    // The manifest has no "qualified" boolean; qualification is evidence.
    let q = qualification("q-1", "sha256:cap", QualificationStatus::Pending);
    assert_ne!(
        q.status,
        QualificationStatus::Passed,
        "pending is not qualified"
    );
}

#[test]
fn g8_lifecycle_separates_asset_from_site() {
    // The asset lifecycle runs DISCOVERED → ... → RELEASED; SITE_BOUND is an
    // admission stage, not a build stage.
    let build_stages = [
        CapabilityLifecycle::Discovered,
        CapabilityLifecycle::Extracted,
        CapabilityLifecycle::Wrapped,
        CapabilityLifecycle::Executable,
        CapabilityLifecycle::Verified,
        CapabilityLifecycle::Qualified,
        CapabilityLifecycle::Released,
    ];
    // Build stages are ordered and do not include SITE_BOUND.
    for stage in build_stages {
        assert_ne!(
            stage,
            CapabilityLifecycle::SiteBound,
            "SITE_BOUND is not a build stage"
        );
    }
    // Deprecated/Revoked are terminal lifecycle states.
    assert_ne!(
        CapabilityLifecycle::Deprecated,
        CapabilityLifecycle::Revoked
    );
}

#[test]
fn g8_strict_profile_gate_blocks_unqualified_capability() {
    // G8: an UNQUALIFIED capability is not allowed into a strict-profile
    // critical binding.
    let store = ControlPlaneStore::in_memory().expect("store");
    let digest = "sha256:unqualified";

    // No qualification at all: blocked.
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(!allowed, "absent qualification blocks the strict gate");

    // A FAILED qualification: blocked.
    QualificationController
        .record(
            &store,
            &qualification("q-fail", digest, QualificationStatus::Failed),
        )
        .unwrap();
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(!allowed, "FAILED qualification blocks the strict gate");

    // A PENDING qualification: blocked.
    QualificationController
        .record(
            &store,
            &qualification("q-pending", digest, QualificationStatus::Pending),
        )
        .unwrap();
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(!allowed, "PENDING qualification blocks the strict gate");
}

#[test]
fn g8_strict_profile_gate_allows_passed_qualification() {
    let store = ControlPlaneStore::in_memory().expect("store");
    let digest = "sha256:qualified";

    // A PASSED qualification enables the strict gate.
    let outcome = QualificationController
        .record(
            &store,
            &qualification("q-pass", digest, QualificationStatus::Passed),
        )
        .unwrap();
    assert!(outcome
        .notes
        .iter()
        .any(|n| n.contains("qualification passed")));
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(allowed, "PASSED qualification enables the strict gate");
}

#[test]
fn g8_expired_qualification_does_not_qualify_new_bindings() {
    let store = ControlPlaneStore::in_memory().expect("store");
    let digest = "sha256:expired";

    // First: PASSED. Then: EXPIRED (the latest qualification wins).
    QualificationController
        .record(
            &store,
            &qualification("q-1", digest, QualificationStatus::Passed),
        )
        .unwrap();
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(allowed);

    QualificationController
        .record(
            &store,
            &qualification("q-2", digest, QualificationStatus::Expired),
        )
        .unwrap();
    let allowed = QualificationController::capability_qualified_for_strict(&store, digest).unwrap();
    assert!(
        !allowed,
        "an EXPIRED qualification does not qualify new bindings"
    );

    // The latest qualification is the expired one (latest by qualified_at).
    let latest = store
        .get_latest_qualification(digest)
        .unwrap()
        .expect("qualification present");
    assert_eq!(latest.0, "\"EXPIRED\"");
}

#[test]
fn g8_qualification_record_round_trips() {
    // The QualificationRecord is a protocol resource: it round-trips through
    // serde with all evidence fields.
    let record = qualification("q-rt", "sha256:rt", QualificationStatus::Passed);
    let json = serde_json::to_string(&record).unwrap();
    let back: QualificationRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back, record);
    assert_eq!(
        back.evaluator_identity.as_deref(),
        Some("qualification-runner")
    );
    assert_eq!(back.input_scope.as_deref(), Some("plant-a/CNC-17"));
}
