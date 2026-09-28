//! resolve_loop — CapabilityResolverController / QualificationController.
//!
//! The resolver is not an LLM guesser: hard constraints filter, scoring ranks.
//! Phase 2 keeps a deterministic resolver with an injected capability catalog;
//! the full resolver (Phase 11) refines eligibility without changing this
//! contract.

use super::ControllerOutcome;
use crate::control::store::ControlPlaneStore;
use crate::core::error::MornError;
use crate::work::{Condition, ConditionType, QualificationRecord, QualificationStatus, WorkSpec};

/// CapabilityResolverController — resolves Work requirements into capability
/// refs, deterministically, from an injected catalog.
pub struct CapabilityResolverController;

impl CapabilityResolverController {
    pub const ID: &'static str = "capability-resolver-controller";

    /// Resolves capabilities for a Work via an injected catalog lookup.
    ///
    /// `catalog` returns the eligible capability refs for the work's
    /// requirements (hard constraints + qualification/site admission are the
    /// caller's responsibility here; Phase 11 centralizes them).
    pub fn resolve(
        &self,
        store: &ControlPlaneStore,
        work_id: &str,
        catalog: impl Fn(&WorkSpec) -> Vec<String>,
    ) -> Result<Option<ControllerOutcome>, MornError> {
        let Some(work) = store.get_work(work_id)? else {
            return Ok(None);
        };
        let resolved = catalog(&work.spec);

        let outcome = if resolved.is_empty() {
            ControllerOutcome::new(Self::ID, work_id)
                .with_condition(Condition::unsatisfied(
                    ConditionType::CapabilityResolved,
                    "no eligible capability in catalog",
                ))
                .with_note("resolver found no eligible capability")
        } else {
            ControllerOutcome::new(Self::ID, work_id)
                .with_condition(Condition::satisfied(ConditionType::CapabilityResolved))
                .with_note(&format!("resolved {} capability(ies)", resolved.len()))
        };
        Ok(Some(outcome))
    }
}

/// QualificationController — records qualification results.
///
/// SAFETY: qualification records are never modified by the evaluated subject
/// itself; the evaluator identity is recorded.
pub struct QualificationController;

impl QualificationController {
    pub const ID: &'static str = "qualification-controller";

    /// Records a qualification result. PASSED qualifications enable binding;
    /// FAILED/EXPIRED do not.
    pub fn record(
        &self,
        store: &ControlPlaneStore,
        record: &QualificationRecord,
    ) -> Result<ControllerOutcome, MornError> {
        let inserted = store.insert_qualification(record)?;
        let note = if inserted {
            match record.status {
                QualificationStatus::Passed => "qualification passed",
                QualificationStatus::Failed => "qualification failed",
                QualificationStatus::Expired => "qualification expired",
                QualificationStatus::Pending => "qualification pending",
            }
        } else {
            "qualification already recorded (idempotent)"
        };
        Ok(ControllerOutcome::new(Self::ID, &record.capability_digest).with_note(note))
    }

    /// G8 strict-profile gate: a capability may enter a strict-profile
    /// critical binding ONLY when its latest qualification is PASSED and not
    /// expired. FAILED/EXPIRED/absent qualifications block the binding.
    pub fn capability_qualified_for_strict(
        store: &ControlPlaneStore,
        capability_digest: &str,
    ) -> Result<bool, MornError> {
        match store.get_latest_qualification(capability_digest)? {
            None => Ok(false),
            Some((status, _qualified_at)) => {
                Ok(serde_json::from_str::<QualificationStatus>(&status)
                    .map(|s| s == QualificationStatus::Passed)
                    .unwrap_or(false))
            }
        }
    }
}
