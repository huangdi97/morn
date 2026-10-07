//! Provider-neutral execution guarantee vocabulary.
//!
//! These are semantic requirements/evidence labels, not implementation names.
//! A provider may satisfy a guarantee with different mechanisms, but it must
//! supply evidence rather than relying on a topology label such as "remote".

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionClass {
    NoIsolation,
    Process,
    Container,
    MicroVm,
    FullVm,
    Remote,
    Physical,
}

impl ExecutionClass {
    pub const fn key(self) -> &'static str {
        match self {
            Self::NoIsolation => "no-isolation",
            Self::Process => "process",
            Self::Container => "container",
            Self::MicroVm => "microvm",
            Self::FullVm => "full-vm",
            Self::Remote => "remote",
            Self::Physical => "physical",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "none" | "noisolation" | "no-isolation" => Some(Self::NoIsolation),
            "process" => Some(Self::Process),
            "container" => Some(Self::Container),
            "microvm" | "micro_vm" | "micro-vm" => Some(Self::MicroVm),
            "fullvm" | "full_vm" | "full-vm" | "vm" => Some(Self::FullVm),
            "remote" => Some(Self::Remote),
            "physical" => Some(Self::Physical),
            _ => None,
        }
    }

    /// Containment compatibility is intentionally partial. Remote and physical
    /// are topology/executor classes, not higher points on a security ladder.
    pub const fn satisfies(self, required: Self) -> bool {
        use ExecutionClass::*;
        match required {
            NoIsolation => true,
            Process => matches!(self, Process | Container | MicroVm | FullVm),
            Container => matches!(self, Container | MicroVm | FullVm),
            MicroVm => matches!(self, MicroVm | FullVm),
            FullVm => matches!(self, FullVm),
            Remote => matches!(self, Remote),
            Physical => matches!(self, Physical),
        }
    }
}

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
    fn execution_class_is_partial_not_ordinal() {
        assert!(ExecutionClass::MicroVm.satisfies(ExecutionClass::Container));
        assert!(!ExecutionClass::Remote.satisfies(ExecutionClass::Container));
        assert_eq!(
            ExecutionClass::parse("micro-vm"),
            Some(ExecutionClass::MicroVm)
        );
        assert_eq!(ExecutionClass::Remote.key(), "remote");
    }

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
