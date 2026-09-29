//! artifact2capability — Artifact2Capability MVP (Phase 12).
//!
//! Proves (Goal §16 / master §17):
//! - two real compilers (OpenAPI2Capability / Repo2Capability) extract from
//!   REAL artifacts deterministically — no LLM anywhere in the pipeline;
//! - failure is a structured, legal state: BLOCKED_MISSING_ARTIFACT,
//!   RESOURCE_ONLY, NON_GENERALIZABLE, WRAPPED_BUT_UNVERIFIED, FAILED_TEST;
//! - a candidate that passes a REAL test becomes a QualifiedCandidate — and
//!   even then it is NOT released: release requires a qualification record
//!   (Phase 10 gate);
//! - every candidate manifest is traceable to its source artifact.

use morn::control::controllers::resolve_loop::QualificationController;
use morn::control::store::ControlPlaneStore;
use morn::studio::artifact2capability::{
    Artifact2CapabilityOutcome, OpenApi2Capability, Repo2Capability,
};
use morn::work::{QualificationRecord, QualificationStatus};
use std::path::PathBuf;

const OPENAPI_DOC: &str = r#"{
  "openapi": "3.0.0",
  "info": { "title": "CMMS", "version": "1.2.0" },
  "paths": {
    "/work-orders": {
      "post": { "operationId": "createWorkOrder" },
      "get": { "operationId": "listWorkOrders" }
    }
  }
}"#;

fn temp_repo() -> PathBuf {
    let dir = std::env::temp_dir().join("morn_a2c_repo");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    dir
}

#[test]
fn g_openapi_compiler_wraps_real_artifacts() {
    let compiler = OpenApi2Capability;
    let outcome = compiler.compile(Some(OPENAPI_DOC), "/work-orders", "post");

    match outcome {
        Artifact2CapabilityOutcome::WrappedButUnverified { source, candidate } => {
            assert_eq!(source, "openapi:/work-orders");
            assert_eq!(candidate.name, "openapi-createWorkOrder");
            assert!(candidate
                .provides
                .contains(&"api.createWorkOrder".to_string()));
            // The wrapper is traceable to the source artifact.
            assert_eq!(candidate.source_ref, "openapi#/work-orders.post");
            assert!(candidate
                .interface_json
                .as_deref()
                .unwrap_or("")
                .contains("\"method\":\"post\""));
        }
        other => panic!("expected WrappedButUnverified, got {other:?}"),
    }
}

#[test]
fn g_openapi_compiler_structured_failures() {
    let compiler = OpenApi2Capability;

    // Missing document.
    let outcome = compiler.compile(None, "/work-orders", "post");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::BlockedMissingArtifact { .. }
    ));

    // Unparseable JSON: RESOURCE_ONLY (doc exists, contributes nothing).
    let outcome = compiler.compile(Some("not json"), "/work-orders", "post");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::ResourceOnly { .. }
    ));

    // A path with no such operation: NON_GENERALIZABLE.
    let outcome = compiler.compile(Some(OPENAPI_DOC), "/nonexistent", "post");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::NonGeneralizable { .. }
    ));
}

#[test]
fn g_openapi_real_test_upgrades_to_qualified_candidate() {
    let compiler = OpenApi2Capability;
    let wrapped = compiler.compile(Some(OPENAPI_DOC), "/work-orders", "get");

    // A REAL test (checks the wrapper's actual interface contract) passes.
    let outcome = compiler.verify(wrapped, |candidate| {
        let interface = candidate.interface_json.as_deref().unwrap_or("");
        if interface.contains("\"method\":\"get\"") && interface.contains("/work-orders") {
            Ok(())
        } else {
            Err("wrapper interface does not match the source operation".to_string())
        }
    });
    match outcome {
        Artifact2CapabilityOutcome::QualifiedCandidate { source, candidate } => {
            assert_eq!(source, "openapi:/work-orders");
            assert_eq!(candidate.name, "openapi-listWorkOrders");
        }
        other => panic!("expected QualifiedCandidate, got {other:?}"),
    }
}

