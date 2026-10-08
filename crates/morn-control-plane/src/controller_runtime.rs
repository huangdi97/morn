//! Durable controller-runtime tick for v11.5 Work resources.
//!
//! The semantic controllers are pure/domain-oriented. This runtime wrapper adds
//! the operational guarantees needed by a real control plane: lease/fencing,
//! reload of canonical Work state, optimistic concurrency, and atomic
//! state+outbox commit.

use serde::{Deserialize, Serialize};
use serde_json::json;

use morn_kernel::error::{Error, Result};
use morn_kernel::{
    EventEnvelope, EventSemanticClass, EventSemanticDescriptor,
};
use morn_kernel::time::Timestamp;
use morn_profile::DomainProfile;
use morn_store::MornStore;
use morn_work::control::{WorkPhase, WorkResource};

use crate::{ControllerInputs, WorkController};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerTickResult {
    pub work_ref: String,
    pub previous_phase: WorkPhase,
    pub next_phase: WorkPhase,
    pub previous_revision: u64,
    pub next_revision: u64,
    pub fencing_token: u64,
    pub event_id: String,
}

#[derive(Debug, Clone)]
pub struct DurableWorkControllerRuntime {
    pub holder: String,
    pub lease_name: String,
    pub lease_ttl_ms: i64,
}

impl DurableWorkControllerRuntime {
    pub fn new(holder: impl Into<String>) -> Self {
        Self {
            holder: holder.into(),
            lease_name: "morn-v115-work-controller".to_string(),
            lease_ttl_ms: 30_000,
        }
    }

