# ADR-038 — Terminal Work phases are monotonic within a Work generation

Status: Accepted for v11.5 convergence.

## Context

Morn receives asynchronous and potentially reordered evidence from harnesses,
workflow engines, event buses and external systems. A late outcome, provider
notification or controller retry must not silently move an already Accepted,
Rejected or Cancelled Work back to Running/Delivered/Blocked.

At the same time, Morn must allow explicit change: a user may revise the Work
specification, migrate Profile/protocol semantics or intentionally create a new
generation.

## Decision

1. `Accepted`, `Rejected` and `Cancelled` are terminal phases for the
   current Work generation.
2. Generic readiness/progress reconciliation is a no-op for a terminal
   generation.
3. Late/reordered executor, outcome or control evidence is still retainable as
   history/evidence but cannot implicitly reopen terminal business state.
4. Reopening requires an explicit desired-state change that increments Work
   generation (for example spec/profile/protocol migration or an explicit future
   reopen operation).
5. `Delivered` is not terminal because independent Acceptance may still
   Accept, Reject or request more evidence.
6. `Reconciling`, `Waiting`, `Blocked` and `Terminating` are not terminal.
7. Historical Attempts/Receipts/Outcomes remain immutable/non-destructive
   evidence when a new generation is created.

## Consequences

- Event reorder cannot silently regress Accepted Work.
- Control-plane retries remain idempotent with respect to terminal state.
- Corrective business action is explicit and attributable rather than hidden in
  a late event.
- Work can still evolve because terminality is generation-scoped, not permanent
  object immutability.
