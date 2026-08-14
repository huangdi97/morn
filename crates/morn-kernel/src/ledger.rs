//! Append-only ledger for auditable events (proposals, approvals, actions, state commits, ...).

use serde::{Deserialize, Serialize};

use crate::ids::{LedgerEntryId, WorkspaceId};
use crate::time::Timestamp;

/// One immutable ledger entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: LedgerEntryId,
    pub workspace_id: WorkspaceId,
    pub seq: u64,
    pub event_type: String,
    pub subject: String,
    pub summary: String,
    pub principal: String,
    pub payload_hash: Option<String>,
    pub refs: Vec<String>,
    pub created_at: Timestamp,
}

/// In-memory append-only ledger. Repository adapters persist entries externally.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    entries: Vec<LedgerEntry>,
    next_seq: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        workspace_id: WorkspaceId,
        event_type: impl Into<String>,
        subject: impl Into<String>,
        summary: impl Into<String>,
        principal: impl Into<String>,
        payload_hash: Option<String>,
        refs: Vec<String>,
    ) -> crate::Result<LedgerEntry> {
        let entry = LedgerEntry {
            id: LedgerEntryId::generate_with("led"),
            workspace_id,
            seq: self.next_seq,
            event_type: event_type.into(),
            subject: subject.into(),
            summary: summary.into(),
            principal: principal.into(),
            payload_hash,
            refs,
            created_at: Timestamp::now(),
        };
        self.next_seq += 1;
        self.entries.push(entry.clone());
        Ok(entry)
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries_for_subject(&self, subject: &str) -> Vec<&LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| e.subject == subject)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_is_append_only_with_monotonic_seq() {
        let ws = WorkspaceId::generate();
        let mut ledger = Ledger::new();
        let e1 = ledger
            .append(
                ws.clone(),
                "action.executed",
                "obj-1",
                "state changed",
                "actor-1",
                None,
                vec![],
            )
            .unwrap();
        let e2 = ledger
            .append(
                ws.clone(),
                "artifact.released",
                "art-1",
                "released v2",
                "actor-1",
                Some("abc".into()),
                vec![],
            )
            .unwrap();
        assert_eq!(e1.seq, 0);
        assert_eq!(e2.seq, 1);
        assert_eq!(ledger.len(), 2);
        assert_eq!(ledger.entries_for_subject("obj-1").len(), 1);
        assert_eq!(
            ledger.entries_for_subject("art-1")[0]
                .payload_hash
                .as_deref(),
            Some("abc")
        );
    }
}
