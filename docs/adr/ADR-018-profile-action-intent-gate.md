# ADR-018 — Domain Profile action intent gates are independent from IAM authorization

Status: Accepted for v11.5 convergence.

## Context

A policy/IAM provider answers whether a principal is allowed to perform an
action. A Domain Profile answers a different question: whether this class of
effect is permitted for this product/operating mode at all.

For example, a plant administrator may technically possess CMMS/MES write
permission while `morn.factory.readonly@1.0.0` intentionally forbids
production write. Conflating those two decisions would let broad enterprise IAM
permissions bypass a Morn product/safety profile.

Fixture/simulation tests also need to exercise write-like state transitions
without turning them into a production-write claim.

## Decision

1. External action intent is classified independently as Read, CandidateOnly,
   SandboxWrite, ShadowWrite, ProductionWrite or PhysicalControl.
2. A Domain Profile semantic gate is evaluated before the external action path.
3. IAM/Authority approval cannot override a profile-level forbidden semantic.
4. Factory read-only explicitly blocks ProductionWrite.
5. SandboxWrite may be used by deterministic fixtures/simulations without
   upgrading the product evidence claim.
6. PhysicalControl fails closed unless the exact profile explicitly opts in.
7. A future write-enabled Factory profile must be a new version/profile with
   its own evidence and admission; it may not mutate the read-only profile.

## Consequences

- Production write remains structurally impossible in the current Factory
  profile even if a test policy or administrator identity would otherwise allow
  it.
- Profile guarantees and identity/policy authorization remain separately
  auditable.
- Simulated external-effect/reconciliation tests no longer rely on ambiguous
  language that could be mistaken for production authorization.
