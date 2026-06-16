//! 信任积分器 — Agent 信用积分记录、排名、历史查询
use crate::core::error::MornError;
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
