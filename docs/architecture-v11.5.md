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


## Reference landscape and boundary decisions

Morn deliberately reuses mature mechanisms rather than turning every adjacent
problem into a Morn-specific subsystem.

| Concern | Reference/standard | Morn decision |
| --- | --- | --- |
| node-local dynamic composition | Cordis | direct reference-runtime dependency behind `runtime/cordis-host`; not business truth |
| agent harness | DeepSeek Harness, Pi, AgentScope-compatible providers | HarnessProvider boundary; no harness session may own Work truth |
| capability description/search | Learnware pattern | Capability = implementation + specification + qualification/admission evidence |
| artifact -> executable capability | Paper2Agent pattern | generalized into Artifact2Capability compilers; output stays candidate until qualified |
| isolated execution | DSec/OpenSandbox/container/microVM pattern | ExecutionEnvironmentProvider chooses isolation from requirements |
| durable control | Kubernetes controller/reconciliation pattern | desired/observed Work state and explicit Conditions |
| optional durable workflow | Temporal/Dapr-class provider | provider behind Work semantics; never canonical Work identity |
| tool/resource protocol | MCP/OpenAPI | capability interface protocols, not Morn-specific RPC |
| agent peer protocol | A2A | optional external-agent provider boundary |
| event envelope | CloudEvents 1.0 shape | Morn event data in standard envelope + Morn extensions |
| telemetry | OpenTelemetry | Morn semantic attributes layered over OTel |
| policy decision | OPA/Cedar/customer IAM | AuthorityProvider; enforcement remains in Morn action boundary |
| workload identity | SPIFFE-class identity | future transport/security provider; credentials never become agent memory |
| package distribution | OCI/ORAS | content-addressed capability package descriptor |
| signatures/provenance | Sigstore/SLSA | external signature/provenance refs, not proprietary formats |

AgentScope and similar production agent frameworks are useful provider examples,
not architectural parents. Their agents, pipelines, SOPs, sandboxes, MCP/A2A
support and service control planes reinforce the need for a neutral Provider
Fabric, but Morn remains Work-first rather than Agent-first.

## What Cordis is and is not

Cordis is the default **composition runtime** for the reference host. It owns
software-lifecycle concerns such as Context, Service slots, dependencies,
plugin/Fiber lifetime and reversible registration effects.

Cordis does **not** own:

- Work identity or generation;
- durable business state;
- authority semantics;
- external action truth;
- receipt/reconciliation;
- accepted outcome;
- capability qualification/site admission;
- historical interpretation.

A Cordis provider hot-swap may change what can be selected for a **future**
binding. It may not mutate an active `ExecutionBinding` or continue an
ambiguous external action under a different provider without an explicit new
binding/migration decision.

## Harness neutrality

The shared HarnessProvider contract is intentionally smaller than DSH, Pi or
AgentScope. A provider may expose richer features internally, but Morn relies
only on the normalized boundary.

Required Morn-side semantics include:

- mount/unmount scoped capability lifecycle;
- start/send/inspect/interrupt/resume/terminate;
- normalized execution events;
- execution receipt;
- explicit runtime context;
- no direct mutation of canonical Work state.

The repository includes a DSH-vs-Pi neutrality test using deterministic fixtures.
This proves contract neutrality only. It does not claim model-output parity or a
successful real DSH/Pi deployment.

## Capability supply chain

The lifecycle is intentionally split because each transition proves a different
fact:

```text
Artifact
  -> Compiler
  -> Declared candidate
  -> Observed in evaluation
  -> Qualified
  -> Released/package-addressable
  -> Profile conformance
  -> Site admitted
  -> Suspended/Retired when required
```

Compilation is not qualification. Qualification is not release. Release is not
site admission. Site admission is scoped to a concrete site/profile and may be
suspended without deleting qualification history.

The repository maps existing CertificationService / CapabilityRelease evidence
into a normalized `QualificationRecord` and then a separate `SiteAdmission`.
This avoids creating a second certification engine while preserving the v11.5
semantic distinction.

## Artifact2Capability

Artifact2Capability is a compiler family, not an automatic production-deployment
mechanism.

Planned/allowed compiler kinds include:

- OpenAPI -> Capability candidate;
- repository -> Capability candidate;
- SOP/procedure -> Procedure capability;
- paper + code -> reviewed skill/MCP-style capability;
- model -> model capability;
- workflow -> workflow capability.

The first concrete implementation is a conservative OpenAPI JSON compiler. It
only discovers operations that actually exist in the source document, records
source provenance, emits a `Declared` candidate and explicitly leaves
qualification/admission unresolved.

## Execution environments

Execution isolation is a requirement in `CapabilityManifest` and a selectable
provider at runtime.

```text
Process < Container < microVM < Full VM
```

