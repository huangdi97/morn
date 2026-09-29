//! workcell_resolver — Capability Resolver / Workcell (Phase 11).
//!
//! Proves (Goal §15 / master §19):
//! - the resolver is NOT an LLM guesser: hard constraints filter, soft
//!   scoring ranks, and the two are separate;
//! - a Workcell may be 0-Agent (Rule + SQL/Human), 1 Agent + Rule, or
//!   Agent + Solver + Human — "Agent is one kind of executor";
//! - every hard-constraint failure is typed: capability missing, expired
//!   qualification, site not admitted, authority ceiling, budget, provider
//!   unavailable;
//! - a Workcell is the dynamic composition of ONE Work, not a department.

use morn::control::workcell::{
    CandidateCapability, ExecutorKind, ResolutionFailure, ResolutionRequirements, Workcell,
    WorkcellResolver,
};
use morn::work::CostAmount;

fn candidate(name: &str, provides: &[&str], kind: ExecutorKind) -> CandidateCapability {
    CandidateCapability {
        name: name.to_string(),
        digest: format!("sha256:{name}"),
        provides: provides.iter().map(|s| s.to_string()).collect(),
        executor_kind: kind,
        provider_id: "fake".to_string(),
        authority_max: vec!["cmms.read".to_string()],
        qualified: true,
        site_admitted: true,
        provider_available: true,
        cost: 1.0,
        latency_p95_seconds: 30.0,
    }
}

fn requirements(work_ref: &str, required: &[&str]) -> ResolutionRequirements {
    ResolutionRequirements {
        work_ref: work_ref.to_string(),
        required_capabilities: required.iter().map(|s| s.to_string()).collect(),
        site: "plant-a".to_string(),
        authority_ceiling: vec!["cmms.read".to_string(), "cmms.create-order".to_string()],
        budget: CostAmount::new(50.0, "CNY"),
    }
}

#[test]
fn g_workcell_zero_agent() {
    // A Workcell does not require an Agent: Rule + Human is sufficient.
    let resolver = WorkcellResolver;
    let candidates = [
        candidate(
            "vibration-rule",
            &["equipment.anomaly.detect"],
            ExecutorKind::Rule,
        ),
        candidate(
            "maintenance-engineer",
            &["equipment.anomaly.decide"],
            ExecutorKind::Human,
        ),
    ];
    let cell = resolver
        .resolve(
            &requirements(
                "work-1",
                &["equipment.anomaly.detect", "equipment.anomaly.decide"],
            ),
            &candidates,
        )
        .expect("resolves");
    assert!(cell.is_zero_agent(), "Rule + Human is a 0-Agent Workcell");
    assert_eq!(cell.members.len(), 2);
    assert_eq!(cell.members[0].executor_kind, ExecutorKind::Rule);
    assert_eq!(cell.members[1].executor_kind, ExecutorKind::Human);
}

#[test]
fn g_workcell_agent_plus_rule_and_agent_solver_human() {
    let resolver = WorkcellResolver;

    // 1 Agent + Rule.
    let candidates = [
        candidate(
            "vibration-rule",
            &["equipment.anomaly.detect"],
            ExecutorKind::Rule,
        ),
        candidate(
            "equipment-investigator",
            &["equipment.anomaly.investigate"],
            ExecutorKind::Agent,
        ),
    ];
    let cell = resolver
        .resolve(
            &requirements(
                "work-2",
                &["equipment.anomaly.detect", "equipment.anomaly.investigate"],
            ),
            &candidates,
        )
        .expect("resolves");
    assert!(!cell.is_zero_agent());
    assert_eq!(cell.members.len(), 2);

    // Agent + Solver + Human.
    let candidates = [
        candidate(
            "equipment-investigator",
            &["equipment.anomaly.investigate"],
            ExecutorKind::Agent,
        ),
        candidate("schedule-solver", &["schedule.plan"], ExecutorKind::Solver),
        candidate(
            "production-owner",
            &["schedule.decide"],
            ExecutorKind::Human,
        ),
    ];
    let cell = resolver
        .resolve(
            &requirements(
                "work-3",
                &[
                    "equipment.anomaly.investigate",
                    "schedule.plan",
                    "schedule.decide",
                ],
            ),
            &candidates,
        )
        .expect("resolves");
    assert_eq!(cell.members.len(), 3);
    assert_eq!(cell.members[1].executor_kind, ExecutorKind::Solver);
    assert_eq!(cell.members[2].executor_kind, ExecutorKind::Human);
}

#[test]
fn g_capability_missing_is_typed() {
    let resolver = WorkcellResolver;
    let candidates = [candidate(
        "vibration-rule",
        &["equipment.anomaly.detect"],
        ExecutorKind::Rule,
    )];
    let failure = resolver
        .resolve(
            &requirements("work-4", &["equipment.anomaly.investigate"]),
            &candidates,
        )
        .expect_err("missing capability is a typed failure");
    assert_eq!(
        failure,
        ResolutionFailure::CapabilityMissing {
            capability: "equipment.anomaly.investigate".to_string()
        }
    );
}

