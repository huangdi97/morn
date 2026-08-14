//! WorldService: the only gateway for canonical Operational World state changes.

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde_json::{json, Value};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    ActionId, GoalId, MetricId, ObjectId, ObjectTypeId, OutcomeRecordId,
    RelationTypeId, WorkspaceId,
};
use morn_kernel::ledger::{Ledger, LedgerEntry};

use crate::action::{Action, ActionStatus};
use crate::diff::StateDiff;
use crate::event::WorldEvent;
use crate::goal::{Goal, Metric};
use crate::object::{Object, ObjectType};
use crate::outcome::OutcomeRecord;
use crate::relation::{Relation, RelationType};
use crate::state::StateSnapshot;

/// Result of a governed state commit.
#[derive(Debug, Clone)]
pub struct StateCommit {
    pub snapshot: StateSnapshot,
    pub diffs: Vec<StateDiff>,
    pub ledger_entry: LedgerEntry,
}

/// In-memory Operational World service. Repository adapters persist externally.
#[derive(Debug, Default)]
pub struct WorldService {
    object_types: HashMap<ObjectTypeId, ObjectType>,
    objects: HashMap<ObjectId, Object>,
    relation_types: HashMap<RelationTypeId, RelationType>,
    relations: Vec<Relation>,
    snapshots: Vec<StateSnapshot>,
    diffs: Vec<StateDiff>,
    events: Vec<WorldEvent>,
    goals: HashMap<GoalId, Goal>,
    metrics: HashMap<MetricId, Metric>,
    outcomes: Vec<OutcomeRecord>,
    ledger: Ledger,
}

impl WorldService {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- object types ----

    pub fn register_object_type(&mut self, object_type: ObjectType) {
        self.object_types.insert(object_type.id.clone(), object_type);
    }

    pub fn object_type(&self, id: &ObjectTypeId) -> Option<&ObjectType> {
        self.object_types.get(id)
    }

    pub fn object_types(&self) -> Vec<&ObjectType> {
        self.object_types.values().collect()
    }

    // ---- objects ----

    pub fn register_object(&mut self, object: Object) {
        self.objects.insert(object.id.clone(), object);
    }

    pub fn object(&self, id: &ObjectId) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn objects(&self) -> Vec<&Object> {
        self.objects.values().collect()
    }

    // ---- relations ----

    pub fn register_relation_type(&mut self, relation_type: RelationType) {
        self.relation_types.insert(relation_type.id.clone(), relation_type);
    }

    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    // ---- events ----

    /// Record a world event (a fact that happened).
    pub fn record_event(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[WorldEvent] {
        &self.events
    }

    // ---- goals / metrics ----

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.insert(goal.id.clone(), goal);
    }

    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.insert(metric.id.clone(), metric);
    }

    pub fn metrics(&self) -> Vec<&Metric> {
        self.metrics.values().collect()
    }

    // ---- outcomes ----

    pub fn record_outcome(&mut self, outcome: OutcomeRecord) {
        self.outcomes.push(outcome);
    }

    pub fn outcome(&self, id: &OutcomeRecordId) -> Option<&OutcomeRecord> {
        self.outcomes.iter().find(|o| o.id == *id)
    }

    pub fn outcomes(&self) -> &[OutcomeRecord] {
        &self.outcomes
    }

    // ---- governed state commit ----

    /// The only way to change canonical object state.
    ///
    /// Requires an already-authorized `Action`. Produces a new immutable
    /// `StateSnapshot`, field-level `StateDiff`s, a world event and a ledger entry.
    pub fn commit_state(
        &mut self,
        action: &Action,
        object_id: ObjectId,
        new_state: BTreeMap<String, Value>,
        reason: Option<String>,
        actor: &str,
    ) -> Result<StateCommit> {
        if action.status != ActionStatus::Authorized {
            return Err(Error::invalid_state(format!(
                "action {} is not authorized (status {:?})",
                action.id, action.status
            )));
        }
        let object = self
            .objects
            .get_mut(&object_id)
            .ok_or_else(|| Error::not_found(format!("object {object_id}")))?;

        let old_state = object.state().clone();
        let new_version = object.apply_commit(new_state.clone());
        let snapshot = StateSnapshot {
            id: morn_kernel::ids::StateSnapshotId::generate_with("snap"),
            object_id: object_id.clone(),
            workspace_id: object.workspace_id.clone(),
            state: new_state.clone(),
            version: new_version,
            created_by: actor.to_string(),
            created_at: morn_kernel::time::Timestamp::now(),
            reason,
            action_id: Some(action.id.clone()),
        };
        let diffs = diff_fields(
            &old_state,
            &new_state,
            object_id.clone(),
            object.workspace_id.clone(),
            snapshot.id.clone(),
            action.id.clone(),
        );
        let ledger_entry = self.ledger.append(
            object.workspace_id.clone(),
            "world.state.commit",
            object_id.to_string(),
            format!("state committed to version {new_version}"),
            actor.to_string(),
            Some(hash_state(&new_state)),
            vec![action.id.to_string(), snapshot.id.to_string()],
        )?;

        self.snapshots.push(snapshot.clone());
        self.diffs.extend(diffs.iter().cloned());
        self.events.push(WorldEvent {
            id: morn_kernel::ids::EventId::generate_with("evt"),
            event_type: "state.committed".to_string(),
            workspace_id: object.workspace_id.clone(),
            related_objects: vec![object_id.to_string()],
            principal: actor.to_string(),
            state_before: Some(old_state),
            state_after: Some(new_state),
            evidence_ref: Some(snapshot.id.to_string()),
            created_at: morn_kernel::time::Timestamp::now(),
        });

        Ok(StateCommit {
            snapshot,
            diffs,
            ledger_entry,
        })
    }

    pub fn snapshots_for(&self, object_id: &ObjectId) -> Vec<&StateSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.object_id == *object_id)
            .collect()
    }

    pub fn diffs(&self) -> &[StateDiff] {
        &self.diffs
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn ledger_entries(&self) -> &[LedgerEntry] {
        self.ledger.entries()
    }
}

