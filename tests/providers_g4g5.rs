//! providers_g4g5 —Harness contract tests + Execution Plane (G4/G5).
//!
//! All HarnessProviders run the SAME contract test (Goal §23.2). The fake
//! provider passes the full contract (control-plane verification); DSH/Pi
//! skeletons pass the honest-blocker contract and stay BLOCKED_EXTERNAL —//! never a fake PASS.
//!
//! Also proves (Goal §8.3—.5): deterministic solver without Agent; human
//! assignment/approval/escalation; Fake CMMS connector with idempotency +
//! intentional timeout-after-commit for reconciliation.

use morn::providers::connector::{ConnectorProvider, FakeCmmsConnector};
use morn::providers::credential::{CredentialProvider, EnvCredentialProvider};
use morn::providers::harness::{
    HarnessCapabilities, HarnessEvent, HarnessHealth, HarnessInput, HarnessProvider,
    HarnessSession, HarnessUsage,
};
use morn::providers::harness_dsh::DshHarnessProvider;
use morn::providers::harness_fake::FakeHarnessProvider;
use morn::providers::harness_pi::PiHarnessProvider;
use morn::providers::human::{
    HumanAssignment, HumanDecision, HumanDecisionType, HumanProvider, HumanTaskKind,
    ReferenceHumanProvider,
};
use morn::providers::solver::{Assignment, ReferenceCpsatSolver, SolverProvider};
use morn::work::ExecutionBinding;

fn binding() -> ExecutionBinding {
    ExecutionBinding::new("binding-g4", "work-g4", "sha256:g4", "fake-provider")
}

/// The shared HarnessProvider contract test. Providers that CAN reach a
/// backend must pass every operation; skeletons that cannot must fail with
/// typed blockers on every operation (honest, no partial fake success).
fn assert_harness_contract_blocker(provider: &dyn HarnessProvider) {
    let caps: HarnessCapabilities = provider.capabilities();
    assert!(
        !caps.real_integration_available,
        "{} must honestly report the integration as unavailable",
        caps.provider_id
    );
    assert!(
        caps.integration_blocker.is_some(),
        "{} must state what is missing (REAL_PROVIDER_INTEGRATION_BLOCKED)",
        caps.provider_id
    );
    assert_eq!(provider.health(), HarnessHealth::ExternalBlocked);

    let session_result = provider.start_session(&binding());
    assert!(
        session_result.is_err(),
        "{} must not fake a session",
        caps.provider_id
    );
    assert!(
        session_result
            .unwrap_err()
            .contains("REAL_PROVIDER_INTEGRATION_BLOCKED"),
        "blocker must be honest"
    );
}

/// The full contract test for a provider that CAN run sessions (the fake).
fn assert_harness_contract_full(provider: &dyn HarnessProvider, expected_id: &str) {
    let caps = provider.capabilities();
    assert_eq!(caps.provider_id, expected_id);
    assert_eq!(provider.health(), HarnessHealth::Healthy);

    // start_session under a binding.
    let session: HarnessSession = provider.start_session(&binding()).expect("session starts");
    assert_eq!(session.binding_ref, "binding-g4");
    assert_eq!(session.provider_id, expected_id);

    // submit: deterministic candidate + completed events.
    let candidate = provider
        .submit(
            &session,
            HarnessInput {
                prompt: "investigate CNC-17".to_string(),
                context_json: None,
            },
        )
        .expect("submit works");
    let events: Vec<HarnessEvent> = provider.get_events(&session, 0).expect("events");
    assert!(
        events.len() >= 3,
        "started + candidate + completed recorded"
    );
    assert!(events.iter().any(|e| e
        .payload_json
        .as_deref()
        .unwrap_or("")
        .contains("fake-candidate:investigate CNC-17")));
    assert!(candidate.seq > 0);

    // checkpoint + usage.
    let checkpoint = provider.checkpoint(&session).expect("checkpoint");
    assert!(checkpoint.contains("event_count"));
    let usage: HarnessUsage = provider.get_usage(&session).expect("usage");
    assert_eq!(usage.event_count, events.len());

    // resume_session.
    let resumed = provider
        .resume_session(&session.session_ref)
        .expect("resume");
    assert_eq!(resumed.session_ref, session.session_ref);

    // cancel →submit refused.
    provider.cancel(&session).expect("cancel");
    assert!(provider
        .submit(
            &session,
            HarnessInput {
                prompt: "x".to_string(),
                context_json: None
            }
        )
        .is_err());

    provider.close().expect("close");
}

