# ADR-015 — SolutionPackage is the reusable blueprint; Work is the runtime instance

Status: Accepted for v11.5 convergence.

## Context

User-facing Morn needs a simple mental model: create something once, reuse it,
and run many instances. Earlier design discussion used terms such as Blueprint,
Instance, Digital Employee and Workcell. If each term becomes a new canonical
record, Morn would create parallel truths beside the existing Work,
SolutionPackage, Capability and ExecutionBinding contracts.

## Decision

Morn keeps the product vocabulary simple while reusing canonical records:

```text
user intent / imported artifact
  -> Studio / Solution Compiler
  -> reviewed + approved SolutionPackage
  -> instantiate
  -> WorkResource
  -> CapabilityResolver
  -> minimum-sufficient Workcell
  -> ExecutionBinding
  -> Attempt(s)
  -> Receipt / Reconciliation
  -> ObservedOutcome
  -> AcceptanceDecision
```

The UI may describe a `SolutionPackage` as a **blueprint**. It is not a second
Blueprint record type.

A runtime **instance** is canonical `WorkResource`. Work carries
`source_solution_ref` so the exact reusable package/version remains
traceable.

A **Workcell** is the minimum sufficient executor mix selected for one Work. It
may contain zero, one or many agents and may also contain rules, programs,
solvers, humans, services or devices. It is not a permanently running digital
employee team by default.

Instantiation does not execute anything. It creates Proposed Work and leaves
capability resolution, qualification/site admission, authority, source-of-truth
binding, execution environment selection and binding as explicit gates.

Profile-derived pre-execution gates are separated from post-execution
guarantees. Receipt, reconciliation, outcome and acceptance must not prevent a
new Work from becoming Ready before an execution binding exists.

## User experience

The minimum Studio flow is:

1. describe a goal or import an artifact;
2. compile a candidate solution/capability;
3. inspect gaps, policy and evidence requirements;
4. approve the SolutionPackage;
5. choose a guarantee profile and optional site;
6. instantiate Work;
7. let the control plane resolve the minimum sufficient executor mix;
8. execute only after the required gates are satisfied;
9. close on independently accepted outcome.

Advanced users can still edit manifests, profiles, capabilities and provider
bindings directly.

## Consequences

Morn can offer “one-click instances” without creating an Agent-first ontology.
The same package may instantiate many Work resources across sites and profiles,
while every execution remains independently governed and auditable.

Product labels such as Digital Employee, Factory Copilot or Research Assistant
become packaged experiences/profile compositions, not new kernel semantics.
