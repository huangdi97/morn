//! Connector / Integration SDK: generic fabric for external systems. Read path
//! and governed write path are distinct; every external write goes through an
//! approved action token (Action Gateway), never direct.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ActionProposalId, Id};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

macro_rules! local_id {
    ($tag:ident, $alias:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $tag;
        pub type $alias = Id<$tag>;
    };
}
local_id!(ConnectorSpecTag, ConnectorSpecId);
local_id!(ConnectorInstanceTag, ConnectorInstanceId);
local_id!(ExternalObjectRefTag, ExternalObjectRef);
local_id!(SyncCursorIdTag, SyncCursorId);
local_id!(ReceiptIdTag, ReceiptId);

/// Connector specification: how to reach and operate an external system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorSpec {
    pub id: ConnectorSpecId,
    pub name: String,
    pub version: Version,
    pub read_capable: bool,
    pub write_capable: bool,
    pub idempotency_supported: bool,
    pub rate_limit_per_min: u32,
    pub created_at: Timestamp,
}

impl ConnectorSpec {
    pub fn new(name: &str, read_capable: bool, write_capable: bool) -> Self {
        Self {
            id: ConnectorSpecId::generate_with("cspec"),
            name: name.to_string(),
            version: Version::v1(),
            read_capable,
            write_capable,
            idempotency_supported: true,
            rate_limit_per_min: 60,
            created_at: Timestamp::now(),
        }
    }
}

/// A configured connector instance with a credential reference (never a secret).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorInstance {
    pub id: ConnectorInstanceId,
    pub spec_id: ConnectorSpecId,
    pub name: String,
    pub external_system_ref: String,
    pub credential_ref: String,
    pub health: bool,
    pub sync_cursor: SyncCursor,
    pub created_at: Timestamp,
}

/// Mapping spec between external and Morn objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingSpec {
    pub external_type: String,
    pub morn_type: String,
    pub field_map: Vec<(String, String)>,
}

/// Sync cursor for incremental reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCursor {
    pub id: SyncCursorId,
    pub last_sequence: u64,
    pub last_timestamp: Option<String>,
}

impl Default for SyncCursor {
    fn default() -> Self {
        Self {
            id: SyncCursorId::generate_with("cur"),
            last_sequence: 0,
            last_timestamp: None,
        }
    }
}

/// An external action request (governed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalActionRequest {
    pub external_ref: ExternalObjectRef,
    pub action: String,
    pub payload: serde_json::Value,
    pub proposal_id: ActionProposalId,
}

/// A receipt for a governed external write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorReceipt {
    pub id: ReceiptId,
    pub external_ref: ExternalObjectRef,
    pub action: String,
    pub external_id: Option<String>,
    pub ok: bool,
    pub error: Option<String>,
    pub created_at: Timestamp,
}

/// The connector provider protocol.
pub trait ConnectorProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    /// Read path: fetch external objects from a cursor.
    fn read(&self, cursor: &SyncCursor, limit: u32) -> Result<Vec<serde_json::Value>>;
    /// Event stream (poll).
    fn poll_events(&self, cursor: &SyncCursor) -> Result<Vec<serde_json::Value>>;
    /// Governed write path: only executes when an approved action token is held.
    fn execute_governed(
        &mut self,
        request: &ExternalActionRequest,
        approved_token: &ApprovedActionToken,
    ) -> Result<ConnectorReceipt>;
    fn health(&self) -> bool;
    fn teardown(&mut self);
}

/// A token proving the action passed the Action Gateway (policy + approval).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedActionToken {
    pub proposal_id: ActionProposalId,
    pub granted_by: String,
}

/// Generic fixture connector for tests: simulated external system with
/// timeout, duplicate, rate-limit and idempotency behavior.
#[derive(Debug)]
pub struct GenericFixtureConnector {
    pub name: String,
    pub external: Vec<serde_json::Value>,
    pub applied_external_ids: Vec<String>,
    pub rate_limit_per_min: u32,
    pub calls_this_minute: u32,
    pub fail_reads: bool,
    pub teardown_called: bool,
}

impl GenericFixtureConnector {
    pub fn new() -> Self {
        Self {
            name: "generic-fixture".to_string(),
            external: vec![serde_json::json!({"id": "ext-1", "kind": "record"})],
            applied_external_ids: Vec::new(),
            rate_limit_per_min: 60,
            calls_this_minute: 0,
            fail_reads: false,
            teardown_called: false,
        }
    }
}

impl Default for GenericFixtureConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectorProvider for GenericFixtureConnector {
    fn provider_name(&self) -> &str {
        &self.name
    }

    fn read(&self, _cursor: &SyncCursor, limit: u32) -> Result<Vec<serde_json::Value>> {
        if self.fail_reads {
            return Err(Error::external("fixture read failure"));
        }
        Ok(self.external.iter().take(limit as usize).cloned().collect())
    }

