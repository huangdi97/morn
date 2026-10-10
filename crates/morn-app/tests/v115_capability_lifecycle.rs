//! v11.5 capability supply-chain persistence and non-destructive lifecycle.

use std::collections::BTreeSet;

use morn_app::AppState;
use morn_assurance::{
    CapabilityDistributionReleaseStatus, QualificationEvidence, SiteAdmissionStatus,
    StrictQualificationRequest,
};
use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, OpenApiJsonCompiler};
use morn_package::SupplyChainVerificationEvidence;
use morn_profile::{evaluate_profile, ConformanceEvidence, DomainProfile, RequirementLevel};

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "morn_v115_capability_{name}_{}.db",
        uuid::Uuid::new_v4()
    ));
    path.to_string_lossy().to_string()
}

fn factory_conformance(profile: &DomainProfile) -> morn_profile::ConformanceReport {
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
fn capability_lifecycle_survives_restart_and_revocation_keeps_history() {
    let db = temp_db("restart");
    let manifest_id;
    let release_id;

    {
        let state = AppState::new(&db).unwrap();
        let source = ArtifactSource {
            kind: ArtifactKind::OpenApi,
            name: "cmms-read".to_string(),
            source_ref: "repo://cmms/openapi.json".to_string(),
            source_digest: Some(format!("sha256:{}", "a".repeat(64))),
            content: r#"{
                "openapi":"3.1.0",
                "paths":{
                    "/orders":{"get":{}}
                }
            }"#
            .to_string(),
        };
        let candidate = OpenApiJsonCompiler.compile(&source).unwrap();

        let mut guard = state.lock();
        guard.v115_capabilities.push(candidate.record);
        let index = 0;
        let observation = {
            let inner = &mut *guard;
            inner
                .v115_admission
                .observe(
                    &mut inner.v115_capabilities[index],
                    vec!["eval://cmms-read/observation".to_string()],
                    "evaluator://independent",
                )
                .unwrap()
        };
        assert!(!observation.evidence_refs.is_empty());

        let qualification = {
            let inner = &mut *guard;
            inner
                .v115_admission
                .qualify_with_evidence(
                    &mut inner.v115_capabilities[index],
                    StrictQualificationRequest {
                        candidate_ref: "candidate://cmms-read@1".to_string(),
                        decision_ref: "decision://cmms-read-qualification".to_string(),
                        evidence_refs: vec!["eval://cmms-read/full".to_string()],
                        qualification_evidence: QualificationEvidence {
                            test_suite_refs: vec!["suite://cmms-read".to_string()],
                            environment_digest: Some(format!("sha256:{}", "b".repeat(64))),
                            expected_properties: vec!["read-only".to_string()],
                            evaluator_identity: Some("evaluator://independent".to_string()),
                            ..Default::default()
                        },
                        context_of_use: vec!["factory-readonly".to_string()],
                        valid_until: None,
                    },
                )
                .unwrap()
        };

        let release = {
            let inner = &mut *guard;
            let digest = format!("sha256:{}", "c".repeat(64));
            inner
                .v115_admission
                .record_verified_release(
                    &mut inner.v115_capabilities[index],
                    &qualification,
                    format!("oci://registry.example/morn/cmms-read@{digest}"),
                    digest.clone(),
                    "sigstore://rekor/cmms-read",
                    "slsa://provenance/cmms-read",
                    SupplyChainVerificationEvidence {
                        subject_digest: digest,
                        verifier_ref: "fixture://release-verifier".to_string(),
                        signature_verified: true,
                        provenance_verified: true,
                        evidence_refs: vec!["fixture://release-verifier/proof".to_string()],
                    },
                )
                .unwrap()
        };
        release_id = release.id.clone();

        let profile = DomainProfile::factory_readonly_v1();
        let report = factory_conformance(&profile);
        assert!(report.passed);
        {
            let inner = &mut *guard;
            inner
                .v115_admission
                .admit(
                    &mut inner.v115_capabilities[index],
                    &qualification,
                    "plant-a",
                    profile.canonical_ref(),
                    &report,
                    "site-owner",
                )
                .unwrap();
        }

        manifest_id = guard.v115_capabilities[index].manifest.id.clone();
        assert_eq!(guard.v115_capabilities[index].admission_refs.len(), 1);
        guard.persist_all().unwrap();
    }

    {
        let state = AppState::new(&db).unwrap();
        let mut guard = state.lock();
        assert_eq!(guard.v115_capabilities.len(), 1);
        assert_eq!(guard.v115_capabilities[0].manifest.id, manifest_id);
        assert_eq!(guard.v115_admission.qualifications.len(), 1);
        assert_eq!(guard.v115_admission.releases.len(), 1);
        assert_eq!(guard.v115_admission.admissions.len(), 1);
        assert!(guard.v115_admission.events.len() >= 4);

        let index = 0;
        {
            let inner = &mut *guard;
            inner
                .v115_admission
                .revoke_release(&mut inner.v115_capabilities[index], &release_id)
                .unwrap();
        }
        assert_eq!(
            guard.v115_admission.releases[0].status,
            CapabilityDistributionReleaseStatus::Revoked
        );
        assert_eq!(
            guard.v115_admission.admissions[0].status,
            SiteAdmissionStatus::Suspended
        );
        assert!(guard.v115_capabilities[0].admission_refs.is_empty());
        guard.persist_all().unwrap();
    }

    {
        let state = AppState::new(&db).unwrap();
        let guard = state.lock();
        assert_eq!(
            guard.v115_admission.releases[0].status,
            CapabilityDistributionReleaseStatus::Revoked
        );
        assert_eq!(
            guard.v115_admission.admissions[0].status,
            SiteAdmissionStatus::Suspended
        );
        assert!(guard
            .v115_admission
            .events
            .iter()
            .any(|event| event.event_type == "ReleaseRevoked"));
        assert!(guard
            .v115_admission
            .events
            .iter()
            .any(|event| event.event_type == "SiteAdmitted"));
    }

    let _ = std::fs::remove_file(db);
}

