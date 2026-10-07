//! v11.5 capability supply-chain persistence and non-destructive lifecycle.

use std::collections::BTreeSet;

use morn_app::AppState;
use morn_assurance::{
    CapabilityDistributionReleaseStatus, QualificationEvidence, SiteAdmissionStatus,
    StrictQualificationRequest,
};
use morn_foundry::{ArtifactCompiler, ArtifactKind, ArtifactSource, OpenApiJsonCompiler};
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
            inner
                .v115_admission
                .record_release(
                    &mut inner.v115_capabilities[index],
                    &qualification,
                    format!("oci://registry.example/morn/cmms-read@sha256:{}", "c".repeat(64)),
                    format!("sha256:{}", "c".repeat(64)),
                    Some("sigstore://rekor/cmms-read".to_string()),
                    Some("slsa://provenance/cmms-read".to_string()),
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
