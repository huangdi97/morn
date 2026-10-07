# ADR-002 — Separate Durable Work Control from Composition Runtime

Status: Accepted for v11.5.

## Decision

Morn separates the durable Work control plane from the node-local composition
runtime. Cordis composes services/plugins and owns software lifecycle; it does
not own durable business Work, external-action truth, reconciliation, outcome,
or acceptance.

## Consequences

- Work survives Cordis/DSH/Pi process restarts.
- Cordis Fiber/effect lifecycle cannot be used as business transaction truth.
- Controllers reconcile desired/observed Work using persisted records.
- Another composition runtime may replace Cordis without redefining Morn Work.
