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
    ValueAssessmentTag,
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
    AccountabilityTag,
    DecisionPolicyAssetTag,
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
    CertificationSpecTag,
    CertificationRunTag,
    CertificationDecisionTag,
    CertifiedWorkCapabilityTag,
    CapabilityReleaseTag,
    ManagedWorkRunTag,
    DeliveryReceiptTag,
    AcceptanceDecisionTag,
    ExistingSystemMappingTag,
    ReplacementRecordTag,
    ReplacementDecisionTag,
    PartialReplaceCandidateTag,
    ReplacementComparisonTag,
    FlywheelPatternTag,
    FlywheelCandidateTag,
    DistillationCandidateTag,
    DistilledProgramTag,
    HumanCorrectionTag,
    TraceRecordTag,
    OperationalEpisodeTag,
    EpisodeDatasetTag,
    DatasetSnapshotTag,
    LabelDefinitionTag,
    FeatureSchemaTag,
    SplitManifestTag,
    DataQualityReportTag,
    PredictorSpecTag,
    PredictorVersionTag,
    TrainingRunTag,
    PredictionTag,
    PredictionEvidenceTag,
    CalibrationReportTag,
    ModelDriftReportTag,
    ContextOfUseTag,
    StateEncoderVersionTag,
    PilotManifestTag,
    MonitoringReportTag,
    RollbackRequestTag,
    RollbackReceiptTag,
    WorkflowDefinitionTag,
    WorkflowRunTag,
    WorkflowStepTag,
    SignalTag,
    TimerWaitTag,
    RetryPolicyTag,
    CompensationPlanTag,
    EscalationTag,
    BudgetGuardTag,
    DriftRecordTag,
    ProblemSpecTag,
    WorkGraphTag,
    WorkNodeTag,
    WorkEdgeTag,
    CapabilityRequirementTag,
    MemberTypePlanTag,
    HarnessPlanTag,
    RuntimeProfileTag,
    EvaluationPlanTag,
    ProposedSolutionTag,
    ValidationReportTag,
    ApprovedSolutionTag,
    SolutionPackageTag,
    SolutionVersionTag,
    ReplayScenarioTag,
    ReplayRunTag,
    ReplayReportTag,
    SimulationScenarioTag,
    SimulationRunTag,
    FaultInjectionTag,
    EvaluationSuiteTag,
    EvaluationRunTag,
    EvaluationResultTag,
    ShadowProfileTag,
    ShadowComparisonTag,
    SessionIdTag,
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
pub type ValueAssessmentId = Id<ValueAssessmentTag>;
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
pub type AccountabilityId = Id<AccountabilityTag>;
pub type DecisionPolicyAssetId = Id<DecisionPolicyAssetTag>;
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
pub type CertificationSpecId = Id<CertificationSpecTag>;
pub type CertificationRunId = Id<CertificationRunTag>;
pub type CertificationDecisionId = Id<CertificationDecisionTag>;
pub type CertifiedWorkCapabilityId = Id<CertifiedWorkCapabilityTag>;
pub type CapabilityReleaseId = Id<CapabilityReleaseTag>;
pub type ManagedWorkRunId = Id<ManagedWorkRunTag>;
pub type DeliveryReceiptId = Id<DeliveryReceiptTag>;
pub type AcceptanceDecisionId = Id<AcceptanceDecisionTag>;
pub type ExistingSystemMappingId = Id<ExistingSystemMappingTag>;
pub type ReplacementRecordId = Id<ReplacementRecordTag>;
pub type ReplacementDecisionId = Id<ReplacementDecisionTag>;
pub type PartialReplaceCandidateId = Id<PartialReplaceCandidateTag>;
pub type ReplacementComparisonId = Id<ReplacementComparisonTag>;
pub type FlywheelPatternId = Id<FlywheelPatternTag>;
pub type FlywheelCandidateId = Id<FlywheelCandidateTag>;
pub type DistillationCandidateId = Id<DistillationCandidateTag>;
pub type DistilledProgramId = Id<DistilledProgramTag>;
pub type HumanCorrectionId = Id<HumanCorrectionTag>;
pub type TraceRecordId = Id<TraceRecordTag>;
pub type OperationalEpisodeId = Id<OperationalEpisodeTag>;
pub type EpisodeDatasetId = Id<EpisodeDatasetTag>;
pub type DatasetSnapshotId = Id<DatasetSnapshotTag>;
pub type LabelDefinitionId = Id<LabelDefinitionTag>;
pub type FeatureSchemaId = Id<FeatureSchemaTag>;
pub type SplitManifestId = Id<SplitManifestTag>;
pub type DataQualityReportId = Id<DataQualityReportTag>;
pub type PredictorSpecId = Id<PredictorSpecTag>;
pub type PredictorVersionId = Id<PredictorVersionTag>;
pub type TrainingRunId = Id<TrainingRunTag>;
pub type PredictionId = Id<PredictionTag>;
pub type PredictionEvidenceId = Id<PredictionEvidenceTag>;
pub type CalibrationReportId = Id<CalibrationReportTag>;
pub type ModelDriftReportId = Id<ModelDriftReportTag>;
pub type ContextOfUseId = Id<ContextOfUseTag>;
pub type StateEncoderVersionId = Id<StateEncoderVersionTag>;
pub type PilotManifestId = Id<PilotManifestTag>;
pub type MonitoringReportId = Id<MonitoringReportTag>;
pub type RollbackRequestId = Id<RollbackRequestTag>;
pub type RollbackReceiptId = Id<RollbackReceiptTag>;
pub type WorkflowDefinitionId = Id<WorkflowDefinitionTag>;
pub type WorkflowRunId = Id<WorkflowRunTag>;
pub type WorkflowStepId = Id<WorkflowStepTag>;
pub type SignalId = Id<SignalTag>;
pub type TimerWaitId = Id<TimerWaitTag>;
pub type RetryPolicyId = Id<RetryPolicyTag>;
pub type CompensationPlanId = Id<CompensationPlanTag>;
pub type EscalationId = Id<EscalationTag>;
pub type BudgetGuardId = Id<BudgetGuardTag>;
pub type DriftRecordId = Id<DriftRecordTag>;
pub type ProblemSpecId = Id<ProblemSpecTag>;
pub type WorkGraphId = Id<WorkGraphTag>;
pub type WorkNodeId = Id<WorkNodeTag>;
pub type WorkEdgeId = Id<WorkEdgeTag>;
pub type CapabilityRequirementId = Id<CapabilityRequirementTag>;
pub type MemberTypePlanId = Id<MemberTypePlanTag>;
pub type HarnessPlanId = Id<HarnessPlanTag>;
pub type RuntimeProfileId = Id<RuntimeProfileTag>;
pub type EvaluationPlanId = Id<EvaluationPlanTag>;
pub type ProposedSolutionId = Id<ProposedSolutionTag>;
pub type ValidationReportId = Id<ValidationReportTag>;
pub type ApprovedSolutionId = Id<ApprovedSolutionTag>;
pub type SolutionPackageId = Id<SolutionPackageTag>;
pub type SolutionVersionId = Id<SolutionVersionTag>;
pub type ReplayScenarioId = Id<ReplayScenarioTag>;
pub type ReplayRunId = Id<ReplayRunTag>;
pub type ReplayReportId = Id<ReplayReportTag>;
pub type SimulationScenarioId = Id<SimulationScenarioTag>;
pub type SimulationRunId = Id<SimulationRunTag>;
pub type FaultInjectionId = Id<FaultInjectionTag>;
pub type EvaluationSuiteId = Id<EvaluationSuiteTag>;
pub type EvaluationRunId = Id<EvaluationRunTag>;
pub type EvaluationResultId = Id<EvaluationResultTag>;
pub type ShadowProfileId = Id<ShadowProfileTag>;
pub type ShadowComparisonId = Id<ShadowComparisonTag>;
