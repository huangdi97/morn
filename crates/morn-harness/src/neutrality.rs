//! Harness-neutrality checks.
//!
//! The benchmark proves only Morn-side semantics: two harness providers can
//! satisfy the same lifecycle/event contract under the same RuntimeContext.
//! It deliberately does not claim model-output equivalence.

use morn_kernel::error::Result;

use crate::context::RuntimeContext;
use crate::contract::{run_provider_contract, ContractReport};
use crate::event::ExecutionEventKind;
use crate::provider::{HarnessProvider, HarnessRuntimeHealthState, ProviderHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessNeutralityReport {
    pub left: ContractReport,
    pub right: ContractReport,
    pub same_contract_surface: bool,
}

impl HarnessNeutralityReport {
    pub fn all_passed(&self) -> bool {
        self.left.all_passed() && self.right.all_passed() && self.same_contract_surface
    }
}

pub fn run_harness_neutrality(
    left: &mut dyn HarnessProvider,
    right: &mut dyn HarnessProvider,
    ctx: &RuntimeContext,
) -> Result<HarnessNeutralityReport> {
    let left_report = run_provider_contract(left, ctx)?;
    let right_report = run_provider_contract(right, ctx)?;
    let left_checks: Vec<&str> = left_report
        .checks
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let right_checks: Vec<&str> = right_report
        .checks
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    Ok(HarnessNeutralityReport {
        same_contract_surface: left_checks == right_checks,
        left: left_report,
        right: right_report,
    })
}

/// Provider-adapter semantic episode that intentionally ignores provider-specific
/// optional lifecycle controls and model-output equivalence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessAdapterEpisode {
    pub provider: String,
    pub work_ref: String,
    pub work_generation: u64,
    pub execution_binding_ref: String,
    pub output_present: bool,
    pub normalized_event_count: usize,
    pub all_events_normalized: bool,
    pub direct_tool_activity: bool,
    pub runtime_version_present: bool,
    pub runtime_digest_present: bool,
    pub runtime_health: HarnessRuntimeHealthState,
    pub settled_turns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessAdapterNeutralityReport {
    pub left: HarnessAdapterEpisode,
    pub right: HarnessAdapterEpisode,
    pub same_work_identity: bool,
    pub same_execution_environment: bool,
    pub both_pinned: bool,
    pub both_e0_only: bool,
    pub both_settled_adapter_turn: bool,
}

impl HarnessAdapterNeutralityReport {
    pub fn all_passed(&self) -> bool {
        self.same_work_identity
            && self.same_execution_environment
            && self.both_pinned
            && self.both_e0_only
            && self.both_settled_adapter_turn
            && self.left.output_present
            && self.right.output_present
            && self.left.all_events_normalized
            && self.right.all_events_normalized
            && !self.left.direct_tool_activity
            && !self.right.direct_tool_activity
            && self.left.runtime_version_present
            && self.right.runtime_version_present
            && self.left.runtime_digest_present
            && self.right.runtime_digest_present
    }
}

fn run_adapter_episode(
    provider: &mut dyn HarnessProvider,
    ctx: &RuntimeContext,
) -> Result<HarnessAdapterEpisode> {
    if !ctx.proves_pinned_work_execution() {
        return Err(morn_kernel::error::Error::validation(
            "adapter-neutrality requires exact Work generation and ExecutionBinding",
        ));
    }
    let scope_id = ctx.scope_id.as_deref().ok_or_else(|| {
        morn_kernel::error::Error::validation("adapter-neutrality requires a mounted scope")
    })?;
    let handle = ProviderHandle {
        provider: provider.provider_name().to_string(),
        scope_id: scope_id.to_string(),
    };
    let session = provider.start(ctx)?;
    let output = provider.send(&session.id, "adapter-neutrality E0 analysis")?;
    let events = provider.stream_events(&session.id);
    let direct_tool_activity = events.iter().any(|event| {
        matches!(
            event.kind,
            ExecutionEventKind::ToolProposed
                | ExecutionEventKind::ToolStarted
                | ExecutionEventKind::ToolCompleted
                | ExecutionEventKind::ToolFailed
        )
    });
    let normalized_event_count = events
        .iter()
        .filter(|event| {
            event.workspace_id == ctx.workspace_id
                && event.session_id == session.id
                && !event.summary.trim().is_empty()
        })
        .count();
    let all_events_normalized = !events.is_empty() && normalized_event_count == events.len();
    let runtime_version_present = provider
        .runtime_version()
        .is_some_and(|version| !version.trim().is_empty());
    let runtime_digest_present = provider
        .runtime_digest()
        .is_some_and(|digest| !digest.trim().is_empty());
    let health = provider.runtime_health_snapshot().ok_or_else(|| {
        morn_kernel::error::Error::validation("adapter-neutrality requires runtime-health evidence")
    })?;
    provider.unmount(&handle)?;

    Ok(HarnessAdapterEpisode {
        provider: provider.provider_name().to_string(),
        work_ref: ctx.work_package_id.to_string(),
        work_generation: ctx.work_generation.unwrap_or_default(),
        execution_binding_ref: ctx
            .execution_binding_ref
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        output_present: output.session_id == session.id && !output.text.trim().is_empty(),
        normalized_event_count,
        all_events_normalized,
        direct_tool_activity,
        runtime_version_present,
        runtime_digest_present,
        runtime_health: health.state,
        settled_turns: health.settled_turns,
    })
}

/// Compare two real-protocol adapter paths at the Morn semantic boundary.
///
/// This proves only local adapter-wire neutrality: both transports execute the
/// same pinned Work identity under E0-only admission, produce normalized
/// executor evidence, and settle a healthy adapter turn. It deliberately does
/// not compare model text and does not establish authenticated live-provider,
/// customer Outcome, Acceptance, or production-write evidence.
pub fn run_harness_adapter_neutrality(
    left: &mut dyn HarnessProvider,
    right: &mut dyn HarnessProvider,
    left_ctx: &RuntimeContext,
    right_ctx: &RuntimeContext,
) -> Result<HarnessAdapterNeutralityReport> {
    let same_work_identity = left_ctx.workspace_id == right_ctx.workspace_id
        && left_ctx.work_package_id == right_ctx.work_package_id
        && left_ctx.work_generation == right_ctx.work_generation
        && left_ctx.execution_binding_ref == right_ctx.execution_binding_ref;
    let same_execution_environment = left_ctx.execution_environment_ref
        == right_ctx.execution_environment_ref
        && left_ctx.execution_class == right_ctx.execution_class
        && left_ctx.execution_guarantees == right_ctx.execution_guarantees;
    let both_pinned =
        left_ctx.proves_pinned_work_execution() && right_ctx.proves_pinned_work_execution();
    let both_e0_only = [
        left.required_scope_restrictions(),
        right.required_scope_restrictions(),
    ]
    .iter()
    .all(|restrictions| restrictions.contains(&"morn.effects<=E0"));

    let left = run_adapter_episode(left, left_ctx)?;
    let right = run_adapter_episode(right, right_ctx)?;
    let both_settled_adapter_turn = [left.clone(), right.clone()].iter().all(|episode| {
        episode.runtime_health == HarnessRuntimeHealthState::Healthy
            && episode.settled_turns > 0
            && episode.normalized_event_count > 0
    });

    Ok(HarnessAdapterNeutralityReport {
        left,
        right,
        same_work_identity,
        same_execution_environment,
        both_pinned,
        both_e0_only,
        both_settled_adapter_turn,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsh_sdk::DshSdkConfig;
    use crate::pi::{PiHarnessProvider, PiMode, PI_REAL_E0_SCOPE_RESTRICTION};
    use crate::pi_rpc::PiRpcConfig;
    use crate::provider::{DeepSeekHarnessProvider, DshMode, DSH_REAL_E0_SCOPE_RESTRICTION};
    use crate::scope::{CapabilityScope, ScopeKind};
    use morn_kernel::ids::{ActorInstanceId, RuntimeBindingId, WorkPackageId, WorkspaceId};
    use morn_kernel::{ExecutionClass, ExecutionGuarantee};

    #[test]
    fn dsh_and_pi_fixture_paths_are_harness_neutral_at_morn_boundary() {
        let ctx = RuntimeContext::new(
            WorkspaceId::generate(),
            ActorInstanceId::generate_with("actor"),
            WorkPackageId::generate_with("work"),
        );
        let mut dsh = DeepSeekHarnessProvider::new(DshMode::Fixture);
        let mut pi = PiHarnessProvider::new(PiMode::Fixture);

        let report = run_harness_neutrality(&mut dsh, &mut pi, &ctx).unwrap();
        assert!(report.all_passed());
        assert_ne!(report.left.provider, report.right.provider);
    }

    #[test]
    fn dsh_sdk_and_pi_rpc_real_adapter_fixtures_are_semantically_neutral() {
        let executable = std::env::current_exe().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let root =
            std::env::temp_dir().join(format!("morn-adapter-neutrality-{}", uuid::Uuid::new_v4()));
        let dsh_home = root.join("dsh-home");
        std::fs::create_dir_all(&dsh_home).unwrap();

        let environment_ref = "env://container/adapter-neutrality";
        let mut dsh_config =
            DshSdkConfig::profile_sdk(cwd.to_string_lossy(), "fixture-provider", "fixture-model")
                .with_dsh_home(dsh_home.to_string_lossy())
                .with_execution_environment_ref(environment_ref)
                .with_profile_configuration_ref(
                    "deepseek-harness-profile@adapter-neutrality#sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                )
                .with_runtime_identity(
                    "fixture-dsh-runtime",
                    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                );
        dsh_config.command = executable.to_string_lossy().to_string();
        dsh_config.args = vec![
            "--exact".to_string(),
            "dsh_sdk::tests::fake_sdk_runtime".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
        ];
        dsh_config.allow_protocol_fixture_transport();
        dsh_config.request_timeout_ms = 10_000;
        dsh_config.turn_timeout_ms = 10_000;

        let pi_config = PiRpcConfig {
            command: executable.to_string_lossy().to_string(),
            args: vec![
                "--exact".to_string(),
                "pi_rpc::tests::fake_pi_rpc_runtime".to_string(),
                "--ignored".to_string(),
                "--quiet".to_string(),
                "--nocapture".to_string(),
            ],
            cwd: Some(cwd.to_string_lossy().to_string()),
            provider: Some("fixture-provider".to_string()),
            model: Some("fixture-model".to_string()),
            execution_environment_ref: Some(environment_ref.to_string()),
            runtime_version: Some("fixture-pi-runtime".to_string()),
            runtime_digest: Some(format!("sha256:{}", "c".repeat(64))),
            request_timeout_ms: 10_000,
            prompt_timeout_ms: 10_000,
            append_route_args: false,
            strict_jsonl: false,
        };

        let workspace = WorkspaceId::generate();
        let actor = ActorInstanceId::generate_with("actor");
        let work = WorkPackageId::generate_with("work");
        let binding = RuntimeBindingId::generate_with("binding");
        let guarantees = vec![
            ExecutionGuarantee::FilesystemReadPolicy,
            ExecutionGuarantee::FilesystemWritePolicy,
            ExecutionGuarantee::ProcessBoundary,
            ExecutionGuarantee::ResourceLimits,
            ExecutionGuarantee::NetworkEgressPolicy,
            ExecutionGuarantee::SecretIndirection,
            ExecutionGuarantee::RuntimeAttestation,
            ExecutionGuarantee::ToolMediation,
        ];

        let mut dsh = DeepSeekHarnessProvider::with_real_sdk(dsh_config);
        let dsh_scope = dsh
            .mount(
                CapabilityScope::new(
                    ScopeKind::ExecutionRun,
                    None,
                    workspace.clone(),
                    "adapter-neutrality-dsh",
                )
                .with_restriction(DSH_REAL_E0_SCOPE_RESTRICTION),
            )
            .unwrap();
        let dsh_ctx = RuntimeContext::new(workspace.clone(), actor.clone(), work.clone())
            .with_work_binding(1, binding.clone())
            .unwrap()
            .with_scope_id(dsh_scope.scope_id)
            .unwrap()
            .with_execution_environment(
                environment_ref,
                ExecutionClass::Container,
                guarantees.clone(),
            )
            .unwrap();

        let mut pi = PiHarnessProvider::with_real_rpc(pi_config);
        let pi_scope = pi
            .mount(
                CapabilityScope::new(
                    ScopeKind::ExecutionRun,
                    None,
                    workspace.clone(),
                    "adapter-neutrality-pi",
                )
                .with_restriction(PI_REAL_E0_SCOPE_RESTRICTION),
            )
            .unwrap();
        let pi_ctx = RuntimeContext::new(workspace, actor, work)
            .with_work_binding(1, binding)
            .unwrap()
            .with_scope_id(pi_scope.scope_id)
            .unwrap()
            .with_execution_environment(environment_ref, ExecutionClass::Container, guarantees)
            .unwrap();

        let report = run_harness_adapter_neutrality(&mut dsh, &mut pi, &dsh_ctx, &pi_ctx).unwrap();
        assert!(report.all_passed(), "adapter-neutrality: {report:?}");
        assert_ne!(report.left.provider, report.right.provider);
        assert_eq!(report.left.work_ref, report.right.work_ref);
        assert_eq!(
            report.left.execution_binding_ref,
            report.right.execution_binding_ref
        );

        dsh.shutdown_real_runtime().unwrap();
        pi.shutdown_real_runtime().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
