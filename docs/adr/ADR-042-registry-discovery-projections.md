# ADR-042 — Registry/discovery standards are projections, not qualification truth

Status: Accepted and implemented as a v11.5 projection contract.

## Context

Morn Hub must discover and distribute heterogeneous capabilities, profiles,
providers, compilers and SolutionPackages. The industry is converging on several
metadata/discovery mechanisms with different scopes:

- xRegistry provides a vendor-neutral metadata registry model with Groups,
  Resources, optional Versions, extensible metadata, and symmetric file/API
  representations.
- A2A Agent Cards describe remote agent identity, skills, supported interfaces
  and authentication/discovery metadata.
- OASF provides an extensible schema/taxonomy for describing agentic content,
  skills and domains.

Morn Capability is intentionally broader than an Agent and adds qualification,
site admission, authority/effect ceilings, execution guarantees, provenance and
business-use constraints. None of the discovery formats above is sufficient to
represent Morn governance truth by itself.

## Decision

1. Morn keeps one canonical Capability/Profile/Release/Admission model.
2. Hub exposes **registry projections/adapters**, not a second registry truth.
3. xRegistry is the preferred future generic metadata-registry projection:
   Morn asset families may map to Groups/Resources/Versions where the mapping is
   lossless enough for discovery.
4. An Agent-kind Capability may import/export A2A Agent Card metadata. Agent
   Card completion, skills or authentication declarations never imply Morn
   qualification/admission/authority.
5. OASF skill/domain metadata may enrich Agent-kind Capability discovery, but
   OASF taxonomy is not the canonical Capability ontology.
6. Discovery metadata is always classified as declared metadata until supported
   by Morn evaluation/qualification evidence.
7. Site admission, effect ceiling, Profile conformance and accepted outcomes are
   Morn-only governance semantics unless a future external standard can express
   them with equivalent guarantees.
8. Registry projections are versioned adapters. A projection upgrade cannot
   silently rewrite a released CapabilityManifest or active ExecutionBinding.

## Consequences

- Hub can integrate with emerging enterprise/agent registries without binding
  Morn semantics to one ecosystem.
- Third-party agents can be discovered through A2A/OASF while deterministic
  programs, solvers, humans, devices and APIs remain first-class Capabilities.
- xRegistry can provide a standards-oriented catalog surface without becoming
  Work truth or qualification truth.
- Search/ranking can combine declared discovery metadata with observed and
  qualified Morn evidence while keeping those evidence classes distinct.


## Implementation

The reference adapter contract lives in
`crates/morn-capability/src/discovery.rs`.

It implements:

- xRegistry-style resource projection for all Capability kinds;
- A2A Agent Card projection for Agent-kind capabilities only;
- OASF skill/domain discovery projection for Agent-kind capabilities only;
- declared-only external metadata import.

The serialized projection types deliberately have no qualification, release,
site-admission, Authority or accepted-outcome fields. A projection therefore
cannot be deserialized back into governed truth by accident.
