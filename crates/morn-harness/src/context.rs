//! RuntimeContext: the minimal, temporary, auditable context handed to a harness.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ActorInstanceId, WorkPackageId, WorkspaceId};
use morn_kernel::{ExecutionClass, ExecutionGuarantee};

/// The context a harness receives for one execution. Morn retains provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub workspace_id: WorkspaceId,
    pub actor_id: ActorInstanceId,
    pub work_package_id: WorkPackageId,
    pub policy_snapshot_version: Option<String>,
    pub provenance_refs: Vec<String>,
    pub scope_id: Option<String>,
    #[serde(default)]
    pub execution_environment_ref: Option<String>,
    #[serde(default)]
    pub execution_class: Option<ExecutionClass>,
    #[serde(default)]
    pub execution_guarantees: Vec<ExecutionGuarantee>,
}

impl RuntimeContext {
    pub fn new(
        workspace_id: WorkspaceId,
        actor_id: ActorInstanceId,
        work_package_id: WorkPackageId,
    ) -> Self {
        Self {
            workspace_id,
            actor_id,
            work_package_id,
            policy_snapshot_version: None,
            provenance_refs: Vec::new(),
            scope_id: None,
            execution_environment_ref: None,
            execution_class: None,
            execution_guarantees: Vec::new(),
        }
    }

    pub fn with_scope_id(mut self, scope_id: impl Into<String>) -> Result<Self, String> {
        let scope_id = scope_id.into();
        if scope_id.trim().is_empty() {
            return Err("runtime scope id must be non-empty".to_string());
        }
        self.scope_id = Some(scope_id);
        Ok(self)
    }

    pub fn with_execution_environment(
        mut self,
        environment_ref: impl Into<String>,
        class: ExecutionClass,
        guarantees: Vec<ExecutionGuarantee>,
    ) -> Result<Self, String> {
        let environment_ref = environment_ref.into();
        if environment_ref.trim().is_empty() {
            return Err("execution environment reference must be non-empty".to_string());
        }
        self.execution_environment_ref = Some(environment_ref);
        self.execution_class = Some(class);
        self.execution_guarantees = guarantees;
        Ok(self)
    }

    pub fn proves_environment(
        &self,
        minimum_class: ExecutionClass,
        required: &[ExecutionGuarantee],
    ) -> bool {
        self.execution_environment_ref
            .as_deref()
            .is_some_and(|reference| !reference.trim().is_empty())
            && self
                .execution_class
                .is_some_and(|class| class.satisfies(minimum_class))
            && required
                .iter()
                .all(|guarantee| self.execution_guarantees.contains(guarantee))
    }

    /// DSH/Pi real adapters can expose local tools without a Morn permission
    /// callback, so they require a containment boundary before startup.
    pub fn proves_real_harness_environment(&self) -> bool {
        self.proves_environment(
            ExecutionClass::Container,
            &[
                ExecutionGuarantee::FilesystemReadPolicy,
                ExecutionGuarantee::FilesystemWritePolicy,
                ExecutionGuarantee::ProcessBoundary,
                ExecutionGuarantee::ResourceLimits,
                ExecutionGuarantee::NetworkEgressPolicy,
                ExecutionGuarantee::SecretIndirection,
                ExecutionGuarantee::RuntimeAttestation,
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> RuntimeContext {
        RuntimeContext::new(
            WorkspaceId::generate(),
            ActorInstanceId::generate(),
            WorkPackageId::generate(),
        )
    }

    #[test]
    fn real_harness_environment_requires_container_and_factory_safety_vector() {
        let base = context();
        assert!(!base.proves_real_harness_environment());
        assert!(context().with_scope_id("   ").is_err());
        assert_eq!(
            context().with_scope_id("scope://one").unwrap().scope_id.as_deref(),
            Some("scope://one")
        );

        let missing_secret = context()
            .with_execution_environment(
                "env://container/a",
                ExecutionClass::Container,
                vec![
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::NetworkEgressPolicy,
                ],
            )
            .unwrap();
        assert!(!missing_secret.proves_real_harness_environment());

        let remote_is_not_a_container_shortcut = context()
            .with_execution_environment(
                "env://remote/a",
                ExecutionClass::Remote,
                vec![
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
            )
            .unwrap();
        assert!(!remote_is_not_a_container_shortcut.proves_real_harness_environment());

        let policy_labels_without_attestation = context()
            .with_execution_environment(
                "env://container/unattested",
                ExecutionClass::Container,
                vec![
                    ExecutionGuarantee::FilesystemReadPolicy,
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::ProcessBoundary,
                    ExecutionGuarantee::ResourceLimits,
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
            )
            .unwrap();
        assert!(!policy_labels_without_attestation.proves_real_harness_environment());

        let isolated = context()
            .with_execution_environment(
                "env://container/a",
                ExecutionClass::Container,
                vec![
                    ExecutionGuarantee::FilesystemReadPolicy,
                    ExecutionGuarantee::FilesystemWritePolicy,
                    ExecutionGuarantee::ProcessBoundary,
                    ExecutionGuarantee::ResourceLimits,
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                    ExecutionGuarantee::RuntimeAttestation,
                ],
            )
            .unwrap();
        assert!(isolated.proves_real_harness_environment());
    }
}