Remote and physical executors are distinct environment classes rather than
pretending every workload is a local sandbox. A profile may require a minimum
isolation floor. A provider that cannot meet the floor is ineligible.

DSH/Pi sandbox features are therefore execution-provider details, not the
Factory safety boundary.

## Authority and enforcement

Morn separates:

```text
AuthorityProvider -> decision
Action/Connector boundary -> enforcement
```

The reference implementation uses the native Policy engine through an
AuthorityProvider adapter. OPA, Cedar or customer IAM can implement the same
decision contract later.

A model, harness or capability may produce a proposal. It does not gain real
authority merely because it knows how to call a tool. Production credentials
belong to the enforcement/execution boundary and should be represented by
secret/workload-identity references rather than copied into prompts or agent
memory.

## Event, telemetry and package interoperability

Integration events use a CloudEvents-compatible core envelope with Morn
extension attributes such as work, attempt, binding, profile and trace ids.

Capability distribution is modeled as content-addressed OCI-style artifacts.
The package descriptor can carry:

- content digest;
- layers/media types;
- SBOM reference;
- SLSA provenance reference;
- signature reference.

Morn does not implement its own signature transparency log or build-provenance
standard.

## Factory Profile as a guarantee profile

`morn.factory.readonly@1.0.0` describes guarantees, not implementation names.
A conformant composition must satisfy the required semantic Conditions,
durability/provenance requirements and isolation floor regardless of whether it
uses DSH, Pi, OPA, Cedar, Temporal, Dapr or customer-specific providers.

The first profile remains read-first. Fixture tests may exercise a simulated
external action/reconciliation path, but this is engineering evidence only and
does not authorize real production writes.

## Reference Factory flow

```text
SCADA/MES/CMMS observation
  -> normalized event
  -> Situation/Work creation
  -> desired Work spec + profile
  -> CapabilityResolver
  -> Qualification + SiteAdmission check
  -> Authority decision
  -> ExecutionEnvironment selection
  -> pinned ExecutionBinding
  -> DSH/Pi/solver/human workcell
  -> candidate recommendation / evidence
  -> optional governed external attempt
  -> timeout-after-commit => OUTCOME_UNKNOWN
  -> ReconciliationController queries authoritative system
  -> Receipt / verified observation
  -> OutcomeRecord
  -> independent AcceptanceSpec + human/business acceptance
```

The same Work remains identifiable if a harness crashes. A replacement provider
requires a new binding; the old attempt is never silently rewritten.

## Product surfaces

All product surfaces are views over the same semantics:

- **Workbench**: Work, desired/observed state, Conditions, binding, attempts,
  evidence, outcomes and acceptance.
- **Studio**: starts from the work goal or an existing artifact, compiles
  candidate capabilities/solutions, exposes unresolved qualification gates.
- **Console**: provider health, composition runtime, profiles, conformance,
  authority, bindings, attempts, reconciliation and external blockers.
- **Hub**: capabilities, releases, harness providers, composition runtimes,
  compilers, profiles and reusable packages.

No surface is allowed to create a parallel domain model.

## Implementation status on the convergence branch

Implemented locally on the v11.5 branch:

- versioned ProtocolSnapshot and explicit history-mutation semantics;
- Work spec/status/generation/Conditions;
- deterministic CapabilityManifest + resolver;
- ExecutionBinding, ActionAttempt and reconciliation;
- durable control-plane persistence adapter;
- exact-pinned Cordis reference host;
- DSH fixture provider and Pi fixture provider;
- DSH/Pi harness-neutrality contract benchmark;
- provider-neutral AuthorityProvider reference adapter;
- ExecutionEnvironmentProvider abstraction;
- profile conformance evaluator;
- QualificationRecord + SiteAdmission bridge;
- OpenAPI2Capability candidate compiler;
- CloudEvents-compatible event envelope;
- OCI/Sigstore/SLSA-oriented capability package descriptor;
- Factory read-only vertical-slice integration test;
- API/UI exposure in Workbench, Studio, Console and Hub.

Not claimed complete without external evidence:

- real DSH process integration;
- real Pi transport integration;
- real factory/customer data;
- real site IAM/OPA/Cedar deployment;
- real OCI registry publishing/signing;
- real microVM/DSec-class execution backend;
- any production write or physical control.

## Engineering invariants

1. Work truth outlives harness sessions.
2. Provider replacement never edits an active binding in place.
3. Unknown external outcome is never blindly retried.
4. Compilation never implies qualification.
5. Qualification never implies site admission.
6. Profile conformance is provider-name independent.
7. Authority decision is separate from enforcement.
8. Cordis reversible lifecycle effects are not business rollback.
9. Current projection may change; historical correction is explicit.
10. External blockers stay external blockers and cannot be converted to PASS by
    fixtures or documentation.
