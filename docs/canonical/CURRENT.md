# Morn Current Canonical Engineering Baseline

Current design line: **v11.5 protocol-driven Work Control Plane convergence**.

The authoritative architecture entry points for this branch are:

1. `docs/architecture-v11.5.md`
2. `docs/adr/ADR-001-v11.5-control-plane-cordis.md`
3. ADR-002 through ADR-010 in `docs/adr/`
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