#[test]
fn g4_dsh_skeleton_honest_blocker_contract() {
    // No credential, no endpoint: honest ExternalBlocked.
    let credentials = EnvCredentialProvider::new();
    let dsh = DshHarnessProvider::new(std::sync::Arc::new(credentials), None);
    assert_harness_contract_blocker(&dsh);

    // Credential present but no endpoint: still blocked, no fake session.
    let credentials = EnvCredentialProvider::new();
    credentials.register("dsh.api-key", "sk-test-000".to_string());
    let dsh = DshHarnessProvider::new(std::sync::Arc::new(credentials), None);
    assert!(dsh
        .capabilities()
        .integration_blocker
        .unwrap()
        .contains("no DSH endpoint"));
    assert!(dsh.start_session(&binding()).is_err());
}

#[test]
fn g4_fake_provider_full_contract() {
    let fake = FakeHarnessProvider::new("fake-dsh");
    assert_harness_contract_full(&fake, "fake-dsh");
}

#[test]
fn g5_pi_skeleton_honest_blocker_contract() {
    let credentials = EnvCredentialProvider::new();
    let pi = PiHarnessProvider::new(std::sync::Arc::new(credentials), None);
    assert_harness_contract_blocker(&pi);
}

#[test]
fn g5_second_harness_same_contract() {
    // Same Work contract, same capability intent, different harness: the fake
    // proves the contract is harness-neutral at the code level. DSH/Pi real
    // swap stays BLOCKED_EXTERNAL until real providers exist.
    let fake_a = FakeHarnessProvider::new("fake-dsh");
    let fake_b = FakeHarnessProvider::new("fake-pi");
    assert_harness_contract_full(&fake_a, "fake-dsh");
    assert_harness_contract_full(&fake_b, "fake-pi");
    // Both implement the same trait object.
    let providers: Vec<Box<dyn HarnessProvider>> = vec![Box::new(fake_a), Box::new(fake_b)];
    assert_eq!(providers.len(), 2);
    for provider in &providers {
        assert!(provider.capabilities().provider_id.starts_with("fake-"));
    }
}

#[test]
fn g5_credential_provider_boundary() {
    // Secrets stay inside the provider; the fingerprint is safe to log.
    let credentials = EnvCredentialProvider::new();
    credentials.register("dsh.api-key", "sk-secret-value-123".to_string());
    let resolved = credentials
        .credential_for("dsh.api-key")
        .expect("resolve")
        .expect("credential");
    assert_eq!(resolved, "sk-secret-value-123");

    // Unknown scope: honest None.
    assert!(credentials
        .credential_for("unknown.scope")
        .expect("resolve")
        .is_none());

    // Fingerprint redacts.
    let fingerprint = EnvCredentialProvider::fingerprint("sk-secret-value-123");
    assert!(fingerprint.starts_with("sk-s"));
    assert!(!fingerprint.contains("secret-value"));
}

#[test]
fn g5_solver_deterministic_without_agent() {
    // Workcell does not require an Agent: the deterministic solver produces
    // the four canonical candidates with no LLM and no Agent.
    let solver = ReferenceCpsatSolver::new();
    let output = solver
        .solve(&Assignment {
            work_ref: "work-1042".to_string(),
            asset: "CNC-17".to_string(),
            affected_order: Some("WO-84729".to_string()),
        })
        .expect("solve works");
    assert_eq!(output.candidates.len(), 4);
    assert_eq!(output.candidates[0].label, "continue_2h_then_maintenance");
    assert_eq!(output.candidates[1].label, "immediate_stop_and_reroute");
    assert_eq!(output.candidates[2].label, "split_batch");
    assert_eq!(output.candidates[3].label, "no_action");
    assert!(output
        .constraints_checked
        .contains(&"no_plc_write".to_string()));

    // Deterministic: same input →same output.
    let again = solver
        .solve(&Assignment {
            work_ref: "work-1042".to_string(),
            asset: "CNC-17".to_string(),
            affected_order: Some("WO-84729".to_string()),
        })
        .expect("solve works");
    assert_eq!(output, again);

    // Invalid input fails typed.
    assert!(solver
        .solve(&Assignment {
            work_ref: String::new(),
            asset: "CNC-17".to_string(),
            affected_order: None,
        })
        .is_err());
}

