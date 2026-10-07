//! M20 Reference Pack E2E: install biolab-reference -> domain types/UI appear ->
//! conformance/E2E -> disable -> uninstall -> Core healthy -> history readable.
//! Runs only with the `domain-biolab` feature.

#![cfg(feature = "domain-biolab")]

use morn_app::AppState;
use morn_domain_sdk::{DomainDefinition, DomainRegistry};
use morn_kernel::version::Version;
use morn_package::{PackLifecycle, PackManifest, PackStatus};

fn temp_db(name: &str) -> String {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("morn_g5_ref_{name}_{}.db", uuid::Uuid::new_v4()));
    path.to_string_lossy().to_string()
}

#[test]
fn reference_pack_e2e() {
    let db = temp_db("ref");

    // 1) Core runs with zero domains.
    {
        let state = AppState::new(&db).unwrap();
        let g = state.lock();
        assert_eq!(g.store.schema_version().unwrap(), 3);
        assert!(g.work.work_packages().is_empty());
    }

    // 2) Install + enable the biolab reference pack (manifest + domain definition).
    let mut lc = PackLifecycle::new();
    let pid = lc
        .init(PackManifest::new(
            "biolab-reference",
            "domain-pack",
            Version::v1(),
        ))
        .unwrap();
    lc.validate(&pid).unwrap();
    lc.build(&pid).unwrap();
    lc.install(&pid).unwrap();
    lc.enable(&pid).unwrap();
    assert_eq!(lc.inspect(&pid).unwrap().status, PackStatus::Enabled);

    let mut reg = DomainRegistry::new();
    reg.install(
        DomainDefinition::new("biolab", "1.0.0", "1.0.0")
            .declare(
                "object_type",
                "Dataset",
                serde_json::json!({"states": ["registered", "locked"]}),
            )
            .declare("work_template", "dataset_to_claim", serde_json::json!({}))
            .declare("ui_extension", "biolab-panel", serde_json::json!({})),
    )
    .unwrap();
    assert!(reg.enable("biolab"));

    // 3) Domain types/UI extensions appear only after enable.
    assert_eq!(reg.declarations_for("biolab", "object_type").len(), 1);
    assert_eq!(reg.declarations_for("biolab", "ui_extension").len(), 1);
    assert_eq!(reg.declarations_for("biolab", "work_template").len(), 1);

    // 4) Reference conformance: the BioLab E2E runs against public SDK services.
    let ws = morn_kernel::ids::WorkspaceId::generate();
    let mut biolab = morn_biolab_reference::service::BioLabService::new(ws.clone());
    let e2e = biolab.run_dataset_to_claim_e2e("aging_pilot", 128).unwrap();
    assert!(
        e2e.all_ok(),
        "reference pack E2E must pass: {:?}",
        e2e.steps
    );

    // 5) Disable -> injected declarations disappear; Core still healthy.
    reg.disable("biolab");
    assert!(reg.declarations_for("biolab", "object_type").is_empty());
    assert!(!reg.is_enabled("biolab"));
    let state = AppState::new(&db).unwrap();
    assert_eq!(state.lock().store.schema_version().unwrap(), 3);

    // 6) Uninstall -> Core healthy; historical provenance/history preserved.
    lc.uninstall(&pid).unwrap();
    assert_eq!(lc.inspect(&pid).unwrap().status, PackStatus::Uninstalled);
    assert!(lc.history.iter().any(|h| h.contains("uninstalled")));
    assert!(
        lc.inspect(&pid).is_some(),
        "manifest record retained after uninstall"
    );
    let state = AppState::new(&db).unwrap();
    assert_eq!(state.lock().work.work_packages().len(), 0);
}
