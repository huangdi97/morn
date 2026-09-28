//! connector — Connector / Action Providers: Fake MES / Fake CMMS (sandbox only).
//!
//! SAFETY: these are FAKE connectors for sandbox verification only — never
//! real production endpoints. All side-effect actions must go through the
//! Action Gateway (Work → proposal → PEP → AuthorityDecision → Binding →
//! Provider → Receipt); an Agent never calls a real production-write path.
//!
//! Deterministic semantics: every action is idempotent by business key
//! (re-submitting the same key returns the existing order, never a
//! duplicate); an intentional timeout mode simulates "committed but
//! unacknowledged" for reconciliation testing (G7).

use serde::{Deserialize, Serialize};

/// One connector action receipt (external system view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorReceipt {
    pub external_ref: String,
    pub accepted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// ConnectorProvider — the external action contract (sandbox fakes only).
pub trait ConnectorProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    /// Creates a work order. Idempotent by business key: the same key returns
    /// the existing order (never a duplicate).
    fn create_order(
        &self,
        business_key: &str,
        payload_json: &str,
    ) -> Result<ConnectorReceipt, String>;
    /// Queries whether an order exists for a business key (readback for
    /// reconciliation).
    fn query_order(&self, business_key: &str) -> Result<Option<String>, String>;
}

#[derive(Default)]
pub(crate) struct FakeStore {
    orders: std::collections::BTreeMap<String, String>,
}

macro_rules! fake_connector {
    ($connector:ident, $id:literal, $ref_prefix:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $connector {
            store: std::sync::Arc<std::sync::Mutex<FakeStore>>,
            /// When true, create_order commits the order but "times out" —
            /// simulating committed-but-unacknowledged for reconciliation.
            timeout_after_commit: std::sync::atomic::AtomicBool,
        }

        impl $connector {
            pub fn new() -> Self {
                Self {
                    store: std::sync::Arc::new(std::sync::Mutex::new(FakeStore::default())),
                    timeout_after_commit: std::sync::atomic::AtomicBool::new(false),
                }
            }

            /// Arms the intentional timeout-after-commit mode (reconciliation
            /// testing only).
            pub fn arm_timeout_after_commit(&self) {
                self.timeout_after_commit
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }

            /// Disarms the timeout mode (re-submit becomes idempotent again).
            pub fn disarm_timeout_after_commit(&self) {
                self.timeout_after_commit
                    .store(false, std::sync::atomic::Ordering::SeqCst);
            }
        }

        impl Default for $connector {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ConnectorProvider for $connector {
            fn provider_id(&self) -> &str {
                $id
            }

            fn create_order(
                &self,
                business_key: &str,
                payload_json: &str,
            ) -> Result<ConnectorReceipt, String> {
                if business_key.is_empty() {
                    return Err("business key is required for external actions".to_string());
                }
                let mut store = self.store.lock().unwrap();
                // INVARIANT: idempotent by business key — the same key returns
                // the existing order, never a duplicate.
                if let Some(existing) = store.orders.get(business_key) {
                    return Ok(ConnectorReceipt {
                        external_ref: existing.clone(),
                        accepted: true,
                        detail: Some("idempotent: existing order returned".to_string()),
                    });
                }
                let external_ref = format!(
                    "{}-{}",
                    $ref_prefix,
                    &business_key[business_key.len().saturating_sub(6)..]
                );
                store
                    .orders
                    .insert(business_key.to_string(), external_ref.clone());
                let _ = payload_json;

                // Intentional timeout-after-commit: the order committed but the
                // caller gets no acknowledgement (reconciliation testing).
                if self
                    .timeout_after_commit
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
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
                Ok(self.store.lock().unwrap().orders.get(business_key).cloned())
            }
        }
    };
}

fake_connector!(
    FakeCmmsConnector,
    "fake-cmms",
    "MO",
    "Fake CMMS connector (sandbox only; idempotent by business key)."
);
fake_connector!(
    FakeMesConnector,
    "fake-mes",
    "WO",
    "Fake MES connector (sandbox only; idempotent by business key)."
);
