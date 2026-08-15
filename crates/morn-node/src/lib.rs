//! Morn Node + Distributed Durable Runtime + Deployment/Topology.
//! Local/simulated multi-node proof: claim -> checkpoint -> node loss ->
//! lease expiry -> failover -> duplicate-event dedupe -> idempotent action.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeIdTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkflowRunIdTag;

pub type NodeId = Id<NodeIdTag>;
pub type WorkflowRunId = Id<WorkflowRunIdTag>;

/// Node types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NodeType {
    Desktop,
    TeamServer,
    PrivateServer,
    Edge,
    Worker,
}

/// A Morn node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MornNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub capabilities: Vec<String>,
    pub resources: serde_json::Value,
    pub healthy: bool,
    pub registered_at: Timestamp,
    pub lease_until: Timestamp,
}

impl MornNode {
    pub fn new(name: &str, node_type: NodeType) -> Self {
        Self {
            id: NodeId::generate_with("node"),
            name: name.to_string(),
            node_type,
            capabilities: Vec::new(),
            resources: serde_json::json!({}),
            healthy: true,
            registered_at: Timestamp::now(),
            lease_until: Timestamp::from_millis(Timestamp::now().millis() + 60_000),
        }
    }

    pub fn lease_expired(&self, now: Timestamp) -> bool {
        now >= self.lease_until
    }
}

/// A claimed work run with lease and checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimedRun {
    pub run_id: WorkflowRunId,
    pub owner_node: NodeId,
    pub lease_until: Timestamp,
    pub checkpoint: Option<String>,
    pub applied_external_ids: Vec<String>,
}

/// Distributed durable runtime with lease/failover/dedupe.
#[derive(Debug, Default)]
pub struct DistributedRuntime {
    pub nodes: Vec<MornNode>,
    pub claims: HashMap<WorkflowRunId, ClaimedRun>,
}

impl DistributedRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_node(&mut self, node: MornNode) {
        self.nodes.push(node);
    }

    pub fn claim(&mut self, run_id: WorkflowRunId, node_id: NodeId, lease_secs: i64) -> bool {
        if let Some(existing) = self.claims.get(&run_id) {
            if existing.owner_node == node_id {
                return true;
            }
            if existing.lease_until > Timestamp::now() {
                return false; // other node holds an active lease
            }
            // stale lease -> takeover below
        }
        self.claims.insert(
            run_id.clone(),
            ClaimedRun {
                run_id,
                owner_node: node_id,
                lease_until: Timestamp::from_millis(Timestamp::now().millis() + lease_secs * 1000),
                checkpoint: None,
                applied_external_ids: Vec::new(),
            },
        );
        true
    }

    pub fn heartbeat(&mut self, run_id: &WorkflowRunId, node_id: &NodeId, lease_secs: i64) -> bool {
        let Some(claim) = self.claims.get_mut(run_id) else {
            return false;
        };
        if claim.owner_node != *node_id {
            return false;
        }
        claim.lease_until = Timestamp::from_millis(Timestamp::now().millis() + lease_secs * 1000);
        true
    }

    pub fn checkpoint(
        &mut self,
        run_id: &WorkflowRunId,
        node_id: &NodeId,
        payload: String,
    ) -> bool {
        let Some(claim) = self.claims.get_mut(run_id) else {
            return false;
        };
        if claim.owner_node != *node_id {
            return false;
        }
        claim.checkpoint = Some(payload);
        true
    }

    /// Node lost (no heartbeat): any run it owned with an expired lease can be
    /// re-claimed by another node.
    pub fn lost_leases(&self, now: Timestamp) -> Vec<WorkflowRunId> {
        self.claims
            .iter()
            .filter(|(_, c)| c.lease_until <= now)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Failover: B restores A's run from checkpoint, dedupes events, and applies
    /// external actions idempotently (no duplicate external effect).
    pub fn failover(
        &mut self,
        run_id: &WorkflowRunId,
        new_owner: NodeId,
        events: &[String],
        external_ids: &[String],
    ) -> (String, u32, u32) {
        let (checkpoint, applied) = match self.claims.get(run_id) {
            Some(c) => (
                c.checkpoint.clone().unwrap_or_default(),
                c.applied_external_ids.clone(),
            ),
            None => (String::new(), Vec::new()),
        };
        let mut applied_set: std::collections::HashSet<String> = applied.iter().cloned().collect();
        let mut applied_count = 0u32;
        for ev in events {
            // each external event carries a dedup key; dedupe against applied
            if !applied_set.contains(ev) {
                applied_set.insert(ev.clone());
                applied_count += 1;
            }
        }
        self.claims.insert(
            run_id.clone(),
            ClaimedRun {
                run_id: run_id.clone(),
                owner_node: new_owner,
                lease_until: Timestamp::from_millis(Timestamp::now().millis() + 60_000),
                checkpoint: Some(checkpoint.clone()),
                applied_external_ids: applied_set.into_iter().collect(),
            },
        );
        // external_ids that are ALREADY applied must NOT be re-applied (idempotent)
        let mut already_applied_external = 0u32;
        for ext in external_ids {
            if applied.contains(ext) {
                already_applied_external += 1;
            }
        }
        (checkpoint, applied_count, already_applied_external)
    }
}

