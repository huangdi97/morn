//! projection — deterministic projection from event history.
//!
//! The current projection is mutable; historical facts are append/supersede.
//! Replaying the event history must deterministically rebuild the same
//! projection (crash/restart consistency, G2).

use crate::work::{WorkPhase, WorkStatus};

/// One recorded semantic event (the store's event row).
#[derive(Debug, Clone, PartialEq)]
pub struct StoredEvent {
    pub id: String,
    pub work_id: Option<String>,
    pub event_type: String,
    pub payload_json: String,
    pub supersedes: Option<String>,
    pub retracts: Option<String>,
    pub valid_time: Option<String>,
    pub recorded_at: String,
}

/// Events that mutate the Work.status projection, keyed by event_type.
pub mod event_types {
    pub const STATUS_PHASE_CHANGED: &str = "work.status.phase_changed";
    pub const CONDITION_CHANGED: &str = "work.condition.changed";
    pub const UNKNOWN_RECORDED: &str = "work.unknown.recorded";
    pub const OUTCOME_RECORDED: &str = "work.outcome.recorded";
    pub const ACCEPTANCE_RECORDED: &str = "work.acceptance.recorded";
}

/// Projects a WorkStatus from the current status plus a replay of semantic
/// events. Deterministic: the same inputs always produce the same output.
///
/// Replay rules:
/// - STATUS_PHASE_CHANGED sets the phase (last write wins by replay order);
/// - UNKNOWN_RECORDED appends to unresolved unknowns;
/// - events with `retracts` cancel the effect of the event they retract;
/// - superseded events keep history but do not change the projection twice.
pub fn project_work(mut status: WorkStatus, events: &[StoredEvent]) -> WorkStatus {
    // SAFETY: a retraction cancels the effect of the event it names — the
    // retracted ids are collected first, then their effects are skipped
    // during replay (regardless of arrival order). The retraction itself
    // changes nothing else.
    let retracted: std::collections::HashSet<&str> = events
        .iter()
        .filter_map(|event| event.retracts.as_deref())
        .collect();
    for event in events {
        if event.retracts.is_some() {
            // The retraction event itself changes nothing.
            continue;
        }
        if retracted.contains(event.id.as_str()) {
            // This event's effect is canceled by a retraction.
            continue;
        }
        let Ok(payload) = serde_json::from_str::<serde_json::Value>(&event.payload_json) else {
            continue;
        };
        match event.event_type.as_str() {
            event_types::STATUS_PHASE_CHANGED => {
                if let Some(phase) = payload.get("phase").and_then(|p| p.as_str()) {
                    // serde_json::from_str would parse the raw identifier as a
                    // JSON document and fail ("expected value"); unit variants
                    // deserialize from a JSON *string value* via from_value.
                    let value = serde_json::Value::String(phase.to_string());
                    if let Ok(phase) = serde_json::from_value::<WorkPhase>(value) {
                        status.phase = phase;
                    }
                }
            }
            event_types::UNKNOWN_RECORDED => {
                if let Some(unknown) = payload.get("unknown").and_then(|u| u.as_str()) {
                    if !status.unresolved_unknowns.iter().any(|u| u == unknown) {
                        status.unresolved_unknowns.push(unknown.to_string());
                    }
                }
            }
            _ => {
                // CONDITION_CHANGED / OUTCOME_RECORDED / ACCEPTANCE_RECORDED
                // are materialized through their dedicated tables; the
                // projection reads them from there, not from raw events.
            }
        }
        status.updated_at = Some(event.recorded_at.clone());
    }
    status
}
