# ADR-039 — Site admission requires a persisted Profile conformance attestation

Status: Accepted for v11.5 convergence.

## Context

A Profile conformance report is meaningful only when the system can answer:
who evaluated it, for which site/Profile, from which evidence, and for what
validity window.

An HTTP admission endpoint that accepts fields such as
`durable_work_state=true`, `provenance_ready=true` or an arbitrary list of
"satisfied semantics" lets the caller self-certify the very guarantees that
SiteAdmission is supposed to protect.

## Decision

1. SiteAdmission through the product/API boundary requires a persisted
   `ProfileConformanceAttestation`.
2. The attestation binds:
   - exact site;
   - exact canonical Profile reference;
   - computed ConformanceReport;
   - concrete evidence references;
   - evaluator identity;
   - evaluation time and optional validity window.
3. A passing ConformanceReport without evidence/evaluator identity is
   insufficient for site admission.
4. The admission endpoint rejects raw conformance booleans/semantic lists.
5. SiteAdmission records the attestation id in `conformance_ref`.
6. Conformance evidence is independent of capability qualification evidence:
   capability tests prove the capability; site/Profile conformance proves that
   the deployment context satisfies the Profile guarantee floor.
7. A failed/expired/site-mismatched attestation fails closed.
8. Fixture conformance attestations are local engineering evidence only and
   cannot be promoted to real-site evidence by relabeling.

## Consequences

- Capability authors cannot self-admit by claiming required semantics.
- A site may reevaluate a Profile independently of rebuilding the capability.
- Profile upgrades naturally require a new/migrated attestation.
- Site admission becomes auditable without hard-coding a specific conformance
  engine vendor.
