//! Durable controller-runtime tick for v11.5 Work resources.
//!
//! The semantic controllers are pure/domain-oriented. This runtime wrapper adds
//! the operational guarantees needed by a real control plane: lease/fencing,
//! reload of canonical Work state, optimistic concurrency, and atomic
//! state+outbox commit.

use serde::{Deserialize, Serialize};
use serde_json::json;

use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use morn_kernel::{EventEnvelope, EventSemanticClass, EventSemanticDescriptor};
use morn_profile::DomainProfile;
use morn_store::store::{ControllerFence, DurableProjectionCommit, MornStore};
use morn_work::control::{WorkPhase, WorkResource};

use crate::{derive_controller_inputs, ConditionEvidence, ControllerInputs, WorkController};

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

    /// Strict v11.5 path: readiness is derived from durable,
    /// generation-scoped evidence instead of caller supplied booleans.
    pub fn reconcile_from_evidence(
        &self,
        store: &MornStore,
        work_id: &str,
        profile: &DomainProfile,
        now: Timestamp,
    ) -> Result<ControllerTickResult> {
        let work: WorkResource = store
            .load_record("work_resource_v115", work_id)?
            .ok_or_else(|| Error::not_found(format!("WorkResource {work_id}")))?;
        let evidence: Vec<ConditionEvidence> = store
            .load_records_in_workspace("condition_evidence_v115", work.workspace_id.as_str())?;
        let inputs = derive_controller_inputs(&work, &evidence, now);
        self.reconcile_once(store, work_id, profile, &inputs, now)
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
            return Err(Error::conflict(
                "controller fencing token is no longer current",
            ));
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
            DurableProjectionCommit {
                kind: "work_resource_v115",
                id: work.id.as_str(),
                workspace_id: work.workspace_id.as_str(),
                created_at: work.created_at.millis(),
                expected_revision: previous_revision,
                record: &persisted,
                envelope: &envelope,
                semantics: &semantics,
            },
            ControllerFence {
                lease_name: &self.lease_name,
                fencing_token: lease.fencing_token,
                fence_at: now.millis(),
            },
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
    fn durable_evidence_path_ignores_stale_generation_witnesses() {
        use crate::ControlPlaneStore;

        let store = MornStore::open_in_memory().unwrap();
        let profile = DomainProfile::lite_v1();
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "summarize evidence",
            profile.canonical_ref(),
        );
        let mut work = WorkResource::new(WorkspaceId::generate(), spec);
        store.save_work_resource_cas(&mut work).unwrap();

        let evidence = ConditionEvidence::new(
            &work,
            "CapabilityResolved",
            true,
            "resolver://fixture",
            vec!["capability://fixture".to_string()],
        )
        .unwrap();
        store.save_condition_evidence(&work, &evidence).unwrap();

        let runtime = DurableWorkControllerRuntime::new("node-evidence");
        let ready = runtime
            .reconcile_from_evidence(
                &store,
                work.id.as_str(),
                &profile,
                Timestamp::from_millis(evidence.observed_at.millis() + 1),
            )
            .unwrap();
        assert_eq!(ready.next_phase, WorkPhase::Ready);

        let mut current: WorkResource = store
            .load_record("work_resource_v115", work.id.as_str())
            .unwrap()
            .unwrap();
        let mut changed = current.spec.clone();
        changed.goal = "changed scope".to_string();
        current.replace_spec(changed);
        store.save_work_resource_cas(&mut current).unwrap();

        let blocked = runtime
            .reconcile_from_evidence(
                &store,
                current.id.as_str(),
                &profile,
                Timestamp::from_millis(evidence.observed_at.millis() + 2),
            )
            .unwrap();
        assert_eq!(blocked.next_phase, WorkPhase::Blocked);
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
