//! host — CompositionHost: Morn's facade over the Cordis Context.
//!
//! The host mounts providers into Morn service slots and exposes typed
//! lifecycle operations. It translates Cordis errors into the Morn contract
//! error type; nothing else in Morn sees raw Cordis types.
//!
//! BOUNDARY (master §7.2): the host must never touch Work bindings, attempts,
//! receipts, or outcomes. Provider reload only changes composition state; the
//! active Attempt stays pinned to its Binding in the Control Plane (proven by
//! `tests/runtime_cordis_g3.rs::g3_provider_reload_does_not_rebind_active_attempt`).

use super::slots::ServiceSlotError;
use cordis::service::ServicePublication;
use cordis::{Context, Service};
use std::sync::Arc;

/// Morn's reference composition host over one Cordis Context.
#[derive(Clone)]
pub struct CompositionHost {
    ctx: Context,
}

/// A mounted provider: the publication handle owns the registration. Dropping
/// it unregisters the provider (Cordis generation-owned cleanup) — a runtime
/// effect, never a business rollback.
pub struct MountedProvider<S: Service> {
    publication: Option<ServicePublication<S>>,
    slot: String,
}

impl<S: Service> MountedProvider<S> {
    /// Unregisters the provider explicitly (idempotent: dropping also cleans
    /// up).
    pub fn unmount(mut self) -> Result<(), ServiceSlotError> {
        if let Some(publication) = self.publication.take() {
            publication
                .remove()
                .map_err(|_| ServiceSlotError::Unavailable {
                    slot: self.slot.clone(),
                })?;
        }
        Ok(())
    }

    /// Upgrades the provider payload in place (publication identity is
    /// preserved, so dependency targets do not drift).
    pub fn upgrade(&self, value: Arc<S>) -> Result<(), ServiceSlotError> {
        self.publication
            .as_ref()
            .ok_or(ServiceSlotError::Unavailable {
                slot: self.slot.clone(),
            })?
            .set(value)
            .map_err(|_| ServiceSlotError::Unavailable {
                slot: self.slot.clone(),
            })
    }

    pub fn slot(&self) -> &str {
        &self.slot
    }
}

impl CompositionHost {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(),
        }
    }

    /// Mounts a provider payload into its Morn service slot. Consumers resolve
    /// the slot by contract (`S::NAME`), not by provider type.
    pub fn mount<S: Service>(&self, value: Arc<S>) -> Result<MountedProvider<S>, ServiceSlotError> {
        let publication = self
            .ctx
            .provide(value)
            .map_err(|_| ServiceSlotError::Unavailable {
                slot: S::NAME.to_string(),
            })?;
        Ok(MountedProvider {
            publication: Some(publication),
            slot: S::NAME.to_string(),
        })
    }

    /// Resolves a slot's visible payload (typed lookup by contract).
    pub fn resolve<S: Service>(&self) -> Result<Arc<S>, ServiceSlotError> {
        self.ctx
            .try_service::<S>()
            .map_err(|_| ServiceSlotError::Unavailable {
                slot: S::NAME.to_string(),
            })
    }

    /// Returns a scope whose resolution of the same contract is isolated —
    /// e.g. per-site provider isolation (master §8 "Morn runtime scope ↔
    /// Cordis context/scope").
    pub fn scoped(&self, service_name: &str) -> Self {
        Self {
            ctx: self.ctx.with_isolated_service(service_name),
        }
    }
}

impl Default for CompositionHost {
    fn default() -> Self {
        Self::new()
    }
}
