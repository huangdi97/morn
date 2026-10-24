# ADR-025 — Profiles are published guarantee assets, not hard-coded modes

Status: Accepted for v11.5 convergence.

## Context

Morn Profiles such as Lite, Enterprise, Factory and Research define minimum
semantic/execution guarantees. Treating them as permanent hard-coded enum values
would contradict the v11.5 rule that implementations and domain contracts may
evolve by explicit version.

At the same time, allowing a Profile payload to be edited in place under the
same id/version would silently reinterpret Work and site admissions.

## Decision

1. A DomainProfile is a versioned published guarantee asset.
2. Its canonical identity is `<profile-id>@<semantic-version>`.
3. A runtime ProfileRegistry may hold multiple versions of one Profile family.
4. Registering the same canonical reference twice is rejected; changed content
   must publish a new version.
5. A Profile validates its semantic requirements, execution class/guarantee
   vector and duplicated convenience flags before publication.
6. Work pins an exact Profile reference.
7. Profile migration follows ADR-024 and creates a new Work desired-state
   generation / new execution bindings when the version changes.
8. Built-in Lite/Enterprise/Factory/Research Profiles are reference assets, not
   an architectural claim that only those four Profiles may ever exist.

## Consequences

- Morn Factory Profile can evolve without changing Morn's semantic constitution.
- Third-party/domain Profiles can be added later without creating a new
  immutable kernel.
- Studio/Console should read the published Profile catalog rather than assume
  one eternal compile-time list.
- SiteAdmission remains scoped to the exact Profile reference that was
  conformance-tested.
