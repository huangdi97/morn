//! profile_conformance — Profiles: Guarantee Profile conformance suite (Phase 14, G9).
//!
//! Proves (Goal §18 / master §25-26):
//! - all four profiles exist (morn.lite / morn.enterprise /
//!   morn.factory.readonly / morn.research) and are guarantee requirements,
//!   not provider lists;
//! - the Factory Readonly profile requires the canonical guarantee chain and
//!   FORBIDS production write;
//! - G9: a Work missing a required condition cannot pass a profile that
//!   requires it — the conformance engine refuses;
//! - the conformance suite produces a machine-readable JSON report.

use morn::control::controllers::state_loop::ProfileConformanceController;
use morn::control::store::ControlPlaneStore;
use morn::work::{Condition, ConditionType, Profile, RequirementLevel, Work, WorkSpec};

fn work(work_id: &str, profile_ref: Option<&str>) -> Work {
    let mut work = Work::draft(
        work_id,
        WorkSpec {
            work_type: "factory.equipment-exception".to_string(),
            goal: "restore_operational_readiness".to_string(),
            target_refs: Vec::new(),
            constraints: Vec::new(),
            acceptance: None,
            authority: None,
            budget: None,
            profile_ref: profile_ref.map(|s| s.to_string()),
            source_refs: Vec::new(),
            owner: None,
            generation: 1,
            contract_version: None,
        },
    );
    let _ = &mut work;
    work
}

fn condition_of(work: &Work, gate: ConditionType) -> Option<bool> {
    work.condition(gate).map(|c| c.is_true())
}

#[test]
fn g9_all_four_profiles_exist_and_are_guarantee_profiles() {
    // The four profiles exist with distinct names and versioned APIs.
    for (profile, expected_name) in [
        (Profile::lite(), "morn.lite"),
        (Profile::enterprise(), "morn.enterprise"),
        (Profile::factory_readonly(), "morn.factory.readonly"),
        (Profile::research(), "morn.research"),
    ] {
        assert_eq!(profile.name, expected_name);
        assert_eq!(profile.api_version, "profile.morn.io/v1alpha1");
        assert!(!profile.version.is_empty());
        // Profiles are guarantee requirements, not provider lists: no
        // requirement names a concrete provider (dsh/pi/opa/sap).
        for requirement in &profile.requirements {
            let r = requirement.requirement.to_lowercase();
            assert!(
                !r.contains("dsh")
                    && !r.contains("pi ")
                    && !r.contains("opa")
                    && !r.contains("sap"),
                "a profile requirement must not name a provider: {}",
                requirement.requirement
            );
        }
    }
}

#[test]
fn g9_factory_readonly_requires_the_canonical_chain() {
    let profile = Profile::factory_readonly();

    // The canonical required guarantees (Goal §18).
    for requirement in [
        "durable_work_state",
        "source_of_truth_binding",
        "provenance",
        "authority_before_external_side_effect",
        "receipt_after_external_action",
        "accepted_outcome_semantics",
        "profile_version_pinning",
    ] {
        assert_eq!(
            profile.level_of(requirement),
            RequirementLevel::Required,
            "'{requirement}' must be required in morn.factory.readonly"
        );
    }

    // SAFETY: production write is FORBIDDEN in the initial profile.
    assert_eq!(
        profile.level_of("production_write"),
        RequirementLevel::Forbidden
    );

    // Human gates for schedule/quality decisions.
    assert!(profile
        .human_gates
        .contains(&"schedule_commitment".to_string()));
    assert!(profile
        .human_gates
        .contains(&"quality_decision".to_string()));

    // reconciliation_on_unknown is conditional (required IF outcome-unknown).
    let reconciliation = profile
        .requirements
        .iter()
        .find(|r| r.requirement == "reconciliation_on_unknown")
        .expect("reconciliation requirement present");
    assert_eq!(reconciliation.level, RequirementLevel::Required);
    assert_eq!(reconciliation.condition.as_deref(), Some("outcome-unknown"));
}

#[test]
fn g9_work_missing_required_condition_cannot_pass_profile() {
    // SAFETY (canonical G9): a Work missing a required condition cannot pass
    // a profile that requires it — the conformance engine refuses.
    let store = ControlPlaneStore::in_memory().expect("store");
    let mut work = work("work-g9a", Some("morn.factory.readonly"));
    store.insert_work(&work).unwrap();

    // No conditions at all: the profile is not conformant.
    let outcome = ProfileConformanceController
        .reconcile(&store, "work-g9a")
        .unwrap()
        .expect("reconcile");
    assert!(
        outcome
            .condition_changes
            .iter()
            .any(|c| c.type_ == ConditionType::ProfileConformant && !c.is_true()),
        "a Work with no conditions is NOT profile-conformant"
    );

    // Satisfy ONE requirement (source_of_truth_binding): still missing the
    // rest → not conformant.
    work.set_condition(Condition::satisfied(ConditionType::ExecutionBound));
    store.update_work_status("work-g9a", &work.status).unwrap();
    let outcome = ProfileConformanceController
        .reconcile(&store, "work-g9a")
        .unwrap()
        .expect("reconcile");
    assert!(
        outcome
            .condition_changes
            .iter()
            .any(|c| c.type_ == ConditionType::ProfileConformant && !c.is_true()),
        "one satisfied gate is not enough for the full guarantee chain"
    );

    // Satisfy ALL required gates: the profile becomes conformant.
    work.set_condition(Condition::satisfied(ConditionType::AuthoritySatisfied));
    work.set_condition(Condition::satisfied(ConditionType::ExternalActionVerified));
    work.set_condition(Condition::satisfied(ConditionType::AcceptanceSatisfied));
    store.update_work_status("work-g9a", &work.status).unwrap();
    let outcome = ProfileConformanceController
        .reconcile(&store, "work-g9a")
        .unwrap()
        .expect("reconcile");
    assert!(
        outcome
            .condition_changes
            .iter()
            .any(|c| c.type_ == ConditionType::ProfileConformant && c.is_true()),
        "all required gates satisfied → profile-conformant"
    );
}

