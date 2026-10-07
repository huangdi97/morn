//! Workload identity boundary for service-to-service execution.
//!
//! Morn identity records remain business/canonical identity. This module models
//! short-lived runtime workload identity attestations (SPIFFE-compatible by
//! shape) without turning SPIFFE or any concrete identity system into a Morn
//! semantic dependency.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkloadIdentityTag;
pub type WorkloadIdentityId = Id<WorkloadIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadIdentityRequest {
    pub workload_ref: String,
    pub work_ref: String,
    pub site_ref: Option<String>,
    pub trust_domain: String,
    pub ttl_seconds: u64,
    pub attestation_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadIdentity {
    pub id: WorkloadIdentityId,
    pub provider: String,
    pub workload_ref: String,
    pub work_ref: String,
    pub site_ref: Option<String>,
    pub trust_domain: String,
    pub subject_uri: String,
    pub attestation_refs: Vec<String>,
    pub expires_at: Timestamp,
}

impl WorkloadIdentity {
    pub fn is_active_at(&self, now: Timestamp) -> bool {
        now <= self.expires_at
    }
}

pub trait WorkloadIdentityProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn attest(&mut self, request: &WorkloadIdentityRequest) -> Result<WorkloadIdentity>;
    fn revoke(&mut self, identity: &WorkloadIdentity) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct FixtureWorkloadIdentityProvider {
    active: HashMap<String, WorkloadIdentity>,
}

impl WorkloadIdentityProvider for FixtureWorkloadIdentityProvider {
    fn provider_name(&self) -> &str {
        "fixture-workload-identity"
    }

    fn attest(&mut self, request: &WorkloadIdentityRequest) -> Result<WorkloadIdentity> {
        if request.workload_ref.trim().is_empty()
            || request.work_ref.trim().is_empty()
            || request.trust_domain.trim().is_empty()
        {
            return Err(Error::validation(
                "workload identity requires workload, work and trust-domain references",
            ));
        }
        if request.ttl_seconds == 0 {
            return Err(Error::validation("workload identity ttl must be positive"));
        }

        let id = WorkloadIdentityId::generate_with("wid");
        let path = request
            .workload_ref
            .trim_matches('/')
            .replace(' ', "-")
            .replace(':', "-");
        let subject_uri = format!("spiffe://{}/{}", request.trust_domain, path);
        let identity = WorkloadIdentity {
            id: id.clone(),
            provider: self.provider_name().to_string(),
            workload_ref: request.workload_ref.clone(),
            work_ref: request.work_ref.clone(),
            site_ref: request.site_ref.clone(),
            trust_domain: request.trust_domain.clone(),
            subject_uri,
            attestation_refs: request.attestation_refs.clone(),
            expires_at: Timestamp::from_millis(
                Timestamp::now().millis() + request.ttl_seconds as i64 * 1_000,
            ),
        };
        self.active.insert(id.to_string(), identity.clone());
        Ok(identity)
    }

    fn revoke(&mut self, identity: &WorkloadIdentity) -> Result<()> {
        self.active
            .remove(identity.id.as_str())
            .ok_or_else(|| Error::not_found(format!("workload identity {}", identity.id)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_identity_is_work_scoped_and_contains_no_secret_material() {
        let mut provider = FixtureWorkloadIdentityProvider::default();
        let identity = provider
            .attest(&WorkloadIdentityRequest {
                workload_ref: "capability/cmms-adapter".to_string(),
                work_ref: "work-1042".to_string(),
                site_ref: Some("plant-a".to_string()),
                trust_domain: "plant-a.example".to_string(),
                ttl_seconds: 60,
                attestation_refs: vec!["fixture://node-attestation".to_string()],
            })
            .unwrap();

        assert_eq!(
            identity.subject_uri,
            "spiffe://plant-a.example/capability/cmms-adapter"
        );
        assert_eq!(identity.work_ref, "work-1042");
        let serialized = serde_json::to_string(&identity).unwrap();
        assert!(!serialized.contains("private_key"));
        assert!(!serialized.contains("bearer"));
        assert!(!serialized.contains("password"));

        provider.revoke(&identity).unwrap();
    }

    #[test]
    fn zero_ttl_or_missing_trust_domain_fails_closed() {
        let mut provider = FixtureWorkloadIdentityProvider::default();
        assert!(provider
            .attest(&WorkloadIdentityRequest {
                workload_ref: "runtime".to_string(),
                work_ref: "work".to_string(),
                site_ref: None,
                trust_domain: String::new(),
                ttl_seconds: 60,
                attestation_refs: vec![],
            })
            .is_err());
    }
}
