//! outcome_acceptance_value — Outcome / Acceptance / Value semantics (Phase 7).
//!
//! Proves (Goal §11 / master §12):
//! - an ObservedOutcome comes only from an identifiable source; the type
//!   system has NO agent/success source — an agent self-report can never
//!   become an outcome;
//! - multi-role acceptance decisions are recorded as business truth; a
//!   rejection or conditional decision does NOT satisfy the acceptance gate
//!   (AcceptanceSatisfied means the outcome was ACCEPTED);
//! - ValueAssessment measures value in accepted outcomes — duration, human
//!   minutes, cost, retries, unknowns, baseline — not agent-run counts.

use morn::control::controllers::outcome_flow::{AcceptanceController, OutcomeController};
use morn::control::store::ControlPlaneStore;
use morn::work::{
    AcceptanceDecision, AcceptanceType, ObservedOutcome, OutcomeMetric, OutcomeSource, Work,
    WorkAcceptanceState, WorkSpec,
};

fn work(work_id: &str) -> Work {
    Work::draft(
        work_id,
        WorkSpec {
            work_type: "factory.equipment-exception".to_string(),
            goal: "restore_operational_readiness".to_string(),
            target_refs: Vec::new(),
            constraints: Vec::new(),
            acceptance: None,
            authority: None,
            budget: None,
            profile_ref: None,
            source_refs: Vec::new(),
            owner: None,
            generation: 1,
            contract_version: None,
        },
    )
}

#[test]
fn agent_self_report_is_never_an_outcome() {
    // The OutcomeSource enum has exactly the four identifiable sources —
    // there is no "AgentSays" variant, so an agent self-report cannot be
    // constructed as an outcome at the type level.
    let sources = [
        OutcomeSource::ExternalSystem,
        OutcomeSource::Sensor,
        OutcomeSource::HumanObservation,
        OutcomeSource::ValidatedComputation,
    ];
    for source in sources {
        let outcome = ObservedOutcome::from_source("oc-1", "work-1", source);
        assert_eq!(outcome.source, source);
    }

    // A sensor observation carries metrics with explicit units.
    let outcome = ObservedOutcome::from_source("oc-2", "work-1", OutcomeSource::Sensor)
        .with_metric(OutcomeMetric::new("vibration", 3.1, "mm/s"))
        .with_metric(OutcomeMetric::new("downtime", 52.0, "min"));
    assert_eq!(outcome.metrics.len(), 2);
    assert_eq!(outcome.metrics[0].unit, "mm/s");
    assert_eq!(outcome.metrics[1].unit, "min");
}

#[test]
fn outcome_controller_records_from_identifiable_source_and_clears_unknowns() {
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&work("work-oa")).unwrap();

    // The Work has an unresolved unknown (e.g. a CMMS timeout).
    let mut w = store.get_work("work-oa").unwrap().unwrap();
    w.status
        .unresolved_unknowns
        .push("cmms-timeout".to_string());
    store.update_work_status("work-oa", &w.status).unwrap();

    // A real observation (external system) is recorded.
    let outcome = ObservedOutcome::from_source("oc-oa", "work-oa", OutcomeSource::ExternalSystem)
        .with_metric(OutcomeMetric::new("vibration", 3.1, "mm/s"));
    let result = OutcomeController.record(&store, &outcome).unwrap();
    assert!(result
        .notes
        .iter()
        .any(|n| n.contains("recorded from ExternalSystem")));

    // The unknown is cleared only by the real observation.
    let w = store.get_work("work-oa").unwrap().unwrap();
    assert!(w.status.unresolved_unknowns.is_empty());
    assert!(w
        .status
        .observed_outcome_refs
        .contains(&"oc-oa".to_string()));

    // Duplicate outcome record is idempotent.
    let dup = OutcomeController.record(&store, &outcome).unwrap();
    assert!(dup.notes.iter().any(|n| n.contains("already recorded")));
}

