//! Predictor Registry + six baseline predictors: Duration, FailureRisk, Cost,
//! HumanIntervention, OutcomeAcceptance, TransitionRisk.
//!
//! Minimum model strategy: deterministic/historical baselines first; only with
//! sufficient data do we ever consider stronger models. Predictions are stored
//! before the actual outcome; the actual is recorded separately and never
//! rewrites the original prediction.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{CalibrationReportId, ModelDriftReportId, PredictionId, PredictorSpecId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::state::FeatureVector;

/// The six predictor targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PredictorTarget {
    Duration,
    FailureRisk,
    Cost,
    HumanIntervention,
    OutcomeAcceptance,
    TransitionRisk,
}

impl PredictorTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            PredictorTarget::Duration => "duration",
            PredictorTarget::FailureRisk => "failure_risk",
            PredictorTarget::Cost => "cost",
            PredictorTarget::HumanIntervention => "human_intervention",
            PredictorTarget::OutcomeAcceptance => "outcome_acceptance",
            PredictorTarget::TransitionRisk => "transition_risk",
        }
    }
}

/// Predictor lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PredictorStatus {
    Draft,
    Trained,
    Evaluated,
    Calibrated,
    Shadow,
    ApprovedForDecisionSupport,
    Restricted,
    Stale,
    Suspended,
    Deprecated,
}

/// A predictor spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictorSpec {
    pub id: PredictorSpecId,
    pub name: String,
    pub target: PredictorTarget,
    pub version: Version,
    pub context_of_use: Vec<String>,
    pub feature_schema_ref: String,
    pub status: PredictorStatus,
    pub created_at: Timestamp,
}

impl PredictorSpec {
    pub fn new(
        name: impl Into<String>,
        target: PredictorTarget,
        context_of_use: Vec<String>,
    ) -> Self {
        Self {
            id: PredictorSpecId::generate_with("pspec"),
            name: name.into(),
            target,
            version: Version::v1(),
            context_of_use,
            feature_schema_ref: "morn.feature.v1".to_string(),
            status: PredictorStatus::Draft,
            created_at: Timestamp::now(),
        }
    }
}

/// A prediction made before the actual outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub id: PredictionId,
    pub predictor_id: PredictorSpecId,
    pub predictor_version: Version,
    pub target: PredictorTarget,
    pub value: f64,
    pub interval_lo: f64,
    pub interval_hi: f64,
    pub confidence: f64,
    pub uncertainty: f64,
    pub context_match: bool,
    pub feature_schema: String,
    pub evidence_refs: Vec<String>,
    pub generated_at: Timestamp,
    /// Actual outcome recorded separately AFTER the prediction (never rewritten).
    pub actual: Option<f64>,
    pub prediction_error: Option<f64>,
}

/// Calibration report (brier-style over stored prediction/actual pairs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub id: CalibrationReportId,
    pub predictor_id: PredictorSpecId,
    pub n: u32,
    pub brier: f64,
    pub buckets: Vec<(String, f64, f64)>, // (bucket, predicted, observed)
    pub created_at: Timestamp,
}

/// Model drift report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDriftReport {
    pub id: ModelDriftReportId,
    pub predictor_id: PredictorSpecId,
    pub dimension: String,
    pub drift_score: f64,
    pub detected_at: Timestamp,
}

/// Baseline parameters learned at training time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BaselineParams {
    pub mean: f64,
    pub std: f64,
    pub rate: f64, // for risk/acceptance/transition targets
    pub n: u32,
    pub insufficient_data: bool,
}

const MIN_DATA: u32 = 3;

/// One predictor's state inside the registry.
#[derive(Debug, Clone)]
pub struct PredictorState {
    pub spec: PredictorSpec,
    pub params: BaselineParams,
    pub predictions: Vec<Prediction>,
    pub calibrations: Vec<CalibrationReport>,
    pub drifts: Vec<ModelDriftReport>,
}

/// Predictor Registry.
#[derive(Debug, Default)]
pub struct PredictorRegistry {
    pub predictors: Vec<PredictorState>,
}

