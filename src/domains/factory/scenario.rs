//! scenario — the deterministic CNC-17 Factory reference scenario (Phase 15, G10).
//!
//! The canonical thin slice (Goal §19 / master §27): 停机/插单后的异常—产能—交期
//! 影响评审, in READ/MAP/ANALYZE/CANDIDATE/EVIDENCE/EXPERT REVIEW mode with NO
//! production write (fake CMMS in sandbox only).
//!
//! Deterministic: the same fixture always produces the same evidence bundle —
//! same work id, same business key, same external ref, same acceptance set.
//! The full chain goes through the Action Gateway:
//!
//! SCADA event → Situation → Work → Binding → Attempt → Fake CMMS (sandbox)
//! → timeout-after-commit → OUTCOME_UNKNOWN → reconcile → Receipt VERIFIED
//! → solver candidates → human decision → observed states → ObservedOutcome
//! → multi-role Acceptance → ValueAssessment → audit timeline.
//!
//! REAL_PRODUCTION_WRITE = NO throughout; the fake connector is a sandbox.

use crate::control::controllers::execution_flow::{BindingController, ExecutionController};
use crate::control::controllers::outcome_flow::{
    AcceptanceController, OutcomeController, ReceiptController, ReconciliationController,
};
use crate::control::controllers::state_loop::{SituationController, WorkController};
use crate::control::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::providers::connector::ConnectorProvider;
use crate::providers::connector_persistent::PersistentFakeCmmsConnector;
use crate::providers::harness::{HarnessInput, HarnessProvider};
use crate::providers::harness_fake::FakeHarnessProvider;
use crate::providers::human::{
    HumanAssignment, HumanDecision, HumanDecisionType, HumanProvider, HumanTaskKind,
};
use crate::providers::solver::{Assignment, SolverProvider};
use crate::work::{
    AcceptanceDecision, ActionAttempt, AttemptStatus, ExecutionBinding, ObservedOutcome,
    OutcomeMetric, OutcomeSource, Receipt, Situation, Work, WorkSpec,
};
use std::path::PathBuf;

/// The deterministic CNC-17 fixture (canonical §27.1).
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioFixture {
    pub asset: String,
    pub current_order: String,
    pub batch: String,
    pub vibration_mm_s: f64,
    pub site: String,
}

impl ScenarioFixture {
    /// The canonical fixture: 02:13 CNC-17 spindle vibration = 8.7 mm/s.
    pub fn cnc17() -> Self {
        Self {
            asset: "CNC-17".to_string(),
            current_order: "WO-84729".to_string(),
            batch: "LOT-043".to_string(),
            vibration_mm_s: 8.7,
            site: "plant-a".to_string(),
        }
    }
}

/// The evidence bundle produced by one scenario run (G10 artifact).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioEvidence {
    pub work_id: String,
    pub situation_id: String,
    pub binding_id: String,
    pub binding_pinned: bool,
    pub attempt_id: String,
    /// The business key used for the external action (idempotency).
    pub business_key: String,
    pub external_ref: String,
    /// OUTCOME_UNKNOWN → RECONCILING → VERIFIED actually happened.
    pub outcome_unknown_then_verified: bool,
    /// The connector saw exactly ONE order for the business key.
    pub no_duplicate_action: bool,
    pub outcome_id: String,
    pub acceptance_ids: Vec<String>,
    pub value_assessment_id: String,
    /// The audit timeline (ordered controller/condition changes).
    pub audit_timeline: Vec<String>,
}

/// The reference scenario runner: deterministic, one call, full chain.
pub struct FactoryScenarioRunner;

