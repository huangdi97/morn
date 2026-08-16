# Morn Developer Guide

This guide covers the SDK surfaces and how to build on Morn.

## Prerequisites

Rust 1.97+ (MSVC on Windows), Node.js 22+, npm. Full verification:
`scripts/run_all.ps1` (Windows PowerShell). CI mirrors it on GitHub Actions.

## Developer CLI (`morn`)

```text
morn doctor|status|node|conformance|domain|package|provider|connector|plugin|compat|migrate
```

- `doctor` — environment + store schema + canonical ledger path checks.
- `status` — current workspace record counts.
- `provider` — intelligence provider fixture + harness smoke.
- `connector list|health` — connector registry/health.
- `plugin validate` — plugin manifest validation.
- `compat` — semantic contract compatibility matrix.
- `migrate status` — schema version / preflight.
- `node list`, `domain list`, `package inspect`, `conformance` — registry and
  conformance kits.

All commands call canonical services; the CLI never writes the database
directly.

## Domain SDK (`morn-domain-sdk`)

A domain declares its ontology and behavior through
`DomainDefinition::declare`, supporting 13 kinds, including `object_type`,
`work_template`, `action_type`, `artifact`, `outcome`, `role`, `work`,
`policy`, `capability`, `connector_requirement`, `evaluation`, and
`ui_extension`. A `DomainRegistry` installs/enables/disables declarations;
enabling a domain surfaces its types and UI extensions.

### Adding a reference domain pack

1. Create a crate under `domain-packs/` depending only on public Morn SDK
   crates (kernel/world/artifact/work/org/actor/harness/runtime/capability).
2. Implement your services against the SDK; keep your own `WorldService` so
   Core stays zero-domain.
3. Add the crate to the workspace and to `morn-app` as an **optional**
   feature-gated dependency (`domain-<name>`).
4. Advertise the pack: `/api/workbench` (and `/api/hub`) return the enabled
   `domain_packs` list; gate any UI extension on that advertisement.
5. Add lifecycle + E2E tests; run `scripts/check_domain_boundary.ps1` and the
   full matrix.

## Pack & Plugin lifecycle (`morn-package`)

`PackLifecycle` supports init → validate → build → install → enable →
disable → upgrade → uninstall → inspect → diff. Uninstall preserves history
and manifest records. `safe_name` rejects empty names, `/`, `\`, `..`, and
control characters to prevent path traversal and command injection. Plugin
manifests declare dependencies, permissions, compatibility, migrations, and
health.

## Provider / Runtime / Connector contracts

- **Provider**: health, scope, invoke, cancel, timeout, normalized
  events/errors, secret redaction. MornNativeHarness and the DSH fixture pass
  one shared contract suite; switching providers preserves canonical records.
- **Runtime**: start, pause, resume, checkpoint, restore, signal, cancel.
  `RuntimeProvider` + `FixtureRuntime` pass conformance.
- **Connector**: read, query, event, mapping, retry, rate limit, idempotency,
  governed write, teardown. Writes require an `ApprovedActionToken` from the
  Action Gateway; duplicates are idempotent.

Conformance kits: `cargo test -p morn-core-tests` (conformance, chaos,
migration/security) plus `crates/morn-harness/tests/contract_tests.rs`.

## Distributed runtime

`morn-node` + `morn-work::DurableRuntime` provide claim/heartbeat/checkpoint/
failover/dedupe. See `docs/architecture.md` §8 for the two-node E2E
guarantees.

## Frontend

Four surfaces (Workbench/Studio/Console/Hub) consume `/api/*` only. State
variants (loading/empty/error/blocked) are components in
`frontend/src/components/ui.tsx`. UI smoke: `cd frontend && node
scripts/ui_smoke.mjs` (requires the server and vite dev/preview running; see
`scripts/run_all.ps1`).

## Testing

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p morn-app                        # zero-domain core
cargo test -p morn-core-tests                  # chaos/conformance/security/pure-core
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check_domain_boundary.ps1
cd frontend && npm run typecheck && npm run lint && npm test && npm run build
```

Do not delete failing tests, do not weaken assertions, and do not mark tests
`#[ignore]` to make CI green.
