//! EvolutionPlannerProvider seam: an external/LLM planner may only produce a
//! structured proposal; schema/rule/policy/capability validation gates it
//! before it becomes an EvolutionCandidate. Proposals never mutate production.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::WorkspaceId;

use crate::flywheel::{FlywheelCandidate, Pattern, PatternKind};

/// Raw structured proposal from a provider (JSON text).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerProposal {
    pub raw: String,
}

impl PlannerProposal {
    pub fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }
}

/// Parsed, validated proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedProposal {
    pub candidate_type: String,
    pub affected_work: String,
    pub proposed_change: String,
    pub risk: String,
    pub required_evaluations: Vec<String>,
    pub expected_benefit: String,
    pub rollback_plan: String,
}

/// A planner provider (real LLM or deterministic test provider).
pub trait EvolutionPlannerProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    /// Produce a structured proposal from evidence. Must not mutate anything.
    fn propose(&self, evidence_window: &Pattern) -> Result<PlannerProposal>;
}

/// Deterministic (rule-based) provider used for tests and local fallback.
#[derive(Debug, Default)]
pub struct DeterministicPlannerProvider;

impl EvolutionPlannerProvider for DeterministicPlannerProvider {
    fn provider_name(&self) -> &str {
        "deterministic-planner"
    }

    fn propose(&self, evidence_window: &Pattern) -> Result<PlannerProposal> {
        let (candidate_type, change, risk, benefit) = match evidence_window.kind {
            PatternKind::RepeatedSuccess => (
                "deterministic_distillation",
                format!(
                    "distill repeated stable step '{}' into a deterministic program with actor fallback",
                    evidence_window.affected_work
                ),
                "low",
                "lower cost/latency, stable quality",
            ),
            PatternKind::RepeatedFailure => (
                "harness_patch",
                format!("patch harness for failing step '{}'", evidence_window.affected_work),
                "medium",
                "raise reliability",
            ),
            PatternKind::RepeatedHumanCorrection => (
                "skill",
                format!(
                    "encode repeated human corrections on '{}' into skill guidance",
                    evidence_window.affected_work
                ),
                "medium",
                "fewer human interventions",
            ),
            _ => (
                "workflow",
                format!("optimize workflow step '{}'", evidence_window.affected_work),
                "low",
                "shorter cycle / lower cost",
            ),
        };
        let raw = serde_json::json!({
            "candidate_type": candidate_type,
            "affected_work": evidence_window.affected_work,
            "proposed_change": change,
            "risk": risk,
            "required_evaluations": ["regression", "acceptance", "policy"],
            "expected_benefit": benefit,
            "rollback_plan": format!("rollback {}", evidence_window.affected_work),
        })
        .to_string();
        Ok(PlannerProposal::new(raw))
    }
}

/// Validator: schema parse -> rule -> policy -> capability/authority gate.
#[derive(Debug, Default)]
pub struct ProposalValidator;

impl ProposalValidator {
    pub fn new() -> Self {
        Self
    }