#[test]
fn g5_human_provider_full_path() {
    let provider = ReferenceHumanProvider::new();

    // Assignment routed to a role.
    let assignment_id = provider
        .assign(HumanAssignment {
            id: "assign-1".to_string(),
            work_ref: "work-1042".to_string(),
            kind: HumanTaskKind::Approval,
            role: "production-owner".to_string(),
            question: "Candidate B: immediate stop + reroute?".to_string(),
            evidence_refs_json: None,
            created_at: morn::work::now_iso(),
        })
        .expect("assign works");
    assert_eq!(assignment_id, "assign-1");

    // Approval decision.
    provider
        .record_decision(HumanDecision::new(
            "decision-1",
            "assign-1",
            HumanDecisionType::Approve,
            "production_owner",
        ))
        .expect("decision recorded");

    // A second decision for the same assignment is rejected (decide once).
    assert!(provider
        .record_decision(HumanDecision::new(
            "decision-2",
            "assign-1",
            HumanDecisionType::Reject,
            "production_owner",
        ))
        .is_err());

    // Escalation path.
    let assignment_id = provider
        .assign(HumanAssignment {
            id: "assign-2".to_string(),
            work_ref: "work-1042".to_string(),
            kind: HumanTaskKind::Acceptance,
            role: "quality-owner".to_string(),
            question: "Accept outcome?".to_string(),
            evidence_refs_json: None,
            created_at: morn::work::now_iso(),
        })
        .expect("assign");
    let escalation = provider
        .escalate(&assignment_id, "SLA expired")
        .expect("escalate");
    assert_eq!(escalation.decision, HumanDecisionType::Escalate);

    // Unknown assignment fails typed.
    assert!(provider
        .record_decision(HumanDecision::new(
            "d",
            "assign-unknown",
            HumanDecisionType::Approve,
            "x"
        ))
        .is_err());
}

#[test]
fn g5_fake_cmms_idempotent_and_no_duplicate() {
    let connector = FakeCmmsConnector::new();

    let receipt = connector
        .create_order("work-1042:cmms.create-order", r#"{"asset":"CNC-17"}"#)
        .expect("order created");
    assert!(receipt.accepted);
    assert!(receipt.external_ref.starts_with("MO-"));

    // Re-submitting the same business key returns the existing order —never a
    // duplicate.
    let again = connector
        .create_order("work-1042:cmms.create-order", r#"{"asset":"CNC-17"}"#)
        .expect("idempotent");
    assert_eq!(again.external_ref, receipt.external_ref);
    assert!(again.detail.unwrap().contains("idempotent"));

    // Readback finds the order.
    let found = connector
        .query_order("work-1042:cmms.create-order")
        .expect("readback")
        .expect("order exists");
    assert_eq!(found, receipt.external_ref);
}

#[test]
fn g5_fake_cmms_timeout_after_commit_simulates_unknown() {
    // G7 preparation: the connector commits the order but the caller gets no
    // acknowledgement —the reconciliation path resolves it by business key.
    let connector = FakeCmmsConnector::new();
    connector.arm_timeout_after_commit();

    let result = connector.create_order("work-t:cmms.create-order", "{}");
    assert!(
        result.is_err(),
        "timeout after commit surfaces as a transport error"
    );
    assert!(result
        .unwrap_err()
        .contains("transport timeout after commit"));

    // The order WAS committed (readback finds it) —the unknown is real.
    let found = connector
        .query_order("work-t:cmms.create-order")
        .expect("readback");
    assert!(found.is_some(), "order committed despite the timeout");

    // Disarm: re-submitting the same key is idempotent (no duplicate order).
    connector.disarm_timeout_after_commit();
    let receipt = connector
        .create_order("work-t:cmms.create-order", "{}")
        .expect("idempotent re-submit");
    assert!(receipt.detail.unwrap().contains("idempotent"));
}
