# ADR-030 — Evidence class is part of every maturity/value claim

Status: Accepted for v11.5 convergence.

## Context

Morn deliberately distinguishes design/spec evidence, deterministic fixtures,
CI/conformance evidence, real runtime evidence, real site/customer evidence and
production-write evidence.

Without a machine-readable evidence class, it is easy for a green fixture or CI
run to be described later as proof of a real runtime or customer outcome. That
would collapse engineering evidence into business evidence and violate the
v11.5 non-claim discipline.

## Decision

1. Evidence claims are explicit records with:
   - subject;
   - evidence class;
   - state (proven / blocked-external / revoked);
   - evidence references;
   - issuer;
   - reason;
   - observation time.
2. Reference evidence classes are:
   - DesignSpec;
   - LocalFixture;
   - CiConformance;
   - RealRuntime;
   - RealSite;
   - ProductionWrite.
3. Lower-class evidence never auto-promotes a higher-class claim.
4. A blocker is visible evidence about missing conditions, but is not proof of
   the blocked class.
5. Production-write claims require explicit production-write evidence; real
   read-only/site evidence is insufficient.
6. Revocation/supersession is append-only/non-destructive history; it does not
   erase the old claim.
7. UI/reports must label evidence class when presenting maturity, value or
   production-readiness statements.

## Consequences

- A passing DSH/Pi fixture + CI contract can prove local Harness neutrality but
  cannot be reported as real DSH/Pi runtime evidence.
- A simulated Factory wedge can prove reconciliation/control-plane invariants
  but cannot become customer/site evidence.
- BLOCKED_EXTERNAL remains a first-class state rather than being converted to
  PASS by mocks.
