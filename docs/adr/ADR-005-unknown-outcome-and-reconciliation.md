# ADR-005 — Model Ambiguous External Effects as OUTCOME_UNKNOWN

Status: Accepted for v11.5.

## Decision

Dispatch, acknowledgement, external commit, observation and verification are
different facts. A timeout after dispatch/commit is not automatically failure.

Morn enters OUTCOME_UNKNOWN and uses reconciliation against the authoritative
external system with a business/idempotency key before any retry.

## Consequences

Blind duplicate writes are prohibited. Reconciliation evidence becomes part of
the Attempt/Receipt trail.
