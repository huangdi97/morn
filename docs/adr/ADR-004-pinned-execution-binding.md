# ADR-004 — Pin Execution Bindings per Attempt

Status: Accepted for v11.5.

## Decision

Every side-effecting Attempt references an immutable-in-scope
ExecutionBinding containing Work generation, capability manifest, provider
version/digest, runtime, authority decision and Profile reference.

Provider replacement creates a new binding. It never edits the binding used by
an active or historical Attempt.

## Consequences

Hot reload affects future selection, not historical execution identity. Any
migration is explicit and attributable.
