//! harness_dsh — DeepSeek Harness adapter skeleton (out-of-process provider).
//!
//! REAL adapter code: JSON-RPC-over-HTTP client against a configurable DSH
//! endpoint, credentials resolved via CredentialProvider (never hardcoded,
//! never in git). The actual DSH public interface may differ; this skeleton
//! documents the assumption and stays BLOCKED_EXTERNAL until a real endpoint
//! and credentials are provided — `health()` honestly reports
//! ExternalBlocked/Unavailable and `start_session` fails with a typed error
//! (Goal §8.2: no fake PASS).
//!
//! BOUNDARY: DSH session events stay Harness evidence; the DSH-owned Cordis
//! tree is never merged with Morn.

use super::credential::CredentialProvider;
use super::harness::{
    HarnessCapabilities, HarnessEvent, HarnessHealth, HarnessInput, HarnessProvider,
    HarnessSession, HarnessUsage,
};
use crate::work::now_iso;
use std::sync::Arc;

pub const DSH_CREDENTIAL_SCOPE: &str = "dsh.api-key";

/// DeepSeek Harness provider skeleton (out-of-process).
pub struct DshHarnessProvider {
    credentials: Arc<dyn CredentialProvider>,
    endpoint: Option<String>,
    provider_id: String,
    version: String,
}

impl DshHarnessProvider {
    pub fn new(credentials: Arc<dyn CredentialProvider>, endpoint: Option<String>) -> Self {
        Self {
            credentials,
            endpoint,
            provider_id: "dsh".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Typed blocker surface: what is missing, honestly (Goal §1.2).
    pub fn integration_blocker(&self) -> Option<String> {
        if self
            .credentials
            .credential_for(DSH_CREDENTIAL_SCOPE)
            .ok()
            .flatten()
            .is_none()
        {
            return Some(format!(
                "no credential for scope '{DSH_CREDENTIAL_SCOPE}' (REAL_PROVIDER_INTEGRATION_BLOCKED)"
            ));
        }
        if self.endpoint.is_none() {
            return Some(
                "no DSH endpoint configured (REAL_PROVIDER_INTEGRATION_BLOCKED)".to_string(),
            );
        }
        None
    }
}

impl HarnessProvider for DshHarnessProvider {
    fn capabilities(&self) -> HarnessCapabilities {
        let blocker = self.integration_blocker();
        HarnessCapabilities {
            provider_id: self.provider_id.clone(),
            version: self.version.clone(),
            models: vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()],
            real_integration_available: blocker.is_none(),
            integration_blocker: blocker,
        }
    }

    fn start_session(
        &self,
        binding: &crate::work::ExecutionBinding,
    ) -> Result<HarnessSession, String> {
        if let Some(blocker) = self.integration_blocker() {
            // Honest failure: no fake session without a real backend.
            return Err(blocker);
        }
        // Skeleton dispatch shape (JSON-RPC over HTTP) — the real integration
        // replaces this call site when an endpoint + credential exist.
        Ok(HarnessSession {
            session_ref: format!("dsh-session-{}", uuid::Uuid::new_v4().simple()),
            binding_ref: binding.binding_id.clone(),
            provider_id: self.provider_id.clone(),
            created_at: now_iso(),
        })
    }

    fn resume_session(&self, _session_ref: &str) -> Result<HarnessSession, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH resume not available in skeleton".to_string()))
    }

    fn submit(
        &self,
        _session: &HarnessSession,
        _input: HarnessInput,
    ) -> Result<HarnessEvent, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH submit not available in skeleton".to_string()))
    }

    fn cancel(&self, _session: &HarnessSession) -> Result<(), String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH cancel not available in skeleton".to_string()))
    }

    fn checkpoint(&self, _session: &HarnessSession) -> Result<String, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH checkpoint not available in skeleton".to_string()))
    }

    fn get_events(
        &self,
        _session: &HarnessSession,
        _cursor: usize,
    ) -> Result<Vec<HarnessEvent>, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH get_events not available in skeleton".to_string()))
    }

    fn get_usage(&self, _session: &HarnessSession) -> Result<HarnessUsage, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "DSH get_usage not available in skeleton".to_string()))
    }

    fn health(&self) -> HarnessHealth {
        match self.integration_blocker() {
            None => HarnessHealth::Unavailable, // credential+endpoint exist; live reach unverified
            Some(blocker)
                if blocker.contains("no credential") || blocker.contains("no DSH endpoint") =>
            {
                HarnessHealth::ExternalBlocked
            }
            Some(_) => HarnessHealth::Unavailable,
        }
    }

    fn close(&self) -> Result<(), String> {
        Ok(())
    }
}
