# ADR-022 — Work termination is two-phase and guarded by finalizers

Status: Accepted for v11.5 convergence.

## Context

A durable Work can outlive processes and can already have external effects when
an operator asks to cancel/delete it. Removing the Work record immediately
would create exactly the failure Morn is designed to avoid: an external order,
notification, schedule change or physical action may still exist while its
governing Work disappears.

Kubernetes finalizers demonstrate a useful generic control-plane pattern:
deletion intent is recorded first; the object remains in a terminating state
while responsible controllers finish cleanup; only then can finalizers be
removed.

Morn adopts the pattern, not Kubernetes resource semantics.

## Decision

1. Work termination is monotonic once requested. It is not silently "unasked".
2. `termination_requested_at` and a reason are durable Work metadata.
3. Work enters `Terminating`; it does not immediately disappear.
4. Qualified finalizer keys identify unresolved control-plane obligations.
5. New finalizers cannot be added after termination is requested; controllers
   must register their obligations before/when they create consequential work.
6. A Work cannot finalize while finalizers remain.
7. External-effect finalization means "the consequence is known and any
   required reconciliation/compensation decision is recorded", not "the world
   has been rolled back".
8. E3 irreversible effects may be fully known and therefore finalizable even
   though they cannot be undone.
9. Cancellation never deletes Attempts, Receipts, Outcomes, Acceptance,
   Authority or historical facts.
10. Business cancellation and physical deletion/storage retention are separate
    concerns.

## Consequences

- "Cancel Work" cannot erase an ambiguous timeout-after-commit.
- Controllers can reconcile or compensate without resurrecting a deleted Work.
- Historical audit remains available after terminal cancellation.
- Morn can later map this contract to Kubernetes/Crossplane-like control planes
  without making Kubernetes a protocol dependency.
