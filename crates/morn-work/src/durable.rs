//! Durable Work Runtime v0.2: workflow runs, signals/waits, retry, compensation,
//! escalation, budget guard, drift detection and attention.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    BudgetGuardId, CompensationPlanId, DriftRecordId, EscalationId, RetryPolicyId, SignalId,
    TimerWaitId, WorkflowDefinitionId, WorkflowRunId,
};
use morn_kernel::time::Timestamp;

use crate::attention::{AttentionItem, AttentionKind, AttentionPriority};
use crate::checkpoint::Checkpoint;
use crate::workflow::{RunStatus, WorkflowDefinition, WorkflowRun};

/// Kinds of signals a run can wait on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SignalKind {
    HumanApproval,
    ExternalEvent,
    ManualResume,
    Cancel,
    DataArrived,
    ReviewerResponse,
}

/// A delivered signal. Delivery is idempotent by signal id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    pub id: SignalId,
    pub run_id: WorkflowRunId,
    pub kind: SignalKind,
    pub payload: String,
    pub identity: String,
    pub authority: String,
    pub schema: String,
    pub created_at: Timestamp,
    pub delivered: bool,
}

impl Signal {
    pub fn new(
        run_id: WorkflowRunId,
        kind: SignalKind,
        payload: impl Into<String>,
        identity: impl Into<String>,
        authority: impl Into<String>,
    ) -> Self {
        Self {
            id: SignalId::generate_with("sig"),
            run_id,
            kind,
            payload: payload.into(),
            identity: identity.into(),
            authority: authority.into(),
            schema: "morn.signal.v1".to_string(),
            created_at: Timestamp::now(),
            delivered: false,
        }
    }
}

/// A timer wait (deadline/waiter) attached to a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerWait {
    pub id: TimerWaitId,
    pub run_id: WorkflowRunId,
    pub kind: String,
    pub wait_until: Timestamp,
    pub fired: bool,
    pub created_at: Timestamp,
}

impl TimerWait {
    pub fn new(run_id: WorkflowRunId, kind: impl Into<String>, timeout_secs: u64) -> Self {
        Self {
            id: TimerWaitId::generate_with("wait"),
            run_id,
            kind: kind.into(),
            wait_until: Timestamp::from_millis(
                Timestamp::now().millis() + (timeout_secs as i64) * 1000,
            ),
            fired: false,
            created_at: Timestamp::now(),
        }
    }
}

/// Retry policy for a step or run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub id: RetryPolicyId,
    pub name: String,
    pub max_attempts: u32,
    pub backoff_base_secs: u64,
    pub retryable_errors: Vec<String>,
    pub budget_impact: f64,
}

impl RetryPolicy {
    pub fn new(name: impl Into<String>, max_attempts: u32) -> Self {
        Self {
            id: RetryPolicyId::generate_with("retry"),
            name: name.into(),
            max_attempts,
            backoff_base_secs: 1,
            retryable_errors: Vec::new(),
            budget_impact: 1.0,
        }
    }
}

/// E2 compensation plan (never used to fake E3 reversibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompensationPlan {
    pub id: CompensationPlanId,
    pub run_id: WorkflowRunId,
    pub original_action: String,
    pub compensation_action: String,
    pub status: String,
    pub result: Option<String>,
    pub residual_risk: String,
    pub created_at: Timestamp,
}

impl CompensationPlan {
    pub fn new(
        run_id: WorkflowRunId,
        original_action: impl Into<String>,
        compensation_action: impl Into<String>,
    ) -> Self {
        Self {
            id: CompensationPlanId::generate_with("comp"),
            run_id,
            original_action: original_action.into(),
            compensation_action: compensation_action.into(),
            status: "pending".to_string(),
            result: None,
            residual_risk: "none".to_string(),
            created_at: Timestamp::now(),
        }
    }
}

/// Escalation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Escalation {
    pub id: EscalationId,
    pub run_id: WorkflowRunId,
    pub level: u32,
    pub reason: String,
    pub to_principal: String,
    pub resolved: bool,
    pub created_at: Timestamp,
}

