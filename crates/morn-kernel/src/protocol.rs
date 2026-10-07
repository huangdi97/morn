//! Versioned Morn semantic protocol metadata.
//!
//! Implementations are replaceable. Published contracts evolve by version, and
//! historical corrections are explicit mutations rather than silent rewrites.

use serde::{Deserialize, Serialize};

use crate::time::Timestamp;
use crate::version::Version;

/// Current design protocol carried by the v11.5 convergence branch.
pub const MORN_PROTOCOL_V11_5: Version = Version::new(11, 5, 0);

/// A published contract reference. The referenced payload is expected to remain
/// addressable by this version/digest even after a newer release exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedContractRef {
    pub kind: String,
    pub id: String,
    pub version: Version,
    pub digest: Option<String>,
}

impl PublishedContractRef {
    pub fn new(kind: impl Into<String>, id: impl Into<String>, version: Version) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            version,
            digest: None,
        }
    }
}

/// Non-destructive ways to change the interpretation or visibility of history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HistoryMutationKind {
    Supersede,
    Retract,
    Redact,
    Migrate,
}

/// Explicit record of a historical correction/migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryMutation {
    pub subject_ref: String,
    pub kind: HistoryMutationKind,
    pub reason: String,
    pub actor: String,
    pub replacement_ref: Option<String>,
    pub created_at: Timestamp,
}

impl HistoryMutation {
    pub fn new(
        subject_ref: impl Into<String>,
        kind: HistoryMutationKind,
        reason: impl Into<String>,
        actor: impl Into<String>,
    ) -> Self {
        Self {
            subject_ref: subject_ref.into(),
            kind,
            reason: reason.into(),
            actor: actor.into(),
            replacement_ref: None,
            created_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticInvariant {
    pub id: String,
    pub summary: String,
}

impl SemanticInvariant {
    pub fn required(id: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            summary: summary.into(),
        }
    }
}

/// Versioned semantic snapshot used by conformance and runtime manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSnapshot {
    pub protocol_version: Version,
    pub semantic_slots: Vec<String>,
    pub invariants: Vec<SemanticInvariant>,
}

impl ProtocolSnapshot {
    pub fn v11_5() -> Self {
        Self {
            protocol_version: MORN_PROTOCOL_V11_5,
            semantic_slots: [
                "Work",
                "Capability",
                "Authority",
                "ExecutionBinding",
                "Attempt",
                "Receipt",
                "Reconciliation",
                "Outcome",
                "Acceptance",
                "Provenance",
                "Profile",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            invariants: vec![
                SemanticInvariant::required(
                    "work-truth-independent-of-executor",
                    "Harness/session/workflow runtime state cannot replace canonical Work truth.",
                ),
                SemanticInvariant::required(
                    "active-binding-pinned",
                    "An active Attempt keeps the exact ExecutionBinding it started with.",
                ),
                SemanticInvariant::required(
                    "profile-effect-gate-before-dispatch",
                    "The exact Domain Profile must permit the external effect mode before dispatch.",
                ),
                SemanticInvariant::required(
                    "authority-before-restricted-effect",
                    "Restricted external side effects require an explicit authority decision before dispatch.",
                ),
                SemanticInvariant::required(
                    "proposal-execution-receipt-outcome-acceptance-distinct",
                    "Proposal, execution, receipt, observed outcome and acceptance are distinct facts.",
                ),
                SemanticInvariant::required(
                    "unknown-requires-reconciliation",
                    "Ambiguous external effects are not blindly retried; they enter reconciliation.",
                ),
                SemanticInvariant::required(
                    "compiler-not-qualification",
                    "Artifact compilation can only create a candidate capability.",
                ),
                SemanticInvariant::required(
                    "qualification-not-admission",
                    "Qualification alone never grants site/profile admission.",
                ),
                SemanticInvariant::required(
                    "profile-version-pinned",
                    "A running Work/Binding is interpreted under an explicit profile version.",
                ),
                SemanticInvariant::required(
                    "history-no-silent-rewrite",
                    "Historical correction is explicit supersession/retraction/redaction/migration, not silent overwrite.",
                ),
                SemanticInvariant::required(
                    "profile-guarantees-not-provider-names",
                    "Domain Profiles constrain guarantees and semantics rather than naming specific vendors.",
                ),
            ],
        }
    }

    pub fn requires(&self, slot: &str) -> bool {
        self.semantic_slots.iter().any(|item| item == slot)
    }

    pub fn requires_invariant(&self, invariant_id: &str) -> bool {
        self.invariants
            .iter()
            .any(|invariant| invariant.id == invariant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v11_5_snapshot_names_control_plane_semantics() {
        let snapshot = ProtocolSnapshot::v11_5();
        assert_eq!(snapshot.protocol_version, MORN_PROTOCOL_V11_5);
        assert!(snapshot.requires("Work"));
        assert!(snapshot.requires("ExecutionBinding"));
        assert!(snapshot.requires("Reconciliation"));
        assert!(snapshot.requires("Acceptance"));
        assert!(snapshot.requires_invariant("active-binding-pinned"));
        assert!(snapshot.requires_invariant("history-no-silent-rewrite"));
        assert!(snapshot
            .invariants
            .iter()
            .all(|invariant| !invariant.summary.trim().is_empty()));
    }

    #[test]
    fn history_change_is_explicit_not_in_place() {
        let old = PublishedContractRef::new("profile", "factory", Version::new(1, 0, 0));
        let mut change = HistoryMutation::new(
            "profile:factory@1.0.0",
            HistoryMutationKind::Supersede,
            "new safety requirement",
            "profile-owner",
        );
        change.replacement_ref = Some("profile:factory@1.1.0".to_string());
        assert_eq!(old.version, Version::new(1, 0, 0));
        assert_eq!(
            change.replacement_ref.as_deref(),
            Some("profile:factory@1.1.0")
        );
    }
}
