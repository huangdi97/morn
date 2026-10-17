# ADR-028 — Durable state transitions and semantic event delivery are atomic

Status: Accepted for v11.5 convergence.

## Context

A controller may update canonical Work/control state and publish a durable
semantic event about that transition. Saving state first and enqueueing the
event later leaves a crash window: the new state can become visible while the
delivery intent is permanently lost.

The reverse order is also unsafe because consumers can observe an event for a
state transition that never committed.

## Decision

1. Durable semantic control/domain/external-observation events use the same
   database transaction as the corresponding mutable projection update whenever
   one state transition and one delivery intent are coupled.
2. MornStore exposes `save_record_cas_with_durable_event` as the reference
   atomic CAS + outbox primitive.
3. The state revision remains optimistic-concurrency controlled.
4. The outbox event id is stable/idempotent.
5. A stale CAS aborts the entire transaction; its event does not leak to the
   outbox.
6. RuntimeSignal and ProjectionNotification are rejected by this durable
   semantic path.
7. Dispatch/retry happens after commit and never invents a new semantic event
   identity.

## Consequences

- Process failure cannot create a committed control projection with a missing
  corresponding durable delivery intent.
- Event delivery remains at-least-once/idempotent at the transport boundary
  while semantic event identity stays stable.
- Runtime telemetry remains separate from business/control event durability.
