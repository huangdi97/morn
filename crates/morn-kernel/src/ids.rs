//! Strongly-typed domain IDs. Each domain concept gets its own `Id<T>` type.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use uuid::Uuid;

/// A strongly-typed identifier for domain object of kind `T`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id<T> {
    value: String,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            _marker: PhantomData,
        }
    }

    /// Generate a random v4 UUID identifier.
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4().to_string())
    }

    /// Generate a random identifier with a readable prefix, e.g. `wp-<uuid>`.
    pub fn generate_with(prefix: &str) -> Self {
        Self::new(format!("{prefix}-{}", Uuid::new_v4()))
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.value)
    }
}

impl<T> FromStr for Id<T> {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Id::new(s))
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.value)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Id::new(s))
    }
}

macro_rules! id_types {
    ($($name:ident),+ $(,)?) => {
        $(
            #[doc = concat!("Tag type for `", stringify!($name), "`.")]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $name;
        )+
    };
}

id_types!(
    WorkspaceTag,
    PrincipalTag,
    IdentityTag,
    PolicyTag,
    ApprovalRequestTag,
    LedgerEntryTag,
    LifecycleRecordTag,
    ObjectTypeTag,
    ObjectTag,
    RelationTypeTag,
    RelationTag,
    EventTag,
    StateSnapshotTag,
    ActionTypeTag,
    ActionProposalTag,
    ActionTag,
    GoalTag,
    MetricTag,
    OutcomeRecordTag,
    ArtifactTag,
    ArtifactVersionTag,
    ReviewTag,
    ArtifactApprovalTag,
    DecisionPackageTag,
    VerificationReportTag,
    CapabilityTag,
    CapabilityProviderTag,
    CapabilityConsumerTag,
    HarnessSpecTag,
    HarnessVersionTag,
    HarnessBindingTag,
    RuntimeBindingTag,
    ScopeTag,
    ExecutionEventTag,
    ExecutionReceiptTag,
    ActorTemplateTag,
    ActorInstanceTag,
    RepresentationContractTag,
    RoleSlotTag,
    MemberBindingTag,
    ResponsibilityBindingTag,
    DelegationTag,
    CommitmentTag,
    WorkcellTag,
    WorkPackageTag,
    WorkContractTag,
    AcceptanceSpecTag,
    OutcomeContractTag,
    CheckpointTag,
    AttentionItemTag,
    RecoveryRecordTag,
    EvolutionCandidateTag,
    EvolutionBranchTag,
    EvolutionEvaluationTag,
    PromotionDecisionTag,
    ShadowRunTag,
    SolutionManifestTag,
    DomainPackTag,
    HubAssetTag,
    DatasetTag,
    SessionIdTag,
    SampleTag,
    AnalysisRunTag,
    QCResultTag,
    ScientificClaimTag,
);

pub type WorkspaceId = Id<WorkspaceTag>;
pub type PrincipalId = Id<PrincipalTag>;
pub type IdentityId = Id<IdentityTag>;
pub type PolicyId = Id<PolicyTag>;
pub type ApprovalRequestId = Id<ApprovalRequestTag>;
pub type LedgerEntryId = Id<LedgerEntryTag>;
pub type LifecycleRecordId = Id<LifecycleRecordTag>;
pub type ObjectTypeId = Id<ObjectTypeTag>;
pub type ObjectId = Id<ObjectTag>;
pub type RelationTypeId = Id<RelationTypeTag>;
pub type RelationId = Id<RelationTag>;
pub type EventId = Id<EventTag>;
pub type StateSnapshotId = Id<StateSnapshotTag>;
pub type ActionTypeId = Id<ActionTypeTag>;
pub type ActionProposalId = Id<ActionProposalTag>;
pub type ActionId = Id<ActionTag>;
pub type GoalId = Id<GoalTag>;
pub type MetricId = Id<MetricTag>;
pub type OutcomeRecordId = Id<OutcomeRecordTag>;
pub type ArtifactId = Id<ArtifactTag>;
pub type ArtifactVersionId = Id<ArtifactVersionTag>;
pub type ReviewId = Id<ReviewTag>;
pub type ArtifactApprovalId = Id<ArtifactApprovalTag>;
pub type DecisionPackageId = Id<DecisionPackageTag>;
pub type VerificationReportId = Id<VerificationReportTag>;
pub type CapabilityId = Id<CapabilityTag>;
pub type CapabilityProviderId = Id<CapabilityProviderTag>;
pub type CapabilityConsumerId = Id<CapabilityConsumerTag>;
pub type HarnessSpecId = Id<HarnessSpecTag>;
pub type HarnessVersionId = Id<HarnessVersionTag>;
pub type HarnessBindingId = Id<HarnessBindingTag>;
pub type RuntimeBindingId = Id<RuntimeBindingTag>;
pub type ScopeId = Id<ScopeTag>;
pub type ExecutionEventId = Id<ExecutionEventTag>;
pub type ExecutionReceiptId = Id<ExecutionReceiptTag>;
pub type ActorTemplateId = Id<ActorTemplateTag>;
pub type ActorInstanceId = Id<ActorInstanceTag>;
pub type RepresentationContractId = Id<RepresentationContractTag>;
pub type RoleSlotId = Id<RoleSlotTag>;
pub type MemberBindingId = Id<MemberBindingTag>;
pub type ResponsibilityBindingId = Id<ResponsibilityBindingTag>;
pub type DelegationId = Id<DelegationTag>;
pub type CommitmentId = Id<CommitmentTag>;
pub type WorkcellId = Id<WorkcellTag>;
pub type WorkPackageId = Id<WorkPackageTag>;
pub type WorkContractId = Id<WorkContractTag>;
pub type AcceptanceSpecId = Id<AcceptanceSpecTag>;
pub type OutcomeContractId = Id<OutcomeContractTag>;
pub type CheckpointId = Id<CheckpointTag>;
pub type AttentionItemId = Id<AttentionItemTag>;
pub type RecoveryRecordId = Id<RecoveryRecordTag>;
pub type EvolutionCandidateId = Id<EvolutionCandidateTag>;
pub type EvolutionBranchId = Id<EvolutionBranchTag>;
pub type EvolutionEvaluationId = Id<EvolutionEvaluationTag>;
pub type PromotionDecisionId = Id<PromotionDecisionTag>;
pub type ShadowRunId = Id<ShadowRunTag>;
pub type SolutionManifestId = Id<SolutionManifestTag>;
pub type DomainPackId = Id<DomainPackTag>;
pub type HubAssetId = Id<HubAssetTag>;
pub type DatasetId = Id<DatasetTag>;
pub type SampleId = Id<SampleTag>;
pub type AnalysisRunId = Id<AnalysisRunTag>;
pub type QCResultId = Id<QCResultTag>;
pub type ScientificClaimId = Id<ScientificClaimTag>;