    pub fn reconcile_once(
        &self,
        store: &MornStore,
        work_id: &str,
        profile: &DomainProfile,
        inputs: &ControllerInputs,
        now: Timestamp,
    ) -> Result<ControllerTickResult> {
        if self.holder.trim().is_empty() || self.lease_ttl_ms <= 0 {
            return Err(Error::validation(
                "controller runtime requires holder identity and positive lease ttl",
            ));
        }

        let lease = store
            .acquire_controller_lease(
                &self.lease_name,
                &self.holder,
                now.millis(),
                self.lease_ttl_ms,
            )?
            .ok_or_else(|| {
                Error::conflict(format!(
                    "controller lease {} is currently held by another runtime",
                    self.lease_name
                ))
            })?;

        if !store.controller_fence_is_current(
            &self.lease_name,
            lease.fencing_token,
            now.millis(),
        )? {
            return Err(Error::conflict("controller fencing token is no longer current"));
        }

        let mut work: WorkResource = store
            .load_record("work_resource_v115", work_id)?
            .ok_or_else(|| Error::not_found(format!("WorkResource {work_id}")))?;

        if work.spec.profile_ref != profile.canonical_ref() {
            return Err(Error::conflict(format!(
                "Work profile {} does not match controller profile {}",
                work.spec.profile_ref,
                profile.canonical_ref()
            )));
        }

        let previous_phase = work.status.phase;
        let previous_revision = work.resource_version;
        WorkController.reconcile(&mut work, profile, inputs);

        let event_id = format!(
            "work:{}:revision:{}:reconcile",
            work.id,
            previous_revision.saturating_add(1)
        );
        let envelope = EventEnvelope::new(
            event_id.clone(),
            format!("morn://controller/{}", self.holder),
            "io.morn.work.reconciled.v1",
            json!({
                "work_ref": work.id.to_string(),
                "generation": work.generation,
                "previous_phase": format!("{:?}", previous_phase),
                "next_phase": format!("{:?}", work.status.phase),
                "observed_generation": work.status.observed_generation,
                "fencing_token": lease.fencing_token,
            }),
        )
        .with_morn_context(
            Some(work.id.as_str()),
            None,
            work.status.active_binding.as_ref().map(|id| id.as_str()),
            Some(&work.spec.profile_ref),
            None,
        );
        let semantics = EventSemanticDescriptor {
            class: EventSemanticClass::DomainFact,
            subject_ref: Some(format!("work://{}", work.id)),
            source_of_truth_ref: None,
            schema_ref: Some("morn://schemas/work-reconciled/v1".to_string()),
        };

        let mut persisted = work.clone();
        persisted.resource_version = previous_revision.saturating_add(1);
        let next_revision = store.save_record_cas_with_durable_event_fenced(
            "work_resource_v115",
            work.id.as_str(),
            work.workspace_id.as_str(),
            work.created_at.millis(),
            previous_revision,
            &persisted,
            &envelope,
            &semantics,
            &self.lease_name,
            lease.fencing_token,
            now.millis(),
        )?;

        Ok(ControllerTickResult {
            work_ref: work.id.to_string(),
            previous_phase,
            next_phase: work.status.phase,
            previous_revision,
            next_revision,
            fencing_token: lease.fencing_token,
            event_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::WorkSpec;

    #[test]
    fn durable_tick_uses_lease_cas_and_atomic_outbox() {
        let store = MornStore::open_in_memory().unwrap();
        let profile = DomainProfile::lite_v1();
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "summarize evidence",
            profile.canonical_ref(),
        );
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let id = work.id.to_string();

        // First durable write establishes revision 1.
        let mut initial = work.clone();
        initial.resource_version = 1;
        store
            .save_record_cas(
                "work_resource_v115",
                initial.id.as_str(),
                initial.workspace_id.as_str(),
                initial.created_at.millis(),
                0,
                &initial,
            )
            .unwrap();

        let runtime = DurableWorkControllerRuntime::new("node-a");
        let result = runtime
            .reconcile_once(
                &store,
                &id,
                &profile,
                &ControllerInputs {
                    capability_resolved: true,
                    capability_qualified: true,
                    authority_satisfied: true,
                    source_of_truth_bound: true,
                    provenance_ready: true,
                },
                Timestamp::from_millis(1_000),
            )
            .unwrap();

        assert_eq!(result.previous_revision, 1);
        assert_eq!(result.next_revision, 2);
        assert_eq!(store.pending_outbox_events(10).unwrap().len(), 1);
        let persisted: WorkResource = store
            .load_record("work_resource_v115", &id)
            .unwrap()
            .unwrap();
        assert_eq!(persisted.resource_version, 2);
        assert_eq!(persisted.status.observed_generation, persisted.generation);
    }

    #[test]
    fn controller_refuses_profile_confusion() {
        let store = MornStore::open_in_memory().unwrap();
        let lite = DomainProfile::lite_v1();
        let enterprise = DomainProfile::enterprise_v1();
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "summarize evidence",
            lite.canonical_ref(),
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        work.resource_version = 1;
        store
            .save_record_cas(
                "work_resource_v115",
                work.id.as_str(),
                work.workspace_id.as_str(),
                work.created_at.millis(),
                0,
                &work,
            )
            .unwrap();

        let runtime = DurableWorkControllerRuntime::new("node-profile");
        assert!(runtime
            .reconcile_once(
                &store,
                work.id.as_str(),
                &enterprise,
                &ControllerInputs::default(),
                Timestamp::from_millis(1_000),
            )
            .is_err());
    }

    #[test]
    fn competing_controller_is_fenced_out() {
        let store = MornStore::open_in_memory().unwrap();
        let profile = DomainProfile::lite_v1();
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "summarize evidence",
            profile.reference(),
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        work.resource_version = 1;
        store
            .save_record_cas(
                "work_resource_v115",
                work.id.as_str(),
                work.workspace_id.as_str(),
                work.created_at.millis(),
                0,
                &work,
            )
            .unwrap();

        store
            .acquire_controller_lease("morn-v115-work-controller", "node-a", 1_000, 30_000)
            .unwrap()
            .unwrap();

        let runtime = DurableWorkControllerRuntime::new("node-b");
        assert!(runtime
            .reconcile_once(
                &store,
                work.id.as_str(),
                &profile,
                &ControllerInputs::default(),
                Timestamp::from_millis(1_001),
            )
            .is_err());
    }
}
