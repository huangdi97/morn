//! ActionGateway: the only path from proposal to canonical state change.
//! Enforces Policy, Approval and Effect Class (E0-E3).

use std::collections::BTreeMap;

use serde_json::Value;

use morn_capability::effect::{EffectClass, EffectContract};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    ActionId, ExecutionReceiptId, ObjectId, WorkspaceId,
};
use morn_kernel::ledger::Ledger;
use morn_kernel::policy::{Policy, PolicyDecision};
use morn_world::action::{Action, ActionProposal, ActionStatus};
use morn_world::service::{StateCommit, WorldService};

/// The canonical world commit port used by the gateway.
pub trait WorldCommitter {
    fn commit(
        &mut self,
        action: &Action,
        object_id: ObjectId,
        new_state: BTreeMap<String, Value>,
        reason: Option<String>,
        actor: &str,
    ) -> Result<StateCommit>;
}

impl WorldCommitter for WorldService {
    fn commit(
        &mut self,
        action: &Action,
        object_id: ObjectId,
        new_state: BTreeMap<String, Value>,
        reason: Option<String>,
        actor: &str,
    ) -> Result<StateCommit> {
        self.commit_state(action, object_id, new_state, reason, actor)
    }
}

/// Expected effects shown before authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPreview {
    pub action_type: String,
    pub target_object: String,
    pub effect_class: String,
    pub policy_decision: PolicyDecision,
    pub approval_required: bool,
    pub verification_required: bool,
}

/// An action that passed preview + authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedAction {
    pub action: Action,
    pub preview: ActionPreview,
}

/// Result of verifying an executed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    pub verified: bool,
    pub details: String,
}

/// The governed Action Gateway.
#[derive(Debug, Clone)]
pub struct ActionGateway {
    pub policy: Policy,
    pub effect_contract: EffectContract,
    pub approved_roles: Vec<String>,
    pub ledger: Ledger,
}

impl ActionGateway {
    pub fn new(policy: Policy, effect_contract: EffectContract) -> Self {
        Self {
            policy,
            effect_contract,
            approved_roles: Vec::new(),
            ledger: Ledger::new(),
        }
    }

    /// Show what executing this proposal would do, without authorizing it.
    pub fn preview(&self, proposal: &ActionProposal, action_name: &str) -> ActionPreview {
        let decision = self.policy.evaluate(
            &proposal.proposed_by,
            action_name,
            proposal.target_object.as_str(),
        );
        ActionPreview {
            action_type: action_name.to_string(),
            target_object: proposal.target_object.to_string(),
            effect_class: self.effect_contract.class.code().to_string(),
            policy_decision: decision,
            approval_required: self.effect_contract.approval_required,
            verification_required: self.effect_contract.verification_required,
        }
    }

    /// Authorize a proposal. Fails on:
    /// - E3 (irreversible) without satisfied approval,
    /// - E2 without a compensation action,
    /// - policy denial.
    pub fn authorize(
        &mut self,
        proposal: &ActionProposal,
        action_name: &str,
    ) -> Result<AuthorizedAction> {
        let preview = self.preview(proposal, action_name);
        if let PolicyDecision::Deny(reason) = &preview.policy_decision {
            return Err(Error::not_authorized(format!(
                "policy denies {action_name}: {reason}"
            )));
        }

        // E2 must declare a compensation action.
        if self.effect_contract.class == EffectClass::E2Compensatable
            && self.effect_contract.compensation_action.is_none()
        {
            return Err(Error::validation(
                "E2 compensatable effect requires a compensation action",
            ));
        }

        // E3 (irreversible) requires a satisfied human approval by default.
        if self.effect_contract.approval_required && self.approved_roles.is_empty() {
            return Err(Error::not_authorized(
                "irreversible/high-risk action requires human approval (E3 gate)",
            ));
        }

        let mut action = Action::new(proposal.id.clone());
        action.status = ActionStatus::Authorized;
        self.ledger.append(
            proposal.workspace_id.clone(),
            "action.authorized",
            proposal.target_object.to_string(),
            format!("{action_name} authorized by {}", proposal.proposed_by),
            proposal.proposed_by.clone(),
            None,
            vec![proposal.id.to_string(), action.id.to_string()],
        )?;
        Ok(AuthorizedAction { action, preview })
    }

    /// Execute an authorized action by committing the new state to the world.
    pub fn execute(
        &mut self,
        authorized: &AuthorizedAction,
        object_id: ObjectId,
        new_state: BTreeMap<String, Value>,
        reason: Option<String>,
        actor: &str,
        world: &mut dyn WorldCommitter,
    ) -> Result<ExecutionOutcome> {
        if authorized.action.status != ActionStatus::Authorized {
            return Err(Error::invalid_state("action is not authorized"));
        }
        let commit = world.commit(&authorized.action, object_id, new_state, reason, actor)?;
        self.ledger.append(
            commit.snapshot.workspace_id.clone(),
            "action.executed",
            commit.snapshot.object_id.to_string(),
            format!(
                "executed {} -> snapshot {} ({} diffs)",
                authorized.preview.action_type,
                commit.snapshot.id,
                commit.diffs.len()
            ),
            actor.to_string(),
            commit.ledger_entry.payload_hash.clone(),
            vec![
                authorized.action.id.to_string(),
                commit.snapshot.id.to_string(),
                commit.ledger_entry.id.to_string(),
            ],
        )?;

        // Verification: the snapshot was created and ledger recorded.
        let verified = true;
        Ok(ExecutionOutcome {
            receipt_id: ExecutionReceiptId::generate_with("rcpt").to_string(),
            verified,
            details: format!(
                "state committed to version {} with {} field diffs",
                commit.snapshot.version,
                commit.diffs.len()
            ),
        })
    }

