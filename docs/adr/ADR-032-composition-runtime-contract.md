# ADR-032 — Cordis is the reference composition runtime, not the Morn runtime contract

Status: Accepted for v11.5 convergence.

## Context

Morn reuses Cordis because Cordis already solves node-local service slots,
dependency composition, scoped lifecycle and reversible software effects. The
architecture must still avoid making Cordis APIs the semantic definition of
Morn, otherwise replacing Cordis later would require rewriting Work/Authority/
Outcome semantics.

## Decision

1. Morn defines a small `CompositionRuntimeProvider` contract above concrete
   composition frameworks.
2. The contract covers only node-local service-slot binding, replacement,
   unmount/lifecycle and inspection.
3. The reference implementation remains exact-pinned Cordis in
   `runtime/cordis-host`.
4. Composition snapshots contain runtime/service metadata only. They contain
   no canonical Work, Authority, ExecutionBinding, Outcome or Acceptance truth.
5. Replacing a service slot in the composition runtime affects provider
   availability for future resolution. It never mutates an already-started
   ExecutionBinding/Attempt.
6. An alternative future composition implementation may pass the same
   conformance contract without changing the Morn semantic protocol.
7. No second composition runtime is implemented merely to prove abstraction;
   the contract is kept small until a real need exists.

## Consequences

- Morn is Cordis-first but not Cordis-locked.
- Cordis-specific Context/Fiber/Effect types stay behind the reference host.
- The Rust/domain/control-plane crates can test composition neutrality without
  importing Cordis.
- Cordis upgrades are runtime/provider migrations, not business-schema
  migrations.