impl Escalation {
    pub fn new(
        run_id: WorkflowRunId,
        level: u32,
        reason: impl Into<String>,
        to_principal: impl Into<String>,
    ) -> Self {
        Self {
            id: EscalationId::generate_with("esc"),
            run_id,
            level,
            reason: reason.into(),
            to_principal: to_principal.into(),
            resolved: false,
            created_at: Timestamp::now(),
        }
    }
}

/// Budget guard: token/model, money, time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetGuard {
    pub id: BudgetGuardId,
    pub run_id: WorkflowRunId,
    pub token_limit: f64,
    pub money_limit: f64,
    pub time_limit_secs: f64,
    pub consumed_tokens: f64,
    pub consumed_money: f64,
    pub started_at: Timestamp,
    pub status: String,
}

impl BudgetGuard {
    pub fn new(
        run_id: WorkflowRunId,
        token_limit: f64,
        money_limit: f64,
        time_limit_secs: f64,
    ) -> Self {
        Self {
            id: BudgetGuardId::generate_with("budget"),
            run_id,
            token_limit,
            money_limit,
            time_limit_secs,
            consumed_tokens: 0.0,
            consumed_money: 0.0,
            started_at: Timestamp::now(),
            status: "ok".to_string(),
        }
    }

    pub fn record(&mut self, tokens: f64, money: f64) {
        self.consumed_tokens += tokens;
        self.consumed_money += money;
    }

    pub fn exceeded(&self, now: Timestamp) -> bool {
        let elapsed = (now.millis() - self.started_at.millis()) as f64 / 1000.0;
        self.consumed_tokens > self.token_limit
            || self.consumed_money > self.money_limit
            || elapsed > self.time_limit_secs
    }
}

/// Drift detection record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftRecord {
    pub id: DriftRecordId,
    pub run_id: WorkflowRunId,
    pub dimension: String,
    pub expected: String,
    pub actual: String,
    pub severity: String,
    pub detected_at: Timestamp,
}

/// The durable runtime service (in-memory core; persistence via MornStore).
#[derive(Debug, Default)]
pub struct DurableRuntime {
    workflows: HashMap<WorkflowDefinitionId, WorkflowDefinition>,
    runs: HashMap<WorkflowRunId, WorkflowRun>,
    signals: Vec<Signal>,
    timer_waits: Vec<TimerWait>,
    retry_policies: HashMap<RetryPolicyId, RetryPolicy>,
    compensations: Vec<CompensationPlan>,
    escalations: Vec<Escalation>,
    budgets: HashMap<WorkflowRunId, BudgetGuard>,
    drifts: Vec<DriftRecord>,
    checkpoints: Vec<Checkpoint>,
    attention: Vec<AttentionItem>,
}