#[test]
fn multi_role_acceptance_accept_satisfies_gate() {
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&work("work-acc")).unwrap();

    // Multi-role acceptance: production, quality, maintenance each decide.
    // All three accept → the Work is Accepted and the gate is satisfied.
    for (i, role) in ["production_owner", "quality_owner", "maintenance_owner"]
        .iter()
        .enumerate()
    {
        let acceptance = AcceptanceDecision::new(
            &format!("acc-{i}"),
            "work-acc",
            AcceptanceType::Accept,
            role,
        );
        let result = AcceptanceController.record(&store, &acceptance).unwrap();
        assert!(
            result
                .condition_changes
                .iter()
                .any(|c| c.type_ == morn::work::ConditionType::AcceptanceSatisfied && c.is_true()),
            "an ACCEPT decision satisfies the acceptance gate"
        );
    }
    let w = store.get_work("work-acc").unwrap().unwrap();
    assert_eq!(w.status.acceptance_state, WorkAcceptanceState::Accepted);
}

#[test]
fn acceptance_reject_does_not_satisfy_the_gate() {
    // SAFETY: a rejection is recorded as business truth but does NOT satisfy
    // the acceptance gate — a rejected work must not pass a profile that
    // requires acceptance.
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&work("work-rej")).unwrap();

    let acceptance = AcceptanceDecision::new(
        "acc-rej",
        "work-rej",
        AcceptanceType::Reject,
        "quality_owner",
    );
    let result = AcceptanceController.record(&store, &acceptance).unwrap();
    assert!(
        result
            .condition_changes
            .iter()
            .any(|c| c.type_ == morn::work::ConditionType::AcceptanceSatisfied && !c.is_true()),
        "a REJECT decision must not satisfy the acceptance gate"
    );
    let w = store.get_work("work-rej").unwrap().unwrap();
    assert_eq!(w.status.acceptance_state, WorkAcceptanceState::Rejected);

    // A conditional decision is also recorded without satisfying the gate.
    let store = ControlPlaneStore::in_memory().expect("store");
    store.insert_work(&work("work-cond")).unwrap();
    let acceptance = AcceptanceDecision::new(
        "acc-cond",
        "work-cond",
        AcceptanceType::Conditional,
        "production_owner",
    );
    let result = AcceptanceController.record(&store, &acceptance).unwrap();
    assert!(
        result
            .condition_changes
            .iter()
            .any(|c| c.type_ == morn::work::ConditionType::AcceptanceSatisfied && !c.is_true()),
        "a CONDITIONAL decision must not satisfy the acceptance gate"
    );
    let w = store.get_work("work-cond").unwrap().unwrap();
    assert_eq!(w.status.acceptance_state, WorkAcceptanceState::Conditional);
}

#[test]
fn value_assessment_measures_accepted_outcomes_not_agent_runs() {
    // Value is measured in accepted outcomes: duration, human minutes, model
    // cost, retries, unknowns, baseline comparison — with explicit units.
    let mut value = morn::work::ValueAssessment::new("va-1", "work-1042");
    value.total_duration_minutes = Some(52.0);
    value.human_minutes = Some(12.0);
    value.model_runtime_cost = Some(morn::work::CostAmount::new(2.5, "CNY"));
    value.retries = 1;
    value.unknown_outcomes = 1;
    value.baseline_comparison_ref = Some("baseline-2026-09".to_string());
    value
        .kpis
        .push(OutcomeMetric::new("late_order_minutes", 11.0, "min"));
    value
        .kpis
        .push(OutcomeMetric::new("duplicate_action_count", 0.0, "count"));
    value.accepted = true;
    value.reopened = false;

    assert_eq!(value.total_duration_minutes, Some(52.0));
    assert_eq!(value.human_minutes, Some(12.0));
    assert_eq!(value.model_runtime_cost.as_ref().unwrap().amount, 2.5);
    assert_eq!(value.model_runtime_cost.as_ref().unwrap().currency, "CNY");
    assert_eq!(value.retries, 1);
    assert_eq!(value.unknown_outcomes, 1);
    assert!(value.accepted);
    assert!(!value.reopened);
    assert_eq!(
        value.kpis[1].value, 0.0,
        "duplicate side effects are measured"
    );

    // Round-trip through serde (the assessment is a protocol resource).
    let json = serde_json::to_string(&value).unwrap();
    let back: morn::work::ValueAssessment = serde_json::from_str(&json).unwrap();
    assert_eq!(back, value);
}
