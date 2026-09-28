//! execution_env — ExecutionEnvironmentSpec + providers (Phase 9, DSec abstraction).
//!
//! SAFETY: a capability verified in one environment is not automatically
//! qualified in another (master §15.3). Admission checks the execution
//! requirements against the provider guarantees; the provider decides where
//! an execution actually lands.
//!
//! Provider staging (Goal §13): local-process is dev-only, container is the
//! preferred default; microVM / full VM / cloud sandbox are FUTURE providers
//! with contracts only — never simulated as complete.

use serde::{Deserialize, Serialize};

/// Isolation level, ordered from weakest to strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IsolationLevel {
    LocalProcess,
    Container,
    MicroVm,
    FullVm,
    CloudSandbox,
}

/// Network policy of an execution environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub mode: String, // "allowlist" | "open" | "offline"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destinations: Vec<String>,
}

/// Filesystem policy of an execution environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesystemPolicy {
    pub mode: String, // "workspace-write" | "read-only" | "full-write"
}

/// ExecutionEnvironmentSpec — what an execution requires (DSec abstraction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionEnvironmentSpec {
    pub isolation: IsolationLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_runtime: Option<String>,
    #[serde(default)]
    pub cpu: u32,
    #[serde(default)]
    pub memory_gib: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    pub filesystem: FilesystemPolicy,
    pub network: NetworkPolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_bindings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persistence_scope: Option<String>, // "work-scoped" | "none" | "site"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    pub side_effect_policy: String, // "none" | "read-only" | "sandbox-write"
}

impl ExecutionEnvironmentSpec {
    /// The dev-only local-process environment (weakest isolation).
    pub fn local_process() -> Self {
        Self {
            isolation: IsolationLevel::LocalProcess,
            os_runtime: None,
            cpu: 2,
            memory_gib: 4,
            gpu: None,
            filesystem: FilesystemPolicy {
                mode: "workspace-write".to_string(),
            },
            network: NetworkPolicy {
                mode: "allowlist".to_string(),
                destinations: Vec::new(),
            },
            secret_bindings: Vec::new(),
            persistence_scope: Some("work-scoped".to_string()),
            timeout_seconds: Some(600),
            side_effect_policy: "read-only".to_string(),
        }
    }

    /// The preferred default container environment.
    pub fn container() -> Self {
        Self {
            isolation: IsolationLevel::Container,
            os_runtime: Some("linux".to_string()),
            cpu: 4,
            memory_gib: 8,
            gpu: None,
            filesystem: FilesystemPolicy {
                mode: "workspace-write".to_string(),
            },
            network: NetworkPolicy {
                mode: "allowlist".to_string(),
                destinations: vec!["historian.internal".to_string()],
            },
            secret_bindings: vec!["historian-read-token".to_string()],
            persistence_scope: Some("work-scoped".to_string()),
            timeout_seconds: Some(1800),
            side_effect_policy: "sandbox-only".to_string(),
        }
    }
}

/// Whether a provider can actually place executions right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentHealth {
    Available,
    Unavailable,
    /// Honest marker: the provider is a FUTURE provider (contract only).
    FutureProvider,
}

/// ExecutionEnvironmentProvider — places executions into environments.
pub trait ExecutionEnvironmentProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn spec(&self) -> ExecutionEnvironmentSpec;
    fn health(&self) -> EnvironmentHealth;
    /// Whether this provider can admit an execution with the given minimum
    /// isolation requirement (capability admission check).
    fn admits(&self, minimum_isolation: IsolationLevel) -> bool {
        match self.health() {
            EnvironmentHealth::Available => self.spec().isolation >= minimum_isolation,
            // Unavailable / future providers admit nothing (honest).
            _ => false,
        }
    }
}

/// Local-process provider (dev only — never a production boundary).
pub struct LocalProcessProvider;

impl ExecutionEnvironmentProvider for LocalProcessProvider {
    fn provider_id(&self) -> &str {
        "local-process"
    }
    fn spec(&self) -> ExecutionEnvironmentSpec {
        ExecutionEnvironmentSpec::local_process()
    }
    fn health(&self) -> EnvironmentHealth {
        EnvironmentHealth::Available // local process always exists (dev)
    }
}

/// Container provider (preferred default). Reports honestly: when Docker is
/// not reachable the health is Unavailable — never a fake green.
pub struct ContainerProvider {
    docker_available: bool,
}

impl ContainerProvider {
    /// Probes Docker availability once at construction.
    pub fn probe() -> Self {
        let available = std::process::Command::new("docker")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        Self {
            docker_available: available,
        }
    }

    /// Forces a known state (for tests and explicit configuration).
    pub fn with_availability(available: bool) -> Self {
        Self {
            docker_available: available,
        }
    }
}

impl ExecutionEnvironmentProvider for ContainerProvider {
    fn provider_id(&self) -> &str {
        "container"
    }
    fn spec(&self) -> ExecutionEnvironmentSpec {
        ExecutionEnvironmentSpec::container()
    }
    fn health(&self) -> EnvironmentHealth {
        if self.docker_available {
            EnvironmentHealth::Available
        } else {
            EnvironmentHealth::Unavailable
        }
    }
}

/// Future providers: contracts exist, no implementation — never simulated.

macro_rules! future_provider {
    ($provider:ident, $id:literal, $isolation:expr, $doc:literal) => {
        #[doc = $doc]
        pub struct $provider;

        impl ExecutionEnvironmentProvider for $provider {
            fn provider_id(&self) -> &str {
                $id
            }
            fn spec(&self) -> ExecutionEnvironmentSpec {
                ExecutionEnvironmentSpec {
                    isolation: $isolation,
                    os_runtime: None,
                    cpu: 0,
                    memory_gib: 0,
                    gpu: None,
                    filesystem: FilesystemPolicy {
                        mode: "read-only".to_string(),
                    },
                    network: NetworkPolicy {
                        mode: "offline".to_string(),
                        destinations: Vec::new(),
                    },
                    secret_bindings: Vec::new(),
                    persistence_scope: Some("none".to_string()),
                    timeout_seconds: None,
                    side_effect_policy: "none".to_string(),
                }
            }
            fn health(&self) -> EnvironmentHealth {
                EnvironmentHealth::FutureProvider
            }
        }
    };
}

future_provider!(
    MicroVmProvider,
    "microvm",
    IsolationLevel::MicroVm,
    "microVM provider — FUTURE provider, contract only (NOT_IMPLEMENTED)."
);
future_provider!(
    FullVmProvider,
    "fullvm",
    IsolationLevel::FullVm,
    "full VM provider — FUTURE provider, contract only (NOT_IMPLEMENTED)."
);
future_provider!(
    CloudSandboxProvider,
    "cloud-sandbox",
    IsolationLevel::CloudSandbox,
    "cloud sandbox provider — FUTURE provider, contract only (NOT_IMPLEMENTED)."
);

/// Capability admission: checks the execution requirements against the
/// provider guarantees. A capability verified in one environment is NOT
/// automatically qualified in another.
pub fn admit_capability(
    provider: &dyn ExecutionEnvironmentProvider,
    minimum_isolation: IsolationLevel,
) -> Result<(), String> {
    if !provider.admits(minimum_isolation) {
        return Err(format!(
            "provider '{}' does not admit executions requiring isolation >= {:?} (health: {:?})",
            provider.provider_id(),
            minimum_isolation,
            provider.health()
        ));
    }
    Ok(())
}
