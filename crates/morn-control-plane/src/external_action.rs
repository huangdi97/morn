//! Governed external-action construction.
//!
//! ActionAttempt is a runtime state machine and can be used in tests/internal
//! code. Real connector attempts must be created through this boundary so the
//! Work/profile/authority permit and pinned ExecutionBinding cannot be skipped.

use morn_kernel::error::{Error, Result};
use morn_runtime::{ActionAttempt, AttemptState, ExecutionBinding};

use crate::ExternalActionPermit;

pub fn begin_external_attempt(
    permit: &ExternalActionPermit,
    binding: &ExecutionBinding,
    business_key: impl Into<String>,
    action_name: impl Into<String>,
) -> Result<ActionAttempt> {
    if permit.work_ref != binding.work_id.to_string() {
        return Err(Error::validation(
            "external action permit Work does not match ExecutionBinding Work",
        ));
    }
    if permit.binding_ref != binding.id.to_string() {
        return Err(Error::validation(
            "external action permit does not match pinned ExecutionBinding",
        ));
    }
    if permit.profile_ref != binding.profile_ref {
        return Err(Error::validation(
            "external action permit profile does not match ExecutionBinding profile",
        ));
    }

    let business_key = business_key.into();
    let action_name = action_name.into();
    if business_key.trim().is_empty() || action_name.trim().is_empty() {
        return Err(Error::validation(
            "external action requires non-empty business key and action name",
        ));
    }

    let mut attempt = ActionAttempt::new(binding.id.clone(), business_key, action_name);
    attempt.transition(AttemptState::Authorized)?;
    Ok(attempt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{AuthorityDecisionTag, Id, WorkPackageId, WorkspaceId};
    use morn_kernel::time::Timestamp;
    use morn_work::control::{WorkResource, WorkSpec};

    #[test]
    fn permit_must_match_exact_work_binding_and_profile() {
        let work = WorkResource::new(
            WorkspaceId::generate(),
            WorkSpec::new(
                WorkPackageId::generate_with("work"),
                "test",
                "morn.factory.readonly@1.0.0",
            ),
        );
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let permit = ExternalActionPermit {
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: Id::<AuthorityDecisionTag>::generate_with("authz").to_string(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            issued_at: Timestamp::now(),
        };
        let attempt =
            begin_external_attempt(&permit, &binding, "business-key", "sandbox-write").unwrap();
        assert_eq!(attempt.state, AttemptState::Authorized);

        let mut wrong = permit.clone();
        wrong.binding_ref = "binding:other".to_string();
        assert!(begin_external_attempt(&wrong, &binding, "business-key", "sandbox-write").is_err());
    }
}
