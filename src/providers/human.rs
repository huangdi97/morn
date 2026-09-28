//! human — HumanProvider: humans are first-class executors.
//!
//! At least supports: assignment, approval, reject/request-more-evidence,
//! acceptance, timeout/escalation. A human decision is business truth made by
//! an identifiable principal — never synthesized.

use serde::{Deserialize, Serialize};

/// The kind of human task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HumanTaskKind {
    Assignment,
    Approval,
    Acceptance,
    Review,
}

/// A human assignment (task routed to a role, not a permanent session).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanAssignment {
    pub id: String,
    pub work_ref: String,
    pub kind: HumanTaskKind,
    /// The role this task is routed to, e.g. production-owner
    pub role: String,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_refs_json: Option<String>,
    pub created_at: String,
}

/// The human decision on an assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HumanDecisionType {
    Approve,
    Reject,
    RequestMoreEvidence,
    Accept,
    Escalate,
}

/// The recorded human decision (business truth by an identifiable actor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanDecision {
    pub id: String,
    pub assignment_ref: String,
    pub decision: HumanDecisionType,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    pub decided_at: String,
}

impl HumanDecision {
    pub fn new(id: &str, assignment_ref: &str, decision: HumanDecisionType, actor: &str) -> Self {
        Self {
            id: id.to_string(),
            assignment_ref: assignment_ref.to_string(),
            decision,
            actor: actor.to_string(),
            reason: None,
            evidence_refs: Vec::new(),
            decided_at: crate::work::now_iso(),
        }
    }
}

/// HumanProvider — routes tasks to roles and records decisions.
pub trait HumanProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    /// Assigns a task to a role. SLA/escalation is the timeout path.
    fn assign(&self, assignment: HumanAssignment) -> Result<String, String>;
    /// Records a human decision (assignment / approval / reject / acceptance /
    /// escalation).
    fn record_decision(&self, decision: HumanDecision) -> Result<(), String>;
    /// Escalates an assignment whose SLA expired (timeout/escalation path).
    fn escalate(&self, assignment_ref: &str, reason: &str) -> Result<HumanDecision, String>;
}

/// Reference human provider: stores assignments and decisions in memory.
/// Decisions are only ever recorded from real HumanDecision values — never
/// synthesized.
#[derive(Default)]
pub struct ReferenceHumanProvider {
    assignments: std::sync::Mutex<
        std::collections::BTreeMap<String, (HumanAssignment, Option<HumanDecision>)>,
    >,
}

impl ReferenceHumanProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HumanProvider for ReferenceHumanProvider {
    fn provider_id(&self) -> &str {
        "reference-human"
    }

    fn assign(&self, assignment: HumanAssignment) -> Result<String, String> {
        if assignment.role.is_empty() || assignment.question.is_empty() {
            return Err("assignment requires role and question".to_string());
        }
        let mut assignments = self.assignments.lock().unwrap();
        let id = assignment.id.clone();
        assignments.insert(id.clone(), (assignment, None));
        Ok(id)
    }

    fn record_decision(&self, decision: HumanDecision) -> Result<(), String> {
        let mut assignments = self.assignments.lock().unwrap();
        let entry = assignments
            .get_mut(&decision.assignment_ref)
            .ok_or_else(|| format!("unknown assignment: {}", decision.assignment_ref))?;
        // INVARIANT: a decision is recorded once; a second decision for the
        // same assignment is an error (the human decides once per round).
        if entry.1.is_some() {
            return Err(format!(
                "assignment {} already has a decision",
                decision.assignment_ref
            ));
        }
        entry.1 = Some(decision);
        Ok(())
    }

    fn escalate(&self, assignment_ref: &str, reason: &str) -> Result<HumanDecision, String> {
        let decision = HumanDecision::new(
            &format!("escalation-{assignment_ref}"),
            assignment_ref,
            HumanDecisionType::Escalate,
            "morn-escalation",
        );
        let decision = HumanDecision {
            reason: Some(reason.to_string()),
            ..decision
        };
        self.record_decision(decision.clone())?;
        Ok(decision)
    }
}
