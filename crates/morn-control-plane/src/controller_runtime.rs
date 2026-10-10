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
use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};
use morn_work::control::{ConditionStatus, WorkCondition, WorkPhase, WorkResource};
use morn_world::ObservedOutcome;

use crate::{
    derive_controller_inputs, require_attested_outcome_provenance, ConditionEvidence,
    ControllerInputs, WorkController,
};

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
    /// generation-scoped evidence. Do not derive before obtaining the controller
    /// lease and reloading Work: a concurrent spec generation could otherwise
    /// receive Conditions that were valid for the previous generation.
    pub fn reconcile_from_evidence(
        &self,
        store: &MornStore,
        work_id: &str,
        profile: &DomainProfile,
        now: Timestamp,
    ) -> Result<ControllerTickResult> {
        self.reconcile_internal(store, work_id, profile, None, now)
    }

    /// Reference/fixture entry point for explicit controller inputs. Production
    /// API routes use reconcile_from_evidence, not caller-provided readiness.
    pub fn reconcile_once(
        &self,
        store: &MornStore,
        work_id: &str,
        profile: &DomainProfile,
        inputs: &ControllerInputs,
        now: Timestamp,
    ) -> Result<ControllerTickResult> {
        self.reconcile_internal(store, work_id, profile, Some(inputs), now)
    }

    fn reconcile_internal(
        &self,
        store: &MornStore,
        work_id: &str,
        profile: &DomainProfile,
        provided_inputs: Option<&ControllerInputs>,
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

        // The evidence and Work snapshot are now from the same generation
        // under the controller lease. A concurrent Work mutation after reload
        // is still rejected by the fenced CAS at commit time.
        let derived_inputs = if provided_inputs.is_none() {
            let evidence: Vec<ConditionEvidence> = store
                .load_records_in_workspace("condition_evidence_v115", work.workspace_id.as_str())?;
            Some(derive_controller_inputs(&work, &evidence, now))
        } else {
            None
        };
        let inputs = provided_inputs
            .or(derived_inputs.as_ref())
            .ok_or_else(|| Error::internal("controller inputs are missing"))?;

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


#[derive(Debug, Clone)]
pub struct DurableBusinessEvidenceControllerRuntime {
    pub holder: String,
    pub lease_name: String,
    pub lease_ttl_ms: i64,
}

impl DurableBusinessEvidenceControllerRuntime {
    pub fn new(holder: impl Into<String>) -> Self {
        Self {
            holder: holder.into(),
            lease_name: "morn-v115-business-evidence-controller".to_string(),
            lease_ttl_ms: 30_000,
        }
    }

    /// Recover the canonical Work projection from already-persisted business
    /// evidence. This controller never replays a Harness turn, never consumes a
    /// deployment bearer again, and never invents an Outcome or Acceptance.
    ///
    /// It exists for the crash/CAS window where immutable evidence was committed
    /// first but the final mutable Work projection did not advance.
    pub fn reconcile_from_persisted_business_evidence(
        &self,
        store: &MornStore,
        work_id: &str,
        now: Timestamp,
    ) -> Result<ControllerTickResult> {
        if self.holder.trim().is_empty() || self.lease_ttl_ms <= 0 {
            return Err(Error::validation(
                "business-evidence controller requires holder identity and positive lease ttl",
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
                "business-evidence controller fencing token is no longer current",
            ));
        }

        let mut work: WorkResource = store
            .load_record("work_resource_v115", work_id)?
            .ok_or_else(|| Error::not_found(format!("WorkResource {work_id}")))?;
        if work.status.phase.is_terminal() {
            return Err(Error::invalid_state(
                "terminal Work does not require business-evidence projection recovery",
            ));
        }

        let outcomes: Vec<ObservedOutcome> = store
            .load_records_in_workspace("observed_outcome_v115", work.workspace_id.as_str())?
            .into_iter()
            .filter(|outcome: &ObservedOutcome| {
                outcome.work_package_id == work.id
                    && outcome.work_generation == work.generation
                    && outcome.workspace_id == work.workspace_id
                    && outcome.is_source_grounded()
            })
            .collect();
        if outcomes.is_empty() {
            return Err(Error::not_found(
                "no persisted source-grounded business outcome exists for this Work generation",
            ));
        }

        let decisions: Vec<AcceptanceDecision> = store
            .load_records_in_workspace("acceptance_decision_v115", work.workspace_id.as_str())?
            .into_iter()
            .filter(|decision: &AcceptanceDecision| {
                decision.work_package_id == work.id
                    && decision.work_generation == work.generation
                    && !decision.evidence_refs.is_empty()
                    && !decision.acting_role.trim().is_empty()
                    && !decision.reason.trim().is_empty()
            })
            .collect();

        let spec_matches = |decision: &AcceptanceDecision| {
            work.spec
                .acceptance_ref
                .as_deref()
                .is_none_or(|expected| expected == decision.acceptance_spec_id.as_str())
        };
        let linked_outcome = |decision: &AcceptanceDecision| {
            outcomes
                .iter()
                .filter(|outcome| decision.outcome_refs.contains(&outcome.id))
                .max_by_key(|outcome| (outcome.observed_at.millis(), outcome.id.to_string()))
        };

        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for decision in &decisions {
            if !spec_matches(decision) {
                continue;
            }
            let Some(outcome) = linked_outcome(decision) else {
                continue;
            };
            match decision.disposition {
                AcceptanceDisposition::Accept
                    if decision.is_final_acceptance()
                        && require_attested_outcome_provenance(store, &work, outcome).is_ok() =>
                {
                    accepted.push((decision, outcome));
                }
                AcceptanceDisposition::Reject => rejected.push((decision, outcome)),
                AcceptanceDisposition::Conditional
                | AcceptanceDisposition::RequestMoreEvidence
                | AcceptanceDisposition::Accept => {}
            }
        }
        if !accepted.is_empty() && !rejected.is_empty() {
            return Err(Error::conflict(
                "conflicting final Accept and Reject evidence exists for the same Work generation",
            ));
        }

        let previous_phase = work.status.phase;
        let previous_revision = work.resource_version;
        let source_of_truth_ref = if let Some((decision, outcome)) = accepted
            .into_iter()
            .max_by_key(|(decision, _)| (decision.decided_at.millis(), decision.id.to_string()))
        {
            let mut observed = WorkCondition::new("OutcomeObservation", ConditionStatus::True);
            observed.reason = format!("persisted authoritative outcome {}", outcome.id);
            observed.evidence_refs = outcome.evidence_refs.clone();
            work.set_condition(observed);

            let mut independent =
                WorkCondition::new("IndependentAcceptance", ConditionStatus::True);
            independent.reason = format!(
                "persisted acceptance decision {} by role {}",
                decision.id, decision.acting_role
            );
            independent.evidence_refs = vec![decision.id.to_string()];
            work.set_condition(independent);

            let mut semantics =
                WorkCondition::new("AcceptedOutcomeSemantics", ConditionStatus::True);
            semantics.reason =
                "persisted final acceptance references deployment-attested business truth"
                    .to_string();
            semantics.evidence_refs = vec![outcome.id.to_string(), decision.id.to_string()];
            work.set_condition(semantics);
            work.status.phase = WorkPhase::Accepted;
            Some(outcome.source_ref.clone())
        } else if let Some((decision, outcome)) = rejected
            .into_iter()
            .max_by_key(|(decision, _)| (decision.decided_at.millis(), decision.id.to_string()))
        {
            let mut observed = WorkCondition::new("OutcomeObservation", ConditionStatus::True);
            observed.reason = format!("persisted source-grounded outcome {}", outcome.id);
            observed.evidence_refs = outcome.evidence_refs.clone();
            work.set_condition(observed);

            let mut independent =
                WorkCondition::new("IndependentAcceptance", ConditionStatus::True);
            independent.reason = format!(
                "persisted rejection decision {} by role {}",
                decision.id, decision.acting_role
            );
            independent.evidence_refs = vec![decision.id.to_string()];
            work.set_condition(independent);

            let mut semantics =
                WorkCondition::new("AcceptedOutcomeSemantics", ConditionStatus::False);
            semantics.reason = "persisted independent review rejected the observed outcome"
                .to_string();
            semantics.evidence_refs = vec![outcome.id.to_string(), decision.id.to_string()];
            work.set_condition(semantics);
            work.status.phase = WorkPhase::Rejected;
            Some(outcome.source_ref.clone())
        } else if let Some(decision) = decisions
            .iter()
            .filter(|decision| {
                spec_matches(decision)
                    && matches!(
                        decision.disposition,
                        AcceptanceDisposition::Conditional
                            | AcceptanceDisposition::RequestMoreEvidence
                    )
                    && linked_outcome(decision).is_some()
            })
            .max_by_key(|decision| (decision.decided_at.millis(), decision.id.to_string()))
        {
            let outcome = linked_outcome(decision)
                .ok_or_else(|| Error::internal("linked review outcome disappeared"))?;
            let mut observed = WorkCondition::new("OutcomeObservation", ConditionStatus::True);
            observed.reason = format!("persisted source-grounded outcome {}", outcome.id);
            observed.evidence_refs = outcome.evidence_refs.clone();
            work.set_condition(observed);

            let mut independent =
                WorkCondition::new("IndependentAcceptance", ConditionStatus::True);
            independent.reason = format!(
                "persisted non-final review decision {} by role {}",
                decision.id, decision.acting_role
            );
            independent.evidence_refs = vec![decision.id.to_string()];
            work.set_condition(independent);
            work.status.phase = WorkPhase::Waiting;
            Some(outcome.source_ref.clone())
        } else {
            let outcome = outcomes
                .iter()
                .max_by_key(|outcome| (outcome.observed_at.millis(), outcome.id.to_string()))
                .ok_or_else(|| Error::internal("business outcome set unexpectedly empty"))?;
            let mut observed = WorkCondition::new("OutcomeObservation", ConditionStatus::True);
            observed.reason = format!("persisted source-grounded outcome {}", outcome.id);
            observed.evidence_refs = outcome.evidence_refs.clone();
            work.set_condition(observed);
            work.status.phase = WorkPhase::Delivered;
            Some(outcome.source_ref.clone())
        };
        work.mark_observed();

        let event_id = format!(
            "work:{}:revision:{}:business-evidence-reconcile",
            work.id,
            previous_revision.saturating_add(1)
        );
        let envelope = EventEnvelope::new(
            event_id.clone(),
            format!("morn://controller/{}", self.holder),
            "io.morn.work.business-evidence-reconciled.v1",
            json!({
                "work_ref": work.id.to_string(),
                "generation": work.generation,
                "previous_phase": format!("{:?}", previous_phase),
                "next_phase": format!("{:?}", work.status.phase),
                "observed_generation": work.status.observed_generation,
                "fencing_token": lease.fencing_token,
                "replayed_external_execution": false,
                "reconsumed_authorization": false,
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
            source_of_truth_ref,
            schema_ref: Some(
                "morn://schemas/work-business-evidence-reconciled/v1".to_string(),
            ),
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

    fn persist_attested_business_outcome(
        store: &MornStore,
        work: &WorkResource,
    ) -> ObservedOutcome {
        use crate::ControlPlaneStore;
        use morn_integration::{
            ConflictPolicy, SourceOfTruthBinding, SourceOfTruthBindingId, TruthAuthorityKind,
        };
        use serde_json::json;

        let binding = SourceOfTruthBinding {
            id: SourceOfTruthBindingId::generate_with("sot"),
            site_ref: work.spec.site_ref.clone(),
            source_ref: "erp://delivery".to_string(),
            authority_kind: TruthAuthorityKind::SystemOfRecord,
            authoritative_fact_types: vec!["delivery.status".to_string()],
            key_mapping_ref: "mapping://delivery@1".to_string(),
            query_capability_ref: "capability://delivery.read@1".to_string(),
            freshness_sla_ms: None,
            conflict_policy: ConflictPolicy::ReconcileBeforeUse,
            version_ref: "binding:test-v1".to_string(),
            created_at: Timestamp::now(),
        };
        store.save_source_of_truth_binding(work, &binding).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "delivery completed",
            morn_world::OutcomeSourceKind::ExternalSystem,
            "erp://delivery/42",
            json!({"status":"delivered"}),
        );
        outcome.pin_work_generation(work.generation).unwrap();
        outcome
            .evidence_refs
            .push("erp://delivery/42/receipt".to_string());
        let attestation_id = "observation-attestation://delivery-42";
        outcome
            .pin_source_provenance(
                binding.id.to_string(),
                attestation_id,
                "delivery.status",
            )
            .unwrap();
        store
            .save_record_immutable(
                "source_observation_attestation_consumed_v115",
                attestation_id,
                work.workspace_id.as_str(),
                outcome.observed_at.millis(),
                &json!({
                    "attestation_id": attestation_id,
                    "work_id": work.id,
                    "source_binding_id": binding.id,
                    "fact_type": "delivery.status",
                    "outcome_id": outcome.id,
                    "consumed_at": outcome.observed_at
                }),
            )
            .unwrap();
        store.save_observed_outcome(work, &outcome).unwrap();
        outcome
    }

    #[test]
    fn durable_business_evidence_recovers_delivered_projection_without_replaying_executor() {
        use crate::ControlPlaneStore;
        use serde_json::json;

        let store = MornStore::open_in_memory().unwrap();
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "observe delivery",
                DomainProfile::lite_v1().canonical_ref(),
            ),
        );
        store.save_work_resource_cas(&mut work).unwrap();

        let mut outcome = ObservedOutcome::new(
            work.workspace_id.clone(),
            work.id.clone(),
            "authoritative delivery result",
            morn_world::OutcomeSourceKind::ExternalSystem,
            "erp://delivery/42",
            json!({"status":"delivered"}),
        );
        outcome.pin_work_generation(work.generation).unwrap();
        outcome
            .evidence_refs
            .push("erp://delivery/42/receipt".to_string());
        store.save_observed_outcome(&work, &outcome).unwrap();

        let runtime = DurableBusinessEvidenceControllerRuntime::new("business-node-a");
        let result = runtime
            .reconcile_from_persisted_business_evidence(
                &store,
                work.id.as_str(),
                Timestamp::from_millis(outcome.observed_at.millis().saturating_add(1)),
            )
            .unwrap();
        assert_eq!(result.previous_phase, WorkPhase::Proposed);
        assert_eq!(result.next_phase, WorkPhase::Delivered);

        let persisted: WorkResource = store
            .load_record("work_resource_v115", work.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(persisted.status.phase, WorkPhase::Delivered);
        assert!(persisted.condition_is_true("OutcomeObservation"));
        let outbox = store.pending_outbox_events(10).unwrap();
        assert_eq!(outbox.len(), 1);
        assert_eq!(
            outbox[0].envelope.event_type,
            "io.morn.work.business-evidence-reconciled.v1"
        );
    }

    #[test]
    fn durable_business_evidence_recovers_attested_acceptance_after_projection_gap() {
        use crate::ControlPlaneStore;
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};

        let store = MornStore::open_in_memory().unwrap();
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "accept delivered result",
                DomainProfile::lite_v1().canonical_ref(),
            ),
        );
        store.save_work_resource_cas(&mut work).unwrap();
        let outcome = persist_attested_business_outcome(&store, &work);

        let mut decision = AcceptanceDecision::new(
            work.id.clone(),
            AcceptanceSpecId::generate_with("acceptance"),
            AcceptanceDisposition::Accept,
            PrincipalId::generate_with("reviewer"),
            "independent-reviewer",
            "deployment-attested delivery accepted",
        );
        decision.pin_work_generation(work.generation).unwrap();
        decision.outcome_refs.push(outcome.id.clone());
        decision
            .evidence_refs
            .push("review://signed/delivery-42".to_string());
        ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();

        // Simulate the crash window: immutable Outcome + Acceptance exist, but
        // the mutable Work projection is still at its old phase/revision.
        let runtime = DurableBusinessEvidenceControllerRuntime::new("business-node-b");
        let result = runtime
            .reconcile_from_persisted_business_evidence(
                &store,
                work.id.as_str(),
                Timestamp::from_millis(decision.decided_at.millis().saturating_add(1)),
            )
            .unwrap();
        assert_eq!(result.next_phase, WorkPhase::Accepted);
        let persisted: WorkResource = store
            .load_record("work_resource_v115", work.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(persisted.status.phase, WorkPhase::Accepted);
        assert!(persisted.condition_is_true("IndependentAcceptance"));
        assert!(persisted.condition_is_true("AcceptedOutcomeSemantics"));
    }

    #[test]
    fn durable_business_evidence_fails_closed_on_conflicting_terminal_reviews() {
        use crate::ControlPlaneStore;
        use morn_kernel::ids::{AcceptanceSpecId, PrincipalId};
        use morn_work::acceptance_decision::{AcceptanceDecision, AcceptanceDisposition};

        let store = MornStore::open_in_memory().unwrap();
        let mut work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "review delivery result",
                DomainProfile::lite_v1().canonical_ref(),
            ),
        );
        store.save_work_resource_cas(&mut work).unwrap();
        let outcome = persist_attested_business_outcome(&store, &work);

        for disposition in [AcceptanceDisposition::Accept, AcceptanceDisposition::Reject] {
            let mut decision = AcceptanceDecision::new(
                work.id.clone(),
                AcceptanceSpecId::generate_with("acceptance"),
                disposition,
                PrincipalId::generate_with("reviewer"),
                "independent-reviewer",
                "independent terminal review",
            );
            decision.pin_work_generation(work.generation).unwrap();
            decision.outcome_refs.push(outcome.id.clone());
            decision
                .evidence_refs
                .push("review://signed/conflict".to_string());
            ControlPlaneStore::save_acceptance_decision(&store, &work, &decision).unwrap();
        }

        let runtime = DurableBusinessEvidenceControllerRuntime::new("business-node-conflict");
        let error = runtime
            .reconcile_from_persisted_business_evidence(
                &store,
                work.id.as_str(),
                Timestamp::now(),
            )
            .unwrap_err();
        assert!(format!("{error}").contains("conflicting final Accept and Reject"));
        let persisted: WorkResource = store
            .load_record("work_resource_v115", work.id.as_str())
            .unwrap()
            .unwrap();
        assert_eq!(persisted.status.phase, WorkPhase::Proposed);
    }

    #[test]
    fn competing_controller_is_fenced_out() {
        let store = MornStore::open_in_memory().unwrap();
        let profile = DomainProfile::lite_v1();
        let spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "summarize evidence",
            profile.canonical_ref(),
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
