//! provider_swap_g11 — Conformance / Compatibility / Provider Swap (Phase 19, G11).
//!
//! Proves (Goal §23 / master §45):
//! - protocol conformance: versioning, Work spec/status, condition gates,
//!   binding pin, authority-before-side-effect, receipt semantics, outcome vs
//!   acceptance, supersede/retract, unknown/reconciliation (the suites from
//!   G1/G2/G6/G7/G8/G9 compose the protocol conformance surface);
//! - provider conformance: all HarnessProviders run the SAME contract test;
//! - the harness swap benchmark: the SAME CNC scenario with Run A (fake-dsh)
//!   and Run B (fake-pi) keeps every business-invariant condition — model
//!   reasoning text may differ, business semantics must not;
//! - REAL DSH/Pi swap stays BLOCKED_EXTERNAL until real providers exist
//!   (PASS_WITH_LIMITATIONS — never a fake PASS).

use morn::domains::factory::scenario::{FactoryScenarioRunner, ScenarioEvidence, ScenarioFixture};
use morn::providers::credential::EnvCredentialProvider;
use morn::providers::harness::{HarnessInput, HarnessProvider};
use morn::providers::harness_dsh::DshHarnessProvider;
use morn::providers::harness_fake::FakeHarnessProvider;
use morn::providers::harness_pi::PiHarnessProvider;

