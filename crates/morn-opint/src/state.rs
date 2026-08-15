//! Typed operational state representation (deterministic, interpretable).
//! No large-model embeddings: categorical/numeric/boolean/temporal/graph/
//! aggregate/missing features.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::StateEncoderVersionId;
use morn_kernel::version::Version;

/// World state: objects/counts/statuses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldState {
    pub object_type_counts: Vec<(String, u64)>,
    pub status_counts: Vec<(String, u64)>,
}

/// Work state: packages/contracts/acceptance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkState {
    pub work_package_count: u64,
    pub open_attention_count: u64,
    pub waiting_approval_count: u64,
    pub blocked_count: u64,
}

/// Organization state: workcell/members.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OrganizationState {
    pub workcell_count: u64,
    pub role_slot_count: u64,
    pub delegation_count: u64,
    pub representation_count: u64,
}

/// Execution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExecutionState {
    pub harnesses_mounted: u64,
    pub active_runs: u64,
    pub retries: u64,
    pub compensations: u64,
    pub escalations: u64,
}

/// Resource state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourceState {
    pub cost_estimate: f64,
    pub human_minutes: f64,
    pub latency_ms: u64,
}

/// Evidence state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EvidenceState {
    pub artifact_versions: u64,
    pub decisions: u64,
    pub approvals: u64,
    pub provenance_refs: u64,
}

/// A feature vector (typed, ordered).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureVector {
    pub features: Vec<(String, f64)>,
    pub missing: Vec<String>,
}

/// Typed state encoder: aggregates the six states into a feature vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEncoder {
    pub version: StateEncoderVersionId,
    pub schema_version: Version,
}

impl StateEncoder {
    pub fn new() -> Self {
        Self {
            version: StateEncoderVersionId::generate_with("enc"),
            schema_version: Version::v1(),
        }
    }

    #[allow(clippy::vec_init_then_push)]
    pub fn encode(
        &self,
        world: &WorldState,
        work: &WorkState,
        org: &OrganizationState,
        exec: &ExecutionState,
        res: &ResourceState,
        evidence: &EvidenceState,
    ) -> FeatureVector {
        let mut features = Vec::new();
        features.push((
            "world.object_type_count".to_string(),
            world.object_type_counts.len() as f64,
        ));
        features.push((
            "world.status_count".to_string(),
            world.status_counts.len() as f64,
        ));
        features.push((
            "work.package_count".to_string(),
            work.work_package_count as f64,
        ));
        features.push((
            "work.open_attention".to_string(),
            work.open_attention_count as f64,
        ));
        features.push((
            "work.waiting_approval".to_string(),
            work.waiting_approval_count as f64,
        ));
        features.push(("work.blocked".to_string(), work.blocked_count as f64));
        features.push(("org.workcell_count".to_string(), org.workcell_count as f64));
        features.push((
            "org.role_slot_count".to_string(),
            org.role_slot_count as f64,
        ));
        features.push((
            "org.delegation_count".to_string(),
            org.delegation_count as f64,
        ));
        features.push((
            "org.representation_count".to_string(),
            org.representation_count as f64,
        ));
        features.push((
            "exec.harnesses_mounted".to_string(),
            exec.harnesses_mounted as f64,
        ));
        features.push(("exec.active_runs".to_string(), exec.active_runs as f64));
        features.push(("exec.retries".to_string(), exec.retries as f64));
        features.push(("exec.compensations".to_string(), exec.compensations as f64));
        features.push(("exec.escalations".to_string(), exec.escalations as f64));
        features.push(("res.cost".to_string(), res.cost_estimate));
        features.push(("res.human_minutes".to_string(), res.human_minutes));
        features.push(("res.latency_ms".to_string(), res.latency_ms as f64));
        features.push((
            "evidence.artifact_versions".to_string(),
            evidence.artifact_versions as f64,
        ));
        features.push(("evidence.decisions".to_string(), evidence.decisions as f64));
        features.push(("evidence.approvals".to_string(), evidence.approvals as f64));
        features.push((
            "evidence.provenance_refs".to_string(),
            evidence.provenance_refs as f64,
        ));
        FeatureVector {
            features,
            missing: Vec::new(),
        }
    }
}

impl Default for StateEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_produces_typed_features() {
        let encoder = StateEncoder::new();
        let mut world = WorldState::default();
        world
            .object_type_counts
            .push(("biolab.Sample".to_string(), 3));
        let work = WorkState {
            work_package_count: 1,
            waiting_approval_count: 1,
            ..Default::default()
        };
        let fv = encoder.encode(
            &world,
            &work,
            &OrganizationState::default(),
            &ExecutionState::default(),
            &ResourceState::default(),
            &EvidenceState::default(),
        );
        assert_eq!(fv.features.len(), 22);
        assert_eq!(fv.features[0], ("world.object_type_count".to_string(), 1.0));
        assert!(fv
            .features
            .iter()
            .any(|(k, v)| k == "work.waiting_approval" && *v == 1.0));
    }
}
