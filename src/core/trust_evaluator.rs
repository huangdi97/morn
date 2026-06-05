use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScoreRecord {
    pub timestamp: DateTime<Utc>,
    pub component_id: String,
    pub output_quality: f64,
    pub execution_success: f64,
    pub latency_score: f64,
    pub user_feedback: f64,
    pub overall: f64,
}

#[derive(Debug, Clone)]
pub struct OutputQuality {
    pub content_relevance: f64,
    pub format_compliance: f64,
    pub completeness: f64,
}

impl OutputQuality {
    pub fn score(&self) -> f64 {
        (self.content_relevance + self.format_compliance + self.completeness) / 3.0
    }
}

#[derive(Debug, Clone)]
pub struct TraceQuality {
    pub call_chain_completeness: f64,
    pub error_rate: f64,
    pub retry_count: u32,
}

impl TraceQuality {
    pub fn score(&self) -> f64 {
        self.call_chain_completeness * 0.5
            + (1.0 - self.error_rate) * 0.3
            + (1.0 - (self.retry_count as f64 / 10.0).min(1.0)) * 0.2
    }
}

#[derive(Debug, Clone)]
pub struct ComponentQuality {
    pub init_success_rate: f64,
    pub uptime_percentage: f64,
    pub resource_efficiency: f64,
}

impl ComponentQuality {
    pub fn score(&self) -> f64 {
        self.init_success_rate * 0.4 + self.uptime_percentage * 0.3 + self.resource_efficiency * 0.3
    }
}

#[derive(Debug, Clone)]
pub struct DriftQuality {
    pub recent_performance: Vec<f64>,
    pub historical_avg: f64,
    pub trend_direction: f64,
}

impl DriftQuality {
    pub fn score(&self) -> f64 {
        if self.recent_performance.is_empty() {
            return self.historical_avg;
        }
        let recent_avg: f64 =
            self.recent_performance.iter().sum::<f64>() / self.recent_performance.len() as f64;
        let drift = (recent_avg - self.historical_avg).abs();
        (1.0 - drift.min(1.0)) * 0.7 + recent_avg * 0.3
    }
}

pub struct TrustEvaluator {
    history: Vec<ScoreRecord>,
}

impl TrustEvaluator {
    pub fn new() -> Self {
        TrustEvaluator {
            history: Vec::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        component_id: &str,
        output: OutputQuality,
        trace: TraceQuality,
        component: ComponentQuality,
        drift: DriftQuality,
        user_feedback: f64,
    ) -> f64 {
        let output_score = output.score();
        let trace_score = trace.score();
        let component_score = component.score();
        let drift_score = drift.score();

        let overall =
            output_score * 0.3 + trace_score * 0.3 + component_score * 0.2 + user_feedback * 0.2;

        let record = ScoreRecord {
            timestamp: Utc::now(),
            component_id: component_id.to_string(),
            output_quality: output_score,
            execution_success: trace_score,
            latency_score: component_score,
            user_feedback,
            overall,
        };

        self.history.push(record);
        overall
    }

    pub fn calculate_trust_score(
        output_quality: f64,
        execution_success_rate: f64,
        avg_latency_ms: f64,
        user_feedback: f64,
    ) -> f64 {
        let latency_score = if avg_latency_ms > 0.0 {
            (1000.0 / avg_latency_ms).min(1.0)
        } else {
            0.0
        };

        output_quality * 0.3
            + execution_success_rate * 0.3
            + latency_score * 0.2
            + user_feedback * 0.2
    }

    pub fn get_history(&self, component_id: &str) -> Vec<&ScoreRecord> {
        self.history
            .iter()
            .filter(|r| r.component_id == component_id)
            .collect()
    }

    pub fn get_trend(&self, component_id: &str, window: usize) -> Vec<f64> {
        let scores: Vec<f64> = self
            .history
            .iter()
            .filter(|r| r.component_id == component_id)
            .map(|r| r.overall)
            .collect();
        let len = scores.len();
        if len <= window {
            return scores;
        }
        scores[len - window..].to_vec()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_formula() {
        let score = TrustEvaluator::calculate_trust_score(0.9, 0.95, 200.0, 0.8);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_high_quality() {
        let score = TrustEvaluator::calculate_trust_score(1.0, 1.0, 100.0, 1.0);
        assert!(score > 0.8);
    }

    #[test]
    fn test_low_quality() {
        let score = TrustEvaluator::calculate_trust_score(0.0, 0.0, 10000.0, 0.0);
        assert!(score < 0.5);
    }

    #[test]
    fn test_evaluator_record() {
        let mut evaluator = TrustEvaluator::new();
        let output = OutputQuality {
            content_relevance: 0.9,
            format_compliance: 0.8,
            completeness: 0.85,
        };
        let trace = TraceQuality {
            call_chain_completeness: 0.95,
            error_rate: 0.05,
            retry_count: 1,
        };
        let component = ComponentQuality {
            init_success_rate: 0.98,
            uptime_percentage: 0.99,
            resource_efficiency: 0.8,
        };
        let drift = DriftQuality {
            recent_performance: vec![0.8, 0.85, 0.9],
            historical_avg: 0.8,
            trend_direction: 0.1,
        };

        let score = evaluator.evaluate("comp-1", output, trace, component, drift, 0.9);
        assert!(score > 0.0);
        assert_eq!(evaluator.get_history("comp-1").len(), 1);
    }

    #[test]
    fn test_trend_analysis() {
        let mut evaluator = TrustEvaluator::new();
        for i in 0..10 {
            let output = OutputQuality {
                content_relevance: 0.5 + i as f64 * 0.05,
                format_compliance: 0.6,
                completeness: 0.7,
            };
            let trace = TraceQuality {
                call_chain_completeness: 0.8,
                error_rate: 0.1,
                retry_count: 1,
            };
            let component = ComponentQuality {
                init_success_rate: 0.9,
                uptime_percentage: 0.95,
                resource_efficiency: 0.7,
            };
            let drift = DriftQuality {
                recent_performance: vec![0.7, 0.75],
                historical_avg: 0.7,
                trend_direction: 0.05,
            };
            evaluator.evaluate("comp-trend", output, trace, component, drift, 0.7);
        }
        let trend = evaluator.get_trend("comp-trend", 5);
        assert_eq!(trend.len(), 5);
    }

    #[test]
    fn test_four_layer_evaluation() {
        let mut evaluator = TrustEvaluator::new();
        let output = OutputQuality {
            content_relevance: 0.85,
            format_compliance: 0.9,
            completeness: 0.8,
        };
        let trace = TraceQuality {
            call_chain_completeness: 0.9,
            error_rate: 0.02,
            retry_count: 0,
        };
        let component = ComponentQuality {
            init_success_rate: 1.0,
            uptime_percentage: 0.99,
            resource_efficiency: 0.85,
        };
        let drift = DriftQuality {
            recent_performance: vec![0.8, 0.82, 0.85],
            historical_avg: 0.8,
            trend_direction: 0.05,
        };

        let score = evaluator.evaluate("comp-4layer", output, trace, component, drift, 0.85);
        assert!(score > 0.7);
        assert!(score <= 1.0);
    }
}