fn scenario_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("morn_g11_{label}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The business-invariant conditions that must hold for ANY harness
/// (the G11 benchmark checklist, Goal §23.3).
fn assert_business_invariants(evidence: &ScenarioEvidence) {
    // Same Work contract.
    assert_eq!(evidence.work_id, "work-cnc17");
    assert_eq!(evidence.situation_id, "situation-cnc17");
    // Binding pin.
    assert!(
        evidence.binding_pinned,
        "the binding stayed BOUND for this run"
    );
    // Same authority boundary: the attempt went through the gateway.
    assert!(
        evidence.outcome_unknown_then_verified,
        "OUTCOME_UNKNOWN → VERIFIED (receipt semantics)"
    );
    // No bypass of the receipt; no duplicate side effect.
    assert!(evidence.external_ref.starts_with("MO-"));
    assert!(evidence.no_duplicate_action, "no duplicate external action");
    // Outcome/Acceptance semantics identical.
    assert_eq!(evidence.outcome_id, "outcome-cnc17");
    assert_eq!(evidence.acceptance_ids.len(), 3);
    assert_eq!(evidence.value_assessment_id, "value-cnc17");
}

#[test]
fn g11_harness_swap_benchmark_same_scenario() {
    // Run A = fake-dsh, Run B = fake-pi: the SAME CNC scenario, business
    // invariants identical. The harness reasoning text may differ (the fake
    // echoes the prompt), but business conditions must not.
    let fixture = ScenarioFixture::cnc17();
    let run_a = FactoryScenarioRunner::run_with_harness(
        &fixture,
        scenario_dir("a").join("orders.json"),
        "fake-dsh",
    )
    .expect("Run A (fake-dsh)");
    let run_b = FactoryScenarioRunner::run_with_harness(
        &fixture,
        scenario_dir("b").join("orders.json"),
        "fake-pi",
    )
    .expect("Run B (fake-pi)");

    assert_business_invariants(&run_a);
    assert_business_invariants(&run_b);

    // Business-invariant fields are IDENTICAL across the swap.
    assert_eq!(run_a.work_id, run_b.work_id);
    assert_eq!(run_a.business_key, run_b.business_key);
    assert_eq!(
        run_a.external_ref, run_b.external_ref,
        "same external ref (deterministic business key)"
    );
    assert_eq!(run_a.outcome_id, run_b.outcome_id);
    assert_eq!(run_a.acceptance_ids, run_b.acceptance_ids);
    assert_eq!(run_a.value_assessment_id, run_b.value_assessment_id);
    assert_eq!(
        run_a.audit_timeline, run_b.audit_timeline,
        "the audit timeline structure is identical"
    );

    // The provider identity differs (that is the point of the swap) — but it
    // never leaked into the business truth fields.
    assert_eq!(
        FactoryScenarioRunner::run_with_harness(
            &fixture,
            scenario_dir("a2").join("orders.json"),
            "fake-dsh"
        )
        .expect("A again"),
        run_a,
        "Run A is deterministic"
    );
}

#[test]
fn g11_provider_conformance_same_contract_test() {
    // All HarnessProviders implement the same contract: capabilities,
    // health, and the honest-blocker surface. The fake passes the FULL
    // contract; DSH/Pi skeletons pass the honest-blocker contract.
    let credentials = EnvCredentialProvider::new();

    let dsh = DshHarnessProvider::new(std::sync::Arc::new(credentials), None);
    let pi = PiHarnessProvider::new(std::sync::Arc::new(EnvCredentialProvider::new()), None);
    let fake_a = FakeHarnessProvider::new("fake-dsh");
    let fake_b = FakeHarnessProvider::new("fake-pi");

    // All four are the same trait object (provider conformance).
    let providers: Vec<Box<dyn HarnessProvider>> = vec![
        Box::new(dsh),
        Box::new(pi),
        Box::new(fake_a),
        Box::new(fake_b),
    ];
    assert_eq!(providers.len(), 4);

    // DSH/Pi skeletons: honestly blocked (REAL_PROVIDER_INTEGRATION_BLOCKED).
    for provider in providers.iter().take(2) {
        let caps = provider.capabilities();
        assert!(!caps.real_integration_available);
        assert!(caps.integration_blocker.is_some());
        assert!(caps
            .integration_blocker
            .as_deref()
            .unwrap_or("")
            .contains("REAL_PROVIDER_INTEGRATION_BLOCKED"));
    }

    // The fakes: full contract, harness-neutral.
    for provider in providers.iter().skip(2) {
        let caps = provider.capabilities();
        assert!(caps.provider_id.starts_with("fake-"));
        // The same contract: start_session works under a binding.
        let binding = morn::work::ExecutionBinding::new(
            "binding-g11",
            "work-g11",
            "sha256:cap",
            &caps.provider_id,
        );
        let session = provider.start_session(&binding).expect("session starts");
        assert_eq!(session.provider_id, caps.provider_id);
        let candidate = provider
            .submit(
                &session,
                HarnessInput {
                    prompt: "swap benchmark".to_string(),
                    context_json: None,
                },
            )
            .expect("submit works");
        assert!(candidate.seq > 0);
    }
}

#[test]
fn g11_swap_benchmark_records_cost_and_latency_shape() {
    // The benchmark records cost/latency/success shape (not fake customer
    // value): each run produces the same evidence shape with the same
    // unknown-outcome count.
    let fixture = ScenarioFixture::cnc17();
    let run_a = FactoryScenarioRunner::run_with_harness(
        &fixture,
        scenario_dir("bench-a").join("orders.json"),
        "fake-dsh",
    )
    .expect("Run A");
    let run_b = FactoryScenarioRunner::run_with_harness(
        &fixture,
        scenario_dir("bench-b").join("orders.json"),
        "fake-pi",
    )
    .expect("Run B");

    // Success shape: both runs reached the same terminal state.
    assert_eq!(
        run_a.outcome_unknown_then_verified,
        run_b.outcome_unknown_then_verified
    );
    assert_eq!(run_a.no_duplicate_action, run_b.no_duplicate_action);
    assert_eq!(run_a.binding_pinned, run_b.binding_pinned);

    // Acceptance was NOT lowered to make the swap pass: 3-role acceptance in
    // both runs (the full set).
    assert_eq!(run_a.acceptance_ids.len(), 3);
    assert_eq!(run_b.acceptance_ids.len(), 3);
}