#[test]
fn v115_capability_hydration_keeps_tenant_manifest_and_historical_event_isolation() {
    use morn_assurance::{CapabilityLifecycleEvent, CapabilityLifecycleEventId};
    use morn_capability::{CapabilityKind, CapabilityManifest, CapabilityRecord, EffectClass};
    use morn_kernel::ids::CapabilityId;
    use morn_kernel::time::Timestamp;

    let db = temp_db("workspace_isolation");
    let own_manifest_id;
    {
        let state = AppState::new(&db).unwrap();
        let guard = state.lock();

        let make_capability = |name: &str| {
            let mut manifest = CapabilityManifest::new(
                CapabilityId::generate_with("cap"),
                name,
                "fixture-provider",
                CapabilityKind::Program,
                EffectClass::E0LifecycleReversible,
            );
            manifest.provenance.source_ref = format!("repo://{name}");
            CapabilityRecord::new(manifest)
        };
        let own = make_capability("owned");
        let foreign = make_capability("foreign");
        own_manifest_id = own.manifest.id.clone();

        for (cap, workspace_id) in [
            (&own, guard.workspace.id.as_str()),
            (&foreign, "other-workspace"),
        ] {
            guard
                .store
                .save_record(
                    "capability_record_v115",
                    cap.manifest.id.as_str(),
                    workspace_id,
                    cap.manifest.declared_at.millis(),
                    cap,
                )
                .unwrap();

            let event = CapabilityLifecycleEvent {
                id: CapabilityLifecycleEventId::generate_with("event"),
                manifest_id: cap.manifest.id.clone(),
                entity_ref: cap.manifest.id.to_string(),
                event_type: "Observed".to_string(),
                reason: "fixture".to_string(),
                actor_ref: "test".to_string(),
                created_at: Timestamp::now(),
            };
            guard
                .store
                .save_record_immutable(
                    "capability_lifecycle_event_v115",
                    event.id.as_str(),
                    "",
                    event.created_at.millis(),
                    &event,
                )
                .unwrap();
        }
    }

    let restarted = AppState::new(&db).unwrap();
    let guard = restarted.lock();
    assert_eq!(guard.v115_capabilities.len(), 1);
    assert_eq!(guard.v115_capabilities[0].manifest.id, own_manifest_id);
    assert_eq!(guard.v115_admission.events.len(), 1);
    assert_eq!(guard.v115_admission.events[0].manifest_id, own_manifest_id);
    drop(guard);
    let _ = std::fs::remove_file(db);
}
