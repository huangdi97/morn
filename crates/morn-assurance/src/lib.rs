//! Morn Assurance: replay, simulation, evaluation and shadow.

pub mod admission;
pub mod certification;
pub mod customer_value;
pub mod evaluation;

pub mod evidence_class;
pub mod managed_work;
pub mod profile_conformance;
pub mod replacement;
pub mod release_readiness;
pub mod replay;
pub mod rollback;
pub mod shadow;
pub mod simulation;

pub use admission::{
    AdmissionService, CapabilityDistributionRelease, CapabilityDistributionReleaseId,
    CapabilityDistributionReleaseStatus, CapabilityLifecycleEvent, CapabilityLifecycleEventId,
    CapabilityObservation, CapabilityObservationId, QualificationEvidence, QualificationRecord,
    QualificationRecordId, QualificationStatus, SiteAdmission, SiteAdmissionId,
    SiteAdmissionStatus, StrictQualificationRequest, VerifiedReleaseRequest,
};
pub use customer_value::{CustomerValueAttestation, CustomerValueMetric};
pub use evidence_class::{
    reference_evidence_ledger, EvidenceClaim, EvidenceClaimId, EvidenceClaimState, EvidenceClass,
    EvidenceLedger,
};
pub use release_readiness::{
    release_readiness_axes, ReadinessState, ReleaseReadinessAxis, RuntimeReadinessEvidence,
};

pub use replay::{ReplayReport, ReplayRunner, ReplayScenario};
pub use shadow::{ShadowComparison, ShadowRun, ShadowRunner};
pub use simulation::{FaultInjection, FaultKind, SimulationScenario};

pub use profile_conformance::{ProfileConformanceAttestation, ProfileConformanceAttestationId};
