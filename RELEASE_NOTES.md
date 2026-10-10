# Release Notes — Morn v1.0.0-rc.1

> **Historical snapshot — not current v11.5 product truth.** This document records the
> v1.0.0-rc.1 / August 2026 state. For the current convergence branch use
> [STATUS.md](STATUS.md) and [BLOCKERS.md](BLOCKERS.md). In particular, statements
> below saying that official DSH was unavailable or that Real mode was only a stub
> are preserved historical observations and have been superseded by the v11.5
> SDK/ACP adapters and current external-evidence blockers.

Release candidate for the Morn v1.0 GA Re-Foundation (Goal
`MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`).

## Highlights

- **Domain-neutral Core** — runs with zero domain packs; reference
  `biolab-reference` pack exercises the full install/enable/disable/uninstall
  lifecycle with preserved history.
- **Stable Semantic Kernel v1** — 22 canonical contracts with compatibility
  matrix and deprecation policy.
- **Capability fabric** — HarnessProvider (MornNative + DSH fixture),
  IntelligenceProvider, RuntimeProvider; one conformance kit for providers,
  runtimes, connectors, plugins, domain packs, and architecture.
- **Distributed durable runtime** — real local two-node claim/checkpoint/
  lease/failover/dedupe E2E.
- **Governed effects** — E0/E1/E2/E3 with approval and compensation rules.
- **Product surfaces** — Workbench / Studio / Console / Hub on one backend,
  zero-domain ready, data-driven domain UI gating.
- **Developer CLI** — doctor/status/node/provider/connector/plugin/domain/
  package/compat/migrate/conformance.
- **Security & chaos** — 10 migration/security + 7 chaos tests; explicit
  recover/block/escalate/compensate, no silent corruption.
- **CI/CD** — GitHub Actions (fmt/clippy/backend/frontend/E2E/desktop).
- **GitHub migration** — old `morn` history archived to
  `legacy/pre-rewrite` + `legacy-pre-rewrite` tag; new Morn on `morn-v1` and
  `main`.

## External blockers (unchanged)

- B-001 real DeepSeek Harness smoke (needs official distribution or
  credentials) — Morn-side provider contract passes; no fake success.
- G4-B-002 real BioLab data pilot (needs a lawful real dataset) — core
  pipeline contracts pass with fixture-controlled records.

## Known limitations

- DSH `Real` mode reports `Error::External` until credentials/install are
  available.
- The real BioLab pilot is blocked on a lawful dataset; no fabricated data.
