# Morn — Protocol-Driven Work Control Plane

Morn is a domain-neutral, outcome-oriented control plane for composing humans,
agents, deterministic programs, solvers, software services and machines into
governed real-world work.

> Current convergence branch: **v11.5 architecture implementation**. The
> existing v1 GA contracts remain a validated baseline; v11.5 is being added
> beside them instead of rewriting history in place. See
> [docs/architecture-v11.5.md](docs/architecture-v11.5.md).
>
> This repository does **not** claim real customer/production evidence merely
> because fixture, conformance or CI tests pass.

## Core model

Morn is not an agent framework and does not require every task to become an
agent. The primary semantics are:

```text
Work
  -> Capability
  -> Authority
  -> ExecutionBinding
  -> Attempt
  -> Receipt / Reconciliation
  -> Observed Outcome
  -> Acceptance
```

Four rules guide the system:

1. **Work** is the unit of coordination.
2. **Capability** is the unit of composition.
3. **Accepted outcome** is the unit of value.
4. **Agent** is one executor kind among rules, programs, solvers, humans,
   services and devices.

## v11.5 architecture

Morn is split into distinct concerns rather than one immutable implementation:

- **Semantic / Specification Plane** — versioned Work, Capability, Authority,
  Binding, Attempt, Receipt, Outcome, Acceptance, Provenance and Profile
  contracts.
- **Durable Control Plane** — desired/observed Work state, generation,
  Conditions, controllers, persistence and reconciliation.
- **Composition Plane** — reference host uses exact-pinned **Cordis** only for
  node-local Context/Service/dependency/Fiber/plugin lifecycle.
- **Execution Plane** — DeepSeek Harness, Pi, deterministic runtimes, solvers,
  workflow engines, connectors, humans and machines are replaceable providers.
- **Trust Plane** — policy decision and enforcement are separate; a
  model/harness cannot acquire real authority merely because it can call a
  tool.
- **Capability Supply Chain** — artifact -> candidate -> observed -> qualified
  -> released -> site-conformant -> admitted.

### Cordis is reused, not reimplemented

The reference composition host lives in `runtime/cordis-host` and currently
pins `@deepseek-ai/cordis@4.0.4`.

Cordis owns software composition/lifecycle. It does **not** own durable Work
truth, authority, external-action truth, reconciliation, outcome or acceptance.
A provider hot-swap may affect a future binding; it cannot silently rewrite an
active `ExecutionBinding`.

### Harnesses are replaceable

The repository contains a shared `HarnessProvider` contract with:

- Morn Native reference provider;
- DeepSeek Harness fixture boundary;
- Pi fixture boundary;
- DSH/Pi neutrality conformance benchmark.

Real DSH/Pi transport remains separate from Morn semantics and is not faked when
the external runtime is unavailable.

## Repository layout

```text
crates/
  morn-kernel/          protocol metadata, identity, policy, event envelope
  morn-world/           operational objects/state/outcomes
  morn-work/            WorkPackage, AcceptanceSpec, durable work
  morn-capability/      CapabilityManifest, resolver, effect contracts
  morn-runtime/         binding, attempts, authority, execution env, reconcile
  morn-control-plane/   desired/observed reference controllers + persistence
  morn-profile/         guarantee profiles + conformance
  morn-harness/         harness provider contract, DSH/Pi/native
  morn-foundry/         Solution compiler + Artifact2Capability
  morn-assurance/       evaluation/certification + qualification/site admission
  morn-package/         packs + OCI/Sigstore/SLSA-oriented descriptors
  ...
runtime/
  cordis-host/          exact-pinned node-local reference composition host
domain-packs/
  biolab-reference/     reference domain pack, feature-gated
frontend/
  Workbench / Studio / Console / Hub
src-tauri/
  desktop shell
```

## Capability model

A Morn capability is broader than an agent:

```text
Capability
= implementation/provider reference
+ machine-readable manifest
+ interface contract
+ execution requirements
+ authority envelope
+ economics
+ provenance
+ qualification/admission evidence
```

The lifecycle is intentionally non-collapsed:

```text
Declared -> Observed -> Qualified -> Admitted -> Suspended/Retired
```

Compilation is not qualification. Qualification is not site admission.

Artifact2Capability compilers cover OpenAPI, structured SOP/procedure,
explicit repository manifests, reviewed paper manifests, pinned model manifests
and durable workflow manifests. Every compiler emits a **Declared** capability
candidate with source provenance; none may claim qualification, site admission,
Outcome or Acceptance merely because the artifact parses or executes. Studio
exposes all six flows under the governed advanced import surface.

### Desktop API routing (reference only)

The Tauri desktop shell loads prebuilt frontend assets, so Vite's `/api`
development proxy is **not** present in the packaged webview. Its API client
therefore resolves `/api` to `http://127.0.0.1:8090/api` under Tauri
origins. The server must currently be started separately with
`cargo run -p morn-app --bin server --all-features` before using the desktop
shell. This is a development/reference configuration, **not** a self-contained
signed installer or an authenticated production deployment.

### Reusable blueprint and runtime instance

Morn does not create separate canonical `Blueprint` or `Instance` objects.
An approved `SolutionPackage` is the reusable blueprint. Instantiating it
creates a normal `WorkResource`, preserving `source_solution_ref` while
leaving profile, capability, authority and execution gates explicit.

A Workcell is the minimum-sufficient executor mix for that Work; it may use
zero, one or many agents alongside rules, programs, solvers, humans, services
or devices.

## Work control and external effects

A v11.5 Work resource has `spec`, `status`, `generation`,
`observed_generation` and Conditions. Harness sessions or workflow cursors are
runtime state, not canonical business state.