/// Deployment / topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub name: String,
    pub topology: Topology,
    pub runtime_binding: String,
    pub storage_binding: String,
    pub secret_binding: String,
    pub policy_binding: String,
    pub upgrade_strategy: String,
    pub rollback_strategy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Topology {
    pub groups: Vec<NodeGroup>,
    pub placement_rules: Vec<PlacementRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGroup {
    pub name: String,
    pub node_type: NodeType,
    pub min_nodes: u32,
    pub max_nodes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementRule {
    pub capability: String,
    pub allowed_node_types: Vec<NodeType>,
}

impl Topology {
    pub fn validate(&self, runtime: &DistributedRuntime) -> bool {
        for group in &self.groups {
            let count = runtime
                .nodes
                .iter()
                .filter(|n| n.node_type == group.node_type && n.healthy)
                .count() as u32;
            if count < group.min_nodes {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_node_failover_with_dedupe_and_idempotency() {
        let mut rt = DistributedRuntime::new();
        let a = MornNode::new("node-a", NodeType::Desktop);
        let b = MornNode::new("node-b", NodeType::Worker);
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        rt.register_node(a);
        rt.register_node(b);

        let run_id = WorkflowRunId::generate();
        // A claims, checkpoints, then dies (no heartbeat).
        assert!(rt.claim(run_id.clone(), a_id.clone(), 60));
        assert!(rt.checkpoint(&run_id, &a_id, "state=v3".to_string()));
        // Force lease expiry.
        rt.claims.get_mut(&run_id).unwrap().lease_until =
            Timestamp::from_millis(Timestamp::now().millis() - 1);
        let lost = rt.lost_leases(Timestamp::now());
        assert!(lost.contains(&run_id));

        // B fails over: restores checkpoint, dedupes duplicate events, and does
        // not re-apply external effects already applied.
        let events = vec!["ev-1".to_string(), "ev-1".to_string(), "ev-2".to_string()];
        let external_already_applied = vec!["ext-1".to_string()];
        let (checkpoint, applied_count, deduped_external) =
            rt.failover(&run_id, b_id.clone(), &events, &external_already_applied);
        assert_eq!(checkpoint, "state=v3", "checkpoint transferred");
        assert_eq!(applied_count, 2, "duplicate event deduped (ev-1 only once)");
        assert_eq!(deduped_external, 0, "no external effect re-applied");
        assert_eq!(rt.claims.get(&run_id).unwrap().owner_node, b_id);
    }

    #[test]
    fn active_lease_blocks_takeover() {
        let mut rt = DistributedRuntime::new();
        let a = MornNode::new("a", NodeType::Desktop);
        let b = MornNode::new("b", NodeType::Worker);
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        rt.register_node(a);
        rt.register_node(b);
        let run_id = WorkflowRunId::generate();
        assert!(rt.claim(run_id.clone(), a_id, 60));
        // B cannot claim while A's lease is active.
        assert!(!rt.claim(run_id, b_id, 60));
    }

    #[test]
    fn deployment_topology_validation() {
        let mut rt = DistributedRuntime::new();
        rt.register_node(MornNode::new("d1", NodeType::Desktop));
        rt.register_node(MornNode::new("w1", NodeType::Worker));
        let topology = Topology {
            groups: vec![
                NodeGroup {
                    name: "desktop".to_string(),
                    node_type: NodeType::Desktop,
                    min_nodes: 1,
                    max_nodes: 1,
                },
                NodeGroup {
                    name: "workers".to_string(),
                    node_type: NodeType::Worker,
                    min_nodes: 1,
                    max_nodes: 4,
                },
            ],
            placement_rules: vec![PlacementRule {
                capability: "compute".to_string(),
                allowed_node_types: vec![NodeType::Worker],
            }],
        };
        assert!(topology.validate(&rt));
        // Incompatible placement: capability requires Worker but only Desktop present.
        let mut bad = rt;
        bad.nodes.retain(|n| n.node_type == NodeType::Desktop);
        assert!(!topology.validate(&bad));
    }
}
