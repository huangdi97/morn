# Morn Current Canonical Engineering Baseline

Current design line: **v11.5 protocol-driven Work Control Plane convergence**.

The authoritative architecture entry points for this branch are:

1. `docs/architecture-v11.5.md`
2. `docs/adr/ADR-001-v11.5-control-plane-cordis.md`
3. ADR-002 through ADR-024 in `docs/adr/`
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
floor. See ADR-020.


## Semantic constitution

Implementations are replaceable; published semantic laws are non-bypassable
within a protocol version. Work/Authority/Binding/external-effect truth/
Outcome/Acceptance/history/profile semantics cannot be silently redefined by a
plugin. See ADR-016.


## Digital employee mapping

“Digital Employee” is a product/organization projection over the existing
`RoleSlot + MemberBinding` semantics. It is not a new AgentInstance/Work source
of truth. The concrete Workcell for a Work still resolves the minimum-sufficient
mix of human, agent, rule, solver, program, service or device capabilities. See
ADR-017.


## Profile action intent

Profile-level effect permission and IAM/Authority are independent gates.
`morn.factory.readonly@1.0.0` structurally forbids ProductionWrite even when a
principal would otherwise have enterprise write permission. Fixture/simulation
writes are classified separately and cannot upgrade the production claim. See
ADR-018.


## Durable event delivery

CloudEvents defines the envelope, not exactly-once processing. The reference
store now has a durable inbound-event claim/dedupe table and outbound-event
outbox. Transport retries retain stable event identity; event delivery success
still does not imply real-world action success. See ADR-019.


## Composition identity boundary

Composition ownership is not business identity. Cordis Context/Fiber selects
software registration/lifecycle ownership only; Morn Work, Actor,
ExecutionBinding and Attempt identities remain explicit typed references.
Provider hot reload cannot infer or mutate business identity. See ADR-014.


## Durable control-plane concurrency

Mutable Work projections use an explicit persistence revision and compare-and-
swap writes. Work generation remains a desired-spec generation, not a storage
revision. Redundant controllers coordinate with expiring leases plus monotonic
fencing tokens; stale leaders cannot overwrite newer control state. This is
separate from business Authority. See ADR-021.


## Work termination

Work termination is a two-phase control-plane operation. Termination intent is
durable and monotonic; finalizers keep the Work in `Terminating` until
outstanding control obligations are resolved. Finalization does not erase
Attempts, Receipts, Outcomes or other history and does not pretend irreversible
effects were rolled back. See ADR-022.


## Execution interpretation provenance

A concrete governed execution binding may emit a persisted
`ExecutionManifest` that pins protocol version, Work generation, Profile,
site, source SolutionPackage, capability, provider version/digest, composition
runtime and Authority reference. It is provenance for replay and migration, not
a second business-truth record. See ADR-023.


## Protocol and Profile evolution

Morn does not freeze one implementation forever. Instead, published protocol and
Profile releases have explicit compatibility semantics. Same-version semantic
mutation is incompatible; patch changes cannot alter guarantee meaning; minor
changes require reevaluation; major/cross-Profile changes are incompatible by
default. Running bindings stay pinned and new versions require explicit migration.
See ADR-024.