impl DurableRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- workflow definitions ----

    pub fn register_workflow(&mut self, definition: WorkflowDefinition) {
        self.workflows.insert(definition.id.clone(), definition);
    }

    pub fn workflow(&self, id: &WorkflowDefinitionId) -> Option<&WorkflowDefinition> {
        self.workflows.get(id)
    }

    pub fn add_retry_policy(&mut self, policy: RetryPolicy) {
        self.retry_policies.insert(policy.id.clone(), policy);
    }

    // ---- runs ----

    pub fn start_run(
        &mut self,
        definition_id: &WorkflowDefinitionId,
        budget: Option<BudgetGuard>,
    ) -> Result<WorkflowRun> {
        let definition = self
            .workflows
            .get(definition_id)
            .ok_or_else(|| Error::not_found(format!("workflow {definition_id}")))?;
        let mut run = WorkflowRun::new(
            definition.id.clone(),
            definition.version,
            definition.workspace_id.clone(),
        );
        run.pending_steps = definition.steps.iter().map(|s| s.name.clone()).collect();
        run.transition(RunStatus::Ready)?;
        run.transition(RunStatus::Running)?;
        if let Some(mut b) = budget {
            b.run_id = run.id.clone();
            self.budgets.insert(run.id.clone(), b);
        }
        self.runs.insert(run.id.clone(), run.clone());
        Ok(run)
    }

    pub fn run(&self, id: &WorkflowRunId) -> Option<&WorkflowRun> {
        self.runs.get(id)
    }

    pub fn runs(&self) -> Vec<&WorkflowRun> {
        self.runs.values().collect()
    }

    /// Restore a previously persisted run (after process restart).
    pub fn restore_run(&mut self, run: WorkflowRun) {
        self.runs.insert(run.id.clone(), run);
    }

    pub fn all_runs(&self) -> Vec<WorkflowRun> {
        self.runs.values().cloned().collect()
    }

    pub fn mark_step_completed(&mut self, run_id: &WorkflowRunId, step: &str) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        if run.status != RunStatus::Running {
            return Err(Error::invalid_state(format!(
                "run {run_id} is not Running (status {:?})",
                run.status
            )));
        }
        run.completed_steps.push(step.to_string());
        run.pending_steps.retain(|s| s != step);
        run.updated_at = Timestamp::now();
        if run.pending_steps.is_empty() {
            run.transition(RunStatus::Completed)?;
        }
        Ok(())
    }

    /// Fail a step; apply retry policy or move to blocked/failed.
    pub fn fail_step(
        &mut self,
        run_id: &WorkflowRunId,
        _step: &str,
        error: &str,
        policy: Option<&RetryPolicy>,
    ) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.attempts += 1;
        let allowed = policy
            .map(|p| run.attempts < p.max_attempts)
            .unwrap_or(false);
        let retryable = policy
            .map(|p| {
                p.retryable_errors.is_empty()
                    || p.retryable_errors.iter().any(|e| error.contains(e))
            })
            .unwrap_or(true);
        if allowed && retryable {
            run.transition(RunStatus::RetryScheduled)?;
            if let Some(p) = policy {
                if let Some(b) = self.budgets.get_mut(run_id) {
                    b.record(0.0, p.budget_impact);
                }
            }
        } else {
            run.transition(RunStatus::Blocked)?;
            let ws = run.workspace_id.clone();
            let item = AttentionItem::new(
                ws,
                AttentionKind::ToolFailure,
                format!("run {run_id} blocked"),
                format!("step failed after retries: {error}"),
                AttentionPriority::High,
            );
            self.attention.push(item);
        }
        Ok(())
    }

    pub fn retry_now(&mut self, run_id: &WorkflowRunId) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::Running)?;
        Ok(())
    }

    // ---- signals / waits ----

    pub fn wait_for_signal(
        &mut self,
        run_id: &WorkflowRunId,
        kind: SignalKind,
        timeout_secs: u64,
    ) -> Result<TimerWait> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::WaitingSignal)?;
        let wait = TimerWait::new(run_id.clone(), format!("{kind:?}"), timeout_secs);
        self.timer_waits.push(wait.clone());
        Ok(wait)
    }

    pub fn wait_for_approval(&mut self, run_id: &WorkflowRunId) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::WaitingApproval)?;
        Ok(())
    }

    /// Deliver a signal. Idempotent by signal id: re-delivery is rejected.
    pub fn deliver_signal(&mut self, signal: Signal) -> Result<()> {
        if self.signals.iter().any(|s| s.id == signal.id) {
            return Err(Error::conflict(format!(
                "signal {} already delivered",
                signal.id
            )));
        }
        let run_id = signal.run_id.clone();
        let run = self
            .runs
            .get_mut(&run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        match signal.kind {
            SignalKind::Cancel => {
                run.transition(RunStatus::Cancelled)?;
            }
            _ => {
                if run.status == RunStatus::WaitingSignal
                    || run.status == RunStatus::WaitingApproval
                {
                    run.transition(RunStatus::Running)?;
                }
            }
        }
        self.signals.push(signal);
        Ok(())
    }

    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    // ---- pause/resume ----

    pub fn pause(&mut self, run_id: &WorkflowRunId) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::Paused)?;
        Ok(())
    }

    pub fn resume(&mut self, run_id: &WorkflowRunId) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::Running)?;
        Ok(())
    }

    // ---- compensation (E2 only) ----

    pub fn compensate(&mut self, run_id: &WorkflowRunId, plan: CompensationPlan) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::Compensating)?;
        self.compensations.push(plan);
        Ok(())
    }

    pub fn complete_compensation(
        &mut self,
        run_id: &WorkflowRunId,
        plan_id: &CompensationPlanId,
        result: &str,
    ) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        if run.status != RunStatus::Compensating {
            return Err(Error::invalid_state("run is not compensating"));
        }
        if let Some(plan) = self.compensations.iter_mut().find(|p| p.id == *plan_id) {
            plan.status = "completed".to_string();
            plan.result = Some(result.to_string());
        }
        run.transition(RunStatus::Completed)?;
        Ok(())
    }

    pub fn compensations(&self) -> &[CompensationPlan] {
        &self.compensations
    }

    // ---- escalation ----

    pub fn escalate(
        &mut self,
        run_id: &WorkflowRunId,
        level: u32,
        reason: &str,
        to: &str,
    ) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        run.transition(RunStatus::Escalated)?;
        self.escalations
            .push(Escalation::new(run_id.clone(), level, reason, to));
        let ws = run.workspace_id.clone();
        self.attention.push(AttentionItem::new(
            ws,
            AttentionKind::DeadlineDrift,
            format!("run {run_id} escalated"),
            reason.to_string(),
            AttentionPriority::Critical,
        ));
        Ok(())
    }

    pub fn escalations(&self) -> &[Escalation] {
        &self.escalations
    }

    // ---- budget ----

    pub fn check_budget(&mut self, run_id: &WorkflowRunId) -> Result<bool> {
        let run = self
            .runs
            .get(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        let Some(budget) = self.budgets.get(run_id) else {
            return Ok(false);
        };
        if budget.exceeded(Timestamp::now()) {
            let ws = run.workspace_id.clone();
            self.attention.push(AttentionItem::new(
                ws,
                AttentionKind::BudgetRisk,
                format!("run {run_id} over budget"),
                format!(
                    "tokens {:.1}/{:.1}, money {:.1}/{:.1}",
                    budget.consumed_tokens,
                    budget.token_limit,
                    budget.consumed_money,
                    budget.money_limit
                ),
                AttentionPriority::High,
            ));
            let run = self.runs.get_mut(run_id).unwrap();
            run.transition(RunStatus::Blocked)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn budget(&self, run_id: &WorkflowRunId) -> Option<&BudgetGuard> {
        self.budgets.get(run_id)
    }

    // ---- checkpoint / restart / drift ----

    pub fn checkpoint(&mut self, run_id: &WorkflowRunId) -> Result<Checkpoint> {
        let run = self
            .runs
            .get(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        let payload = serde_json::to_string(run).map_err(|e| Error::internal(e.to_string()))?;
        let cp = Checkpoint::new(
            run.workspace_id.clone(),
            morn_kernel::ids::WorkPackageId::new(run.id.to_string()),
            format!("{:?}", run.status),
            payload,
        );
        self.checkpoints.push(cp.clone());
        Ok(cp)
    }

    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// Load a persisted run (after restart) and resume with a drift check.
    pub fn load_and_resume(
        &mut self,
        run_id: &WorkflowRunId,
        expected_world_version: &str,
        actual_world_version: &str,
        harness_available: bool,
    ) -> Result<()> {
        let run = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found(format!("run {run_id}")))?;
        if expected_world_version != actual_world_version {
            let ws = run.workspace_id.clone();
            self.drifts.push(DriftRecord {
                id: DriftRecordId::generate_with("drift"),
                run_id: run_id.clone(),
                dimension: "world_version".to_string(),
                expected: expected_world_version.to_string(),
                actual: actual_world_version.to_string(),
                severity: "high".to_string(),
                detected_at: Timestamp::now(),
            });
            self.attention.push(AttentionItem::new(
                ws,
                AttentionKind::ResumeDrift,
                format!("run {run_id} drift detected"),
                format!("world version {expected_world_version} -> {actual_world_version}"),
                AttentionPriority::High,
            ));
            run.transition(RunStatus::Blocked)?;
            return Ok(());
        }
        if !harness_available {
            self.attention.push(AttentionItem::new(
                run.workspace_id.clone(),
                AttentionKind::CapabilityGap,
                format!("run {run_id} harness unavailable"),
                "required harness is not available on resume".to_string(),
                AttentionPriority::High,
            ));
            run.transition(RunStatus::Blocked)?;
            return Ok(());
        }
        run.transition(RunStatus::Running)?;
        Ok(())
    }

    pub fn drifts(&self) -> &[DriftRecord] {
        &self.drifts
    }

    pub fn attention_items(&self) -> &[AttentionItem] {
        &self.attention
    }

    pub fn open_attention(&self) -> Vec<&AttentionItem> {
        self.attention
            .iter()
            .filter(|a| a.status == "open")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowStep, WorkflowStepKind};
    use morn_kernel::ids::WorkspaceId;

    fn workflow(ws: WorkspaceId, name: &str) -> WorkflowDefinition {
        WorkflowDefinition::new(ws, name)
            .add_step(WorkflowStep::new("analyze", WorkflowStepKind::Auto))
            .add_step(WorkflowStep::new("review", WorkflowStepKind::SignalWait))
            .add_step(WorkflowStep::new("release", WorkflowStepKind::Auto))
    }

    fn run(runtime: &mut DurableRuntime, ws: &WorkspaceId) -> WorkflowRunId {
        let def = workflow(ws.clone(), "dataset-to-claim");
        let def_id = def.id.clone();
        runtime.register_workflow(def);
        runtime.start_run(&def_id, None).unwrap().id
    }

    #[test]
    fn wait_human_signal_and_resume() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let run_id = run(&mut rt, &ws);
        rt.wait_for_signal(&run_id, SignalKind::HumanApproval, 3600)
            .unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::WaitingSignal);
        let signal = Signal::new(
            run_id.clone(),
            SignalKind::HumanApproval,
            "approved",
            "pi-1",
            "pi",
        );
        rt.deliver_signal(signal).unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Running);
    }

    #[test]
    fn duplicate_signal_is_rejected() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let run_id = run(&mut rt, &ws);
        rt.wait_for_signal(&run_id, SignalKind::ReviewerResponse, 3600)
            .unwrap();
        let signal = Signal::new(
            run_id.clone(),
            SignalKind::ReviewerResponse,
            "ok",
            "reviewer-1",
            "reviewer",
        );
        rt.deliver_signal(signal.clone()).unwrap();
        assert!(
            rt.deliver_signal(signal).is_err(),
            "duplicate signal must be rejected"
        );
    }

    #[test]
    fn timeout_wait_moves_to_blocked_when_no_signal() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let run_id = run(&mut rt, &ws);
        rt.wait_for_signal(&run_id, SignalKind::ExternalEvent, 1)
            .unwrap();
        // no signal delivered; run stays waiting (a timer would fire and block in production)
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::WaitingSignal);
        // cancel path
        let cancel = Signal::new(
            run_id.clone(),
            SignalKind::Cancel,
            "timeout",
            "system",
            "runtime",
        );
        rt.deliver_signal(cancel).unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Cancelled);
    }

    #[test]
    fn retry_then_exhausted_blocks() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let run_id = run(&mut rt, &ws);
        let policy = RetryPolicy::new("qc", 3);
        let policy_id = policy.id.clone();
        rt.add_retry_policy(policy);
        let pol = rt.retry_policies.get(&policy_id).cloned();
        // attempt 1 -> scheduled
        rt.fail_step(&run_id, "analyze", "tool error", pol.as_ref())
            .unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::RetryScheduled);
        rt.retry_now(&run_id).unwrap();
        // attempt 2 -> scheduled
        rt.fail_step(&run_id, "analyze", "tool error", pol.as_ref())
            .unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::RetryScheduled);
        rt.retry_now(&run_id).unwrap();
        // attempt 3 -> exhausted -> blocked
        rt.fail_step(&run_id, "analyze", "tool error", pol.as_ref())
            .unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Blocked);
        assert_eq!(rt.open_attention().len(), 1);
    }

    #[test]
    fn e2_compensation_completes() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let run_id = run(&mut rt, &ws);
        let plan = CompensationPlan::new(run_id.clone(), "create_order", "cancel_order");
        let plan_id = plan.id.clone();
        rt.compensate(&run_id, plan).unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Compensating);
        rt.complete_compensation(&run_id, &plan_id, "order cancelled")
            .unwrap();
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Completed);
        assert_eq!(rt.compensations()[0].status, "completed");
    }

    #[test]
    fn budget_stop_blocks_run() {
        let ws = WorkspaceId::generate();
        let mut rt = DurableRuntime::new();
        let def = workflow(ws.clone(), "budget-run");
        let def_id = def.id.clone();
        rt.register_workflow(def);
        let run_id = rt
            .start_run(
                &def_id,
                Some(BudgetGuard::new(
                    WorkflowRunId::generate(),
                    100.0,
                    10.0,
                    3600.0,
                )),
            )
            .unwrap()
            .id;
        if let Some(b) = rt.budgets.get_mut(&run_id) {
            b.record(250.0, 5.0);
        }
        assert!(rt.check_budget(&run_id).unwrap());
        assert_eq!(rt.run(&run_id).unwrap().status, RunStatus::Blocked);
        assert!(rt
            .open_attention()
            .iter()
            .any(|a| a.kind == AttentionKind::BudgetRisk));
    }

    #[test]
    fn restart_resume_via_checkpoint_and_drift() {
        let ws = WorkspaceId::generate();
        // Process A: run, wait for signal, checkpoint.
        let mut rt_a = DurableRuntime::new();
        let run_id = run(&mut rt_a, &ws);
        rt_a.wait_for_signal(&run_id, SignalKind::HumanApproval, 3600)
            .unwrap();
        let cp = rt_a.checkpoint(&run_id).unwrap();
        let payload = cp.payload_json.clone();

        // "Process restart": fresh runtime, restore run from checkpoint payload.
        let restored: WorkflowRun = serde_json::from_str(&payload).unwrap();
        let mut rt_b = DurableRuntime::new();
        rt_b.runs.insert(restored.id.clone(), restored.clone());

        // Drift: world version changed -> blocked + attention.
        rt_b.load_and_resume(&run_id, "world-v1", "world-v2", true)
            .unwrap();
        assert_eq!(rt_b.run(&run_id).unwrap().status, RunStatus::Blocked);
        assert!(rt_b.drifts().iter().any(|d| d.dimension == "world_version"));
        assert!(rt_b
            .open_attention()
            .iter()
            .any(|a| a.kind == AttentionKind::ResumeDrift));

        // No drift -> resumes to Running, then signal can be delivered.
        let mut rt_c = DurableRuntime::new();
        rt_c.runs.insert(restored.id.clone(), restored);
        rt_c.load_and_resume(&run_id, "world-v1", "world-v1", true)
            .unwrap();
        assert_eq!(rt_c.run(&run_id).unwrap().status, RunStatus::Running);
        rt_c.deliver_signal(Signal::new(
            run_id.clone(),
            SignalKind::HumanApproval,
            "ok",
            "pi",
            "pi",
        ))
        .unwrap();
        assert_eq!(rt_c.run(&run_id).unwrap().status, RunStatus::Running);
    }
}
