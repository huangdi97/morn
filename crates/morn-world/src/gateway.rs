//! ActionGateway: the public governed path into canonical World state.
//! Raw WorldService mutation stays crate-private; callers must pass this Policy/Approval/Effect gate.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::action::{Action, ActionProposal, ActionStatus};
use crate::service::WorldService;
use morn_capability::effect::{EffectClass, EffectContract};
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ActionId, ExecutionReceiptId, ObjectId, WorkspaceId};
use morn_kernel::ledger::Ledger;
use morn_kernel::policy::{Policy, PolicyDecision};
use morn_kernel::status::LifecycleStatus;

fn world_has_commit_evidence(
    world: &WorldService,
    workspace_ref: &str,
    object_ref: &str,
    action_ref: &str,
    snapshot_ref: &str,
    ledger_ref: &str,
) -> bool {
    let object_id = ObjectId::new(object_ref.to_string());
    let snapshot_matches = world.snapshots_for(&object_id).iter().any(|snapshot| {
        snapshot.id.to_string() == snapshot_ref
            && snapshot.workspace_id.as_str() == workspace_ref
            && snapshot
                .action_id
                .as_ref()
                .is_some_and(|action_id| action_id.as_str() == action_ref)
    });
    let ledger_matches = world.ledger_entries().iter().any(|entry| {
        entry.id.to_string() == ledger_ref
            && entry.workspace_id.as_str() == workspace_ref
            && entry.subject == object_ref
            && entry.refs.iter().any(|reference| reference == action_ref)
            && entry.refs.iter().any(|reference| reference == snapshot_ref)
    });
    snapshot_matches && ledger_matches
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
    action: Action,
    preview: ActionPreview,
}

impl AuthorizedAction {
    pub fn action_id(&self) -> &ActionId {
        &self.action.id
    }

    pub fn preview(&self) -> &ActionPreview {
        &self.preview
    }
}

/// Result of verifying an executed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    /// Canonical world commit was durably recorded. This is execution truth,
    /// not independent verification or accepted business outcome.
    pub committed: bool,
    pub workspace_ref: String,
    pub object_ref: String,
    pub action_ref: String,
    pub snapshot_ref: String,
    pub ledger_ref: String,
    pub details: String,
}

/// The governed Action Gateway.
#[derive(Debug, Clone)]
pub struct ActionGateway {
    policy: Policy,
    effect_contract: EffectContract,
    ledger: Ledger,
}

