# Morn Architecture

Morn v1.0 GA Re-Foundation. This document describes the stable architecture
enforced by tests in this repository (`scripts/check_domain_boundary.ps1`,
`crates/morn-core-tests/tests/conformance.rs`,
`crates/morn-core-tests/tests/architecture_*.rs`).

## 1. Layering

```text
Kernel / World / Work contracts
        ↑
Actor / Organization
        ↑
Harness / Runtime adapters
        ↑
Product surfaces (Workbench / Studio / Console / Hub)
```

Dependency direction is enforced: Core crates must not depend on concrete
domain packs; domain packs depend only on the public Morn SDK; the UI only
talks to application APIs and never writes the database directly.

## 2. Workspace crates

| Crate | Responsibility |
| --- | --- |
| `morn-kernel` | IDs, Version, timestamps, status enums, structured error, Identity, Workspace, Policy, Approval, append-only Ledger, frozen semantic contracts |
| `morn-world` | ObjectType/Object (private state), Relation, Event, StateSnapshot, ActionType/ActionProposal, ActionGateway effects, StateDiff, Outcome |
| `morn-artifact` | Immutable Artifact versions, Review, Approval, Provenance, DecisionPackage, VerificationReport |
| `morn-capability` | CapabilityDefinition/Provider/Consumer, EffectClass E0–E3 |
| `morn-harness` | HarnessSpec/Version/Binding, CapabilityScope, ExecutionEvent normalization, ExecutionReceipt, HarnessProvider (MornNative + DSH fixture), IntelligenceProvider |
| `morn-runtime` | ActionGateway (preview/authorize/execute/verify/recover), RuntimeProvider, FixtureRuntime |
| `morn-actor` | ActorTemplate/Instance, ActorOrigin, RepresentationContract |
| `morn-organization` | RoleSlot, MemberBinding, Delegation, Commitment, Workcell, DecisionPolicyAsset, AccountabilityChain |
| `morn-work` | WorkPackage, AcceptanceSpec, OutcomeContract, ExecutionMode, WorkService, DurableRuntime (checkpoint/pause/resume/signals/retry/compensation) |
| `morn-evolution` | Candidate/Branch/Evaluation/PromotionDecision/Rollback, Flywheel, Distillation, EvolutionPlannerProvider |
| `morn-foundry` | Domain-neutral Solution Compiler, WorkGraph, Manifest |
| `morn-assurance` | Certification, Evaluation, Managed Work, Replacement Pilot, Replay, Shadow, Simulation, Rollback |
| `morn-opint` | Operational intelligence: episodes, datasets, predictors, calibration, drift |
| `morn-integration` | Connector SDK: ConnectorSpec/Instance/Mapping/SyncCursor/ExternalActionRequest, governed writes via ApprovedActionToken |
| `morn-process` | Generic process intelligence: ObservedEvent/Trace/WorkGraph, bottleneck/wait/rework/loop signals (no domain ontology) |
| `morn-node` | MornNode, NodeType, health, lease, topology/placement |
| `morn-domain-sdk` | DomainDefinition/Declaration/Registry (13 declaration kinds) |
| `morn-package` | PackLifecycle (init/validate/build/install/enable/disable/upgrade/uninstall/inspect/diff), PluginManifest, safe_name validation |
| `morn-store` | SQLite adapter (schema v2, migrations, workspace isolation, immutable receipts) |
| `morn-app` | Shared axum API + server binary; zero-domain by default, `domain-biolab` feature |
| `morn-cli` | Developer CLI: doctor/status/node/provider/connector/plugin/domain/package/compat/migrate/conformance |
| `morn-core-tests` | Zero-domain migration/security/chaos/conformance + pure-core E2E + reference pack E2E |
| `morn-biolab-reference` (domain-packs) | Reference domain pack behind the public SDK (feature-gated) |
| `src-tauri` (morn-desktop) | Tauri v2 desktop shell loading frontend/dist |

## 3. Canonical state pipeline

```text
Proposal
→ Schema Validation
→ Domain Validation
→ Policy
→ Approval / Simulation (by risk)
→ Action Gateway
→ Commit
→ Verify
→ Ledger / Outcome
```

Runtimes, harnesses, providers, and connectors may only produce proposals,
events, and receipts. The only write path to canonical world state is the
governed service/action gateway (`morn-world::WorldService::commit_state`,
`morn-runtime::ActionGateway`). Tests assert this
(`runtime_cannot_commit_world_directly`).

## 4. Effect classes

- E0 lifecycle_reversible — provider mount/unmount cleanup.
- E1 transactional.
- E2 compensatable — a compensation plan is required; compensation completes
  on failure.
- E3 irreversible — default Preview + Approval + Verify; unapproved E3 is
  denied.

## 5. Evolution vs production

Evolution runs on branches/candidates only. A failed evaluation cannot
promote; a successful promotion creates a **new** production version plus a
rollback point. Evolution never mutates production in place.

## 6. Domain neutrality

- Core is compiled and tested with zero domain packs (`cargo check -p
  morn-app`, `cargo test -p morn-core-tests`).
- The compiler uses generic governed-deliverable signals, never domain words.
- UI surfaces are gated by backend advertisement (`/api/workbench`
  `domain_packs`), not by hardcoded domain names.
- A reference pack (`biolab-reference`) exercises the whole lifecycle
  (install/enable/disable/uninstall) while Core stays healthy and history is
  preserved.

## 7. Persistence

- SQLite via `morn-store` (schema v2, bundled rusqlite). The kernel and
  services depend on repository ports, not on SQLite directly, so a
  PostgreSQL adapter can replace it without touching contracts.
- Ledger is append-only; receipts and artifact versions are immutable;
  migrations are versioned with preflight/dry-run/apply/verify and restore
  plans; downgrade without a restore plan is rejected.

## 8. Distributed durable runtime

Two real local nodes: Node A claims work → checkpoint → Node A failure →
lease expiry → Node B takeover → restore → duplicate event → dedupe (no
duplicated external effect) → work completes → audit failover record. Stale
worker completion, capability mismatch, duplicate signals, and checkpoint
drift are rejected or explicitly surfaced.

## 9. UI

Workbench / Studio / Console / Hub are four views of one backend. The UI
implements loading / empty / error / blocked states and never fabricates
records.
