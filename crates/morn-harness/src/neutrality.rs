//! Harness-neutrality checks.
//!
//! The benchmark proves only Morn-side semantics: two harness providers can
//! satisfy the same lifecycle/event contract under the same RuntimeContext.
//! It deliberately does not claim model-output equivalence.

use morn_kernel::error::Result;

use crate::context::RuntimeContext;
use crate::contract::{run_provider_contract, ContractReport};
use crate::provider::HarnessProvider;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pi::{PiHarnessProvider, PiMode};
    use crate::provider::{DeepSeekHarnessProvider, DshMode};
    use morn_kernel::ids::{ActorInstanceId, WorkPackageId, WorkspaceId};

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
}
