# CHANGELOG.md

## [0.1.0] — 2026-08-15 — Morn v10.2 Tonight Baseline

### Added
- Rust workspace (13 crates) implementing the Morn Stable Semantic Kernel and domain layers:
  - `morn-kernel`: strong IDs, Version, Timestamp, status enums, structured Error, Identity, Workspace, Policy, Approval, append-only Ledger, LifecycleTracker.
  - `morn-world`: ObjectType/Object (private state), Relation, StateSnapshot, WorldEvent, ActionType/ActionProposal/Action, Goal/Metric, OutcomeRecord, WorldService (governed `commit_state`).
  - `morn-artifact`: immutable Artifact versions, Review, ArtifactApproval, ProvenanceGraph, DecisionPackage, VerificationReport, ArtifactService.
  - `morn-capability`: CapabilityDefinition/Provider/Consumer, EffectClass E0–E3, EffectContract.
  - `morn-actor`: ActorTemplate/Instance, ActorOrigin, RepresentationContract.
  - `morn-organization`: RoleSlot, MemberBinding, ResponsibilityBinding, Delegation, Commitment, Workcell.
  - `morn-work`: WorkPackage, AcceptanceSpec, OutcomeContract, WorkContract, ExecutionMode, AttentionItem, Checkpoint, RecoveryRecord, WorkService + DurableWorkService.
  - `morn-harness`: HarnessSpec/HarnessVersion, HarnessBinding/RuntimeBinding, CapabilityScope, ExecutionEvent(+normalization), ExecutionReceipt, RuntimeContext, HarnessProvider trait, MornNativeHarness, DeepSeekHarnessProvider (Fixture/Real), shared provider contract suite.
  - `morn-runtime`: ActionGateway (preview/authorize/execute/verify/recover) enforcing Policy + Approval + Effect Class.
  - `morn-evolution`: EvolutionCandidate/Branch/Evaluation/PromotionDecision/RollbackRecord + governed EvolutionEngine.
  - `morn-biolab`: BioLab domain schema + Dataset→Reviewed Scientific Claim E2E service.
  - `morn-store`: SQLite adapter (rusqlite bundled) with versioned schema, append-only ledger, typed JSON records, restart persistence.
  - `morn-app`: shared axum HTTP API (Workbench/Studio/Console/Hub/BioLab/Evolution) + server binary.
- Frontend: React + Vite + TypeScript with four surfaces (Workbench/Studio/Console/Hub) on the shared API, loading/empty/error states, BioLab E2E runner.
- `scripts/run_all.ps1`: fmt/check/clippy(-D warnings)/test + frontend typecheck/lint/test/build + server build + demo smoke (health + BioLab E2E).

### Fixed
- UTF-8 BOM issue on strict JSON parsing (KF-004).
- `all_ok` serialized field on BioLab E2E result.

### Known limitations
- Real DeepSeek Harness smoke: external/credential blocker (B-001); Morn-side provider contract passes for both providers.
- Tauri desktop shell and browser-level UI QA deferred (KF-002 / KF-003).