    /// Schema parse: proposal must be valid JSON with required fields.
    pub fn parse(&self, proposal: &PlannerProposal) -> Result<ParsedProposal> {
        let v: serde_json::Value = serde_json::from_str(&proposal.raw).map_err(|e| {
            Error::validation(format!("proposal is not valid structured JSON: {e}"))
        })?;
        let get = |k: &str| {
            v.get(k)
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        };
        let candidate_type = get("candidate_type")
            .ok_or_else(|| Error::validation("proposal missing candidate_type"))?;
        let affected_work = get("affected_work")
            .ok_or_else(|| Error::validation("proposal missing affected_work"))?;
        let proposed_change = get("proposed_change")
            .ok_or_else(|| Error::validation("proposal missing proposed_change"))?;
        let risk = get("risk").unwrap_or_else(|| "low".to_string());
        let expected_benefit = get("expected_benefit").unwrap_or_default();
        let rollback_plan = get("rollback_plan").unwrap_or_else(|| "rollback".to_string());
        let required_evaluations = v
            .get("required_evaluations")
            .and_then(serde_json::Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        Ok(ParsedProposal {
            candidate_type,
            affected_work,
            proposed_change,
            risk,
            required_evaluations,
            expected_benefit,
            rollback_plan,
        })
    }

    /// Rule validation: candidate_type must be known and change non-empty.
    pub fn rule_validate(&self, parsed: &ParsedProposal) -> Result<()> {
        const KNOWN: [&str; 7] = [
            "skill",
            "workflow",
            "harness_patch",
            "deterministic_distillation",
            "workcell",
            "role_suggestion",
            "software_replacement",
        ];
        if !KNOWN.contains(&parsed.candidate_type.as_str()) {
            return Err(Error::validation(format!(
                "unknown candidate_type '{}'",
                parsed.candidate_type
            )));
        }
        if parsed.proposed_change.trim().is_empty() {
            return Err(Error::validation("proposed_change must not be empty"));
        }
        Ok(())
    }

    /// Policy validation: proposal must not claim production mutation or deploy.
    pub fn policy_validate(&self, parsed: &ParsedProposal) -> Result<()> {
        let lower = parsed.proposed_change.to_lowercase();
        for forbidden in [
            "auto deploy",
            "deploy production",
            "mutate production",
            "self-modify",
        ] {
            if lower.contains(forbidden) {
                return Err(Error::validation(format!(
                    "proposal violates policy (forbidden: {forbidden})"
                )));
            }
        }
        Ok(())
    }

    /// Capability/authority validation: proposed capability must be known or a gap recorded.
    pub fn capability_validate(&self, parsed: &ParsedProposal, available: &[String]) -> Result<()> {
        // Role/software suggestions are advisory only and require no capability.
        if matches!(
            parsed.candidate_type.as_str(),
            "role_suggestion" | "software_replacement"
        ) {
            return Ok(());
        }
        // For capability candidates, at least a generic capability must exist.
        if available.is_empty() && !parsed.required_evaluations.is_empty() {
            return Err(Error::validation(
                "no available capability and no evaluation path: do not fabricate",
            ));
        }
        Ok(())
    }

    /// Convert a validated proposal into a FlywheelCandidate.
    pub fn to_candidate(
        &self,
        parsed: ParsedProposal,
        workspace_id: WorkspaceId,
        evidence_window: &Pattern,
        provider_name: &str,
    ) -> FlywheelCandidate {
        FlywheelCandidate {
            id: morn_kernel::ids::FlywheelCandidateId::generate_with("fwc"),
            workspace_id,
            candidate_type: parsed.candidate_type,
            affected_work: parsed.affected_work,
            source_evidence_window: vec![
                evidence_window
                    .evidence_window_start
                    .clone()
                    .unwrap_or_default(),
                evidence_window
                    .evidence_window_end
                    .clone()
                    .unwrap_or_default(),
            ],
            baseline_metrics: format!(
                "quality={:.2} latency_ms={} cost={:.2} human={}",
                evidence_window.baseline_quality,
                evidence_window.baseline_latency_ms,
                evidence_window.baseline_cost,
                evidence_window.baseline_human_interventions
            ),
            proposed_change: parsed.proposed_change,
            risk: parsed.risk,
            required_evaluations: parsed.required_evaluations,
            expected_benefit: parsed.expected_benefit,
            rollback_plan: parsed.rollback_plan,
            status: format!("proposed_via_{provider_name}"),
            created_at: morn_kernel::time::Timestamp::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flywheel::PatternKind;

    fn pattern() -> Pattern {
        Pattern {
            id: morn_kernel::ids::FlywheelPatternId::generate(),
            kind: PatternKind::RepeatedSuccess,
            affected_work: "qc".to_string(),
            evidence_window_start: Some("t0".to_string()),
            evidence_window_end: Some("t1".to_string()),
            count: 5,
            baseline_quality: 1.0,
            baseline_latency_ms: 1200,
            baseline_cost: 4.0,
            baseline_human_interventions: 0,
            created_at: morn_kernel::time::Timestamp::now(),
        }
    }

    #[test]
    fn deterministic_provider_proposal_validates_to_candidate() {
        let provider = DeterministicPlannerProvider::default();
        let validator = ProposalValidator::new();
        let p = pattern();
        let proposal = provider.propose(&p).unwrap();
        let parsed = validator.parse(&proposal).unwrap();
        validator.rule_validate(&parsed).unwrap();
        validator.policy_validate(&parsed).unwrap();
        validator
            .capability_validate(&parsed, &["*".to_string()])
            .unwrap();
        let candidate = validator.to_candidate(
            parsed,
            WorkspaceId::generate(),
            &p,
            provider.provider_name(),
        );
        assert_eq!(candidate.candidate_type, "deterministic_distillation");
        assert!(candidate.status.contains("deterministic-planner"));
    }

    #[test]
    fn policy_violating_proposal_rejected() {
        let validator = ProposalValidator::new();
        let proposal = PlannerProposal::new(
            r#"{"candidate_type":"workflow","affected_work":"qc","proposed_change":"auto deploy production"}"#,
        );
        let parsed = validator.parse(&proposal).unwrap();
        assert!(validator.policy_validate(&parsed).is_err());
    }

    #[test]
    fn malformed_or_unknown_proposal_rejected() {
        let validator = ProposalValidator::new();
        // Not JSON
        assert!(validator.parse(&PlannerProposal::new("not json")).is_err());
        // Unknown candidate_type
        let proposal = PlannerProposal::new(
            r#"{"candidate_type":"world_model","affected_work":"qc","proposed_change":"x"}"#,
        );
        let parsed = validator.parse(&proposal).unwrap();
        assert!(validator.rule_validate(&parsed).is_err());
    }

    #[test]
    fn provider_failure_is_surfaced_not_silenced() {
        struct FailingProvider;
        impl EvolutionPlannerProvider for FailingProvider {
            fn provider_name(&self) -> &str {
                "failing"
            }
            fn propose(&self, _: &Pattern) -> Result<PlannerProposal> {
                Err(Error::external("provider unavailable"))
            }
        }
        let provider = FailingProvider;
        assert!(provider.propose(&pattern()).is_err());
    }
}
