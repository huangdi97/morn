//! The SAME provider contract suite must pass for at least two provider paths:
//! MornNativeHarness and DeepSeekHarnessProvider (fixture mode).

use morn_harness::contract::{run_provider_contract, test_context};
use morn_harness::provider::{DeepSeekHarnessProvider, DshMode, HarnessProvider, MornNativeHarness};
use morn_kernel::ids::WorkspaceId;

#[test]
fn morn_native_passes_provider_contract() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = MornNativeHarness::new();
    let report = run_provider_contract(&mut provider, &ctx).expect("contract suite should run");
    assert!(report.all_passed(), "checks: {:?}", report.checks);
    assert_eq!(report.provider, "morn-native");
}

#[test]
fn deepseek_harness_fixture_passes_provider_contract() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = DeepSeekHarnessProvider::new(DshMode::Fixture);
    let report = run_provider_contract(&mut provider, &ctx).expect("contract suite should run");
    assert!(report.all_passed(), "checks: {:?}", report.checks);
    assert_eq!(report.provider, "deepseek-harness");
}

#[test]
fn deepseek_harness_real_mode_reports_external_blocker() {
    let ws = WorkspaceId::generate();
    let ctx = test_context(&ws);
    let mut provider = DeepSeekHarnessProvider::new(DshMode::Real);
    let err = provider.start(&ctx);
    assert!(err.is_err(), "real DSH is not available in this environment");
    let msg = format!("{}", err.unwrap_err());
    assert!(msg.contains("not installed"), "unexpected error: {msg}");
}
