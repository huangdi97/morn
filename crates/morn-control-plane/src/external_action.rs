//! Governed external-action construction.
//!
//! ActionAttempt is a runtime state machine and can be used in tests/internal
//! code. Real connector attempts must be created through this boundary so the
//! Work/profile/authority permit and pinned ExecutionBinding cannot be skipped.

use morn_capability::effect::{EffectClass, EffectContract};
use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use morn_runtime::{ActionAttempt, AttemptState, ExecutionBinding};

use crate::ExternalActionPermit;

pub fn begin_external_attempt(
    permit: &ExternalActionPermit,
    binding: &ExecutionBinding,
    business_key: impl Into<String>,
    action_name: impl Into<String>,
) -> Result<ActionAttempt> {
    begin_external_attempt_with_effect(
        permit,
        binding,
        EffectContract::e0(),
        business_key,
        action_name,
    )
}

pub fn begin_external_attempt_with_effect(
    permit: &ExternalActionPermit,
    binding: &ExecutionBinding,
    effect: EffectContract,
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
    if permit.site_ref != binding.site_ref {
        return Err(Error::validation(
            "external action permit site does not match ExecutionBinding site",
        ));
    }
    let now = Timestamp::now();
    if permit.not_before.is_some_and(|start| now < start)
        || permit.expires_at.is_some_and(|end| now > end)
    {
        return Err(Error::not_authorized(
            "external action permit is outside its validity window",
        ));
    }

    let business_key = business_key.into();
    let action_name = action_name.into();
    if business_key.trim().is_empty() || action_name.trim().is_empty() {
        return Err(Error::validation(
            "external action requires non-empty business key and action name",
        ));
    }

    if action_name != permit.action {
        return Err(Error::not_authorized(format!(
            "permit authorizes action {}, not {action_name}",
            permit.action
        )));
    }

    if matches!(
        permit.mode,
        crate::ExternalActionMode::SandboxWrite
            | crate::ExternalActionMode::ShadowWrite
            | crate::ExternalActionMode::ProductionWrite
            | crate::ExternalActionMode::PhysicalControl
    ) && effect.class == EffectClass::E0LifecycleReversible
    {
        return Err(Error::validation(
            "write-like external action modes cannot be represented as E0 lifecycle effects",
        ));
    }

    let effect_ceiling = binding
        .effect_ceiling
        .unwrap_or(EffectClass::E0LifecycleReversible);
    if !effect_ceiling.permits(effect.class) {
        return Err(Error::not_authorized(format!(
            "binding effect ceiling {} does not permit requested effect {}",
            effect_ceiling, effect.class
        )));
    }
    if effect.class == EffectClass::E2Compensatable {
        let requested_compensation = effect
            .compensation_action
            .as_deref()
            .ok_or_else(|| Error::validation("E2 external action requires compensation action"))?;
        if binding.compensation_ref.as_deref() != Some(requested_compensation) {
            return Err(Error::validation(
                "E2 attempt requires an exact compensation action pinned in the binding",
            ));
        }
    }

    let mut attempt = ActionAttempt::new(binding.id.clone(), business_key, action_name);
    attempt.set_effect_contract(effect)?;
    attempt.resource_ref = Some(permit.resource.clone());
    attempt.site_ref = permit.site_ref.clone();
    attempt.authority_decision_ref = Some(permit.authority_decision_ref.clone());
    attempt.external_action_permit_ref = Some(permit.id.to_string());
    attempt.transition(AttemptState::Authorized)?;
    Ok(attempt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use morn_kernel::ids::{WorkPackageId, WorkspaceId};
    use morn_work::control::{WorkResource, WorkSpec};

    #[test]
    fn effect_ceiling_blocks_write_from_read_only_binding() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:read", "provider:a", "1");
        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "cmms.create-order".to_string(),
            resource: "cmms://fixture".to_string(),
            site_ref: binding.site_ref.clone(),
            scope: vec![],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };
        assert!(begin_external_attempt_with_effect(
            &permit,
            &binding,
            EffectContract::e2("cmms.cancel-order"),
            "business-key",
            "cmms.create-order",
        )
        .is_err());
    }

    #[test]
    fn e2_attempt_must_match_pinned_compensation() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding = ExecutionBinding::for_work(&work, "capability:write", "provider:a", "1");
        binding.effect_ceiling = Some(EffectClass::E2Compensatable);
        binding.compensation_ref = Some("cmms.cancel-order".to_string());

        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "cmms.create-order".to_string(),
            resource: "cmms://fixture".to_string(),
            site_ref: binding.site_ref.clone(),
            scope: vec![],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };

        assert!(begin_external_attempt_with_effect(
            &permit,
            &binding,
            EffectContract::e2("cmms.other-cancel"),
            "business-key",
            "cmms.create-order",
        )
        .is_err());
        assert!(begin_external_attempt_with_effect(
            &permit,
            &binding,
            EffectContract::e2("cmms.cancel-order"),
            "business-key",
            "cmms.create-order",
        )
        .is_ok());
    }

    #[test]
    fn e2_attempt_rejects_unpinned_compensation_even_under_e3_ceiling() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let mut binding = ExecutionBinding::for_work(&work, "capability:write", "provider:a", "1");
        // A permissive effect ceiling is not evidence that a compensating
        // operation was qualified/pinned when the binding was created.
        binding.effect_ceiling = Some(EffectClass::E3Irreversible);
        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "cmms.create-order".to_string(),
            resource: "cmms://fixture".to_string(),
            site_ref: binding.site_ref.clone(),
            scope: vec![],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };
        assert!(begin_external_attempt_with_effect(
            &permit,
            &binding,
            EffectContract::e2("cmms.cancel-order"),
            "business-key",
            "cmms.create-order",
        )
        .is_err());

        binding.compensation_ref = Some("cmms.cancel-order".to_string());
        assert!(begin_external_attempt_with_effect(
            &permit,
            &binding,
            EffectContract::e2("cmms.cancel-order"),
            "business-key",
            "cmms.create-order",
        )
        .is_ok());
    }

    #[test]
    fn legacy_e0_entrypoint_cannot_smuggle_a_write_mode() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:legacy", "provider:a", "1");
        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "cmms.sandbox.write".to_string(),
            resource: "cmms://fixture".to_string(),
            site_ref: binding.site_ref.clone(),
            scope: vec![],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };
        assert!(
            begin_external_attempt(&permit, &binding, "business-key", "cmms.sandbox.write",)
                .is_err()
        );
    }

    #[test]
    fn permit_cannot_be_replayed_for_a_different_action() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::SandboxWrite,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "cmms.create-order".to_string(),
            resource: "cmms://fixture".to_string(),
            site_ref: binding.site_ref.clone(),
            scope: vec![],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };

        assert!(
            begin_external_attempt(&permit, &binding, "business-key", "cmms.delete-order").is_err()
        );
    }

    #[test]
    fn permit_must_match_exact_work_binding_and_profile() {
        let mut spec = WorkSpec::new(
            WorkPackageId::generate_with("work"),
            "test",
            "morn.factory.readonly@1.0.0",
        );
        spec.site_ref = Some("plant-a".to_string());
        let work = WorkResource::new(WorkspaceId::generate(), spec);
        let binding = ExecutionBinding::for_work(&work, "capability:a", "provider:a", "1");
        let permit = ExternalActionPermit {
            id: crate::ExternalActionPermitId::generate_with("permit"),
            profile_ref: binding.profile_ref.clone(),
            mode: crate::ExternalActionMode::Read,
            authority_decision_ref: "authz:test".to_string(),
            principal: "controller".to_string(),
            acting_for: None,
            action: "historian.read".to_string(),
            resource: "historian://fixture".to_string(),
            site_ref: Some("plant-a".to_string()),
            scope: vec!["maintenance-order:create".to_string()],
            parameter_envelope: BTreeMap::new(),
            work_ref: work.id.to_string(),
            binding_ref: binding.id.to_string(),
            not_before: None,
            expires_at: None,
            issued_at: Timestamp::now(),
        };
        let attempt =
            begin_external_attempt(&permit, &binding, "business-key", "historian.read").unwrap();
        assert_eq!(attempt.state, AttemptState::Authorized);
        assert_eq!(attempt.resource_ref.as_deref(), Some("historian://fixture"));
        assert_eq!(attempt.site_ref.as_deref(), Some("plant-a"));
        assert_eq!(
            attempt.external_action_permit_ref.as_deref(),
            Some(permit.id.as_str())
        );

        let mut wrong = permit.clone();
        wrong.binding_ref = "binding:other".to_string();
        assert!(
            begin_external_attempt(&wrong, &binding, "business-key", "historian.read").is_err()
        );
    }
}