#[test]
fn g9_profile_conformance_is_refused_for_non_matching_profile() {
    // A Work with a profile_ref that is not factory-readonly is not checked
    // by the factory conformance controller (reconcile returns None).
    let store = ControlPlaneStore::in_memory().expect("store");
    store
        .insert_work(&work("work-g9b", Some("morn.lite")))
        .unwrap();
    let outcome = ProfileConformanceController
        .reconcile(&store, "work-g9b")
        .unwrap();
    assert!(
        outcome.is_none(),
        "a non-factory profile is not factory-conformance-checked"
    );

    // A Work with no profile_ref at all: also not checked.
    store.insert_work(&work("work-g9c", None)).unwrap();
    let outcome = ProfileConformanceController
        .reconcile(&store, "work-g9c")
        .unwrap();
    assert!(outcome.is_none());
}

#[test]
fn g9_machine_readable_conformance_report() {
    // The conformance suite produces a machine-readable JSON report over all
    // four profiles (the G9 evidence artifact).
    let store = ControlPlaneStore::in_memory().expect("store");

    let mut report = serde_json::Map::new();
    for (profile, expected_name) in [
        (Profile::lite(), "morn.lite"),
        (Profile::enterprise(), "morn.enterprise"),
        (Profile::factory_readonly(), "morn.factory.readonly"),
        (Profile::research(), "morn.research"),
    ] {
        // A Work under this profile with no conditions: not conformant.
        // A unique Work per profile iteration: store inserts are idempotent
        // by id, so a shared id would keep the FIRST profile for all four.
        let work_id = format!("work-report-{}", profile.name);
        let work = work(&work_id, Some(&profile.name));
        store.insert_work(&work).unwrap();
        let outcome = ProfileConformanceController
            .reconcile(&store, &work_id)
            .unwrap();

        // Lite/research only require durable_work_state (no mapped gate), so
        // the controller reports conformant for them; enterprise/factory
        // require the full chain and report NOT conformant.
        let conformant = outcome
            .map(|o| {
                o.condition_changes
                    .iter()
                    .find(|c| c.type_ == ConditionType::ProfileConformant)
                    .map(|c| c.is_true())
            })
            .unwrap_or(None);

        let gate_count = profile
            .requirements
            .iter()
            .filter(|r| r.level == RequirementLevel::Required)
            .count();
        report.insert(
            profile.name.clone(),
            serde_json::json!({
                "version": profile.version,
                "required_guarantees": gate_count,
                "human_gates": profile.human_gates,
                "production_write_forbidden": profile.level_of("production_write") == RequirementLevel::Forbidden,
                "empty_work_conformant": conformant,
            }),
        );
        // Clean up for the next profile iteration.
        let _ = expected_name;
    }

    // Machine-readable report assertions.
    let report = serde_json::Value::Object(report);
    let factory = report.get("morn.factory.readonly").expect("factory entry");
    assert_eq!(
        factory["production_write_forbidden"],
        serde_json::json!(true)
    );
    assert_eq!(factory["required_guarantees"], serde_json::json!(8));
    assert_eq!(
        factory["empty_work_conformant"],
        serde_json::json!(false),
        "an empty Work is not factory-conformant"
    );

    let lite = report.get("morn.lite").expect("lite entry");
    assert_eq!(
        lite["production_write_forbidden"],
        serde_json::json!(false),
        "lite does not forbid production write (it is optional there)"
    );

    // The report round-trips through serde (machine-readable evidence).
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
}

#[test]
fn g9_profile_version_is_frozen_and_pinned() {
    // Released profile versions are frozen: the factory_readonly profile
    // carries a version, and the requirement is profile_version_pinning.
    let profile = Profile::factory_readonly();
    assert_eq!(profile.version, "v1alpha1");
    assert_eq!(
        profile.level_of("profile_version_pinning"),
        RequirementLevel::Required
    );

    // The profile is content-stable: two constructions produce identical
    // requirements (the same profile content, always).
    let again = Profile::factory_readonly();
    assert_eq!(profile, again, "profile content is deterministic");
}