#[test]
fn g_qualification_expired_blocks_resolution() {
    let resolver = WorkcellResolver;
    let mut unqualified = candidate(
        "equipment-investigator",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    unqualified.qualified = false;
    let failure = resolver
        .resolve(
            &requirements("work-5", &["equipment.anomaly.investigate"]),
            &[unqualified],
        )
        .expect_err("unqualified capability blocks");
    assert!(matches!(
        failure,
        ResolutionFailure::QualificationMissing { .. }
    ));
}

#[test]
fn g_site_not_admitted_blocks_resolution() {
    let resolver = WorkcellResolver;
    let mut not_admitted = candidate(
        "equipment-investigator",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    not_admitted.site_admitted = false;
    let failure = resolver
        .resolve(
            &requirements("work-6", &["equipment.anomaly.investigate"]),
            &[not_admitted],
        )
        .expect_err("site not admitted blocks");
    match failure {
        ResolutionFailure::SiteNotAdmitted { capability, site } => {
            assert_eq!(capability, "equipment.anomaly.investigate");
            assert_eq!(site, "plant-a");
        }
        other => panic!("expected SiteNotAdmitted, got {other:?}"),
    }
}

#[test]
fn g_authority_ceiling_insufficient_blocks_resolution() {
    let resolver = WorkcellResolver;
    let mut overreaching = candidate(
        "cmms-writer",
        &["cmms.write-order"],
        ExecutorKind::Connector,
    );
    overreaching.authority_max = vec!["cmms.write-order".to_string(), "plc.write".to_string()];
    // The Work's ceiling does not include plc.write.
    // A dedicated ceiling that includes cmms.write-order so that plc.write is
    // the ONLY action over the ceiling (isolating the assertion).
    let requirements = ResolutionRequirements {
        work_ref: "work-7".to_string(),
        required_capabilities: vec!["cmms.write-order".to_string()],
        site: "plant-a".to_string(),
        authority_ceiling: vec![
            "cmms.read".to_string(),
            "cmms.create-order".to_string(),
            "cmms.write-order".to_string(),
        ],
        budget: CostAmount::new(50.0, "CNY"),
    };
    let failure = resolver
        .resolve(&requirements, std::slice::from_ref(&overreaching))
        .expect_err("authority ceiling blocks");
    match failure {
        ResolutionFailure::AuthorityCeilingInsufficient { capability, action } => {
            assert_eq!(action, "plc.write");
            // The failure names the REQUIRED capability interface (the
            // caller's view), per the resolver's typed-failure semantics.
            assert_eq!(capability, "cmms.write-order");
        }
        other => panic!("expected AuthorityCeilingInsufficient, got {other:?}"),
    }
}

#[test]
fn g_budget_insufficient_blocks_resolution() {
    let resolver = WorkcellResolver;
    let mut expensive = candidate(
        "equipment-investigator",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    expensive.cost = 99.0;
    // Budget is 50 CNY; the candidate costs 99.
    let failure = resolver
        .resolve(
            &requirements("work-8", &["equipment.anomaly.investigate"]),
            &[expensive],
        )
        .expect_err("budget blocks");
    match failure {
        ResolutionFailure::BudgetInsufficient {
            required,
            currency,
            available,
        } => {
            assert_eq!(required, 99.0);
            assert_eq!(currency, "CNY");
            assert_eq!(available, 50.0);
        }
        other => panic!("expected BudgetInsufficient, got {other:?}"),
    }
}

#[test]
fn g_provider_unavailable_fallback() {
    // The candidate's provider is down: a typed failure (never a silent
    // resolution). With a second provider available, resolution succeeds.
    let resolver = WorkcellResolver;
    let mut down = candidate(
        "equipment-investigator",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    down.provider_available = false;
    let failure = resolver
        .resolve(
            &requirements("work-9", &["equipment.anomaly.investigate"]),
            std::slice::from_ref(&down),
        )
        .expect_err("provider unavailable is typed");
    assert!(matches!(
        failure,
        ResolutionFailure::ProviderUnavailable { .. }
    ));

    // Fallback: a fresh catalog whose provider is available resolves — the
    // typed failure is the caller's signal to retry with the fallback.
    let mut fallback = candidate(
        "equipment-investigator-pi",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    fallback.provider_available = true;
    fallback.provider_id = "pi".to_string();
    let cell = resolver
        .resolve(
            &requirements("work-9", &["equipment.anomaly.investigate"]),
            std::slice::from_ref(&fallback),
        )
        .expect("fallback catalog resolves");
    assert_eq!(cell.members[0].provider_id, "pi");
}

#[test]
fn g_hard_constraints_and_soft_scoring_are_separate() {
    // Soft scoring ranks eligible candidates but NEVER changes eligibility:
    // an expensive-but-qualified candidate still resolves; scoring is
    // informational.
    let resolver = WorkcellResolver;
    let mut expensive = candidate(
        "slow-expensive",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    expensive.cost = 10.0;
    expensive.latency_p95_seconds = 120.0;
    let mut cheap = candidate(
        "fast-cheap",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    cheap.cost = 1.0;
    cheap.latency_p95_seconds = 10.0;

    let cell = resolver
        .resolve(
            &requirements("work-10", &["equipment.anomaly.investigate"]),
            &[expensive, cheap],
        )
        .expect("both are eligible; soft scoring is informational");
    assert_eq!(cell.members.len(), 1);
    // The resolver picks the first eligible candidate (deterministic);
    // soft scores are recorded as informational metadata.
    assert!(!cell.soft_scores.is_empty());

    // Deterministic: the same input resolves the same Workcell.
    let mut expensive2 = candidate(
        "slow-expensive",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    expensive2.cost = 10.0;
    expensive2.latency_p95_seconds = 120.0;
    let mut cheap2 = candidate(
        "fast-cheap",
        &["equipment.anomaly.investigate"],
        ExecutorKind::Agent,
    );
    cheap2.cost = 1.0;
    cheap2.latency_p95_seconds = 10.0;
    let cell2 = resolver
        .resolve(
            &requirements("work-10", &["equipment.anomaly.investigate"]),
            &[expensive2, cheap2],
        )
        .expect("resolves");
    assert_eq!(cell, cell2, "resolution is deterministic");

    // A Workcell is a protocol resource: it round-trips through serde.
    let json = serde_json::to_string(&cell).unwrap();
    let back: Workcell = serde_json::from_str(&json).unwrap();
    assert_eq!(back, cell);
}
