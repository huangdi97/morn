# ADR-036 — Revocation blocks future effects but never strands reconciliation

Status: Accepted for v11.5 convergence.

## Context

A capability release, qualification, site admission or provider may be revoked/
suspended after an ExecutionBinding was created. Two unsafe reactions are both
possible:

- continue issuing new real-world effects because the old binding is pinned;
- invalidate the binding so aggressively that Morn refuses to reconcile a
  side effect that may already have happened.

The first ignores current trust state. The second loses consequence accounting.

## Decision

1. Before a **new external effect**, strict Enterprise/Factory execution
   re-checks the pinned binding against:
   - exact capability manifest;
   - current site + Profile admission;
   - current qualification/release state behind that admission;
   - provider liveness/freshness;
   - exact pinned provider version/digest.
2. A revoked/suspended/stale dependency blocks new effects and requires an
   explicit replacement/rebind before future execution.
3. Existing ExecutionBinding/Attempt history is never rewritten by revocation.
4. Reconciliation of an already-dispatched/unknown attempt remains allowed
   when the capability identity still matches, even if admission/provider
   selection has since been revoked.
5. Reconciliation may use a separate currently eligible source-of-truth
   connector/provider; it does not re-authorize the original side effect.
6. A capability identity mismatch blocks both new execution and reconciliation
   under that binding because the evidence can no longer be attributed safely.

## Consequences

- Revocation is effective for future consequences without erasing history.
- `OUTCOME_UNKNOWN` cannot become permanently unresolvable merely because a
  runtime/provider is suspended.
- Provider/capability replacement remains an explicit binding migration.
