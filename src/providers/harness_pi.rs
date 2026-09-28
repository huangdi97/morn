//! harness_pi — Pi adapter skeleton (second harness provider).
//!
//! Goal §8.2: the point is not "support one more model" but proving
//! `same Work contract / same capability intent / same business acceptance`
//! with a different harness. Same HarnessProvider contract as DSH; honest
//! BLOCKED_EXTERNAL until Pi access + credentials exist. Skeleton with real
//! credential plumbing; no fake PASS.
//!
//! BOUNDARY: Pi session events stay Harness evidence; never Work truth.

use super::credential::CredentialProvider;
use super::harness::{
    HarnessCapabilities, HarnessEvent, HarnessHealth, HarnessInput, HarnessProvider,
    HarnessSession, HarnessUsage,
};
use crate::work::now_iso;
use std::sync::Arc;

pub const PI_CREDENTIAL_SCOPE: &str = "pi.api-key";

/// Pi harness provider skeleton (out-of-process).
pub struct PiHarnessProvider {
    credentials: Arc<dyn CredentialProvider>,
    endpoint: Option<String>,
    provider_id: String,
    version: String,
}

impl PiHarnessProvider {
    pub fn new(credentials: Arc<dyn CredentialProvider>, endpoint: Option<String>) -> Self {
        Self {
            credentials,
            endpoint,
            provider_id: "pi".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Typed blocker surface (honest, Goal §1.2).
    pub fn integration_blocker(&self) -> Option<String> {
        if self
            .credentials
            .credential_for(PI_CREDENTIAL_SCOPE)
            .ok()
            .flatten()
            .is_none()
        {
            return Some(format!(
                "no credential for scope '{PI_CREDENTIAL_SCOPE}' (REAL_PROVIDER_INTEGRATION_BLOCKED)"
            ));
        }
        if self.endpoint.is_none() {
            return Some(
                "no Pi endpoint configured (REAL_PROVIDER_INTEGRATION_BLOCKED)".to_string(),
            );
        }
        None
    }
}

impl HarnessProvider for PiHarnessProvider {
    fn capabilities(&self) -> HarnessCapabilities {
        let blocker = self.integration_blocker();
        HarnessCapabilities {
            provider_id: self.provider_id.clone(),
            version: self.version.clone(),
            models: vec!["pi-model".to_string()],
            real_integration_available: blocker.is_none(),
            integration_blocker: blocker,
        }
    }

    fn start_session(
        &self,
        binding: &crate::work::ExecutionBinding,
    ) -> Result<HarnessSession, String> {
        if let Some(blocker) = self.integration_blocker() {
            return Err(blocker);
        }
        Ok(HarnessSession {
            session_ref: format!("pi-session-{}", uuid::Uuid::new_v4().simple()),
            binding_ref: binding.binding_id.clone(),
            provider_id: self.provider_id.clone(),
            created_at: now_iso(),
        })
    }

    fn resume_session(&self, _session_ref: &str) -> Result<HarnessSession, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi resume not available in skeleton".to_string()))
    }

    fn submit(
        &self,
        _session: &HarnessSession,
        _input: HarnessInput,
    ) -> Result<HarnessEvent, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi submit not available in skeleton".to_string()))
    }

    fn cancel(&self, _session: &HarnessSession) -> Result<(), String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi cancel not available in skeleton".to_string()))
    }

    fn checkpoint(&self, _session: &HarnessSession) -> Result<String, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi checkpoint not available in skeleton".to_string()))
    }

    fn get_events(
        &self,
        _session: &HarnessSession,
        _cursor: usize,
    ) -> Result<Vec<HarnessEvent>, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi get_events not available in skeleton".to_string()))
    }

    fn get_usage(&self, _session: &HarnessSession) -> Result<HarnessUsage, String> {
        Err(self
            .integration_blocker()
            .unwrap_or_else(|| "Pi get_usage not available in skeleton".to_string()))
    }

    fn health(&self) -> HarnessHealth {
        match self.integration_blocker() {
            Some(blocker)
                if blocker.contains("no credential") || blocker.contains("no Pi endpoint") =>
            {
                HarnessHealth::ExternalBlocked
            }
            _ => HarnessHealth::Unavailable,
        }
    }

    fn close(&self) -> Result<(), String> {
        Ok(())
    }
}