fn diff_fields(
    before: &BTreeMap<String, Value>,
    after: &BTreeMap<String, Value>,
    object_id: ObjectId,
    workspace_id: WorkspaceId,
    snapshot_id: morn_kernel::ids::StateSnapshotId,
    action_id: ActionId,
) -> Vec<StateDiff> {
    let mut diffs = Vec::new();
    let mut keys: Vec<&String> = before.keys().chain(after.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let old_val = before.get(key).cloned();
        let new_val = after.get(key).cloned();
        if old_val != new_val {
            diffs.push(StateDiff {
                id: morn_kernel::ids::StateSnapshotId::generate_with("diff"),
                object_id: object_id.clone(),
                workspace_id: workspace_id.clone(),
                field: key.clone(),
                from: old_val,
                to: new_val,
                snapshot_id: snapshot_id.clone(),
                action_id: action_id.clone(),
                created_at: morn_kernel::time::Timestamp::now(),
            });
        }
    }
    diffs
}

fn hash_state(state: &BTreeMap<String, Value>) -> String {
    let bytes = serde_json::to_vec(state).unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hasher;
    hasher.write(&bytes);
    format!("{:016x}", hasher.finish())
}

/// Helper used by tests / domain services to build initial states.
pub fn state(entries: &[(&str, Value)]) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    for (k, v) in entries {
        map.insert((*k).to_string(), v.clone());
    }
    map
}

pub fn str_state(key: &str, value: &str) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    map.insert(key.to_string(), json!(value));
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionProposal;
    use morn_kernel::ids::ActionTypeId;
    use serde_json::json;

    #[test]
    fn canonical_state_requires_authorized_action() {
        let mut world = WorldService::new();
        let ws = WorkspaceId::generate();
        let obj_type = ObjectType::new("biolab.Sample", ws.clone(), vec!["qc_status".into()], vec!["ok".into(), "failed".into()], vec![]);
        world.register_object_type(obj_type.clone());
        let obj_id = ObjectId::generate_with("sample");
        world.register_object(Object::new(
            obj_id.clone(),
            obj_type.id.clone(),
            ws.clone(),
            str_state("qc_status", "ok"),
        ));

        let proposal = ActionProposal::new(
            ActionTypeId::generate(),
            obj_id.clone(),
            ws.clone(),
            BTreeMap::new(),
            "pipeline",
        );
        let unauthorized = Action::new(proposal.id.clone());
        let err = world.commit_state(
            &unauthorized,
            obj_id.clone(),
            str_state("qc_status", "failed"),
            None,
            "pipeline",
        );
        assert!(err.is_err(), "un-authorized action must not commit state");
    }

    #[test]
    fn commit_creates_snapshot_diff_and_ledger() {
        let mut world = WorldService::new();
        let ws = WorkspaceId::generate();
        let obj_type = ObjectType::new("biolab.Sample", ws.clone(), vec!["qc_status".into()], vec![], vec![]);
        world.register_object_type(obj_type.clone());
        let obj_id = ObjectId::generate_with("sample");
        world.register_object(Object::new(
            obj_id.clone(),
            obj_type.id.clone(),
            ws.clone(),
            str_state("qc_status", "ok"),
        ));

        let proposal = ActionProposal::new(
            ActionTypeId::generate(),
            obj_id.clone(),
            ws.clone(),
            BTreeMap::new(),
            "pipeline",
        );
        let mut action = Action::new(proposal.id.clone());
        action.status = ActionStatus::Authorized;

        let commit = world
            .commit_state(&action, obj_id.clone(), str_state("qc_status", "failed"), Some("qc failed".to_string()), "pipeline")
            .unwrap();
        assert_eq!(commit.snapshot.version, 2);
        assert_eq!(commit.diffs.len(), 1);
        assert_eq!(commit.diffs[0].field, "qc_status");
        assert_eq!(commit.diffs[0].to, Some(json!("failed")));
        assert_eq!(world.object(&obj_id).unwrap().state().get("qc_status"), Some(&json!("failed")));
        assert_eq!(world.snapshots_for(&obj_id).len(), 1);
        assert_eq!(world.ledger_entries().len(), 1);
    }
}





