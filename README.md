# Morn — Work & Organization Control Plane

Morn v1.0 GA Re-Foundation. A from-scratch, domain-neutral work and
organization control plane: a stable semantic kernel for governed work,
actors, artifacts, outcomes, evolution, and distributed durable execution.

> Status: **v1.0.0-rc.1** — local GA complete; GitHub takeover to `main` per
> `MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`. See
> `reports/morn_v1_ga_final_report.md`.

## What this is

- **Stable Semantic Kernel** (`morn-kernel`) — identity, workspace, policy,
  approval, append-only ledger, versioned contracts. Provider/plugin cannot
  redefine these.
- **Operational World** (`morn-world`) — objects, relations, events, state
  snapshots, governed action gateway (E0–E3 effects), state diffs, outcomes.
- **Work & Organization** (`morn-work`, `morn-organization`, `morn-actor`) —
  work packages, acceptance specs, durable runtimes, role slots, members,
  actors, representation contracts.
- **Evolution** (`morn-evolution`, `morn-foundry`, `morn-assurance`) —
  candidate/branch/evaluation/promotion with production separation, solution
  compiler, certification, managed work, replacement pilots, rollback.
- **Capability Fabric** (`morn-capability`, `morn-harness`, `morn-runtime`,
  `morn-integration`) — replaceable providers, runtimes and connectors behind
  stable contracts; connectors write only through governed tokens.
- **Node / Distributed Durable Runtime** (`morn-node`, `morn-process`) —
  two-node claim/checkpoint/lease/failover/dedupe, process intelligence.
- **Domain SDK / Pack / Plugin** (`morn-domain-sdk`, `morn-package`) —
  install/enable/disable/upgrade/uninstall domain packs and plugins with
  preserved history and path/name safety.
- **Product surfaces** — Workbench, Studio, Console, Hub on one shared axum
  backend + React/Vite frontend, plus a Tauri desktop shell and a developer
  CLI (`morn`).

## Repository layout

```text
crates/            Rust workspace (kernel -> world/work -> capability -> app)
domain-packs/      Reference domain pack (biolab-reference, feature-gated)
frontend/          React + Vite + TypeScript (four surfaces)
src-tauri/         Tauri v2 desktop shell (loads frontend/dist)
scripts/           run_all.ps1 (full verification), domain boundary guard
docs/              Architecture, developer guide, deployment, security
reports/           Goal 1–5 + v1 GA reports and audits
.github/workflows/ CI (fmt/clippy/backend/frontend/E2E/desktop)
```

## Quickstart (Windows PowerShell)

```powershell
# 1. Install dependencies
#    Rust 1.97+ (MSVC): https://rustup.rs   |   Node.js 22+: https://nodejs.org
#    Chromium for UI smoke: cd frontend; npx playwright install chromium

# 2. Backend
cargo build --workspace --all-features
cargo run -p morn-app --bin server --all-features   # http://127.0.0.1:8090

# 3. Frontend
cd frontend
npm install
npm run dev          # http://127.0.0.1:5173

# 4. Developer CLI
cargo run -p morn-cli -- doctor
cargo run -p morn-cli -- status

# 5. Full verification (fmt / clippy / tests / frontend / Tauri / UI+E2E smokes)
powershell -ExecutionPolicy Bypass -File scripts/run_all.ps1
```

Environment variables for the server: `MORN_DB` (SQLite path, default
`morn.db`), `MORN_PORT` (default `8090`).

## Key invariants

- Core is **domain-neutral**: it starts and runs with zero domain packs; domain
  packs (e.g. `biolab-reference`) are installed through the public SDK and
  advertised to the UI via `/api/workbench` `domain_packs`.
- All canonical state changes flow
  `Proposal -> Schema -> Domain -> Policy -> Approval/Simulation -> Action
  Gateway -> Commit -> Verify -> Ledger/Outcome`. Runtimes/providers cannot
  commit canonical world state directly.
- Artifacts are immutable: edits create new versions with lineage; old
  versions stay readable.
- Evolution never mutates production: promotion creates a new version and a
  rollback point.
- E3 (irreversible) effects require approval; E2 requires a compensation plan.

## Test

```text
scripts/run_all.ps1  -> exit 0
Rust: 218+ tests, 0 ignored (incl. chaos 7, conformance 6, migration/security 10,
      pure-core E2E, reference pack E2E, Goal1–5 regressions)
Frontend: typecheck / lint / 4 tests / build
Desktop: cargo build -p morn-desktop
UI smoke: Playwright — Workbench/Studio/Console/Hub + BioLab E2E + Goal2/3 interactions
Demo smoke: health / bootstrap / BioLab E2E / workbench records
```

## Docs

- [Architecture](docs/architecture.md)
- [Developer guide (SDK / plugin / provider / connector / runtime)](docs/developer-guide.md)
- [Deployment / upgrade / troubleshooting](docs/deployment.md)
- [Security](docs/security.md)
- [Release notes](RELEASE_NOTES.md)

## External blockers (not fixable locally)

- **B-001**: real DeepSeek Harness smoke requires an official DSH distribution
  or real credentials. Morn-side provider contract passes (MornNative + DSH
  fixture). No fake success is reported.
- **G4-B-002**: real BioLab data pilot requires a lawful real dataset. All
  core pipeline contracts pass with fixture-controlled records.

## License

`MIT OR Apache-2.0`
