//! workcell — Capability Resolver / Workcell (Phase 11).
//!
//! Resolver flow (Goal §15): Work requirements → hard constraints → eligible
//! capabilities → qualification / site admission → authority compatibility →
//! execution environment compatibility → cost/latency/risk scoring → minimum
//! sufficient executor mix → bindings.
//!
//! Hard constraints and soft scoring are SEPARATE: eligibility is never an
//! LLM guess (the LLM may help parse natural-language requirements, but the
//! final eligibility check is deterministic). A Workcell is the dynamic
//! execution composition of ONE Work — not a permanent multi-agent department.

use serde::{Deserialize, Serialize};

/// The executor kind of a Workcell member. "Agent is one kind of executor" —
/// a Workcell may contain zero agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutorKind {
    Agent,
    Rule,
    Solver,
    Human,
    Connector,
}

/// One member of a Workcell: a resolved executor for one required capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkcellMember {
    pub role: String,
    pub executor_kind: ExecutorKind,
    pub capability_name: String,
    pub capability_digest: String,
    pub provider_id: String,
}

/// The dynamic execution composition of one Work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workcell {
    pub work_ref: String,
    pub members: Vec<WorkcellMember>,
    /// Soft scores (informational, never eligibility).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soft_scores: Vec<(String, f64)>,
}

impl Workcell {
    /// Whether this Workcell contains zero agents (Rule + Solver + Human only).
    pub fn is_zero_agent(&self) -> bool {
        !self
            .members
            .iter()
            .any(|m| m.executor_kind == ExecutorKind::Agent)
    }
}

/// Typed resolution failures — a capability shortage is never hidden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolutionFailure {
    CapabilityMissing {
        capability: String,
    },
    QualificationMissing {
        capability: String,
    },
    SiteNotAdmitted {
        capability: String,
        site: String,
    },
    AuthorityCeilingInsufficient {
        capability: String,
        action: String,
    },
    BudgetInsufficient {
        required: f64,
        currency: String,
        available: f64,
    },
    ProviderUnavailable {
        capability: String,
    },
}

/// One candidate capability from the catalog (a snapshot of its supply-chain
/// state at resolution time).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateCapability {
    pub name: String,
    pub digest: String,
    pub provides: Vec<String>,
    pub executor_kind: ExecutorKind,
    pub provider_id: String,
    /// Authority actions this candidate may ever be granted.
    pub authority_max: Vec<String>,
    /// Whether the candidate has a PASSED, non-expired qualification.
    pub qualified: bool,
    /// Whether the candidate is admitted at the required site.
    pub site_admitted: bool,
    /// Whether the candidate's provider is available right now.
    pub provider_available: bool,
    /// Soft scoring inputs (cost / latency).
    pub cost: f64,
    pub latency_p95_seconds: f64,
}

/// Hard requirements for a Work (eligibility is decided against these).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionRequirements {
    pub work_ref: String,
    /// Required capability interfaces (provides).
    pub required_capabilities: Vec<String>,
    pub site: String,
    pub authority_ceiling: Vec<String>,
    pub budget: crate::work::CostAmount,
}

/// The deterministic resolver: hard filter first, soft scoring second,
/// minimum sufficient mix last.
pub struct WorkcellResolver;

impl WorkcellResolver {
    /// Resolves a Workcell from the candidate catalog.
    ///
    /// Hard constraints (in order): capability existence, qualification,
    /// site admission, authority ceiling, budget, provider availability.
    /// Any failure is a typed `ResolutionFailure`. Soft scoring then ranks
    /// the eligible candidates by cost/latency (informational).
    pub fn resolve(
        &self,
        requirements: &ResolutionRequirements,
        candidates: &[CandidateCapability],
    ) -> Result<Workcell, ResolutionFailure> {
        let mut members = Vec::new();
        let mut soft_scores = Vec::new();

        for required in &requirements.required_capabilities {
            // Hard filter: candidates that provide the required capability.
            let eligible: Vec<&CandidateCapability> = candidates
                .iter()
                .filter(|c| c.provides.iter().any(|p| p == required))
                .collect();

            let Some(candidate) = eligible.first() else {
                return Err(ResolutionFailure::CapabilityMissing {
                    capability: required.clone(),
                });
            };

            // Hard: qualification.
            if !candidate.qualified {
                return Err(ResolutionFailure::QualificationMissing {
                    capability: required.clone(),
                });
            }
            // Hard: site admission.
            if !candidate.site_admitted {
                return Err(ResolutionFailure::SiteNotAdmitted {
                    capability: required.clone(),
                    site: requirements.site.clone(),
                });
            }
            // Hard: authority ceiling (every candidate action must be within
            // the Work's ceiling).
            for action in &candidate.authority_max {
                if !requirements.authority_ceiling.iter().any(|a| a == action) {
                    return Err(ResolutionFailure::AuthorityCeilingInsufficient {
                        capability: required.clone(),
                        action: action.clone(),
                    });
                }
            }
            // Hard: budget.
            if candidate.cost > requirements.budget.amount {
                return Err(ResolutionFailure::BudgetInsufficient {
                    required: candidate.cost,
                    currency: requirements.budget.currency.clone(),
                    available: requirements.budget.amount,
                });
            }
            // Hard: provider availability.
            if !candidate.provider_available {
                return Err(ResolutionFailure::ProviderUnavailable {
                    capability: required.clone(),
                });
            }

            // Soft scoring: cost/latency rank (informational only).
            let score = 1.0 / (1.0 + candidate.cost / 10.0 + candidate.latency_p95_seconds / 60.0);
            soft_scores.push((candidate.name.clone(), (score * 1000.0).round() / 1000.0));

            members.push(WorkcellMember {
                role: required.clone(),
                executor_kind: candidate.executor_kind,
                capability_name: candidate.name.clone(),
                capability_digest: candidate.digest.clone(),
                provider_id: candidate.provider_id.clone(),
            });
        }

        Ok(Workcell {
            work_ref: requirements.work_ref.clone(),
            members,
            soft_scores,
        })
    }
}
