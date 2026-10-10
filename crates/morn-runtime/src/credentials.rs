//! Opaque credential handles for least-privilege execution.
//!
//! Harnesses and capabilities receive references/handles scoped to Work, site
//! and action. Secret values are resolved only at the enforcement/execution
//! boundary and are never serialized into Morn events or prompts.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CredentialHandleTag;
pub type CredentialHandleId = Id<CredentialHandleTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRequest {
    pub work_ref: String,
    pub site_ref: Option<String>,
    pub provider_ref: String,
    pub resource: String,
    /// Optional protocol audience/resource URI (for example an MCP protected
    /// resource indicator). This scopes a credential to the intended server
    /// rather than allowing bearer-token passthrough across resources.
    pub audience_ref: Option<String>,
    pub action: String,
    pub requested_scopes: Vec<String>,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialHandle {
    pub id: CredentialHandleId,
    pub provider: String,
    pub audience_ref: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Timestamp,
    pub opaque_ref: String,
}

pub trait CredentialProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn issue(&mut self, request: &CredentialRequest) -> Result<CredentialHandle>;
    fn revoke(&mut self, handle: &CredentialHandle) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct FixtureCredentialProvider {
    active: HashMap<String, CredentialHandle>,
}

impl CredentialProvider for FixtureCredentialProvider {
    fn provider_name(&self) -> &str {
        "fixture-credential-provider"
    }

    fn issue(&mut self, request: &CredentialRequest) -> Result<CredentialHandle> {
        if request.work_ref.trim().is_empty()
            || request.provider_ref.trim().is_empty()
            || request.resource.trim().is_empty()
            || request.action.trim().is_empty()
        {
            return Err(Error::validation(
                "credential request requires work, provider, resource and action scope",
            ));
        }
        if request.ttl_seconds == 0 {
            return Err(Error::validation("credential ttl must be positive"));
        }
        let id = CredentialHandleId::generate_with("cred");
        let expires_at = Timestamp::from_millis(
            Timestamp::now().millis() + (request.ttl_seconds as i64 * 1_000),
        );
        let handle = CredentialHandle {
            id: id.clone(),
            provider: self.provider_name().to_string(),
            audience_ref: request.audience_ref.clone(),
            scopes: request.requested_scopes.clone(),
            expires_at,
            opaque_ref: format!("secret-handle://{}", id),
        };
        self.active.insert(id.to_string(), handle.clone());
        Ok(handle)
    }

    fn revoke(&mut self, handle: &CredentialHandle) -> Result<()> {
        self.active
            .remove(handle.id.as_str())
            .ok_or_else(|| Error::not_found(format!("credential handle {}", handle.id)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_handle_contains_no_secret_value() {
        let mut provider = FixtureCredentialProvider::default();
        let handle = provider
            .issue(&CredentialRequest {
                work_ref: "work-1".to_string(),
                site_ref: Some("plant-a".to_string()),
                provider_ref: "cmms".to_string(),
                resource: "maintenance-order".to_string(),
                audience_ref: Some("https://cmms.example/mcp".to_string()),
                action: "read".to_string(),
                requested_scopes: vec!["cmms.read".to_string()],
                ttl_seconds: 60,
            })
            .unwrap();
        let serialized = serde_json::to_string(&handle).unwrap();
        assert!(serialized.contains("secret-handle://"));
        assert!(serialized.contains("https://cmms.example/mcp"));
        assert!(!serialized.contains("api_key"));
        assert!(!serialized.contains("password"));
        provider.revoke(&handle).unwrap();
    }

    #[test]
    fn credential_is_resource_audience_bound_without_exposing_secret() {
        let mut provider = FixtureCredentialProvider::default();
        let handle = provider
            .issue(&CredentialRequest {
                work_ref: "work-2".to_string(),
                site_ref: None,
                provider_ref: "mcp-server".to_string(),
                resource: "mcp-protected-resource".to_string(),
                audience_ref: Some("https://tools.example/mcp".to_string()),
                action: "call".to_string(),
                requested_scopes: vec!["tools.call".to_string()],
                ttl_seconds: 30,
            })
            .unwrap();
        assert_eq!(
            handle.audience_ref.as_deref(),
            Some("https://tools.example/mcp")
        );
        assert!(handle.opaque_ref.starts_with("secret-handle://"));
    }
}
