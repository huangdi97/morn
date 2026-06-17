//! Trust scorer tests.
use super::*;
use crate::core::error::MornError;

#[test]
fn test_calculate_score() {
    let score = TrustScorer::calculate(0.9, 0.95, 200.0, 0.8);
    assert!(score > 0.0 && score <= 1.0);
}

#[test]
fn test_high_quality() {
    let score = TrustScorer::calculate(1.0, 1.0, 100.0, 1.0);
    assert!(score > 0.8);
}

#[test]
fn test_low_quality() {
    let score = TrustScorer::calculate(0.0, 0.0, 10000.0, 0.0);
    assert!(score < 0.3);
}

#[test]
fn test_record_and_get() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-1", 0.8, 0.9, 300.0, 0.7);
    let s = scorer.get_score("agent-1");
    assert!(s.is_some());
    assert!(s.unwrap() > 0.0);
}

#[test]
fn test_get_nonexistent() {
    let scorer = TrustScorer::new();
    assert!(scorer.get_score("ghost").is_none());
}

#[test]
fn test_multiple_records() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-1", 0.8, 0.9, 300.0, 0.7);
    scorer.record("agent-1", 0.9, 0.95, 200.0, 0.8);
    let avg = scorer.get_score("agent-1").unwrap();
    assert!(avg > 0.7 && avg < 1.0);
}

#[test]
fn test_rankings() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-a", 0.5, 0.5, 500.0, 0.5);
    scorer.record("agent-b", 0.9, 0.9, 100.0, 0.9);
    let rankings = scorer.get_rankings();
    assert_eq!(rankings.len(), 2);
    assert!(rankings[0].overall_score > rankings[1].overall_score);
    assert_eq!(rankings[0].agent_id, "agent-b");
}

#[test]
fn test_get_history() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-1", 0.8, 0.9, 300.0, 0.7);
    scorer.record("agent-1", 0.9, 0.95, 200.0, 0.8);
    assert_eq!(scorer.get_history("agent-1").len(), 2);
    assert!(scorer.get_history("ghost").is_empty());
}

#[test]
fn test_recent_score() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-1", 0.5, 0.5, 500.0, 0.5);
    scorer.record("agent-1", 0.9, 0.9, 100.0, 0.9);
    scorer.record("agent-1", 0.95, 0.95, 50.0, 0.95);
    let recent = scorer.get_recent_score("agent-1", 2).unwrap();
    let overall = scorer.get_score("agent-1").unwrap();
    assert!(recent > overall);
}

#[test]
fn test_clear() {
    let mut scorer = TrustScorer::new();
    scorer.record("agent-1", 0.8, 0.9, 300.0, 0.7);
    scorer.clear();
    assert!(scorer.get_score("agent-1").is_none());
}

#[test]
fn test_all_scores() {
    let mut scorer = TrustScorer::new();
    scorer.record("a1", 0.8, 0.9, 300.0, 0.7);
    scorer.record("a2", 0.7, 0.8, 400.0, 0.6);
    let all = scorer.get_all_scores();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_evaluator_trust_formula() {
    let score = TrustEvaluator::calculate_trust_score(0.9, 0.95, 200.0, 0.8);
    assert!(score > 0.0 && score <= 1.0);
}

#[test]
fn test_evaluator_high_quality() {
    let score = TrustEvaluator::calculate_trust_score(1.0, 1.0, 100.0, 1.0);
    assert!(score > 0.8);
}

#[test]
fn test_evaluator_low_quality() {
    let score = TrustEvaluator::calculate_trust_score(0.0, 0.0, 10000.0, 0.0);
    assert!(score < 0.5);
}

#[test]
fn test_evaluator_records_history() {
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
fn test_evaluator_trend_analysis() {
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
fn test_evaluator_four_layer_evaluation() {
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