    fn poll_events(&self, _cursor: &SyncCursor) -> Result<Vec<serde_json::Value>> {
        Ok(vec![
            serde_json::json!({"type": "external_event", "seq": 1}),
        ])
    }

    fn execute_governed(
        &mut self,
        request: &ExternalActionRequest,
        approved_token: &ApprovedActionToken,
    ) -> Result<ConnectorReceipt> {
        if approved_token.proposal_id != request.proposal_id {
            return Err(Error::not_authorized(
                "action token does not match proposal",
            ));
        }
        if self.calls_this_minute >= self.rate_limit_per_min {
            return Err(Error::external("rate limit exceeded"));
        }
        self.calls_this_minute += 1;
        // Idempotency: an external id already applied is not re-applied.
        if self
            .applied_external_ids
            .contains(&request.external_ref.to_string())
        {
            return Ok(ConnectorReceipt {
                id: ReceiptId::generate_with("rcpt"),
                external_ref: request.external_ref.clone(),
                action: request.action.clone(),
                external_id: Some("dup".to_string()),
                ok: true,
                error: Some("duplicate request ignored (idempotent)".to_string()),
                created_at: Timestamp::now(),
            });
        }
        self.applied_external_ids
            .push(request.external_ref.to_string());
        Ok(ConnectorReceipt {
            id: ReceiptId::generate_with("rcpt"),
            external_ref: request.external_ref.clone(),
            action: request.action.clone(),
            external_id: Some(format!("ext-id-{}", self.applied_external_ids.len())),
            ok: true,
            error: None,
            created_at: Timestamp::now(),
        })
    }

    fn health(&self) -> bool {
        !self.fail_reads
    }

    fn teardown(&mut self) {
        self.teardown_called = true;
    }
}

/// A registry of connector providers.
#[derive(Debug, Default)]
pub struct ConnectorRegistry {
    pub instances: Vec<ConnectorInstance>,
    pub mappings: HashMap<String, MappingSpec>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_instance(&mut self, instance: ConnectorInstance) {
        self.instances.push(instance);
    }

    pub fn add_mapping(&mut self, external_type: &str, mapping: MappingSpec) {
        self.mappings.insert(external_type.to_string(), mapping);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(req: &ExternalActionRequest) -> ApprovedActionToken {
        ApprovedActionToken {
            proposal_id: req.proposal_id.clone(),
            granted_by: "action-gateway".to_string(),
        }
    }

    #[test]
    fn read_and_events() {
        let c = GenericFixtureConnector::new();
        assert_eq!(c.read(&SyncCursor::default(), 10).unwrap().len(), 1);
        assert_eq!(c.poll_events(&SyncCursor::default()).unwrap().len(), 1);
        assert!(c.health());
    }

    #[test]
    fn governed_write_requires_matching_token() {
        let mut c = GenericFixtureConnector::new();
        let req = ExternalActionRequest {
            external_ref: ExternalObjectRef::generate_with("ext"),
            action: "create".to_string(),
            payload: serde_json::json!({"x": 1}),
            proposal_id: ActionProposalId::generate(),
        };
        let wrong = ApprovedActionToken {
            proposal_id: ActionProposalId::generate(),
            granted_by: "x".to_string(),
        };
        assert!(c.execute_governed(&req, &wrong).is_err());
        let receipt = c.execute_governed(&req, &token(&req)).unwrap();
        assert!(receipt.ok);
    }

    #[test]
    fn duplicate_external_request_is_idempotent() {
        let mut c = GenericFixtureConnector::new();
        let req = ExternalActionRequest {
            external_ref: ExternalObjectRef::generate_with("ext"),
            action: "create".to_string(),
            payload: serde_json::json!({"x": 1}),
            proposal_id: ActionProposalId::generate(),
        };
        let _r1 = c.execute_governed(&req, &token(&req)).unwrap();
        let r2 = c.execute_governed(&req, &token(&req)).unwrap();
        assert_eq!(
            c.applied_external_ids.len(),
            1,
            "duplicate must not re-apply"
        );
        assert_eq!(r2.external_id.as_deref(), Some("dup"));
    }

    #[test]
    fn rate_limit_and_teardown() {
        let mut c = GenericFixtureConnector::new();
        c.rate_limit_per_min = 1;
        let req = ExternalActionRequest {
            external_ref: ExternalObjectRef::generate_with("ext"),
            action: "create".to_string(),
            payload: serde_json::json!({}),
            proposal_id: ActionProposalId::generate(),
        };
        assert!(c.execute_governed(&req, &token(&req)).is_ok());
        let second = ExternalActionRequest {
            external_ref: ExternalObjectRef::generate_with("ext2"),
            action: "create".to_string(),
            payload: serde_json::json!({}),
            proposal_id: ActionProposalId::generate(),
        };
        assert!(c.execute_governed(&second, &token(&second)).is_err());
        c.teardown();
        assert!(c.teardown_called);
    }
}