External action truth is explicit:

```text
PROPOSED
-> AUTHORIZED
-> DISPATCHED
-> ACKNOWLEDGED
-> COMMITTED
-> OBSERVED
-> VERIFIED
```

Ambiguous outcomes are first-class:

```text
DISPATCHED / ACKNOWLEDGED / COMMITTED
-> OUTCOME_UNKNOWN
-> RECONCILING
```

A timeout after a remote commit is never blindly retried. The
ReconciliationController queries the authoritative external system using the
business/idempotency key.

## Version and history semantics

Morn does not require every object to be permanently immutable.

- current projections may change;
- published contracts evolve by explicit version;
- a running binding is pinned for its execution scope;
- historical facts may be superseded, retracted, migrated or redacted;
- past interpretation is never silently overwritten.

The rule is **non-destructive historical evolution**, not “nothing can ever
change”.

## Standards-first interoperability

Morn prefers existing standards/mechanisms instead of proprietary reinvention:

| Concern | Preferred mechanism |
| --- | --- |
| local composition | Cordis |
| agent harness | DSH / Pi / other HarnessProvider |
| tools/resources | MCP / OpenAPI |
| agent peer | A2A |
| event envelope | CloudEvents-compatible core fields |
| telemetry | OpenTelemetry |
| policy decision | native reference / OPA / Cedar / customer IAM |
| package distribution | OCI / ORAS |
| signature/provenance | Sigstore / SLSA |
| isolation | process / container / microVM / VM / remote / physical provider |
| execution guarantees | typed provider-neutral guarantee vector (network, filesystem, secrets, boundaries, attestation, state) |
| optional durable workflow | Temporal/Dapr-class provider |

## Factory first product slice

The first Factory profile remains **brownfield and read-first**:

> outage / insert-order exception -> capacity -> delivery-impact review

`morn.factory.readonly@1.0.0` is a **guarantee profile**, not a plugin list. It
requires durable Work state, source-of-truth binding, capability qualification,
authority before side effect, receipt/reconciliation, outcome observation,
independent acceptance, provenance and a minimum isolation level.

Fixture tests include the classic “remote order committed, client timed out”
case and prove that Morn enters `OUTCOME_UNKNOWN`, reconciles the external
reference and does not blindly redispatch.

**Production write remains out of scope.**

## Product surfaces

- **Workbench** — real Work, Conditions, runtime/control-plane status, outcomes,
  acceptance and attention.
- **Studio** — starts from a work goal or existing artifact; compiles solutions
  and candidate capabilities while keeping qualification gates visible.
- **Console** — provider/composition/trust/conformance/external-blocker view.
- **Hub** — capability/harness/runtime/compiler/package registry surface.

All surfaces share one backend/domain model; they must not create separate
business truth.

## Quickstart

```powershell
# Backend
cargo build --workspace --all-features
cargo run -p morn-app --bin server --all-features

# Frontend
cd frontend
npm install
npm run dev

# Full verification
powershell -ExecutionPolicy Bypass -File scripts/run_all.ps1
```

Environment variables: `MORN_DB` (SQLite path, default `morn.db`) and
`MORN_PORT` (default `8090`).

### Windows reference desktop build

The Desktop GitHub Actions job builds both the Tauri shell and the Windows
backend and publishes an unsigned `morn-windows-reference-<SHA>` artifact
containing `morn-desktop.exe` and `server.exe`. This is for engineering
acceptance, **not** an authenticated production release or signed installer.

Extract both executables, run `server.exe` first (it binds only to
`127.0.0.1:8090`), then launch `morn-desktop.exe`. The backend uses
`morn.db` in its working directory unless `MORN_DB` is set.
Keep the backend local; no production credentials or industrial writes should
be exposed through this reference package.

The v11.5 metadata endpoint is:

```text
GET /api/v115/status
```

and Studio's first Artifact2Capability endpoint is:

```text
POST /api/v115/artifact/openapi/compile
```

## Key invariants

- zero-domain Core remains runnable without domain packs;
- Work truth outlives harness sessions;
- provider replacement never edits an active binding in place;
- unknown external outcome is never blindly retried;
- compilation never implies qualification;
- qualification never implies site admission;
- profiles specify guarantees, not provider names;
- authority decision is separate from enforcement;
- Cordis reversible lifecycle effects are not business rollback;
- fixtures never upgrade an external/production evidence claim.

## Verification

GitHub Actions runs backend fmt/clippy/tests, zero-domain checks, frontend
typecheck/lint/tests/build, Cordis host tests/smoke, E2E UI smoke and desktop
build. The v11.5 branch also adds provider-neutrality, profile-conformance,
Artifact2Capability, execution-environment and Factory read-only integration
tests.

## External blockers / non-claims

- real DeepSeek Harness process/configuration is external to the current
  deterministic fixture contract;
- real Pi transport is not configured by the reference runtime;
- real Factory/customer pilot requires lawful site data and explicit authority;
- OCI publication/signing, real microVM/DSec-class execution and customer
  OPA/Cedar/IAM integration require external infrastructure;
- **REAL_PRODUCTION_WRITE = NO**.

## Docs

- [v11.5 architecture](docs/architecture-v11.5.md)
- [Language-neutral v11.5 protocol](spec/v11.5/README.md)
- [Architecture baseline](docs/architecture.md)
- [Developer guide](docs/developer-guide.md)
- [Deployment](docs/deployment.md)
- [Security](docs/security.md)
- [Release notes](RELEASE_NOTES.md)

## License

`MIT OR Apache-2.0`
