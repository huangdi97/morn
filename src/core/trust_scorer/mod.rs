//! trust_scorer - Scores agents and actions using trust-related metrics.
pub mod evaluator;
pub mod scorer;

pub use evaluator::{
    ComponentQuality, DriftQuality, OutputQuality, ScoreRecord, TraceQuality, TrustEvaluator,
};
pub use scorer::{AgentRanking, ScoreEntry, TrustScorer};

#[cfg(test)]
mod tests;