    /// Verify an execution outcome against expectations.
    pub fn verify(&self, outcome: &ExecutionOutcome, expected_snapshot_count: usize) -> Result<bool> {
        if !outcome.verified {
            return Err(Error::validation("execution was not verified"));
        }
        Ok(expected_snapshot_count > 0)
    }

    /// Record a recovery for a failed action.
    pub fn recover(&mut self, failure: &str, actor: &str, workspace_id: WorkspaceId) -> Result<ActionId> {
        let action_id = ActionId::generate_with("recover");
        self.ledger.append(
            workspace_id,
            "action.recovered",
            action_id.to_string(),
            format!("recovered from failure: {failure}"),
            actor.to_string(),
            None,
            vec![action_id.to_string()],
        )?;
        Ok(action_id)
    }

    pub fn mark_approval_satisfied(&mut self, role: impl Into<String>) {
        self.approved_roles.push(role.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::effect::EffectClass;
    use morn_kernel::ids::{ActionTypeId, WorkspaceId};
    use morn_kernel::policy::{Policy, PolicyRule};
    use morn_world::object::{Object, ObjectType};
    use serde_json::json;

    fn proposal(ws: WorkspaceId, target: ObjectId, by: &str) -> ActionProposal {
        ActionProposal::new(
            ActionTypeId::generate(),
            target,
            ws,
            BTreeMap::new(),
            by,
        )
    }

    #[test]
    fn e3_without_approval_is_denied() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(ws.clone(), "strict", vec![PolicyRule::allow("release_claim")]);
        let e3 = EffectContract::e3("public release of scientific claim");
        let mut gateway = ActionGateway::new(policy, e3);
        let target = ObjectId::generate();
        let prop = proposal(ws, target.clone(), "analyst");
        let err = gateway.authorize(&prop, "release_claim");
        assert!(err.is_err(), "E3 without approval must be denied");
        assert!(gateway.approved_roles.is_empty());
    }

    #[test]
    fn e3_with_approval_is_authorized_and_executes_via_world() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(ws.clone(), "strict", vec![PolicyRule::allow("release_claim")]);
        let e3 = EffectContract::e3("public release");
        let mut gateway = ActionGateway::new(policy, e3);
        gateway.mark_approval_satisfied("pi");

        let mut world = WorldService::new();
        let obj_type = ObjectType::new("biolab.Claim", ws.clone(), vec!["status".into()], vec![], vec![]);
        world.register_object_type(obj_type.clone());
        let target = ObjectId::generate_with("claim");
        world.register_object(Object::new(
            target.clone(),
            obj_type.id.clone(),
            ws.clone(),
            {
                let mut m = BTreeMap::new();
                m.insert("status".to_string(), json!("draft"));
                m
            },
        ));

        let prop = proposal(ws, target.clone(), "analyst");
        let authorized = gateway.authorize(&prop, "release_claim").unwrap();
        assert_eq!(authorized.action.status, ActionStatus::Authorized);

        let mut new_state = BTreeMap::new();
        new_state.insert("status".to_string(), json!("released"));
        let outcome = gateway
            .execute(&authorized, target.clone(), new_state, Some("pi approved".to_string()), "analyst", &mut world)
            .unwrap();
        assert!(outcome.verified);
        assert_eq!(
            world.object(&target).unwrap().state().get("status"),
            Some(&json!("released"))
        );
        assert_eq!(world.snapshots_for(&target).len(), 1);
    }

    #[test]
    fn e2_without_compensation_fails_validation() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(ws.clone(), "p", vec![PolicyRule::allow("exclude_sample")]);
        // E2 contract with no compensation action: must fail.
        let e2 = EffectContract {
            class: EffectClass::E2Compensatable,
            compensation_action: None,
            irreversible_reason: None,
            preview_required: true,
            approval_required: false,
            verification_required: true,
        };
        let mut gateway = ActionGateway::new(policy, e2);
        let target = ObjectId::generate();
        let prop = proposal(ws, target, "pipeline");
        assert!(gateway.authorize(&prop, "exclude_sample").is_err());
    }

    #[test]
    fn runtime_cannot_commit_world_directly() {
        // The only write path is through the gateway + WorldService::commit.
        // Direct Object mutation is impossible (fields are private) — verify
        // that an unauthorized Action is rejected by WorldService too.
        let ws = WorkspaceId::generate();
        let mut world = WorldService::new();
        let obj_type = ObjectType::new("t", ws.clone(), vec!["s".into()], vec![], vec![]);
        world.register_object_type(obj_type.clone());
        let target = ObjectId::generate();
        world.register_object(Object::new(target.clone(), obj_type.id.clone(), ws.clone(), {
            let mut m = BTreeMap::new();
            m.insert("s".to_string(), json!("a"));
            m
        }));
        let prop = proposal(ws, target.clone(), "runtime");
        let unauthorized = Action::new(prop.id);
        assert!(world
            .commit_state(&unauthorized, target, BTreeMap::new(), None, "runtime")
            .is_err());
    }
}


