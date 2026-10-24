# ADR-024 — Protocol and Profile evolution requires explicit compatibility

Status: Accepted for v11.5 convergence.

## Context

Morn intentionally avoids an immutable implementation core. Protocol semantics
and Domain Profiles may evolve, but a running Work cannot be silently
reinterpreted under a newer contract simply because software was upgraded.

Version numbers alone are insufficient if the content behind an already
published version can change without detection.

## Decision

### Protocol

- Same protocol version + changed semantic slots/invariants is incompatible
  because it is a silent mutation.
- Patch version changes are compatible only when the semantic contract is
  equivalent.
- Minor protocol changes require explicit reevaluation.
- Major protocol changes are incompatible by default.
- Protocol migration may require Profile reevaluation and always creates new
  execution bindings when the protocol version changes.

### Domain Profile

- Same Profile id/version + changed guarantees is incompatible.
- A patch may not change guarantee semantics; a semantic patch mutation is
  incompatible.
- A minor Profile change requires explicit reevaluation and site readmission.
- A Profile major change or cross-Profile migration is incompatible by default.
- Changing Profile version creates a new Work desired-state generation and new
  ExecutionBindings; an active Attempt stays on the binding/profile it started
  with.

### Execution provenance

ExecutionManifest records the protocol version and invariant identifiers used by
the execution scope, plus the exact Profile/binding/provider/runtime
interpretation.

## Consequences

1. There is no permanently immutable Morn implementation.
2. Published version identity has real meaning and cannot be silently reused for
   changed semantics.
3. Current state can migrate, but migration is an explicit decision with
   reevaluation/readmission where required.
4. Historical Work/Attempt evidence remains interpretable under the contract
   that existed when it executed.
