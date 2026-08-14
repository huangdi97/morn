# STATUS.md — Morn v10.2 Tonight

Last updated: 2026-08-15

## Current State

- Design baseline: Morn v10.2-R1 (greenfield; no pre-existing source repo)
- Repository baseline: INSPECTED — no `.git`, no source; Rust/Node toolchains installed during M0
- Tonight gate: M0–M12 executed; `scripts/run_all.ps1` GREEN (exit 0); final report generated

## Milestones

- [x] M0 Inspect & Baseline (git init, toolchain: rustc 1.97.1 / cargo 1.97.1 / Node 22 / pnpm 11)
- [x] M1 Freeze Contracts (strong IDs, Version, Timestamp, status enums, structured Error)
- [x] M2 Kernel + SQLite (Identity/Workspace/Policy/Approval/Ledger/Lifecycle + MornStore adapter)
- [x] M3 World + Controlled Action (Object/Relation/Event/StateSnapshot/ActionProposal/ActionGateway E0–E3)
- [x] M4 Artifact / Decision (immutable versions, review/approval, provenance, DecisionPackage, VerificationReport)
- [x] M5 Work / Organization Minimum (WorkPackage/AcceptanceSpec/OutcomeContract/ExecutionMode/RoleSlot/MemberBinding/Workcell/ActorTemplate/Instance/Origin)
- [x] M6 Harness Fabric + DSH Spike (HarnessSpec/Binding/RuntimeBinding/CapabilityScope/ExecutionEvent/Receipt; MornNative + DeepSeekHarness providers share one contract suite)
- [x] M7 Durable Minimum (checkpoint/pause/resume/recovery/attention; SQLite persistence incl. restart tests)
- [x] M8 Evolution v0.1 (candidate/branch/evaluation/promotion/rollback; production guard)
- [x] M9 BioLab Vertical Slice (Domain schema + Dataset→Reviewed Claim E2E, lineage asserted)
- [x] M10 Product Surfaces (Workbench/Studio/Console/Hub on one axum backend; React/Vite frontend)
- [x] M11 Full Verification (`scripts/run_all.ps1`: fmt/check/clippy/test/frontend/build/demo smoke — all pass)
- [x] M12 Closeout (STATUS/DECISIONS/BLOCKERS/KNOWN_FAILURES/CHANGELOG + `reports/tonight_final_report.md`)

## Baseline Commands

```text
backend build:  cargo build --workspace
backend test:   cargo test --workspace --all-features
backend lint:   cargo clippy --workspace --all-targets --all-features -- -D warnings
backend fmt:    cargo fmt --all -- --check
frontend install: npm install (in frontend/)
frontend test:  npm test
frontend lint:  npm run lint
frontend build: npm run build
server:         cargo run -p morn-app --bin server   (default http://127.0.0.1:8090, db morn.db)
full verify:    powershell -ExecutionPolicy Bypass -File scripts/run_all.ps1
```

## Latest Test Results (2026-08-15, `scripts/run_all.ps1`, exit 0)

```text
cargo fmt --check            -> pass
cargo check                  -> pass
cargo clippy -D warnings     -> pass
cargo test                   -> all suites pass (kernel 4, world 3, capability 2, artifact 5,
                                work 6, actor 1, organization 1, harness 5 (incl. 2-provider
                                contract + switch/event invariants), runtime 4, evolution 4,
                                biolab 1, store 5, + app/doc tests)
frontend typecheck           -> pass
frontend lint                -> pass
frontend test                -> 2 passed
frontend build               -> pass (dist/ built)
build server binary          -> pass
demo smoke                   -> health=ok, BioLab E2E 7/7 steps, workbench objects=3
```

## Current Blocker Summary

- B-001: real DeepSeek Harness smoke blocked (credential/external; no official DSH install in this environment; the only PyPI `deepseek-harness` is a third-party OpenAI-compatible client requiring a DeepSeek API key). Morn-side provider contract passes for both providers. See `BLOCKERS.md`.

## Current Known Failures

- KF-001: vitest/esbuild `spawn EPERM` inside the sandbox (environment boundary; runs with approval). See `KNOWN_FAILURES.md`.
- KF-002: Tauri desktop shell not built tonight (deferred; web surfaces share the same backend; Tauri = next P1 step). See `KNOWN_FAILURES.md`.
- KF-003: browser-level UI runtime QA (console errors/overflow) not automated tonight (no headless browser harness configured).

## Completion Rule

`ACCEPTANCE.md` A-Gate: all locally-fixable items pass; the only not-passed item is the real external DSH smoke which is an explicit external/credential blocker (B-001) — the Morn-side DSH provider contract is complete. Tauri desktop build is recorded as deferred (not claimed as done).
