//! credential — CredentialProvider interface (Goal §12.4).
//!
//! SAFETY: secrets are not written to git; capability only obtains
//! work-scope/minimal-permission credentials; harnesses do not naturally hold
//! production credentials; logs redact secrets. This interface is the seam —
//! implementations resolve credentials without exposing them to business
//! layers or logs.

use std::collections::HashMap;
use std::sync::Mutex;

/// Resolves a scoped credential. Returns `Ok(None)` when no credential is
/// available for the scope (honest unavailability, never a fake value).
pub trait CredentialProvider: Send + Sync {
    /// `scope` names the minimal permission scope, e.g.
    /// "dsh.api-key" / "cmms.read-token" / "historian.read".
    fn credential_for(&self, scope: &str) -> Result<Option<String>, String>;

    /// Whether this provider can resolve credentials at all right now.
    fn available(&self) -> bool;
}

/// Environment-based credential provider (dev/reference).
///
/// PROVIDER: reads from a process-level map (populated from env vars at
/// startup by the app shell); values never leave this boundary. A redacted
/// fingerprint is safe to log; the secret itself is not.
pub struct EnvCredentialProvider {
    credentials: Mutex<HashMap<String, String>>,
}

impl EnvCredentialProvider {
    pub fn new() -> Self {
        Self {
            credentials: Mutex::new(HashMap::new()),
        }
    }

    /// Registers one scoped credential (called by the app shell at startup).
    /// The scope→value mapping stays inside this provider.
    pub fn register(&self, scope: &str, value: String) {
        self.credentials
            .lock()
            .unwrap()
            .insert(scope.to_string(), value);
    }

    /// Redacted fingerprint safe for logs (first 4 chars + length), never the
    /// secret itself.
    pub fn fingerprint(value: &str) -> String {
        let head: String = value.chars().take(4).collect();
        format!("{}…({} chars)", head, value.len())
    }
}

impl Default for EnvCredentialProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialProvider for EnvCredentialProvider {
    fn credential_for(&self, scope: &str) -> Result<Option<String>, String> {
        Ok(self.credentials.lock().unwrap().get(scope).cloned())
    }

    fn available(&self) -> bool {
        true
    }
}
