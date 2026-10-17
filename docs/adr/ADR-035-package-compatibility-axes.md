# ADR-035 — Package compatibility follows protocol/profile/runtime axes

Status: Accepted for v11.5 convergence.

## Context

The v1 package/plugin manifests use `core_compat: 1.x`. v11.5 no longer
defines Morn by an immutable software Core, so treating that legacy field as
the semantic compatibility contract would contradict the current architecture.

Packages actually cross several independent boundaries:

- public SDK / packaging ABI;
- Morn semantic protocol;
- Domain Profile guarantee contract;
- composition runtime/provider host.

## Decision

1. `core_compat` is retained only as a legacy v1 packaging-ABI field so old
   serialized manifests and tests remain readable.
2. New manifests carry explicit `protocol_compat`.
3. Packages/plugins may declare `profile_compat` and
   `composition_runtime_compat` independently.
4. No package may infer protocol compatibility merely from the Rust crate/API
   version or from Cordis compatibility.
5. Profile compatibility never grants SiteAdmission or Authority; it only
   declares the package's intended compatibility envelope.
6. Runtime/provider compatibility does not rewrite already-pinned execution
   bindings.
7. A future package-format major version may remove the legacy `core_compat`
   field through an explicit migration rather than silently changing its
   meaning.

## Consequences

- Packages can evolve with Morn Protocol and Profiles without resurrecting a
  forever-immutable Core concept.
- A Cordis/DSH/Pi upgrade is distinguishable from a Morn semantic protocol
  migration.
- Legacy v1 package history remains interpretable.
