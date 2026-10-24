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
3. Evidence classes are categorical, not a scalar maturity ladder. One class never implicitly proves another.
4. A blocker is visible evidence about missing conditions, but is not proof of
   the blocked class.
5. Production-write claims require explicit production-write evidence; real
   read-only/site evidence is insufficient.
6. Revocation/supersession is append-only/non-destructive history; it does not
   erase the old claim.
7. UI/reports must label evidence class when presenting maturity, value or
   production-readiness statements.

## Consequences

- A passing DSH/Pi fixture + CI contract can prove those explicit engineering evidence classes but
  cannot be reported as real DSH/Pi runtime evidence.
- A simulated Factory wedge can prove reconciliation/control-plane invariants
  but cannot become customer/site evidence.
- BLOCKED_EXTERNAL remains a first-class state rather than being converted to
  PASS by mocks.

## Categorical semantics

The class names are ordered only for deterministic display/serialization. They
do not define inheritance. In particular:

- CIConformance does not imply LocalFixture unless both claims exist;
- RealRuntime does not imply RealSite;
- RealSite does not imply ProductionWrite;
- ProductionWrite evidence does not retroactively prove design/CI claims.

A report that needs several classes must require every class explicitly.

## Deployment-coherence rule

Evidence-class semantics remain categorical and append-only; this rule does not
rewrite historical RealSite or ProductionWrite facts. However, when those
classes are used to make a **current deployment-readiness** decision, Morn also
requires identity coherence with the deployment being evaluated.

- RealSite and ProductionWrite readiness use an exact site-scoped subject.
- A Proven RealSite or ProductionWrite claim used by release readiness must cite
  the exact `MORN_BUILD_IDENTITY_REF` of the running deployment.
- A claim from an older build remains auditable history, but is projected as an
  identity mismatch for a different running build until fresh deployment-owned
  evidence is appended.
- RealRuntime coherence remains stricter and independent: the current provider
  artifact plus its profile/route/environment identities and a fresh Healthy
  lease must all match.
- ProductionWrite evidence still does not grant action authority. The exact
  Work generation/action/resource/site must independently pass Profile /
  Authority and consume its one-action ExternalActionPermit.

This prevents a readiness result from being assembled out of individually valid
but deployment-incoherent evidence epochs without turning EvidenceClass into an
inheritance or maturity ladder.
