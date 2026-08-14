//! Provenance graph (W3C PROV inspired): entities, activities, agents and relations.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::ArtifactVersionId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ProvRelation {
    WasGeneratedBy,
    Used,
    WasDerivedFrom,
    WasAttributedTo,
    WasAssociatedWith,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceEdge {
    pub from: String,
    pub to: String,
    pub relation: ProvRelation,
}

/// Lineage of artifact versions: which version derived from which.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProvenanceGraph {
    pub edges: Vec<ProvenanceEdge>,
}

impl ProvenanceGraph {
    pub fn add(&mut self, from: String, to: String, relation: ProvRelation) {
        self.edges.push(ProvenanceEdge { from, to, relation });
    }

    pub fn ancestors_of(&self, node: &ArtifactVersionId) -> Vec<&ProvenanceEdge> {
        self.edges
            .iter()
            .filter(|e| e.to == node.to_string())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}
