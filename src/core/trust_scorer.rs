//! trust_scorer — Scores agents and actions using trust-related metrics.
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreEntry {
    pub agent_id: String,
    pub output_quality: f64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub user_feedback: f64,
    pub overall: f64,
    pub recorded_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentRanking {
    pub agent_id: String,
    pub agent_name: String,
    pub overall_score: f64,
    pub total_evaluations: u64,
}

pub struct TrustScorer {
    scores: HashMap<String, Vec<ScoreEntry>>,
}

impl TrustScorer {
    pub fn new() -> Self {
        TrustScorer {
            scores: HashMap::new(),
        }
    }

    pub fn calculate(
        output_quality: f64,
        success_rate: f64,
        avg_latency_ms: f64,
        user_feedback: f64,
    ) -> f64 {
        let latency_score = if avg_latency_ms > 0.0 {
            (1000.0 / avg_latency_ms).min(1.0)
        } else {
            0.0
        };
        output_quality * 0.3 + success_rate * 0.3 + latency_score * 0.2 + user_feedback * 0.2
    }

    pub fn record(
        &mut self,
        agent_id: &str,
        output_quality: f64,
        success_rate: f64,
        avg_latency_ms: f64,
        user_feedback: f64,
    ) -> f64 {
        let overall = Self::calculate(output_quality, success_rate, avg_latency_ms, user_feedback);
        let entry = ScoreEntry {
            agent_id: agent_id.to_string(),
            output_quality,
            success_rate,
            avg_latency_ms,
            user_feedback,
            overall,
            recorded_at: chrono::Utc::now().timestamp(),
        };
        self.scores
            .entry(agent_id.to_string())
            .or_default()
            .push(entry);
        overall
    }

    pub fn get_score(&self, agent_id: &str) -> Option<f64> {
        self.scores.get(agent_id).and_then(|entries| {
            if entries.is_empty() {
                return None;
            }
            let sum: f64 = entries.iter().map(|e| e.overall).sum();
            Some(sum / entries.len() as f64)
        })
    }

    pub fn get_recent_score(&self, agent_id: &str, window: usize) -> Option<f64> {
        self.scores.get(agent_id).and_then(|entries| {
            if entries.is_empty() {
                return None;
            }
            let start = entries.len().saturating_sub(window);
            let recent: f64 = entries[start..].iter().map(|e| e.overall).sum();
            Some(recent / (entries.len() - start) as f64)
        })
    }

    pub fn get_all_scores(&self) -> Vec<(&str, f64)> {
        self.scores
            .iter()
            .filter_map(|(id, entries)| {
                if entries.is_empty() {
                    return None;
                }
                let sum: f64 = entries.iter().map(|e| e.overall).sum();
                Some((id.as_str(), sum / entries.len() as f64))
            })
            .collect()
    }

    pub fn get_history(&self, agent_id: &str) -> Vec<&ScoreEntry> {
        self.scores
            .get(agent_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_rankings(&self) -> Vec<AgentRanking> {
        let mut rankings: Vec<AgentRanking> = self
            .scores
            .iter()
            .filter_map(|(id, entries)| {
                if entries.is_empty() {
                    return None;
                }
                let sum: f64 = entries.iter().map(|e| e.overall).sum();
                Some(AgentRanking {
                    agent_id: id.clone(),
                    agent_name: id.clone(),
                    overall_score: sum / entries.len() as f64,
                    total_evaluations: entries.len() as u64,
                })
            })
            .collect();
        rankings.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        rankings
    }

    pub fn clear(&mut self) {
        self.scores.clear();
    }
}

impl Default for TrustScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