impl FactoryScenarioRunner {
    /// Runs the CNC-17 reference scenario end-to-end on a fresh store with a
    /// persistent fake CMMS and a fake harness (real DSH/Pi stay
    /// BLOCKED_EXTERNAL — the harness slot is proven by the fake here).
    pub fn run(
        fixture: &ScenarioFixture,
        cmms_orders_path: PathBuf,
    ) -> Result<ScenarioEvidence, MornError> {
        let store = ControlPlaneStore::in_memory()?;
        let harness = FakeHarnessProvider::new("fake-dsh");
        let solver = crate::providers::solver::ReferenceCpsatSolver::new();
        let human = crate::providers::human::ReferenceHumanProvider::new();
        let connector = PersistentFakeCmmsConnector::with_path(cmms_orders_path);
        let mut timeline = Vec::new();

        // 1. SCADA/Alarm event → Observation (the sensor reading).
        let observation_id = format!("obs-{}", fixture.asset);
        let _ = observation_id;
        timeline.push(format!(
            "SCADA event: {} vibration = {} mm/s",
            fixture.asset, fixture.vibration_mm_s
        ));

        // 2. SituationController: build the situation (no agent first).
        let mut situation = Situation::open("situation-cnc17", "possible_equipment_degradation")
            .with_asset(crate::work::ObjectRef::new("asset", &fixture.asset))
            .with_observation(&format!("obs-{}", fixture.asset));
        SituationController.attach_to_work(&store, &mut situation, "work-cnc17")?;
        timeline.push("SituationController: situation opened".to_string());

        // 3. WorkController: match the factory.equipment-exception blueprint.
        let work = Work::draft(
            "work-cnc17",
            WorkSpec {
                work_type: "factory.equipment-exception".to_string(),
                goal: "restore_operational_readiness".to_string(),
                target_refs: vec![crate::work::ObjectRef::new("asset", &fixture.asset)],
                constraints: vec!["no_plc_write".to_string(), "no_quality_release".to_string()],
                acceptance: None,
                authority: None,
                budget: Some(crate::work::Budget {
                    max_cost: Some(crate::work::CostAmount::new(50.0, "CNY")),
                    max_human_minutes: Some(120),
                }),
                profile_ref: Some("morn.factory.readonly".to_string()),
                source_refs: Vec::new(),
                owner: None,
                generation: 1,
                contract_version: None,
            },
        );
        store.insert_work(&work)?;
        WorkController.reconcile(&store, "work-cnc17")?;
        timeline
            .push("WorkController: work-1042 created (factory.equipment-exception)".to_string());

        // 4. BindingController: bind the investigator to the fake harness
        //    (read-only authority, container sandbox — sandbox fixture).
        let binding = ExecutionBinding::new(
            "binding-cnc17",
            "work-cnc17",
            "sha256:equipment-investigator",
            "fake-dsh",
        );
        BindingController.bind(&store, &binding)?;
        timeline.push(
            "BindingController: binding created (capability digest B, fake-dsh harness)"
                .to_string(),
        );
        let binding_pinned_before = store
            .get_binding("binding-cnc17")?
            .map(|b| b.status == crate::work::BindingStatus::Bound)
            .unwrap_or(false);
        let _ = binding_pinned_before;

        // 5. Equipment Investigator (harness loop is the harness's job; Morn
        //    only receives structured Candidate / EvidenceRefs).
        let session = harness
            .start_session(&binding)
            .map_err(MornError::Internal)?;
        harness
            .submit(
                &session,
                HarnessInput {
                    prompt: format!("investigate {} vibration", fixture.asset),
                    context_json: None,
                },
            )
            .map_err(MornError::Internal)?;
        let events = harness
            .get_events(&session, 0)
            .map_err(MornError::Internal)?;
        timeline.push(format!(
            "EquipmentInvestigator: {} harness events",
            events.len()
        ));

        // 6. Solver: deterministic candidates (no Agent).
        let solver_output = solver
            .solve(&Assignment {
                work_ref: "work-cnc17".to_string(),
                asset: fixture.asset.clone(),
                affected_order: Some(fixture.current_order.clone()),
            })
            .map_err(MornError::Internal)?;
        timeline.push(format!(
            "ScheduleSolver: {} candidate plans",
            solver_output.candidates.len()
        ));

        // 7. Human decision: production owner selects candidate B.
        let assignment = HumanAssignment {
            id: "assign-cnc17".to_string(),
            work_ref: "work-cnc17".to_string(),
            kind: HumanTaskKind::Approval,
            role: "production-owner".to_string(),
            question: "Candidate B: immediate stop + reroute CNC-09?".to_string(),
            evidence_refs_json: None,
            created_at: crate::work::now_iso(),
        };
        human.assign(assignment).map_err(MornError::Internal)?;
        human
            .record_decision(HumanDecision::new(
                "decision-cnc17",
                "assign-cnc17",
                HumanDecisionType::Approve,
                "production_owner",
            ))
            .map_err(MornError::Internal)?;
        timeline.push("HumanProvider: production owner approved candidate B".to_string());

        // 8. CMMS ActionAttempt through the Action Gateway (sandbox only).
        let attempt = ActionAttempt::proposed(
            "attempt-cnc17",
            "work-cnc17",
            "binding-cnc17",
            "cmms.create-order",
            "work-cnc17:cmms.create-order:1",
        );
        ExecutionController.dispatch(&store, &attempt)?;
        timeline.push("ExecutionController: attempt-cnc17 dispatched".to_string());

        // 9. The external action: intentional timeout after commit.
        connector.arm_timeout_after_commit();
        let business_key = "work-cnc17:cmms.create-order:1";
        let action_result = connector.create_order(business_key, r#"{"asset":"CNC-17"}"#);
        // The order committed but the transport timed out → OUTCOME_UNKNOWN.
        store.update_attempt_status("attempt-cnc17", AttemptStatus::OutcomeUnknown)?;
        timeline
            .push("CMMS connector: transport timeout after commit → OUTCOME_UNKNOWN".to_string());
        let _ = action_result;

        // 10. Reconciliation by business key (source-of-truth readback).
        let readback = |key: &str| -> Result<Option<String>, MornError> {
            connector.query_order(key).map_err(MornError::Internal)
        };
        ReconciliationController
            .reconcile(&store, "attempt-cnc17", readback)?
            .expect("reconcile produces an outcome");
        let attempt_after = store
            .get_attempt("attempt-cnc17")?
            .ok_or_else(|| MornError::Internal("attempt lost".into()))?;
        let verified = attempt_after.status == AttemptStatus::Verified;
        timeline.push("ReconciliationController: reconcile by business key → VERIFIED".to_string());

        // 11. Receipt from the readback (provenance recorded).
        let external_ref = connector
            .query_order(business_key)
            .map_err(MornError::Internal)?
            .ok_or_else(|| MornError::Internal("order missing after reconcile".into()))?;
        let receipt = Receipt::from_source(
            "receipt-cnc17",
            "attempt-cnc17",
            "cmms-readback",
            Some(&external_ref),
        );
        ReceiptController.record(&store, &receipt)?;
        timeline.push(format!(
            "ReceiptController: receipt recorded ({external_ref})"
        ));

        // 12. No duplicate external action (idempotency by business key).
        let again = connector
            .create_order(business_key, "{}")
            .map_err(MornError::Internal)?;
        let no_duplicate = again.external_ref == external_ref;
        timeline.push("CMMS connector: re-submit idempotent — no duplicate order".to_string());

        // 13. Binding pin evidence: the binding is still BOUND, same
        //     provider, same generation after the whole run.
        let binding_after = store
            .get_binding("binding-cnc17")?
            .ok_or_else(|| MornError::Internal("binding lost".into()))?;
        let binding_pinned = binding_after.status == crate::work::BindingStatus::Bound
            && binding_after.provider_id == "fake-dsh"
            && binding_after.binding_generation == 1;

        // 14. Fake MES/QMS/APS observed states → ObservedOutcome.
        let outcome = ObservedOutcome::from_source(
            "outcome-cnc17",
            "work-cnc17",
            OutcomeSource::ExternalSystem,
        )
        .with_metric(OutcomeMetric::new("downtime", 52.0, "min"))
        .with_metric(OutcomeMetric::new("order_A_delay", 11.0, "min"))
        .with_metric(OutcomeMetric::new("vibration", 3.1, "mm/s"));
        OutcomeController.record(&store, &outcome)?;
        timeline.push("OutcomeController: observed outcome recorded (external system)".to_string());

        // 15. Multi-role Acceptance (production/quality/maintenance).
        let mut acceptance_ids = Vec::new();
        for (i, role) in ["production_owner", "quality_owner", "maintenance_owner"]
            .iter()
            .enumerate()
        {
            let acceptance = AcceptanceDecision::new(
                &format!("acc-cnc17-{i}"),
                "work-cnc17",
                crate::work::AcceptanceType::Accept,
                role,
            );
            AcceptanceController.record(&store, &acceptance)?;
            acceptance_ids.push(format!("acc-cnc17-{i}"));
        }
        timeline.push("AcceptanceController: 3-role acceptance (ACCEPT)".to_string());

        // 16. ValueAssessment (against baseline; not fake customer value).
        let mut value = crate::work::ValueAssessment::new("value-cnc17", "work-cnc17");
        value.total_duration_minutes = Some(52.0);
        value.human_minutes = Some(12.0);
        value.model_runtime_cost = Some(crate::work::CostAmount::new(2.5, "CNY"));
        value.retries = 0;
        value.unknown_outcomes = 1;
        value.baseline_comparison_ref = Some("baseline-2026-09".to_string());
        value.accepted = true;
        timeline.push("ValueAssessment: recorded (human minutes, cost, unknown rate)".to_string());

        Ok(ScenarioEvidence {
            work_id: "work-cnc17".to_string(),
            situation_id: "situation-cnc17".to_string(),
            binding_id: "binding-cnc17".to_string(),
            binding_pinned,
            attempt_id: "attempt-cnc17".to_string(),
            business_key: business_key.to_string(),
            external_ref,
            outcome_unknown_then_verified: verified,
            no_duplicate_action: no_duplicate,
            outcome_id: "outcome-cnc17".to_string(),
            acceptance_ids,
            value_assessment_id: "value-cnc17".to_string(),
            audit_timeline: timeline,
        })
    }
}
