# ADR-019 — Durable controller events use inbox dedupe and transactional-intent outbox

Status: Accepted for v11.5 convergence.

## Context

The v11.5 control plane is reconciliation-driven and must tolerate duplicate
delivery, restart and partial transport failure. CloudEvents gives a common
event envelope, but an envelope alone does not guarantee exactly-once
processing. External brokers also cannot be assumed to deliver exactly once.

## Decision

1. Inbound control events are claimed by stable event id in a durable inbox.
   Redelivery of the same event id is safe and does not rerun a non-idempotent
   controller transition.
2. Outbound event delivery is represented by a durable outbox row keyed by the
   same stable event id. Transport retry reuses the event id rather than
   fabricating a new business event.
3. Outbox dispatch status is transport state, not business Work truth.
4. Controller logic remains idempotent/reconciling even with inbox dedupe;
   dedupe is defense-in-depth, not permission to write non-idempotent
   controllers.
5. Event delivery success does not imply external business effect success.
   ActionAttempt/Receipt/Reconciliation remain the real-world effect model.
6. The reference SQLite store implements the minimum inbox/outbox contract.
   Future Kafka/Postgres/cloud transports may replace delivery mechanics
   without changing semantic event identity.

## Consequences

- At-least-once buses and client retries do not require an exactly-once broker.
- Work/controller restart can replay safely.
- Message delivery and business effect truth remain separate.
