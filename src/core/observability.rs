//! observability — Tracks trace spans and token usage for runtime workflows.
use crate::core::event_bus::{Event, SimpleEventBus};
use serde_json::Value;
use std::collections::HashMap;

pub const EVENT_TRACE_SPAN_STARTED: &str = "observability.trace.span_started";
pub const EVENT_TRACE_SPAN_ENDED: &str = "observability.trace.span_ended";
pub const EVENT_TOKEN_USAGE_RECORDED: &str = "observability.token_usage.recorded";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraceSpanType {
    UserInput,
    IntentParse,
    LLMInvoke,
    ToolExec,
    KnowledgeRetrieval,
    MemoryAccess,
    WorkflowStep,
    AgentDispatch,
    ConsensusVote,
    ChannelSend,
    AuthCheck,
    SecurityScan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceSpan {
    pub id: String,
    pub span_type: TraceSpanType,
    pub start_time: i64,
    pub duration_ms: f64,
    pub status: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenStats {
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub by_model: HashMap<String, u64>,
    pub by_agent: HashMap<String, u64>,
}

impl Default for TokenStats {
    fn default() -> Self {
        Self {
            total_tokens: 0,
            cost_usd: 0.0,
            by_model: HashMap::new(),
            by_agent: HashMap::new(),
        }
    }
}

pub struct ObservabilityManager {
    active_spans: HashMap<String, TraceSpan>,
    completed_spans: Vec<TraceSpan>,
    token_stats: TokenStats,
    max_spans: usize,
    event_bus: Option<SimpleEventBus>,
}

impl ObservabilityManager {
    pub fn new() -> Self {
        Self::with_max_spans(1_000)
    }

    pub fn with_max_spans(max_spans: usize) -> Self {
        Self {
            active_spans: HashMap::new(),
            completed_spans: Vec::new(),
            token_stats: TokenStats::default(),
            max_spans,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: SimpleEventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn start_span(
        &mut self,
        span_type: TraceSpanType,
        metadata: HashMap<String, String>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let span = TraceSpan {
            id: id.clone(),
            span_type,
            start_time: chrono::Utc::now().timestamp_millis(),
            duration_ms: 0.0,
            status: "running".to_string(),
            metadata,
        };

        self.active_spans.insert(id.clone(), span.clone());
        self.publish_span_event(EVENT_TRACE_SPAN_STARTED, &span);
        id
    }

    pub fn end_span(
        &mut self,
        span_id: &str,
        status: &str,
        metadata: HashMap<String, String>,
    ) -> Option<TraceSpan> {
        let mut span = self.active_spans.remove(span_id)?;
        let now = chrono::Utc::now().timestamp_millis();
        span.duration_ms = (now - span.start_time).max(0) as f64;
        span.status = status.to_string();
        span.metadata.extend(metadata);

        self.completed_spans.push(span.clone());
        self.trim_spans();
        self.publish_span_event(EVENT_TRACE_SPAN_ENDED, &span);
        Some(span)
    }

    pub fn record_token_usage(&mut self, model: &str, agent: &str, tokens: u64, cost_usd: f64) {
        self.token_stats.total_tokens = self.token_stats.total_tokens.saturating_add(tokens);
        self.token_stats.cost_usd += cost_usd;
        *self
            .token_stats
            .by_model
            .entry(model.to_string())
            .or_insert(0) += tokens;
        *self
            .token_stats
            .by_agent
            .entry(agent.to_string())
            .or_insert(0) += tokens;

        if let Some(bus) = &self.event_bus {
            bus.publish_event(
                EVENT_TOKEN_USAGE_RECORDED,
                "observability",
                serde_json::json!({
                    "model": model,
                    "agent": agent,
                    "tokens": tokens,
                    "cost_usd": cost_usd,
                }),
            );
        }
    }

    pub fn get_recent_spans(&self, limit: usize) -> Vec<TraceSpan> {
        self.completed_spans
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_token_stats(&self) -> TokenStats {
        self.token_stats.clone()
    }

    pub fn get_span_distribution(&self) -> HashMap<TraceSpanType, u64> {
        let mut distribution = HashMap::new();
        for span in &self.completed_spans {
            *distribution.entry(span.span_type.clone()).or_insert(0) += 1;
        }
        distribution
    }

    pub fn handle_event(&mut self, event: &Event) {
        if event.event_type == "chat_agent.response" {
            let model = event
                .data
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let tokens = event
                .data
                .get("tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            self.record_token_usage(model, &event.source, tokens, 0.0);
        }
    }

    fn publish_span_event(&self, event_type: &str, span: &TraceSpan) {
        if let Some(bus) = &self.event_bus {
            bus.publish_event(
                event_type,
                "observability",
                serde_json::json!({
                    "id": span.id,
                    "span_type": format!("{:?}", span.span_type),
                    "start_time": span.start_time,
                    "duration_ms": span.duration_ms,
                    "status": span.status,
                    "metadata": span.metadata,
                }),
            );
        }
    }

    fn trim_spans(&mut self) {
        if self.max_spans == 0 {
            self.completed_spans.clear();
        } else if self.completed_spans.len() > self.max_spans {
            let excess = self.completed_spans.len() - self.max_spans;
            self.completed_spans.drain(0..excess);
        }
    }
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_and_end_span() {
        let mut manager = ObservabilityManager::new();
        let span_id = manager.start_span(TraceSpanType::LLMInvoke, HashMap::new());
        let span = manager
            .end_span(
                &span_id,
                "ok",
                HashMap::from([("model".into(), "local".into())]),
            )
            .unwrap();

        assert_eq!(span.id, span_id);
        assert_eq!(span.status, "ok");
        assert_eq!(span.metadata.get("model"), Some(&"local".to_string()));
        assert_eq!(manager.get_recent_spans(1).len(), 1);
    }

    #[test]
    fn test_token_stats_and_distribution() {
        let mut manager = ObservabilityManager::new();
        manager.record_token_usage("llama3", "agent-a", 42, 0.01);
        manager.record_token_usage("llama3", "agent-b", 8, 0.02);

        let id = manager.start_span(TraceSpanType::ToolExec, HashMap::new());
        manager.end_span(&id, "ok", HashMap::new()).unwrap();

        let stats = manager.get_token_stats();
        assert_eq!(stats.total_tokens, 50);
        assert_eq!(stats.by_model.get("llama3"), Some(&50));
        assert_eq!(stats.by_agent.get("agent-a"), Some(&42));
        assert_eq!(
            manager
                .get_span_distribution()
                .get(&TraceSpanType::ToolExec),
            Some(&1)
        );
    }

    #[test]
    fn test_handle_event_records_tokens() {
        let mut manager = ObservabilityManager::new();
        let event = Event::new(
            "chat_agent.response",
            "chat_agent",
            serde_json::json!({"model": "mistral", "tokens": 12}),
        );

        manager.handle_event(&event);

        let stats = manager.get_token_stats();
        assert_eq!(stats.total_tokens, 12);
        assert_eq!(stats.by_model.get("mistral"), Some(&12));
    }
}
