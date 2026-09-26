//! principal — Principal / Actor / Role / Commitment (Workforce semantics).
//!
//! Role is an organizational and UX interface, not a permanent agent session:
//! the same role may invoke different Harnesses/Agents/Solvers/Humans in
//! different Works, and may involve zero agents.

use serde::{Deserialize, Serialize};

/// A Principal is the accountable party (person or system) that owns actions
/// and commitments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Principal {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

impl Principal {
    pub fn new(id: &str, display_name: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            kind: None,
        }
    }
}

/// An Actor is anything that can execute: agent, human, solver, rule, model,
/// connector, robot. Agent is one kind of executor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// executor kind, e.g. agent / human / solver / rule / connector
    pub kind: String,
}

impl Actor {
    pub fn new(id: &str, kind: &str) -> Self {
        Self {
            id: id.to_string(),
            principal_id: None,
            kind: kind.to_string(),
        }
    }
}

/// A Role is an organizational interface (e.g. production owner, quality
/// reviewer, planner). Roles are decoupled from agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actor_kinds: Vec<String>,
}

impl Role {
    pub fn new(id: &str, display_name: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            actor_kinds: Vec::new(),
        }
    }
}

/// A Commitment binds an owner to delivering a Work under stated constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commitment {
    pub id: String,
    pub work_ref: String,
    pub principal_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Commitment {
    pub fn new(id: &str, work_ref: &str, principal_id: &str) -> Self {
        Self {
            id: id.to_string(),
            work_ref: work_ref.to_string(),
            principal_id: principal_id.to_string(),
            due_at: None,
            active: true,
            created_at: Some(crate::work::now_iso()),
        }
    }
}
