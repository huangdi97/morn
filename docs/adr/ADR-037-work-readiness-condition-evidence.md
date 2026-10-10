# ADR-037 — Work readiness conditions require generation-scoped evidence

Status: Accepted for v11.5 convergence.

## Context

The v11.5 Work controller exposes semantic Conditions such as
`CapabilityResolved`, `CapabilityQualified`, `AuthoritySatisfied`,
`SourceOfTruthBound` and `ProvenanceReady`.

A dangerous implementation shortcut is to let an API caller post booleans such
as `authority_satisfied=true`. That turns the control API into a second source
of truth and allows strict Profile gates to be bypassed without the records
that actually prove them.

Kubernetes-style Conditions are projections of controller observations; they
are not arbitrary user assertions. Morn must preserve the same separation.

## Decision

1. Positive readiness Conditions are derived from durable
   `ConditionEvidence`, never directly from caller-supplied booleans.
2. Condition evidence is pinned to the exact Work id and Work generation.
3. Positive evidence requires at least one evidence reference and an explicit
   producer reference.
4. Replacing Work desired state creates a new generation; evidence from an old
   generation cannot satisfy the new generation.
5. The latest active evidence for a Condition determines its current projected
   value. A later negative/revocation observation can therefore withdraw
   readiness without deleting history.
6. Strict condition producers are typed:
   - CapabilityResolved <- complete WorkcellPlan/resolver result;
   - CapabilityQualified <- active qualification/release/site+Profile admission;
   - action Authority <- currently valid BoundAuthorityDecision for the exact Work/site/action/resource, enforced when an action permit is issued rather than as a generic readiness bit;
   - SourceOfTruthBound <- validated SourceOfTruthBinding for the Work site;
   - ProvenanceReady <- selected capability/source provenance available before execution. The later ExecutionManifest separately pins binding/runtime provenance after a binding exists.
7. The generic `/api/v115/work/reconcile` endpoint rejects readiness boolean
   fields and reconciles from persisted ConditionEvidence.
8. ConditionEvidence is an immutable observation record. Current Work
   Conditions remain mutable projections derived from those records.

## Consequences

- UI/API clients cannot make Factory Work Ready by posting five booleans.
- Controller state can be rebuilt after restart from durable evidence.
- Work generation changes invalidate stale readiness observations without
  deleting them.
- Runtime/provider/controller implementations remain replaceable because the
  evidence record carries semantic meaning rather than vendor-specific state.
- Future condition producers may be added, but a Profile semantic gate remains
  unsatisfied until an accepted producer emits evidence for the current Work
  generation.
