//! Materialize a resolved minimum-sufficient capability plan as the existing
//! canonical Workcell organization object.
//!
//! CapabilityResolver owns selection. Workcell owns the execution-unit
//! lifecycle. This bridge intentionally does not create a parallel AgentTeam or
//! Blueprint record.

use morn_capability::WorkcellPlan;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{WorkPackageId, WorkspaceId};
use morn_organization::Workcell;

pub fn materialize_workcell_plan(
    workspace_id: WorkspaceId,
    work_package_id: WorkPackageId,
    name: impl Into<String>,
    plan: &WorkcellPlan,
) -> Result<Workcell> {
    if !plan.is_complete() {
        return Err(Error::invalid_state(format!(
            "cannot materialize incomplete workcell; uncovered={:?}",
            plan.uncovered
        )));
    }
    if plan.members.is_empty() {
        return Err(Error::invalid_state(
            "complete workcell plan must contain at least one executor capability",
        ));
    }

    let mut workcell = Workcell::new(workspace_id, name, work_package_id);
    workcell.capabilities = plan
        .members
        .iter()
        .map(|member| member.capability.manifest_id.to_string())
        .collect();
    workcell.capabilities.sort();
    workcell.capabilities.dedup();
    Ok(workcell)
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_capability::{
        CapabilityKind, CapabilityManifest, CapabilityRecord, CapabilityRequest,
        CapabilityResolver, CapabilityStage, EffectClass, IsolationLevel, WorkcellRequest,
    };
    use morn_kernel::ids::CapabilityId;

    fn admitted(
        name: &str,
        provider: &str,
        kind: CapabilityKind,
        provides: &[&str],
    ) -> CapabilityRecord {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            name,
            provider,
            kind,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provides = provides.iter().map(|value| value.to_string()).collect();
        manifest.execution.minimum_isolation = IsolationLevel::Container;
        manifest.economics.estimated_cost_micros = Some(1);
        let mut record = CapabilityRecord::new(manifest);
        record.stage = CapabilityStage::Admitted;
        record.qualification_refs.push(format!("qual:{name}"));
        record.admitted_sites.push("plant-a".to_string());
        record
    }

    #[test]
    fn zero_agent_plan_materializes_as_normal_workcell() {
        let candidates = vec![
            admitted(
                "alarm-rule",
                "rule-provider",
                CapabilityKind::Rule,
                &["alarm.classify"],
            ),
            admitted(
                "capacity-solver",
                "solver-provider",
                CapabilityKind::Solver,
                &["capacity.optimize"],
            ),
        ];
        let plan = CapabilityResolver.resolve_minimum_workcell(
            &WorkcellRequest {
                required_provides: vec!["alarm.classify".into(), "capacity.optimize".into()],
                site_ref: Some("plant-a".into()),
                ..Default::default()
            },
            &candidates,
        );
        assert!(plan.is_complete());
        assert_eq!(plan.agent_count(), 0);

        let workcell = materialize_workcell_plan(
            WorkspaceId::generate(),
            WorkPackageId::generate_with("work"),
            "factory exception review",
            &plan,
        )
        .unwrap();
        assert_eq!(workcell.capabilities.len(), 2);
    }

    #[test]
    fn incomplete_plan_cannot_be_materialized() {
        let plan = CapabilityResolver.resolve_minimum_workcell(
            &WorkcellRequest {
                required_provides: vec!["missing".into()],
                ..Default::default()
            },
            &[],
        );
        assert!(!plan.is_complete());
        assert!(materialize_workcell_plan(
            WorkspaceId::generate(),
            WorkPackageId::generate_with("work"),
            "incomplete",
            &plan,
        )
        .is_err());
    }

    #[test]
    fn capability_resolution_request_type_remains_independent() {
        let _request = CapabilityRequest::default();
    }
}
