# ADR-010 — Historical Evolution Is Explicit, Not Silently Immutable

Status: Accepted for v11.5.

## Decision

Morn does not freeze all state forever. Current projections may change.
Published contracts evolve by version; active bindings are pinned within their
execution scope; historical facts are corrected via supersede/retract/redact or
explicit migration.

Bitemporal fields distinguish when a fact applied from when Morn recorded it.

## Consequences

The system can explain decisions using what was known at the time while still
supporting correction, privacy redaction and schema migration.
