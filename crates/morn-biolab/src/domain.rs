//! BioLab domain schema: object types, action types, roles and work templates.

use std::collections::BTreeMap;

use serde_json::json;

use morn_kernel::ids::{ActionTypeId, WorkspaceId};
use morn_kernel::status::MemberType;
use morn_world::object::ObjectType;

/// Register the BioLab operational object types in a world service.
pub fn register_object_types(ws: &WorkspaceId, world: &mut morn_world::WorldService) {
    let dataset_actions = vec![
        ActionTypeId::generate_with("actt"),
    ];
    world.register_object_type(ObjectType::new(
        "biolab.Dataset",
        ws.clone(),
        vec!["name".into(), "rows".into(), "locked_version".into(), "qc_status".into()],
        vec!["registered".into(), "locked".into(), "qc_failed".into()],
        dataset_actions,
    ));
    world.register_object_type(ObjectType::new(
        "biolab.AnalysisRun",
        ws.clone(),
        vec!["dataset".into(), "status".into(), "summary".into()],
        vec!["pending".into(), "running".into(), "completed".into(), "failed".into()],
        vec![],
    ));
    world.register_object_type(ObjectType::new(
        "biolab.ScientificClaim",
        ws.clone(),
        vec!["statement".into(), "status".into(), "dataset".into(), "analysis_run".into(), "artifact".into()],
        vec!["draft".into(), "reviewed".into(), "released".into()],
        vec![],
    ));
}

/// Role blueprints used by the BioLab demo workcell.
pub fn role_blueprints() -> Vec<(&'static str, Vec<MemberType>)> {
    vec![
        ("analyst", vec![MemberType::Actor]),
        ("pipeline", vec![MemberType::DeterministicWorker]),
        ("statistical_reviewer", vec![MemberType::Actor, MemberType::Human]),
        ("pi", vec![MemberType::Human]),
    ]
}

/// Initial state for a locked dataset fixture.
pub fn dataset_state(name: &str, rows: u64) -> BTreeMap<String, serde_json::Value> {
    let mut s = BTreeMap::new();
    s.insert("name".to_string(), json!(name));
    s.insert("rows".to_string(), json!(rows));
    s.insert("locked_version".to_string(), json!("ds-v1"));
    s.insert("qc_status".to_string(), json!("locked"));
    s
}
