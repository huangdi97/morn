//! Accountability chain: who judged, delegated, verified, approved, executed,
//! retained responsibility, and when authority was terminated/revoked.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{AccountabilityId, WorkspaceId};
use morn_kernel::time::Timestamp;

/// Role a principal played in an accountability chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AccountabilityRole {
    Judged,
    Delegated,
    Verified,
    Approved,
    Executed,
    ResponsibilityRetained,
    AuthorityTerminated,
    AuthorityRevoked,
}

/// One link in the accountability chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountabilityEntry {
    pub id: AccountabilityId,
    pub workspace_id: WorkspaceId,
    pub subject: String,
    pub role: AccountabilityRole,
    pub principal: String,
    pub note: String,
    pub created_at: Timestamp,
}

/// A full accountability chain for a piece of work/decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccountabilityChain {
    pub entries: Vec<AccountabilityEntry>,
}

impl AccountabilityChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        workspace_id: WorkspaceId,
        subject: impl Into<String>,
        role: AccountabilityRole,
        principal: impl Into<String>,
        note: impl Into<String>,
    ) -> &mut Self {
        self.entries.push(AccountabilityEntry {
            id: AccountabilityId::generate_with("acc"),
            workspace_id,
            subject: subject.into(),
            role,
            principal: principal.into(),
            note: note.into(),
            created_at: Timestamp::now(),
        });
        self
    }

    /// The final responsibility always stays with the original accountable principal.
    pub fn retained_by(&self) -> Vec<&AccountabilityEntry> {
        self.entries
            .iter()
            .filter(|e| e.role == AccountabilityRole::ResponsibilityRetained)
            .collect()
    }

    pub fn terminated(&self) -> Vec<&AccountabilityEntry> {
        self.entries
            .iter()
            .filter(|e| {
                matches!(
                    e.role,
                    AccountabilityRole::AuthorityTerminated | AccountabilityRole::AuthorityRevoked
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accountability_chain_tracks_full_path_and_retained_responsibility() {
        let ws = WorkspaceId::generate();
        let mut chain = AccountabilityChain::new();
        chain
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::Judged,
                "pi",
                "judgment",
            )
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::Delegated,
                "analyst",
                "delegated analysis",
            )
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::Executed,
                "analyst",
                "ran analysis",
            )
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::Verified,
                "reviewer",
                "independent review",
            )
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::Approved,
                "pi",
                "approval",
            )
            .add(
                ws.clone(),
                "claim-release",
                AccountabilityRole::ResponsibilityRetained,
                "pi",
                "final responsibility",
            );
        assert_eq!(chain.retained_by().len(), 1);
        assert_eq!(chain.retained_by()[0].principal, "pi");
        assert_eq!(chain.entries.len(), 6);
    }

    #[test]
    fn terminated_authority_is_recorded() {
        let ws = WorkspaceId::generate();
        let mut chain = AccountabilityChain::new();
        chain
            .add(
                ws.clone(),
                "x",
                AccountabilityRole::Delegated,
                "analyst",
                "delegated",
            )
            .add(
                ws,
                "x",
                AccountabilityRole::AuthorityRevoked,
                "pi",
                "revoked after incident",
            );
        assert_eq!(chain.terminated().len(), 1);
    }
}
