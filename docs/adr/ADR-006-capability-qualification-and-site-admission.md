# ADR-006 — Separate Capability Compilation, Observation, Qualification, Release and Site Admission

Status: Accepted for v11.5.

## Decision

The capability supply chain is intentionally non-collapsed:

```text
Artifact
 -> compile
 -> Declared
 -> evaluate / observe
 -> Observed
 -> independently qualify
 -> Qualified
 -> content-addressed release
 -> Released
 -> Profile conformance
 -> SiteAdmission(site, profile)
 -> Admitted
```

Each transition proves a different fact.

Compilation proves only that a source artifact can be represented as a candidate
manifest. It cannot qualify itself.

Observation records evaluation evidence and evaluator identity. The strict
qualification path rejects a candidate that has never reached `Observed`.

Qualification records carry test suites, environment digest, expected
properties, known failure modes, evaluator identity, context of use and
validity. Legacy/minimal qualification may remain readable for compatibility
but is insufficient for strict site admission.

Release binds a qualified manifest to a content-addressed package digest and
build/provenance reference. Qualification is therefore not a package identity.

Site admission binds an active release to an exact site and guarantee-profile
reference after Profile conformance. A capability qualified or admitted for one
site/profile does not automatically inherit admission elsewhere.

Release revocation suspends dependent site admissions without deleting the
qualification, release or historical decision records.

## Consequences

A generated skill, agent, API wrapper, paper-derived capability or repository
entrypoint cannot self-promote to production use.

Paper2Agent-style generation is useful in the compilation/evaluation half of
the lifecycle, while Morn retains independent qualification, release,
conformance and admission gates.

Fixtures may prove the lifecycle mechanics but cannot turn local engineering
evidence into a customer/site evidence claim.
