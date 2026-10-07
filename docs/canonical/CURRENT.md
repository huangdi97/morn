# Morn Current Canonical Engineering Baseline

Current design line: **v11.5 protocol-driven Work Control Plane convergence**.

The authoritative architecture entry points for this branch are:

1. `docs/architecture-v11.5.md`
2. `docs/adr/ADR-001-v11.5-control-plane-cordis.md`
3. ADR-002 through ADR-015 in `docs/adr/`
4. `docs/security/MORN_V11_5_THREAT_MODEL.md`

Historical v10.x/v11.3 documents and v1 GA evidence remain readable evidence
and compatibility baselines. They are not silently inherited as current v11.5
requirements. Existing validated code/contracts are preserved or explicitly
mapped rather than rewritten merely to match new terminology.

## Current architecture definition

Morn =
Semantic Specification
+ Durable Work Control Plane
+ Reference Composition Runtime
+ Provider Fabric
+ Capability Supply Chain
+ Domain Guarantee Profiles.

Reference composition runtime: Cordis, behind an adapter/boundary.
Harnesses: replaceable providers (Morn Native, DSH, Pi, others).
Factory first wedge: brownfield/read-first outage/insert-order -> capacity ->
delivery-impact review.

## Evidence discipline

Local fixture/conformance/CI evidence proves engineering properties only.
It does not imply real customer data, production write, real factory outcome or
customer value validation.


## Interoperability boundary

MCP/OpenAPI/A2A are interoperability protocols, not replacements for Morn
business semantics. MCP tools contribute capability interfaces; A2A Tasks and
Artifacts contribute executor state/evidence. Neither protocol can grant
Authority or make an external task completion equal Morn Outcome/Acceptance.
See `ADR-011-mcp-a2a-interoperability-boundaries.md`.


## Execution trust clarification

Execution topology and containment guarantees are not one scalar rank. Remote
and physical executors do not automatically satisfy container/microVM/VM
requirements. Runtime workload identity is separate from canonical business
identity; the reference provider shape is SPIFFE-compatible, while secrets stay
behind opaque credential handles. See ADR-012 and ADR-013.


## Blueprint / instance convergence

User-facing “blueprint” maps to an approved `SolutionPackage`; a runtime
“instance” maps to canonical `WorkResource`. Work stores
`source_solution_ref` for package provenance. Workcell remains a
minimum-sufficient executor composition for a Work, not a second source of
business truth. See ADR-015.

## Execution guarantees

Execution environment topology and security guarantees are separate. Profiles,
capability manifests and environment providers use a typed execution-guarantee
vector; Factory read-only currently requires filesystem-write policy,
network-egress policy and secret indirection in addition to its containment
floor. See ADR-014.
