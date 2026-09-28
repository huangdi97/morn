//! execution_environment — ExecutionEnvironmentSpec + providers (Phase 9).
//!
//! Proves (Goal §13 / master §15):
//! - the spec models isolation, resources, network, filesystem, secrets,
//!   persistence, timeout, and side-effect policy with explicit values;
//! - local-process is dev-only and available; container is the preferred
//!   default and reports honestly when Docker is unreachable;
//! - microVM / full VM / cloud sandbox are FUTURE providers — contract only,
//!   honestly marked FutureProvider, never simulated as complete;
//! - capability admission checks execution requirements against provider
//!   guarantees: a capability verified in one environment is NOT
//!   automatically admitted in a weaker one.

use morn::providers::execution_env::{
    admit_capability, CloudSandboxProvider, ContainerProvider, EnvironmentHealth,
    ExecutionEnvironmentProvider, ExecutionEnvironmentSpec, FilesystemPolicy, FullVmProvider,
    IsolationLevel, LocalProcessProvider, MicroVmProvider, NetworkPolicy,
};

#[test]
fn g_execution_env_spec_models_all_fields() {
    // The spec carries every field the DSec abstraction requires, with
    // explicit units and modes.
    let spec = ExecutionEnvironmentSpec::container();
    assert_eq!(spec.isolation, IsolationLevel::Container);
    assert_eq!(spec.os_runtime.as_deref(), Some("linux"));
    assert_eq!(spec.cpu, 4);
    assert_eq!(spec.memory_gib, 8);
    assert!(spec.gpu.is_none());
    assert_eq!(spec.filesystem.mode, "workspace-write");
    assert_eq!(spec.network.mode, "allowlist");
    assert!(spec
        .network
        .destinations
        .contains(&"historian.internal".to_string()));
    assert!(spec
        .secret_bindings
        .contains(&"historian-read-token".to_string()));
    assert_eq!(spec.persistence_scope.as_deref(), Some("work-scoped"));
    assert_eq!(spec.timeout_seconds, Some(1800));
    assert_eq!(spec.side_effect_policy, "sandbox-only");

    // Round-trip through serde (protocol resource).
    let json = serde_json::to_string(&spec).unwrap();
    let back: ExecutionEnvironmentSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back, spec);
}

#[test]
fn g_local_process_is_dev_only_and_available() {
    let provider = LocalProcessProvider;
    assert_eq!(provider.provider_id(), "local-process");
    assert_eq!(provider.health(), EnvironmentHealth::Available);
    assert_eq!(provider.spec().isolation, IsolationLevel::LocalProcess);
    assert_eq!(provider.spec().side_effect_policy, "read-only");

    // It admits executions requiring the weakest isolation only.
    assert!(provider.admits(IsolationLevel::LocalProcess));
    assert!(
        !provider.admits(IsolationLevel::Container),
        "local-process must not admit container-isolated executions"
    );
}

#[test]
fn g_container_is_preferred_default_and_honest() {
    // With Docker forced unavailable: Unavailable, admits nothing — no fake.
    let provider = ContainerProvider::with_availability(false);
    assert_eq!(provider.health(), EnvironmentHealth::Unavailable);
    assert!(!provider.admits(IsolationLevel::LocalProcess));
    assert!(!provider.admits(IsolationLevel::Container));
    let result = admit_capability(&provider, IsolationLevel::Container);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not admit"));

    // With Docker available: admits container-isolated executions.
    let provider = ContainerProvider::with_availability(true);
    assert_eq!(provider.health(), EnvironmentHealth::Available);
    assert!(provider.admits(IsolationLevel::LocalProcess));
    assert!(provider.admits(IsolationLevel::Container));
    assert!(!provider.admits(IsolationLevel::MicroVm));
    assert!(admit_capability(&provider, IsolationLevel::Container).is_ok());
}

#[test]
fn g_future_providers_are_never_simulated() {
    // SAFETY: microVM / full VM / cloud sandbox are contract-only providers —
    // honestly marked FutureProvider; they admit nothing and are never
    // simulated as complete.
    for provider in [
        &MicroVmProvider as &dyn ExecutionEnvironmentProvider,
        &FullVmProvider,
        &CloudSandboxProvider,
    ] {
        assert_eq!(provider.health(), EnvironmentHealth::FutureProvider);
        assert!(
            !provider.admits(IsolationLevel::LocalProcess),
            "future providers admit nothing"
        );
        let result = admit_capability(provider, IsolationLevel::LocalProcess);
        assert!(result.is_err());
    }
    assert_eq!(MicroVmProvider.spec().isolation, IsolationLevel::MicroVm);
}

#[test]
fn g_admission_checks_requirements_against_guarantees() {
    // A capability verified in one environment is NOT automatically admitted
    // in a weaker one: admission compares the requirement against the
    // provider's guarantee.
    let local = LocalProcessProvider;
    let container = ContainerProvider::with_availability(true);

    // A capability requiring container isolation: local refuses.
    let result = admit_capability(&local, IsolationLevel::Container);
    assert!(
        result.is_err(),
        "a container-isolated capability is not admitted on local-process"
    );
    assert!(result.unwrap_err().contains("local-process"));

    // The same capability on container: admitted.
    assert!(admit_capability(&container, IsolationLevel::Container).is_ok());

    // An unavailable provider admits nothing regardless of its isolation.
    let down = ContainerProvider::with_availability(false);
    let result = admit_capability(&down, IsolationLevel::LocalProcess);
    assert!(result.is_err(), "an unavailable provider admits nothing");
}

#[test]
fn g_isolation_levels_are_ordered() {
    // The ordering matters for admission: LocalProcess < Container < MicroVm
    // < FullVm < CloudSandbox.
    assert!(IsolationLevel::LocalProcess < IsolationLevel::Container);
    assert!(IsolationLevel::Container < IsolationLevel::MicroVm);
    assert!(IsolationLevel::MicroVm < IsolationLevel::FullVm);
    assert!(IsolationLevel::FullVm < IsolationLevel::CloudSandbox);
}

#[test]
fn g_network_and_filesystem_policies_round_trip() {
    // Explicit network/filesystem policies survive serde round-trips.
    let policy = NetworkPolicy {
        mode: "allowlist".to_string(),
        destinations: vec!["a.internal".to_string(), "b.internal".to_string()],
    };
    let json = serde_json::to_string(&policy).unwrap();
    let back: NetworkPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(back, policy);

    let fs = FilesystemPolicy {
        mode: "read-only".to_string(),
    };
    let json = serde_json::to_string(&fs).unwrap();
    let back: FilesystemPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(back, fs);
}
