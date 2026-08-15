//! IntelligenceProvider: model/LLM/solver are provider capabilities. Two local
//! fixtures run the same conformance; the protocol never owns canonical state.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};

/// A structured completion request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntelligenceRequest {
    pub prompt: String,
    pub context_refs: Vec<String>,
}

/// A normalized completion result (no private chain-of-thought).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceResult {
    pub text: String,
    pub model_version: String,
    pub latency_ms: u64,
    pub cost: f64,
}

/// Provider-neutral intelligence protocol.
pub trait IntelligenceProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn complete(&self, request: &IntelligenceRequest) -> Result<IntelligenceResult>;
}

/// Deterministic fixture A.
#[derive(Debug, Default)]
pub struct RuleIntelligence;

impl IntelligenceProvider for RuleIntelligence {
    fn provider_name(&self) -> &str {
        "rule-intelligence"
    }
    fn complete(&self, request: &IntelligenceRequest) -> Result<IntelligenceResult> {
        if request.prompt.trim().is_empty() {
            return Err(Error::validation("empty prompt"));
        }
        let lower = request.prompt.to_lowercase();
        let text = if lower.contains("deterministic") {
            "deterministic path".to_string()
        } else {
            format!(
                "analyzed: {}",
                request.prompt.chars().take(40).collect::<String>()
            )
        };
        Ok(IntelligenceResult {
            text,
            model_version: "rule-1.0".to_string(),
            latency_ms: 1,
            cost: 0.0,
        })
    }
}

/// Deterministic fixture B.
#[derive(Debug, Default)]
pub struct SolverIntelligence;

impl IntelligenceProvider for SolverIntelligence {
    fn provider_name(&self) -> &str {
        "solver-intelligence"
    }
    fn complete(&self, request: &IntelligenceRequest) -> Result<IntelligenceResult> {
        if request.prompt.trim().is_empty() {
            return Err(Error::validation("empty prompt"));
        }
        Ok(IntelligenceResult {
            text: format!("solved({})", request.prompt.len()),
            model_version: "solver-1.0".to_string(),
            latency_ms: 2,
            cost: 0.0,
        })
    }
}

/// Conformance: both fixtures must pass the same protocol checks.
pub fn run_intelligence_conformance(provider: &dyn IntelligenceProvider) -> Result<()> {
    let req = IntelligenceRequest {
        prompt: "deterministic work".to_string(),
        context_refs: vec!["wp-1".to_string()],
    };
    let res = provider.complete(&req)?;
    if res.text.is_empty() {
        return Err(Error::internal("empty completion"));
    }
    if res.model_version.is_empty() {
        return Err(Error::internal("missing model version"));
    }
    // Failure path: empty prompt is rejected.
    let bad = IntelligenceRequest {
        prompt: String::new(),
        context_refs: vec![],
    };
    if provider.complete(&bad).is_ok() {
        return Err(Error::validation("empty prompt must be rejected"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_fixtures_pass_same_conformance() {
        run_intelligence_conformance(&RuleIntelligence).unwrap();
        run_intelligence_conformance(&SolverIntelligence).unwrap();
    }
}