impl PredictorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, spec: PredictorSpec) -> PredictorSpec {
        self.predictors.push(PredictorState {
            spec: spec.clone(),
            params: BaselineParams::default(),
            predictions: Vec::new(),
            calibrations: Vec::new(),
            drifts: Vec::new(),
        });
        spec
    }

    pub fn predictor(&self, id: &PredictorSpecId) -> Option<&PredictorState> {
        self.predictors.iter().find(|p| p.spec.id == *id)
    }

    pub fn predictor_mut(&mut self, id: &PredictorSpecId) -> Option<&mut PredictorState> {
        self.predictors.iter_mut().find(|p| p.spec.id == *id)
    }

    /// Train a historical baseline from episodes. `values` are the observed
    /// outcomes per episode for the target. Insufficient data -> insufficient_data.
    pub fn train(&mut self, id: &PredictorSpecId, values: &[f64], successes: usize) -> Result<()> {
        let state = self
            .predictor_mut(id)
            .ok_or_else(|| Error::not_found(format!("predictor {id}")))?;
        if values.len() < MIN_DATA as usize {
            state.params.insufficient_data = true;
            state.params.n = values.len() as u32;
            state.spec.status = PredictorStatus::Draft;
            return Err(Error::validation(format!(
                "insufficient data ({} episodes) for {}; baseline-only, do not fake performance",
                values.len(),
                state.spec.target.as_str()
            )));
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let rate = successes as f64 / n;
        state.params = BaselineParams {
            mean,
            std: variance.sqrt(),
            rate,
            n: values.len() as u32,
            insufficient_data: false,
        };
        state.spec.status = PredictorStatus::Trained;
        Ok(())
    }

    /// Predict with uncertainty and context-match. Restricted/out-of-context
    /// predictors return a low-confidence prediction that cannot auto-decide.
    pub fn predict(
        &mut self,
        id: &PredictorSpecId,
        _features: &FeatureVector,
        context: &str,
        evidence_refs: Vec<String>,
    ) -> Result<Prediction> {
        let state = self
            .predictor_mut(id)
            .ok_or_else(|| Error::not_found(format!("predictor {id}")))?;
        if state.params.insufficient_data || state.params.n == 0 {
            return Err(Error::validation(format!(
                "predictor {} has insufficient data; refusing to fabricate a prediction",
                state.spec.name
            )));
        }
        let target = state.spec.target;
        let (value, interval_lo, interval_hi, uncertainty) = match target {
            PredictorTarget::Duration
            | PredictorTarget::Cost
            | PredictorTarget::HumanIntervention => {
                let std = state.params.std.max(0.001);
                (
                    state.params.mean,
                    state.params.mean - 1.96 * std,
                    state.params.mean + 1.96 * std,
                    std,
                )
            }
            PredictorTarget::FailureRisk
            | PredictorTarget::OutcomeAcceptance
            | PredictorTarget::TransitionRisk => {
                let r = state.params.rate.clamp(0.0, 1.0);
                let se = (r * (1.0 - r) / state.params.n as f64).sqrt().max(0.001);
                (r, (r - 1.96 * se).max(0.0), (r + 1.96 * se).min(1.0), se)
            }
        };
        let context_match = state.spec.context_of_use.is_empty()
            || state.spec.context_of_use.iter().any(|c| c == context);
        let confidence = if context_match {
            1.0 - uncertainty.min(0.99)
        } else {
            0.05
        };
        let prediction = Prediction {
            id: PredictionId::generate_with("pred"),
            predictor_id: state.spec.id.clone(),
            predictor_version: state.spec.version,
            target,
            value,
            interval_lo,
            interval_hi,
            confidence,
            uncertainty,
            context_match,
            feature_schema: state.spec.feature_schema_ref.clone(),
            evidence_refs,
            generated_at: Timestamp::now(),
            actual: None,
            prediction_error: None,
        };
        state.predictions.push(prediction.clone());
        Ok(prediction)
    }

    /// Record the actual outcome for a stored prediction. The original
    /// prediction is never rewritten; error is derived.
    pub fn record_actual(
        &mut self,
        id: &PredictorSpecId,
        prediction_id: &PredictionId,
        actual: f64,
    ) -> Result<()> {
        let state = self
            .predictor_mut(id)
            .ok_or_else(|| Error::not_found(format!("predictor {id}")))?;
        let pred = state
            .predictions
            .iter_mut()
            .find(|p| p.id == *prediction_id)
            .ok_or_else(|| Error::not_found(format!("prediction {prediction_id}")))?;
        if pred.actual.is_some() {
            return Err(Error::conflict(format!(
                "prediction {prediction_id} already has an actual"
            )));
        }
        pred.actual = Some(actual);
        pred.prediction_error = Some(actual - pred.value);
        Ok(())
    }

    /// Calibration (Brier score over stored prediction/actual pairs).
    pub fn calibrate(&mut self, id: &PredictorSpecId) -> Result<CalibrationReport> {
        let state = self
            .predictor_mut(id)
            .ok_or_else(|| Error::not_found(format!("predictor {id}")))?;
        let pairs: Vec<(f64, f64)> = state
            .predictions
            .iter()
            .filter_map(|p| p.actual.map(|a| (p.value, a)))
            .collect();
        if pairs.is_empty() {
            return Err(Error::validation("no prediction/actual pairs to calibrate"));
        }
        let n = pairs.len() as f64;
        let brier = pairs.iter().map(|(p, a)| (p - a).powi(2)).sum::<f64>() / n;
        let buckets = vec![(
            "all".to_string(),
            pairs[0].0,
            pairs.iter().map(|x| x.1).sum::<f64>() / n,
        )];
        let report = CalibrationReport {
            id: CalibrationReportId::generate_with("cal"),
            predictor_id: state.spec.id.clone(),
            n: pairs.len() as u32,
            brier,
            buckets,
            created_at: Timestamp::now(),
        };
        state.calibrations.push(report.clone());
        if brier <= 0.25 {
            state.spec.status = PredictorStatus::Calibrated;
        }
        Ok(report)
    }

    /// Drift check: high brier or context mismatch rate -> drift report + Stale.
    pub fn drift_check(&mut self, id: &PredictorSpecId) -> Result<ModelDriftReport> {
        let state = self
            .predictor_mut(id)
            .ok_or_else(|| Error::not_found(format!("predictor {id}")))?;
        let mismatch_rate = if state.predictions.is_empty() {
            0.0
        } else {
            state
                .predictions
                .iter()
                .filter(|p| !p.context_match)
                .count() as f64
                / state.predictions.len() as f64
        };
        let last_brier = state.calibrations.last().map(|c| c.brier).unwrap_or(0.5);
        let drift_score = last_brier * 0.7 + mismatch_rate * 0.3;
        let report = ModelDriftReport {
            id: ModelDriftReportId::generate_with("drift"),
            predictor_id: state.spec.id.clone(),
            dimension: if drift_score > 0.35 {
                "calibration_or_context".to_string()
            } else {
                "none".to_string()
            },
            drift_score,
            detected_at: Timestamp::now(),
        };
        state.drifts.push(report.clone());
        if drift_score > 0.35 {
            state.spec.status = PredictorStatus::Stale;
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateEncoder;

    fn reg(target: PredictorTarget) -> PredictorRegistry {
        let mut r = PredictorRegistry::new();
        r.register(PredictorSpec::new(
            format!("{}-predictor", target.as_str()),
            target,
            vec!["biolab".to_string()],
        ));
        r
    }

    fn features() -> FeatureVector {
        StateEncoder::new().encode(
            &crate::state::WorldState::default(),
            &crate::state::WorkState::default(),
            &crate::state::OrganizationState::default(),
            &crate::state::ExecutionState::default(),
            &crate::state::ResourceState::default(),
            &crate::state::EvidenceState::default(),
        )
    }

    #[test]
    fn duration_predictor_train_predict_uncertainty() {
        let mut r = reg(PredictorTarget::Duration);
        let id = r.predictors[0].spec.id.clone();
        r.train(&id, &[100.0, 120.0, 110.0, 130.0], 4).unwrap();
        let p = r
            .predict(&id, &features(), "biolab", vec!["ep-1".to_string()])
            .unwrap();
        assert_eq!(p.target, PredictorTarget::Duration);
        assert!(p.uncertainty > 0.0);
        assert!(p.context_match);
        assert!(p.interval_lo < p.value && p.value < p.interval_hi);
    }

    #[test]
    fn outcome_acceptance_predictor_uses_rate_and_calibrates() {
        let mut r = reg(PredictorTarget::OutcomeAcceptance);
        let id = r.predictors[0].spec.id.clone();
        // 3 accepted out of 4.
        r.train(&id, &[1.0, 0.0, 1.0, 1.0], 3).unwrap();
        let p = r.predict(&id, &features(), "biolab", vec![]).unwrap();
        assert!((p.value - 0.75).abs() < 0.01);
        r.record_actual(&id, &p.id, 1.0).unwrap();
        let cal = r.calibrate(&id).unwrap();
        assert!(cal.n >= 1);
        // Re-recording actual is rejected (prediction immutable).
        assert!(r.record_actual(&id, &p.id, 0.0).is_err());
    }

    #[test]
    fn insufficient_data_refuses_prediction() {
        let mut r = reg(PredictorTarget::Cost);
        let id = r.predictors[0].spec.id.clone();
        // Only 1 episode -> insufficient data.
        assert!(r.train(&id, &[50.0], 1).is_err());
        assert!(r.predictor(&id).unwrap().params.insufficient_data);
        assert!(r.predict(&id, &features(), "biolab", vec![]).is_err());
    }

    #[test]
    fn context_mismatch_low_confidence_and_drift() {
        let mut r = reg(PredictorTarget::FailureRisk);
        let id = r.predictors[0].spec.id.clone();
        r.train(&id, &[1.0, 0.0, 0.0, 0.0], 1).unwrap();
        let p = r.predict(&id, &features(), "pharma", vec![]).unwrap();
        assert!(!p.context_match);
        assert!(p.confidence <= 0.05, "out-of-context cannot auto-decide");
        r.record_actual(&id, &p.id, 1.0).unwrap();
        r.calibrate(&id).unwrap();
        let drift = r.drift_check(&id).unwrap();
        assert!(drift.drift_score >= 0.0);
    }

    #[test]
    fn all_six_targets_registered_and_typed() {
        for target in [
            PredictorTarget::Duration,
            PredictorTarget::FailureRisk,
            PredictorTarget::Cost,
            PredictorTarget::HumanIntervention,
            PredictorTarget::OutcomeAcceptance,
            PredictorTarget::TransitionRisk,
        ] {
            let r = reg(target);
            assert_eq!(r.predictors[0].spec.target, target);
            assert_eq!(
                target.as_str(),
                r.predictors[0].spec.name.split('-').next().unwrap()
            );
        }
    }
}

/// Serializable snapshot of a predictor state (spec + learned params).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictorSnapshot {
    pub spec: PredictorSpec,
    pub params: BaselineParams,
}

impl PredictorRegistry {
    /// Snapshot all predictor states for persistence.
    pub fn snapshot_all(&self) -> Vec<PredictorSnapshot> {
        self.predictors
            .iter()
            .map(|p| PredictorSnapshot {
                spec: p.spec.clone(),
                params: p.params.clone(),
            })
            .collect()
    }

    /// Restore predictor states (spec + params) from a snapshot. Existing
    /// predictions are preserved by matching spec id.
    pub fn restore_all(&mut self, snapshots: Vec<PredictorSnapshot>) {
        for snap in snapshots {
            if let Some(existing) = self
                .predictors
                .iter_mut()
                .find(|p| p.spec.id == snap.spec.id)
            {
                existing.spec = snap.spec;
                existing.params = snap.params;
            } else {
                self.predictors.push(PredictorState {
                    spec: snap.spec,
                    params: snap.params,
                    predictions: Vec::new(),
                    calibrations: Vec::new(),
                    drifts: Vec::new(),
                });
            }
        }
    }
}
