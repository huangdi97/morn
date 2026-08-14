//! Shared status enums for Morn records.

use serde::{Deserialize, Serialize};

/// Lifecycle status shared by most Morn objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LifecycleStatus {
    Active,
    Suspended,
    Retired,
    Archived,
    Terminated,
}

/// Artifact lifecycle: Draft -> Submitted -> InReview -> ChangesRequested | Approved
/// -> Locked -> Superseded -> Archived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ArtifactStatus {
    Draft,
    Submitted,
    InReview,
    ChangesRequested,
    Approved,
    Locked,
    Superseded,
    Archived,
}

impl ArtifactStatus {
    pub fn can_transition_to(self, next: ArtifactStatus) -> bool {
        use ArtifactStatus::*;
        matches!(
            (self, next),
            (Draft, Submitted)
                | (Submitted, InReview)
                | (Submitted, ChangesRequested)
                | (InReview, Approved)
                | (InReview, ChangesRequested)
                | (ChangesRequested, Submitted)
                | (ChangesRequested, InReview)
                | (Approved, Locked)
                | (Locked, Superseded)
                | (Locked, Archived)
                | (Superseded, Archived)
                | (Draft, Archived)
        )
    }
}

/// Work delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkStatus {
    Proposed,
    Planned,
    InProgress,
    WaitingApproval,
    Blocked,
    Completed,
    Accepted,
    Rejected,
    Cancelled,
}

/// Harness/runtime operational status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RuntimeStatus {
    Unmounted,
    Mounted,
    Running,
    Paused,
    Failed,
    Terminated,
}

/// Approval request status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    ChangesRequested,
    Withdrawn,
}

/// Member types in a mixed organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MemberType {
    Human,
    Actor,
    DeterministicWorker,
    ExternalService,
    Device,
}
