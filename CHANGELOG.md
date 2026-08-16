# CHANGELOG.md

## [1.0.0-rc.1] — 2026-08-16 — Morn v1.0 GA Re-Foundation

### Added
- Goal 6 finalization (`MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`):
  - Full local regression + reality/backlog/architecture audits
    (`reports/final_reality_audit.md`, `reports/final_backlog_audit.md`).
  - CI/CD: `.github/workflows/ci.yml` — fmt/clippy/backend tests/
    zero-domain core/CLI smoke/frontend/E2E+Playwright/desktop.
  - Documentation: `README.md`, `docs/architecture.md`,
    `docs/developer-guide.md`, `docs/deployment.md`, `docs/security.md`,
    `RELEASE_NOTES.md`.
  - Hub now advertises enabled `domain_packs` consistently with Workbench
    (D-028); zero-domain Hub no longer hardcodes an empty pack list.
  - `.gitattributes` (LF normalization) and `.gitignore` entry for Tauri
    generated schemas (`src-tauri/gen/`); generated schemas untracked.
  - `scripts/run_all.ps1` runs the domain boundary guard with `-NoProfile` to
    avoid nested PowerShell profile noise.
  - Performance baseline recorded (`reports/performance_baseline.md`).
  - GitHub migration: old `morn` history archived to `legacy/pre-rewrite` +
    `legacy-pre-rewrite` annotated tag; new Morn on `morn-v1` and `main`;
    final report `reports/morn_v1_ga_final_report.md`.

### Fixed
- CI (GitHub Actions, Linux) portability (KF-012/KF-013):
  - 3 `Cargo.toml` files carried UTF-8 BOM (Linux cargo/toml rejects);
    stripped and `scripts/check_no_bom.ps1` guard added to CI.
  - Tauri Linux system deps installed (libwebkit2gtk-4.1-dev etc.) so the full
    workspace clippy/tests run on Ubuntu.
  - `frontend/dist` built before workspace clippy (tauri `generate_context!`
    embeds the frontend assets).
  - Added PNG icon set (icon.png/32x32/128x128/128x128@2x) and updated
    `tauri.conf.json` bundle.icon so `generate_context!` works on Linux.
  - `check_domain_boundary.ps1` test-path filter made cross-platform.
  - E2E UI smoke: vite `--host 127.0.0.1` + readiness loops (IPv6/localhost
    binding on Ubuntu).
  - Result: all four CI jobs green on GitHub Actions.
- Hub `/api/hub` returned a hardcoded empty `domain_packs`; now derived from
  the same feature advertisement as `/api/workbench`.
- `crates/morn-app/tests/e2e_goal4.rs`: removed a misleading `placeholder`
  comment in the external-blocked pilot phase.

### Notes
- External blockers unchanged: B-001 (real DeepSeek Harness smoke),
  G4-B-002 (real BioLab data pilot). Both are credential/data blockers, not
  local defects.

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
