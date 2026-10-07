# ADR-014 — Composition ownership is not Work / Actor identity

Status: Accepted for v11.5 convergence.

## Context

Cordis Context/Fiber identify software-composition ownership and cleanup scope.
Modern DSH architecture now makes the same distinction explicitly: an Agent's
Cordis Context owns registrations, while Agent identity is passed explicitly at
runtime boundaries.

Morn has an even stronger need for this separation. A provider/plugin context
must never implicitly choose the business Work, accountable Actor, Authority
decision or execution binding.

## Decision

1. Cordis Context/Fiber identity is **composition ownership only**.
2. Morn Work/Actor/ExecutionBinding/Attempt identity is carried explicitly in
   typed Morn requests/records.
3. Selecting a Cordis context cannot grant Authority or infer a business
   Principal.
4. Harness sessions inherit an explicit `RuntimeContext`; they do not derive
   Work identity by reverse-scanning the composition tree.
5. Provider hot reload changes software availability only. Existing Work and
   active binding identity remain unchanged until an explicit migration
   decision creates a new binding.
6. Browser/UI composition, if later moved onto Cordis, remains a projection
   layer and may not own canonical Work state.

## Consequences

- Morn can directly reuse Cordis without conflating plugin lifecycle with
  business lifecycle.
- DSH/Pi/other harnesses can maintain their own internal context models while
  Morn keeps one explicit runtime identity envelope.
- Distributed/remote providers can cross process boundaries without depending
  on in-process Context object identity.
- Audit/replay can explain which Work/Actor/Binding generated an attempt even
  after the original composition runtime has been restarted.
