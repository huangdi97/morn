//! Provider-neutral execution guarantee vocabulary.
//!
//! These are semantic requirements/evidence labels, not implementation names.
//! A provider may satisfy a guarantee with different mechanisms, but it must
//! supply evidence rather than relying on a topology label such as "remote".

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionGuarantee {
    FilesystemReadPolicy,
    FilesystemWritePolicy,
    NetworkEgressPolicy,
    ProcessBoundary,
    KernelBoundary,
    ResourceLimits,
    SecretIndirection,
    WorkloadIdentity,
    StatefulExecution,
    CheckpointResume,
    RuntimeAttestation,
}

impl ExecutionGuarantee {
    pub const fn key(self) -> &'static str {
        match self {
            Self::FilesystemReadPolicy => "filesystem-read-policy",
            Self::FilesystemWritePolicy => "filesystem-write-policy",
            Self::NetworkEgressPolicy => "network-egress-policy",
            Self::ProcessBoundary => "process-boundary",
            Self::KernelBoundary => "kernel-boundary",
            Self::ResourceLimits => "resource-limits",
            Self::SecretIndirection => "secret-indirection",
            Self::WorkloadIdentity => "workload-identity",
            Self::StatefulExecution => "stateful-execution",
            Self::CheckpointResume => "checkpoint-resume",
            Self::RuntimeAttestation => "runtime-attestation",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_keys_are_stable_and_provider_neutral() {
        assert_eq!(
            serde_json::to_string(&ExecutionGuarantee::NetworkEgressPolicy).unwrap(),
            ""network-egress-policy""
        );
        assert_eq!(
            ExecutionGuarantee::RuntimeAttestation.key(),
            "runtime-attestation"
        );
    }
}
