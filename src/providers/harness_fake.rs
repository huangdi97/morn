//! harness_fake — deterministic fake harness provider.
//!
//! Verifies the control plane without any external system (Goal §8.2). The
//! fake echoes input, records events per session, and checkpoints by event
//! count. It is deterministic: the same input always produces the same event
//! sequence.

use super::harness::{
    HarnessCapabilities, HarnessEvent, HarnessEventKind, HarnessHealth, HarnessInput,
    HarnessProvider, HarnessSession, HarnessUsage,
};
use crate::work::now_iso;
use crate::work::ExecutionBinding;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct SessionLog {
    events: Vec<HarnessEvent>,
    cancelled: bool,
}

/// Deterministic fake harness provider (control-plane verification only).
pub struct FakeHarnessProvider {
    provider_id: String,
    version: String,
    sessions: Arc<Mutex<BTreeMap<String, SessionLog>>>,
}

impl FakeHarnessProvider {
    pub fn new(provider_id: &str) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            version: "0.1.0".to_string(),
            sessions: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl HarnessProvider for FakeHarnessProvider {
    fn capabilities(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            provider_id: self.provider_id.clone(),
            version: self.version.clone(),
            models: vec!["fake-model".to_string()],
            real_integration_available: false,
            integration_blocker: Some(
                "deterministic fake provider (control-plane verification only)".to_string(),
            ),
        }
    }

    fn start_session(&self, binding: &ExecutionBinding) -> Result<HarnessSession, String> {
        let session_ref = format!(
            "fake-session-{}-{}",
            binding.binding_id,
            uuid::Uuid::new_v4().simple()
        );
        let session = HarnessSession {
            session_ref: session_ref.clone(),
            binding_ref: binding.binding_id.clone(),
            provider_id: self.provider_id.clone(),
            created_at: now_iso(),
        };
        let started = HarnessEvent {
            seq: 0,
            kind: HarnessEventKind::Started,
            payload_json: Some(
                serde_json::json!({ "binding_ref": binding.binding_id }).to_string(),
            ),
            recorded_at: now_iso(),
        };
        self.sessions.lock().unwrap().insert(
            session_ref,
            SessionLog {
                events: vec![started],
                cancelled: false,
            },
        );
        Ok(session)
    }

    fn resume_session(&self, session_ref: &str) -> Result<HarnessSession, String> {
        let sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get(session_ref)
            .ok_or_else(|| format!("unknown fake session: {session_ref}"))?;
        if log.cancelled {
            return Err(format!("fake session {session_ref} was cancelled"));
        }
        let binding_ref = log
            .events
            .first()
            .and_then(|e| e.payload_json.as_ref())
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .and_then(|p| {
                p.get("binding_ref")
                    .and_then(|b| b.as_str())
                    .map(String::from)
            })
            .unwrap_or_default();
        Ok(HarnessSession {
            session_ref: session_ref.to_string(),
            binding_ref,
            provider_id: self.provider_id.clone(),
            created_at: now_iso(),
        })
    }

    fn submit(
        &self,
        session: &HarnessSession,
        input: HarnessInput,
    ) -> Result<HarnessEvent, String> {
        let mut sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get_mut(&session.session_ref)
            .ok_or_else(|| format!("unknown fake session: {}", session.session_ref))?;
        if log.cancelled {
            return Err(format!(
                "fake session {} was cancelled",
                session.session_ref
            ));
        }
        let seq = log.events.len();
        // Deterministic: one candidate event derived from the input, then the
        // session is complete. The candidate echoes the prompt hash-free.
        let candidate = HarnessEvent {
            seq,
            kind: HarnessEventKind::Candidate,
            payload_json: Some(
                serde_json::json!({ "candidate": format!("fake-candidate:{}", input.prompt) })
                    .to_string(),
            ),
            recorded_at: now_iso(),
        };
        log.events.push(candidate.clone());
        let completed_seq = seq + 1;
        let completed = HarnessEvent {
            seq: completed_seq,
            kind: HarnessEventKind::Completed,
            payload_json: None,
            recorded_at: now_iso(),
        };
        log.events.push(completed);
        Ok(candidate)
    }

    fn cancel(&self, session: &HarnessSession) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get_mut(&session.session_ref)
            .ok_or_else(|| format!("unknown fake session: {}", session.session_ref))?;
        log.cancelled = true;
        let seq = log.events.len();
        log.events.push(HarnessEvent {
            seq,
            kind: HarnessEventKind::Cancelled,
            payload_json: None,
            recorded_at: now_iso(),
        });
        Ok(())
    }

    fn checkpoint(&self, session: &HarnessSession) -> Result<String, String> {
        let sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get(&session.session_ref)
            .ok_or_else(|| format!("unknown fake session: {}", session.session_ref))?;
        Ok(serde_json::json!({ "event_count": log.events.len() }).to_string())
    }

    fn get_events(
        &self,
        session: &HarnessSession,
        cursor: usize,
    ) -> Result<Vec<HarnessEvent>, String> {
        let sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get(&session.session_ref)
            .ok_or_else(|| format!("unknown fake session: {}", session.session_ref))?;
        Ok(log.events.iter().skip(cursor).cloned().collect())
    }

    fn get_usage(&self, session: &HarnessSession) -> Result<HarnessUsage, String> {
        let sessions = self.sessions.lock().unwrap();
        let log = sessions
            .get(&session.session_ref)
            .ok_or_else(|| format!("unknown fake session: {}", session.session_ref))?;
        Ok(HarnessUsage {
            event_count: log.events.len(),
            token_estimate: Some(log.events.len() as i64 * 10),
        })
    }

    fn health(&self) -> HarnessHealth {
        HarnessHealth::Healthy
    }

    fn close(&self) -> Result<(), String> {
        Ok(())
    }
}
