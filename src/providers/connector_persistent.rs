//! connector_persistent — file-backed persistent fake connector.
//!
//! SAFETY: simulates a REAL external system's server-side state: an order
//! committed on the "server" survives the client process crashing. This is
//! what makes crash-after-commit reconciliation tests real (G7) — the
//! reconciliation readback queries the persistent server state, not the
//! client's in-memory map.
//!
//! Deterministic + idempotent by business key, same action semantics as
//! `FakeCmmsConnector` (sandbox only, never a real production endpoint), with
//! state persisted to a JSON file via temp-file + rename.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use super::connector::{ConnectorProvider, ConnectorReceipt};

#[derive(Default, Serialize, Deserialize)]
struct PersistentOrders {
    orders: BTreeMap<String, String>,
}

/// File-backed persistent fake CMMS connector (sandbox verification only).
pub struct PersistentFakeCmmsConnector {
    path: PathBuf,
    /// When true, create_order commits the order but "times out" —
    /// simulating committed-but-unacknowledged for reconciliation testing.
    timeout_after_commit: AtomicBool,
}

impl PersistentFakeCmmsConnector {
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        // SAFETY: the persistent store directory is created eagerly — a fake
        // external system must never fail on a missing local directory.
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self {
            path,
            timeout_after_commit: AtomicBool::new(false),
        }
    }

    /// Arms the intentional timeout-after-commit mode (reconciliation testing).
    pub fn arm_timeout_after_commit(&self) {
        self.timeout_after_commit.store(true, Ordering::SeqCst);
    }

    /// Disarms the timeout mode (re-submit becomes idempotent again).
    pub fn disarm_timeout_after_commit(&self) {
        self.timeout_after_commit.store(false, Ordering::SeqCst);
    }

    fn load(&self) -> PersistentOrders {
        std::fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Atomic-ish save: write to a temp file, then rename over the target.
    fn save(&self, orders: &PersistentOrders) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(orders).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &self.path).map_err(|e| e.to_string())
    }
}

impl ConnectorProvider for PersistentFakeCmmsConnector {
    fn provider_id(&self) -> &str {
        "fake-cmms-persistent"
    }

    fn create_order(
        &self,
        business_key: &str,
        payload_json: &str,
    ) -> Result<ConnectorReceipt, String> {
        if business_key.is_empty() {
            return Err("business key is required for external actions".to_string());
        }
        let mut orders = self.load();
        // INVARIANT: idempotent by business key — the same key returns the
        // existing order, never a duplicate.
        if let Some(existing) = orders.orders.get(business_key) {
            return Ok(ConnectorReceipt {
                external_ref: existing.clone(),
                accepted: true,
                detail: Some("idempotent: existing order returned".to_string()),
            });
        }
        let external_ref = format!(
            "MO-{}",
            &business_key[business_key.len().saturating_sub(6)..]
        );
        orders
            .orders
            .insert(business_key.to_string(), external_ref.clone());
        self.save(&orders)?;
        let _ = payload_json;

        // Intentional timeout-after-commit: the order is committed on the
        // persistent "server" but the caller gets no acknowledgement.
        if self.timeout_after_commit.load(Ordering::SeqCst) {
            return Err(format!(
                "transport timeout after commit (order {external_ref} exists)"
            ));
        }

        Ok(ConnectorReceipt {
            external_ref,
            accepted: true,
            detail: None,
        })
    }

    fn query_order(&self, business_key: &str) -> Result<Option<String>, String> {
        Ok(self.load().orders.get(business_key).cloned())
    }
}