#[test]
fn g_openapi_real_test_failure_is_structured() {
    let compiler = OpenApi2Capability;
    let wrapped = compiler.compile(Some(OPENAPI_DOC), "/work-orders", "post");

    // A REAL test runs and FAILS: FAILED_TEST with the test detail.
    let outcome = compiler.verify(wrapped, |_candidate| {
        Err("assertion failed: create-order returned 500".to_string())
    });
    match outcome {
        Artifact2CapabilityOutcome::FailedTest {
            source,
            test_detail,
            ..
        } => {
            assert_eq!(source, "openapi:/work-orders");
            assert!(test_detail.contains("500"));
        }
        other => panic!("expected FailedTest, got {other:?}"),
    }
}

#[test]
fn g_repo_compiler_discovers_real_entrypoints() {
    let compiler = Repo2Capability;
    let repo = temp_repo();

    let outcome = compiler.compile(Some(&repo), "demo-crate");
    match outcome {
        Artifact2CapabilityOutcome::WrappedButUnverified { candidate, .. } => {
            assert_eq!(candidate.name, "repo-demo-crate");
            assert_eq!(candidate.digest, "sha256:repo:demo-crate:Cargo.toml");
            assert!(candidate.source_ref.contains("Cargo.toml"));
        }
        other => panic!("expected WrappedButUnverified, got {other:?}"),
    }
}

#[test]
fn g_repo_compiler_structured_failures() {
    let compiler = Repo2Capability;

    // Missing path.
    let outcome = compiler.compile(None, "demo");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::BlockedMissingArtifact { .. }
    ));

    // Non-directory path.
    let outcome = compiler.compile(Some(std::path::Path::new("definitely/not/a/dir")), "demo");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::BlockedMissingArtifact { .. }
    ));

    // A directory with no entrypoint: RESOURCE_ONLY.
    let empty = std::env::temp_dir().join("morn_a2c_empty");
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();
    let outcome = compiler.compile(Some(&empty), "empty-dir");
    assert!(matches!(
        outcome,
        Artifact2CapabilityOutcome::ResourceOnly { .. }
    ));
}

#[test]
fn g_qualified_candidate_requires_qualification_record_before_release() {
    // SAFETY: even a QualifiedCandidate is NOT released — release requires a
    // qualification record through the Phase 10 gate. The compiler NEVER
    // writes a release on its own.
    let compiler = OpenApi2Capability;
    let wrapped = compiler.compile(Some(OPENAPI_DOC), "/work-orders", "post");
    let qualified = compiler.verify(wrapped, |_| Ok(()));

    let candidate = match qualified {
        Artifact2CapabilityOutcome::QualifiedCandidate { candidate, .. } => candidate,
        other => panic!("expected QualifiedCandidate, got {other:?}"),
    };

    // Before the qualification record: the strict gate blocks.
    let store = ControlPlaneStore::in_memory().expect("store");
    let allowed =
        QualificationController::capability_qualified_for_strict(&store, &candidate.digest)
            .unwrap();
    assert!(!allowed, "no qualification record → not releasable");

    // Record a qualification (evaluator identity required — the compiler is
    // not the evaluator).
    let record = QualificationRecord {
        id: "q-a2c".to_string(),
        capability_digest: candidate.digest.clone(),
        runtime_provider: Some("container".to_string()),
        execution_environment: Some("env-1".to_string()),
        environment_digest: Some("sha256:env".to_string()),
        input_scope: Some("work-orders".to_string()),
        expected_outputs: vec!["operation response".to_string()],
        failure_modes: vec![],
        cost_evidence: None,
        evaluator_identity: Some("qualification-runner".to_string()),
        status: QualificationStatus::Passed,
        version: Some("1".to_string()),
        qualified_at: Some("2026-09-28T00:00:00Z".to_string()),
        limitations: None,
    };
    QualificationController.record(&store, &record).unwrap();
    let allowed =
        QualificationController::capability_qualified_for_strict(&store, &candidate.digest)
            .unwrap();
    assert!(
        allowed,
        "PASSED qualification record enables the strict gate"
    );
}