impl ActionGateway {
    pub fn new(policy: Policy, effect_contract: EffectContract) -> Self {
        Self {
            policy,
            effect_contract,
            ledger: Ledger::new(),
        }
    }

    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    pub fn effect_contract(&self) -> &EffectContract {
        &self.effect_contract
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
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
        if proposal.workspace_id != self.policy.workspace_id {
            return Err(Error::not_authorized(
                "proposal workspace does not match the governing policy workspace",
            ));
        }
        if self.policy.status != LifecycleStatus::Active {
            return Err(Error::not_authorized(
                "inactive policy cannot authorize canonical World actions",
            ));
        }
        let preview = self.preview(proposal, action_name);
        if let PolicyDecision::Deny(reason) = &preview.policy_decision {
            return Err(Error::not_authorized(format!(
                "policy denies {action_name}: {reason}"
            )));
        }

        if self.effect_contract.class.rank() > EffectClass::E1Transactional.rank() {
            return Err(Error::not_authorized(
                "legacy World ActionGateway is limited to E0/E1 canonical state transitions; E2/E3 effects require the v11.5 ExternalAction authority/permit/attempt path",
            ));
        }

        let mut action = Action::new(proposal.id.clone());
        action.status = ActionStatus::Authorized;
        let refs = vec![proposal.id.to_string(), action.id.to_string()];
        self.ledger.append(
            proposal.workspace_id.clone(),
            "action.authorized",
            proposal.target_object.to_string(),
            format!("{action_name} authorized by {}", proposal.proposed_by),
            proposal.proposed_by.clone(),
            None,
            refs,
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
        world: &mut WorldService,
    ) -> Result<ExecutionOutcome> {
        if authorized.action.status != ActionStatus::Authorized {
            return Err(Error::invalid_state("action is not authorized"));
        }
        if actor.trim().is_empty() {
            return Err(Error::validation(
                "canonical World execution requires an actor",
            ));
        }
        let object_ref = object_id.to_string();
        if object_ref != authorized.preview.target_object {
            return Err(Error::not_authorized(
                "authorized action is bound to a different target object",
            ));
        }
        let object = world
            .object(&object_id)
            .ok_or_else(|| Error::not_found(format!("object {object_id}")))?;
        if object.workspace_id != self.policy.workspace_id {
            return Err(Error::not_authorized(
                "target object workspace does not match the governing policy",
            ));
        }
        let commit = world.commit_state_authorized_internal(
            &authorized.action,
            object_id,
            new_state,
            reason,
            actor,
        )?;
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

        Ok(ExecutionOutcome {
            receipt_id: ExecutionReceiptId::generate_with("rcpt").to_string(),
            committed: true,
            workspace_ref: commit.snapshot.workspace_id.to_string(),
            object_ref,
            action_ref: authorized.action.id.to_string(),
            snapshot_ref: commit.snapshot.id.to_string(),
            ledger_ref: commit.ledger_entry.id.to_string(),
            details: format!(
                "state committed to version {} with {} field diffs; verification remains separate",
                commit.snapshot.version,
                commit.diffs.len()
            ),
        })
    }

    /// Verify an execution outcome against expectations.
    pub fn verify(&self, outcome: &ExecutionOutcome, world: &WorldService) -> Result<bool> {
        if !outcome.committed
            || outcome.workspace_ref.trim().is_empty()
            || outcome.object_ref.trim().is_empty()
            || outcome.action_ref.trim().is_empty()
            || outcome.snapshot_ref.trim().is_empty()
            || outcome.ledger_ref.trim().is_empty()
        {
            return Err(Error::validation(
                "execution outcome lacks exact durable commit identity",
            ));
        }
        Ok(world_has_commit_evidence(
            world,
            &outcome.workspace_ref,
            &outcome.object_ref,
            &outcome.action_ref,
            &outcome.snapshot_ref,
            &outcome.ledger_ref,
        ))
    }

    /// Record a recovery for a failed action.
    pub fn recover(
        &mut self,
        failure: &str,
        actor: &str,
        workspace_id: WorkspaceId,
    ) -> Result<ActionId> {
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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectType};
    use morn_kernel::ids::{ActionTypeId, WorkspaceId};
    use morn_kernel::policy::{Policy, PolicyRule};
    use serde_json::json;

    fn proposal(ws: WorkspaceId, target: ObjectId, by: &str) -> ActionProposal {
        ActionProposal::new(ActionTypeId::generate(), target, ws, BTreeMap::new(), by)
    }

    #[test]
    fn legacy_gateway_rejects_e2_and_e3_even_with_policy_allow() {
        let ws = WorkspaceId::generate();
        for contract in [
            EffectContract::e2("compensate"),
            EffectContract::e3("irreversible"),
        ] {
            let policy = Policy::new(
                ws.clone(),
                "strict",
                vec![PolicyRule::allow("write")],
            );
            let mut gateway = ActionGateway::new(policy, contract);
            let prop = proposal(
                ws.clone(),
                ObjectId::generate_with("target"),
                "principal://operator",
            );
            let err = gateway.authorize(&prop, "write").unwrap_err();
            assert!(
                err.to_string().contains("v11.5 ExternalAction"),
                "legacy high-risk effects must route through the canonical v11.5 authority path"
            );
        }
    }

    #[test]
    fn policy_workspace_cannot_authorize_foreign_workspace_proposal() {
        let policy_workspace = WorkspaceId::generate();
        let foreign_workspace = WorkspaceId::generate();
        let policy = Policy::new(
            policy_workspace,
            "workspace-bound",
            vec![PolicyRule::allow("update")],
        );
        let mut gateway = ActionGateway::new(policy, EffectContract::e1());
        let proposal = proposal(
            foreign_workspace,
            ObjectId::generate_with("foreign"),
            "principal://operator",
        );
        assert!(gateway.authorize(&proposal, "update").is_err());
    }

    #[test]
    fn authorization_for_one_target_cannot_be_replayed_to_another_object() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(ws.clone(), "strict", vec![PolicyRule::allow("update")]);
        let mut gateway = ActionGateway::new(policy, EffectContract::e1());
        let mut world = WorldService::new();
        let object_type =
            ObjectType::new("asset", ws.clone(), vec!["status".into()], vec![], vec![]);
        world.register_object_type(object_type.clone());
        let target_a = ObjectId::generate_with("asset-a");
        let target_b = ObjectId::generate_with("asset-b");
        for target in [target_a.clone(), target_b.clone()] {
            world.register_object(Object::new(target, object_type.id.clone(), ws.clone(), {
                let mut state = BTreeMap::new();
                state.insert("status".to_string(), json!("before"));
                state
            }));
        }

        let proposal = proposal(ws, target_a.clone(), "principal://operator");
        let authorized = gateway.authorize(&proposal, "update").unwrap();
        let mut next = BTreeMap::new();
        next.insert("status".to_string(), json!("after"));
        assert!(gateway
            .execute(
                &authorized,
                target_b.clone(),
                next,
                Some("wrong target replay".to_string()),
                "runtime://executor",
                &mut world,
            )
            .is_err());
        assert_eq!(
            world.object(&target_b).unwrap().state().get("status"),
            Some(&json!("before"))
        );
        assert!(world.snapshots_for(&target_b).is_empty());
    }

    #[test]
    fn commit_receipt_is_not_self_verification() {
        let ws = WorkspaceId::generate();
        let policy = Policy::new(ws, "p", vec![]);
        let gateway = ActionGateway::new(policy, EffectContract::e1());
        let world = WorldService::new();
        let committed_without_witness = ExecutionOutcome {
            receipt_id: "rcpt:test".to_string(),
            committed: true,
            workspace_ref: String::new(),
            object_ref: String::new(),
            action_ref: String::new(),
            snapshot_ref: String::new(),
            ledger_ref: String::new(),
            details: "commit claimed without witness".to_string(),
        };
        assert!(gateway.verify(&committed_without_witness, &world).is_err());

        let forged_commit = ExecutionOutcome {
            receipt_id: "rcpt:test".to_string(),
            committed: true,
            workspace_ref: "workspace:w".to_string(),
            object_ref: "object:o".to_string(),
            action_ref: "action:a".to_string(),
            snapshot_ref: "snapshot:s".to_string(),
            ledger_ref: "ledger:l".to_string(),
            details: "unbacked commit".to_string(),
        };
        assert!(!gateway.verify(&forged_commit, &world).unwrap());
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
        world.register_object(Object::new(
            target.clone(),
            obj_type.id.clone(),
            ws.clone(),
            {
                let mut m = BTreeMap::new();
                m.insert("s".to_string(), json!("a"));
                m
            },
        ));
        let prop = proposal(ws, target.clone(), "runtime");
        let unauthorized = Action::new(prop.id);
        assert!(world
            .commit_state_authorized_internal(
                &unauthorized,
                target,
                BTreeMap::new(),
                None,
                "runtime",
            )
            .is_err());
    }
}
