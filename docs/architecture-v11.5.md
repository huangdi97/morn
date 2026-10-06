# Morn v11.5 Architecture — Protocol-driven Work Control Plane

Status: implementation convergence baseline. This document does not upgrade any
customer/production evidence claim.

## Product definition

Morn is a protocol-driven, outcome-oriented work control plane. It composes
humans, agents, deterministic rules, solvers, software services and machines
into governed real-world work without making any one harness or workflow engine
the canonical source of business truth.

Four rules guide the architecture:

1. Work is the unit of coordination.
2. Capability is the unit of composition.
3. Accepted outcome is the unit of value.
4. Agent is one kind of executor.

## Planes

### Semantic / Specification Plane
Versioned contracts describe Work, Capability, Authority, ExecutionBinding,
Attempt, Receipt, Reconciliation, Outcome, Acceptance, Provenance and Profile.

### Durable Control Plane
Work uses desired/observed state, generation and Conditions. Controllers
reconcile observations toward desired state. Agent sessions, workflow cursors
and sandbox instances are runtime state and cannot overwrite Work truth.

### Composition Plane
The reference runtime uses Cordis for local Context/Service/dependency/Fiber
lifecycle. Cordis reversible effects are software lifecycle effects, not a
claim that real-world business effects are reversible. Domain/control-plane
crates do not import Cordis directly.

### Execution Plane
Harnesses (DeepSeek Harness, Pi), solvers, deterministic programs, humans,
workflow engines, connectors and physical executors are providers. DSH is
integrated out-of-process through its public SDK/ACP wire boundary.

### Trust Plane
Policy decision and policy enforcement are separate. External side effects pass
through an enforcement point; a model/harness never gains production authority
merely because it can call a tool.

### Capability Supply Chain
Artifacts can become candidate capabilities, then be evaluated, qualified,
released and admitted. Declared, observed, qualified and admitted are distinct.

## Work control model

A Work resource has `spec` and `status` plus a monotonically increasing
`generation`. Controllers update Conditions and `observed_generation`.
Changing desired Work creates a new generation.

A running action never hot-swaps providers. `ExecutionBinding` pins Work
id/generation, capability manifest, provider version/digest, runtime reference,
authority decision and profile. Provider change creates a new binding and
usually a new attempt or explicit migration.

## External action truth

`PROPOSED -> AUTHORIZED -> DISPATCHED -> ACKNOWLEDGED -> COMMITTED -> OBSERVED -> VERIFIED`

Ambiguity is first-class:

`DISPATCHED/ACKNOWLEDGED/COMMITTED -> OUTCOME_UNKNOWN -> RECONCILING`

A timeout after a remote commit is not automatically failure and must not be
blindly retried. Reconciliation queries the authoritative external system using
a business/idempotency key and records evidence.

## History and versioning

Current projections are mutable. Published contracts evolve by version. A
running binding is pinned within its execution scope. Historical facts may be
superseded, retracted, migrated or redacted, but not silently overwritten.

## Capability model

A Morn Capability contains implementation/provider reference, machine-readable
manifest, interface contract, execution requirements, authority envelope,
economics, provenance and qualification/admission evidence. Capabilities include
agents, rules, programs, solvers, models, APIs, services, humans, devices and
hybrids.

## Profiles

A Domain Profile is a guarantee set, not a plugin list. The first Factory
read-only profile requires durable Work state, source-of-truth binding,
capability qualification, authority before side effect, receipt/reconciliation,
outcome observation, independent acceptance and provenance. It does not grant
production-write authority.

## Reuse before reinvention

Prefer Cordis for local composition; DSH/Pi for harnesses; MCP/OpenAPI/A2A for
interfaces; CloudEvents and OpenTelemetry for events/telemetry; OPA/Cedar for
policy decisions; OCI/ORAS plus Sigstore/SLSA for capability distribution and
provenance; Temporal/Dapr when a durable workflow provider is actually needed.

## First Factory wedge

The first product slice remains brownfield and read-first: outage/insert-order
exception -> capacity -> delivery-impact review. Production write remains out
of scope until explicit site authority and evidence gates exist.
