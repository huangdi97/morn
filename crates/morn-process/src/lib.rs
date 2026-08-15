//! Generic Process Intelligence: domain-neutral observed events -> traces ->
//! observed work graph with bottleneck/rework/loop/wait/handoff detection.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ObservedEventTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ObservedWorkTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ProcessTraceTag;

pub type ObservedEventId = Id<ObservedEventTag>;
pub type ObservedWorkId = Id<ObservedWorkTag>;
pub type ProcessTraceId = Id<ProcessTraceTag>;

/// A generic observed event (no domain ontology).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedEvent {
    pub id: ObservedEventId,
    pub case_ref: String,
    pub activity: String,
    pub actor: String,
    pub timestamp: Timestamp,
    pub event_type: String, // start | complete | handoff | wait | rework | approval | manual_copy
}

impl ObservedEvent {
    pub fn new(case_ref: &str, activity: &str, actor: &str, event_type: &str) -> Self {
        Self {
            id: ObservedEventId::generate_with("evt"),
            case_ref: case_ref.to_string(),
            activity: activity.to_string(),
            actor: actor.to_string(),
            timestamp: Timestamp::now(),
            event_type: event_type.to_string(),
        }
    }
}

/// An observed unit of work aggregated from events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWork {
    pub id: ObservedWorkId,
    pub case_ref: String,
    pub activity: String,
    pub occurrences: u32,
    pub total_wait_ms: u64,
    pub rework: bool,
    pub looped: bool,
    pub handoffs: u32,
    pub duplicate_approvals: u32,
    pub manual_copy: bool,
}

/// A trace of one case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessTrace {
    pub id: ProcessTraceId,
    pub case_ref: String,
    pub events: Vec<ObservedEvent>,
}

/// Observed work graph across cases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObservedWorkGraph {
    pub works: Vec<ObservedWork>,
    pub transitions: Vec<(String, String, u32)>,
}

/// Detected process signals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProcessSignals {
    pub repeated_handoff: Vec<String>,
    pub wait_bottleneck: Vec<String>,
    pub rework: Vec<String>,
    pub loop_detected: Vec<String>,
    pub duplicate_approval: Vec<String>,
    pub manual_copy: Vec<String>,
}

/// Generic process miner: builds traces and detects signals without any domain
/// knowledge.
#[derive(Debug, Default)]
pub struct ProcessMiner {
    pub traces: Vec<ProcessTrace>,
    pub graph: ObservedWorkGraph,
    pub signals: ProcessSignals,
}

impl ProcessMiner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, events: Vec<ObservedEvent>) -> ProcessTrace {
        let trace = ProcessTrace {
            id: ProcessTraceId::generate_with("trace"),
            case_ref: events
                .first()
                .map(|e| e.case_ref.clone())
                .unwrap_or_default(),
            events,
        };
        self.traces.push(trace.clone());
        trace
    }

    /// Build the observed graph and detect signals from all traces.
    pub fn mine(&mut self) {
        let mut by_activity: HashMap<String, ObservedWork> = HashMap::new();
        let mut transitions: HashMap<(String, String), u32> = HashMap::new();
        let mut signals = ProcessSignals::default();
        for trace in &self.traces {
            let evs = &trace.events;
            for (i, e) in evs.iter().enumerate() {
                let w = by_activity
                    .entry(e.activity.clone())
                    .or_insert(ObservedWork {
                        id: ObservedWorkId::generate_with("w"),
                        case_ref: e.case_ref.clone(),
                        activity: e.activity.clone(),
                        occurrences: 0,
                        total_wait_ms: 0,
                        rework: false,
                        looped: false,
                        handoffs: 0,
                        duplicate_approvals: 0,
                        manual_copy: false,
                    });
                w.occurrences += 1;
                if e.event_type == "handoff" {
                    w.handoffs += 1;
                }
                if e.event_type == "wait" {
                    w.total_wait_ms += 1000;
                }
                if e.event_type == "rework" {
                    w.rework = true;
                }
                if e.event_type == "approval" {
                    w.duplicate_approvals += 1;
                }
                if e.event_type == "manual_copy" {
                    w.manual_copy = true;
                }
                if i > 0 {
                    let key = (evs[i - 1].activity.clone(), e.activity.clone());
                    *transitions.entry(key).or_insert(0) += 1;
                }
            }
            // loop detection: same activity appears twice in one case
            for w in by_activity.values_mut() {
                if w.case_ref == trace.case_ref && w.occurrences > 1 {
                    w.looped = true;
                }
            }
        }
        let works: Vec<ObservedWork> = by_activity.into_values().collect();
        for w in &works {
            if w.handoffs >= 2 {
                signals.repeated_handoff.push(w.activity.clone());
            }
            if w.total_wait_ms >= 2000 {
                signals.wait_bottleneck.push(w.activity.clone());
            }
            if w.rework {
                signals.rework.push(w.activity.clone());
            }
            if w.looped {
                signals.loop_detected.push(w.activity.clone());
            }
            if w.duplicate_approvals >= 2 {
                signals.duplicate_approval.push(w.activity.clone());
            }
            if w.manual_copy {
                signals.manual_copy.push(w.activity.clone());
            }
        }
        let transitions = transitions
            .into_iter()
            .map(|((a, b), c)| (a, b, c))
            .collect();
        self.graph = ObservedWorkGraph { works, transitions };
        self.signals = signals;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_fixture_detects_signals() {
        let mut miner = ProcessMiner::new();
        miner.ingest(vec![
            ObservedEvent::new("case-1", "approve", "mgr-a", "approval"),
            ObservedEvent::new("case-1", "approve", "mgr-b", "approval"),
            ObservedEvent::new("case-1", "copy", "clerk", "manual_copy"),
            ObservedEvent::new("case-1", "review", "r1", "handoff"),
            ObservedEvent::new("case-1", "review", "r2", "handoff"),
            ObservedEvent::new("case-1", "review", "r1", "rework"),
            ObservedEvent::new("case-1", "wait", "sys", "wait"),
            ObservedEvent::new("case-1", "wait", "sys", "wait"),
            ObservedEvent::new("case-1", "review", "r1", "handoff"),
        ]);
        miner.mine();
        assert!(miner.signals.repeated_handoff.iter().any(|a| a == "review"));
        assert!(miner.signals.wait_bottleneck.iter().any(|a| a == "wait"));
        assert!(miner.signals.rework.iter().any(|a| a == "review"));
        assert!(miner.signals.loop_detected.iter().any(|a| a == "review"));
        assert!(miner
            .signals
            .duplicate_approval
            .iter()
            .any(|a| a == "approve"));
        assert!(miner.signals.manual_copy.iter().any(|a| a == "copy"));
        assert!(!miner.graph.transitions.is_empty());
    }

    #[test]
    fn no_domain_ontology_required() {
        // Activities are arbitrary strings; miner knows no domain.
        let mut miner = ProcessMiner::new();
        miner.ingest(vec![ObservedEvent::new("c", "alpha", "a", "start")]);
        miner.mine();
        assert_eq!(miner.graph.works.len(), 1);
    }
}
