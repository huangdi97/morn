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

Trust evidence follows the same append-only rule. `EvidenceClaim` history is
scoped by subject + evidence class. The latest appended claim is the current
state for that pair: `Proven`, `BlockedExternal`, or `Revoked`. A revoked
or newly blocked claim immediately stops satisfying current gates, while the
older proof remains auditable. Re-establishing trust requires an explicit later
`Proven` claim with fresh evidence; stale older proof cannot be replayed.
Historical `CustomerValidated` ValueAssessment records are never rewritten,
but product surfaces must separately project whether their exact RealSite
support is **currently** proven.

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

The repository has two provider-neutrality layers:

- deterministic Fixture providers prove the shared baseline `HarnessProvider`
  contract independently of any external runtime;
- DSH SDK and Pi RPC **real-adapter wire fixtures** drive the actual subprocess
  adapters with the same pinned Workspace/Work generation/ExecutionBinding and
  execution-environment guarantees. This second gate requires exact runtime
  version/digest identity, E0-only admission, fully normalized execution events,
  no direct tool activity, and a settled healthy adapter turn.

Neither layer compares model text. The adapter-wire gate proves Morn-side
transport/semantic neutrality only; it does not claim authenticated model
parity, live credentials, customer Outcome/Acceptance, or a successful
production DSH/Pi deployment.

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

Implemented compiler kinds include:

- OpenAPI -> Capability candidate;
- repository -> Capability candidate;
- SOP/procedure -> Procedure capability;
- paper + code -> reviewed skill/MCP-style capability;
- model -> model capability;
- workflow -> workflow capability.

The implemented compiler family includes conservative OpenAPI JSON,
structured SOP/procedure, explicit repository-manifest, reviewed
paper-manifest, pinned model-manifest and durable workflow-manifest compilers. They only consume declared/reviewed source facts,
record provenance, emit `Declared` candidates and explicitly leave
qualification/release/site-admission unresolved. Paper-derived executable
candidates require a real code binding; free-form paper text alone is not
promoted into an executable capability.

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

Capability distribution is modeled as content-addressed OCI artifacts. Morn's
distribution boundary invokes ORAS as an explicit subprocess with an empty
inherited environment, parses its JSON receipt and records only a digest-pinned
subject reference. The package descriptor can carry:

- content digest;
- layers/media types;
- SBOM reference;
- SLSA provenance reference;
- signature reference.

Signature and provenance are independent trust axes. The reference verifier
uses Cosign against the exact digest and an explicit certificate identity/OIDC
issuer; SLSA provenance is verified separately and may be merged only when both
proofs bind the same subject digest. Deployment-generated verification evidence
is loaded out-of-band and HTTP callers cannot self-assert it. Site admission
requires both verified axes.

Morn does not implement its own signature transparency log or build-provenance
standard and does not mint Sigstore/OIDC identity itself.

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
- DSH fixture providers plus real out-of-process DSH SDK and Pi JSONL-RPC
  transport adapters with bounded turns, explicit runtime configuration and
  scrubbed child-process environments;
- DSH/Pi real-provider E0 authority ceiling at the HarnessProvider seam; E1/E2/E3
  effects remain governed ExternalAction operations;
- durable DSH public event normalization into Morn audit events without copying
  assistant text, tool arguments/results or private reasoning;
- DSH/Pi harness-neutrality contract benchmark;
- provider-neutral AuthorityProvider reference adapter;
- Work-scoped credential and SPIFFE-compatible workload-identity provider seams;
- ExecutionEnvironmentProvider abstraction with explicit guarantee-vector enforcement;
- profile conformance evaluator;
- QualificationRecord + SiteAdmission bridge;
- OpenAPI2Capability, SOP2ProcedureCapability, Repo2Capability and
  ReviewedPaper2Capability candidate compilers;
- CloudEvents-compatible event envelope;
- OCI/Sigstore/SLSA-oriented capability package descriptor;
- ORAS publication adapter with explicit environment isolation and digest-pinned
  publication receipts;
- independent Cosign signature and SLSA-provenance verification bound to the
  exact OCI digest, deployment-owned verification catalog, and SiteAdmission
  enforcement;
- simple Creator draft -> existing solution pipeline;
- approved SolutionPackage -> persisted Work instantiation;
- Factory read-only vertical-slice integration test;
- API/UI exposure in Workbench, Studio, Console and Hub.

Not claimed complete without external evidence:

- authenticated live DSH runtime/model/credential smoke and runtime health lease;
- installed/live Pi runtime/model/credential smoke and runtime health lease;
- a real execution-environment backend proving the isolation/egress/secret
  guarantees required by the selected Profile (the repository fixture provider
  is not a production sandbox);
- real factory/customer data;
- real site IAM/OPA/Cedar deployment;
- authorized live OCI registry publication plus real Sigstore/OIDC signature
  and SLSA provenance verification evidence (the adapters/gates exist locally,
  but no registry identity or deployment credentials are fabricated in CI);
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


## Execution trust model refinement

Recent sandbox/runtime systems reinforce that execution backends are
heterogeneous rather than one security ladder. Morn therefore does not rank
`Remote` or `Physical` above `microVM`/`FullVM`. Capability resolution
and Profile conformance require an explicitly compatible execution class, and
future network/kernel/attestation guarantees are independent properties.

Runtime workload identity is also separated from business identity. Morn
retains Principal/Actor accountability while a replaceable
`WorkloadIdentityProvider` can issue short-lived SPIFFE-compatible identities
for service-to-service execution. Credentials remain opaque, Work/site/action
scoped handles resolved only at the enforcement boundary.

AgentScope Runtime, DSec-like elastic sandbox infrastructure and other
production agent runtimes are treated as possible execution/provider families,
not as architectural parents. Their state persistence, interruption, sandbox,
MCP/A2A and deployment services can be integrated behind Morn contracts without
turning Agent state into Work truth.


## Agent is an explicit executor kind

Morn does not infer "agent" from the presence of an LLM or model. Capability
kinds distinguish `Agent`, `Llm` and `Model`:

- `Agent`: an autonomous/semi-autonomous executor with an agent/harness loop;
- `Llm`: a language-model inference capability;
- `Model`: a non-agent model/predictor/classifier capability.

A Workcell's agent count therefore counts only explicit `Agent` capabilities.
A model + deterministic controller does not become an agent by naming
convention, and a zero-agent Workcell remains a first-class valid plan.


## Blueprint and instance semantics

Morn does not add parallel `Blueprint` or `Instance` canonical records.
An approved `SolutionPackage` is the reusable blueprint; instantiation creates
a normal `WorkResource` with `source_solution_ref` pointing back to the exact
package/version.

```text
Intent / Artifact
 -> Studio
 -> SolutionPackage
 -> WorkResource
 -> CapabilityResolver
 -> Workcell
 -> ExecutionBinding
 -> Attempt
 -> Outcome
 -> Acceptance
```

Instantiation is deliberately non-executing. The Work starts `Proposed`.
Profile-derived pre-execution gates are attached to Work readiness; binding,
receipt, reconciliation, outcome and acceptance remain later control-plane
conditions. This lets a user create simple reusable experiences without making
Morn Agent-first.

## Multidimensional execution guarantees

The reference design now separates execution topology from security/operational
guarantees. A typed `ExecutionGuarantee` vocabulary covers filesystem policy,
network egress, process/kernel boundaries, resource limits, secret indirection,
workload identity, stateful execution, checkpoint/resume and runtime
attestation.

Capability requests/manifests, Domain Profiles and
`ExecutionEnvironmentProvider` use the same guarantee vocabulary. A provider
that cannot prove a required guarantee is ineligible even if its topology label
sounds stronger. The Factory read-only profile currently requires
filesystem-write policy, network-egress policy and secret indirection.

This is intentionally compatible with heterogeneous execution systems such as
containers, microVMs, DSec-like elastic sandbox platforms and AgentScope-style
runtime providers without hard-wiring any one of them into Morn semantics.


## Creator product layer

Creator is the lightweight entry point for users who do not want to manually
assemble WorkGraph, harness, runtime and provider details. A Creator request
contains goal, guarantee profile, acceptance criteria, optional site,
capability hints and autonomy posture. It translates into the existing
ProblemSpec/WorkGraph/ProposedSolution pipeline.

Creator does not write canonical Work and does not create a new AgentInstance
truth model. Review/approval produces the existing SolutionPackage; only
SolutionPackage instantiation creates canonical WorkResource. This keeps the
product simple while preserving one business object model.

See ADR-014, ADR-015, ADR-016 and ADR-020.


## Digital employee / role interface

Digital Employee is retained as a useful organizational/product surface, but it
does not become a new canonical record family. The reference mapping is:

```text
Digital Employee view
  = RoleSlot
  + MemberBinding
  + projected responsibilities/capability requirements/authority ceiling
```

The role is stable while execution is dynamic. One role may participate in
different Workcells and may use DSH, Pi, deterministic programs, solvers or
other providers across Works. A Workcell is assembled around a concrete Work
using the minimum-sufficient capability mix and can legitimately contain zero
agents.

This keeps Role as the human-facing organizational interface, Work as the
coordination truth, Capability as the composition unit, and Harness sessions as
replaceable runtime state.


## Profile action intent vs authority

Morn evaluates two independent questions before a governed external effect:

1. **Profile gate** — is this effect mode permitted by the exact Domain Profile?
2. **Authority gate** — is this principal/delegation allowed to perform the
   specific action/resource under the current Work/Binding context?

The current Factory read-only profile forbids `ProductionWrite` regardless of
IAM permissions. Deterministic fixture paths use `SandboxWrite`, which allows
reconciliation/failure tests without implying production authority. Physical
control fails closed unless a future exact profile explicitly enables it.


## Durable controller event delivery

The reconciliation control plane assumes at-least-once event delivery and
restart. The reference store therefore provides:

```text
inbound event
 -> stable event-id claim / dedupe
 -> idempotent controller reconcile
 -> canonical state mutation
 -> durable outbox delivery intent
 -> transport retry with same event id
```

This is deliberately separate from `ActionAttempt` effect idempotency. An
event being delivered exactly once does not prove an ERP/MES/CMMS action
happened exactly once; external-effect ambiguity still uses business keys,
Receipt and Reconciliation.


## Composition identity is explicit

Cordis Context/Fiber scope and Morn business identity are deliberately
orthogonal. A composition context owns plugin registrations and cleanup; it does
not select the Work, Principal, Actor, Authority or ExecutionBinding for an
operation.

This mirrors the direction in recent DeepSeek Harness architecture, where
runtime Agent identity is passed explicitly rather than inferred from a Cordis
Context. Morn applies the same principle to a broader Work-first system.

Therefore:

```text
Cordis Context/Fiber
    -> software ownership / lifecycle

Morn RuntimeContext
    -> workspace + actor + work + policy/provenance scope

ExecutionBinding / Attempt
    -> exact capability/provider/runtime/authority identity
```

No reverse lookup from Cordis Context to canonical Work identity is permitted.
If the composition provider changes, business identity survives and a new
binding is required before future execution can use the replacement.


## Provider fabric registry

Provider identity is runtime/composition metadata, not business identity.
The reference runtime therefore maintains a typed provider catalog with:

- provider family;
- exact version and optional content digest;
- endpoint/protocol metadata;
- declared feature set;
- current health projection;
- observation/evidence history.

Only a healthy provider that satisfies the requested feature contract is
selectable for a future binding. Provider health changes do not rewrite existing
`ExecutionBinding` or `ActionAttempt`; they affect future resolution or
trigger an explicit rebind/migration decision.

The provider registry is intentionally separate from the Capability Registry:
one provider may implement many capabilities, and one capability may be
available through multiple providers. Cordis service presence is one local
provider observation source, not the canonical provider/business database.

## Accepted outcome closure

Final acceptance is stricter than an `Accept` enum value. A Work reaches the
Accepted phase only when:

1. the AcceptanceDecision belongs to the Work;
2. its AcceptanceSpec matches the Work's pinned acceptance reference when one
   is declared;
3. it explicitly references a source-grounded ObservedOutcome for that Work;
4. that same outcome is present as control-plane evidence.

An agent/task/session reporting success can therefore never close Work by
manufacturing an unlinked outcome id.


## Control-plane concurrency and fencing

A Kubernetes-like desired/observed model also needs Kubernetes-like conflict
discipline. Morn therefore separates:

```text
Work.generation       desired-spec generation
resource_version      mutable projection CAS revision
controller fence      controller leadership epoch
ExecutionBinding      exact execution identity
```

They are not interchangeable.

The reference SQLite store now provides:

- compare-and-swap projection writes;
- durable controller leases;
- monotonic fencing tokens on takeover;
- inbox dedupe and transactional-intent outbox.

A stale controller cannot overwrite a newer Work projection merely because it
started earlier. A stale controller lease also does not grant Authority or make
an external effect valid: real actions still traverse profile, authority,
credential and binding enforcement gates.


## Graceful Work termination

A Work is not an ephemeral process and therefore cannot be deleted like an
agent session. The reference control resource carries monotonic termination
intent plus qualified finalizers. Controllers register cleanup/reconciliation
obligations before termination; a terminating Work remains addressable until
the finalizer set is empty. Terminal cancellation is still retained as
historical business state.

This borrows the finalizer pattern used by mature reconciliation control planes
while preserving Morn-specific semantics: finalization means consequence
accounting is complete, not that every real-world effect was undone.


## Execution interpretation manifest

Provider-neutral composition still needs reproducible interpretation. A v11.5
`ExecutionManifest` is therefore derived from a pinned ExecutionBinding and
records the protocol version, Work generation, Profile/site, source
SolutionPackage, capability manifest, provider version/digest, composition
runtime and Authority decision reference.

The manifest is durable provenance only. It does not own Work phase, external
action truth, Outcome or Acceptance. If a Work generation/profile/site changes,
the old binding cannot mint a current manifest; explicit rebind/migration is
required.


## Protocol / Profile migration semantics

The architecture intentionally has no forever-immutable implementation.
Stability comes from explicit published contract identity and migration.

The reference compatibility rules are:

- same version + changed semantic content = incompatible silent mutation;
- semantic-equivalent patch = compatible;
- minor semantic/guarantee change = requires reevaluation;
- major or cross-Profile change = incompatible by default.

A Profile version change creates a new Work desired-state generation and new
ExecutionBindings; active Attempts remain pinned to the old binding. Site
admission is repeated when the Profile guarantee contract changes. Execution
manifests record protocol version plus invariant ids so historical execution can
be interpreted under the contract that actually governed it.


## Profile registry and extension

Domain Profiles are published guarantee assets. The reference runtime ships
Lite, Enterprise, Factory-readonly and Research Profiles, but they are not a
closed enum. `ProfileRegistry` can hold explicit multiple versions of a
Profile family and rejects same-reference overwrite.

This lets Factory or future domain Profiles evolve independently of provider
implementations while preserving exact Work/SiteAdmission interpretation.
Studio profile selection is driven by the backend versioned catalog rather than
assuming one permanent UI list.


## Runtime event semantics and provider liveness

Cordis and harness runtimes may expose high-volume plugin/session/tool/model
events. These do not share one truth level with Morn's durable Work history.
Morn classifies event meaning explicitly:

```text
RuntimeSignal            -> ephemeral runtime/telemetry evidence
ControlIntent            -> durable desired-state/control request
DomainFact               -> durable Morn semantic fact
ExternalObservation      -> durable source-grounded fact/receipt
ProjectionNotification   -> derived best-effort UI/projection signal
```

CloudEvents remains the envelope. The durable outbox is reserved for semantic
event classes that require durability; a runtime signal cannot become business
truth merely because it is persisted or delivered.

Provider selection is similarly temporal. ProviderRegistry health can have a
validity deadline. Strict Work compositions apply a ProviderGate before
CapabilityResolver, so an admitted capability backed by an unavailable or stale
provider is not selectable for a new binding. Provider loss affects future
selection; it does not mutate the binding of an already-started Attempt.

For HTTP MCP, credentials are resource/audience-bound opaque handles. Morn does
not store/pass raw bearer tokens through Work, Capability or agent state.


## Atomic control transition delivery

The reference SQLite store now provides an atomic CAS + durable semantic outbox
primitive. When a controller transition and semantic event belong to the same
logical state change, both are written in one transaction. Transport dispatch
happens after commit with stable event identity. Runtime-only signals are
rejected from this path.

## Pi public RPC provider boundary

The reference runtime now contains a Rust client for Pi's current public
subprocess protocol:

```text
pi --mode rpc --no-session
stdin/stdout = strict LF-delimited JSON
```

The boundary implements correlated commands, `get_state`, prompt acceptance,
abort and collection through `agent_settled`. Pi's prompt response is only
acceptance/queue status; `agent_settled` is only executor-idle status. Neither
becomes Morn Outcome or Acceptance.

Morn intentionally binds to the CLI/wire contract rather than an in-process
TypeScript package name so Pi packaging/repository changes do not redefine the
Morn protocol.


## Evidence classes and value evidence are orthogonal

Morn now treats deployment/engineering evidence and business-value evaluation as
two independent axes.

```text
EvidenceClass
  DesignSpec | LocalFixture | CiConformance | RealRuntime | RealSite | ProductionWrite

ValueEvidenceClass
  Fixture | Simulation | Shadow | ObservedOperational | CustomerValidated
```

Neither enum is an inheritance ladder. A RealSite claim does not imply
ProductionWrite; CI evidence does not imply RealRuntime. A customer-value claim
requires the customer-validated value class, independent Acceptance, explicit
value evidence and an explicit RealSite evidence claim. This still permits a
read-only customer pilot to prove value without production write authority.

The reference runtime exposes these claims and external blockers in the Console
so fixture/CI success cannot silently become a customer or production claim.

## Durable workflow engines are providers

The pre-v11.5 `DurableRuntime/WorkflowRun` remains useful for waits, signals,
retry, checkpoint and recovery, but it is not a second Work truth model.
Temporal/Dapr-class engines would have the same boundary.

```text
WorkResource (canonical)
  -> ExecutionBinding
     -> DurableWorkflowBinding
        -> workflow run/cursor (provider state)
        -> DurableWorkflowEvidence
  -> source-grounded Outcome
  -> independent Acceptance
```

A completed workflow can move a Work to a waiting/evidence-ready state, but it
cannot mark the Work accepted. The repository includes an adapter that consumes
the legacy Morn DurableRuntime through this provider boundary.

## Cordis reference runtime contract

Morn directly reuses Cordis for the reference node-local composition host, while
also defining a deliberately small `CompositionRuntimeProvider` contract.
That contract covers service-slot mount/replace/unmount/snapshot only. It has no
Work, Authority, ExecutionBinding, Outcome or Acceptance fields.

This captures the current DeepSeek Harness lesson precisely: Cordis makes
services/plugins replaceable through Context, service dependencies, Fiber
lifecycle and reversible effects, while authoritative host/business state can
remain outside the plugin tree. Cordis is therefore the default implementation
rather than the semantic definition of Morn.

## Declarative UI composition

Recent DeepSeek Harness client architecture also demonstrates a second,
browser-side Cordis tree with typed host communication and UI slots. Morn adopts
the **slot** idea but not unrestricted client code loading in v11.5.

Domain packs contribute typed `UiExtensionSpec` declarations:

```text
surface: Workbench | Studio | Console | Hub
slot
title
safe renderer
optional in-app data endpoint
in-app actions
optional required profile
priority
```

Remote arbitrary JavaScript is rejected. An action declaration is presentation
metadata only; backend Profile/Authority/credential enforcement remains the
actual trust boundary. The reference Workbench renders these declarations
generically. A future trusted client-plugin runtime may sit behind the same slot
contract if real product pressure justifies the larger supply-chain boundary.

## External runtime research boundary

The reference landscape continues to support provider reuse rather than Morn
reinvention:

- DeepSeek Harness: pluginized model/tool/session/agent-loop services on Cordis;
- AgentScope Runtime: production agent-as-service, sandbox, persistence,
  interruption and observability — a Provider/Execution-plane reference rather
  than a new Morn core;
- Paper2Agent: reviewed Paper2Skill/Paper2MCP generation, with executable MCP
  tools required to bind to existing repository code — reflected in
  Artifact2Capability and qualification separation;
- DSec: heterogeneous FnCall/container/microVM/full-VM sandbox backends and
  stateful rollout lifecycle — reflected in ExecutionEnvironmentProvider and
  multidimensional execution guarantees.

These projects strengthen the v11.5 boundary rather than collapse it: mature
agent/sandbox runtimes should become replaceable executors, while Morn focuses
on durable Work, binding, authority, reconciliation, outcome and acceptance.


## Revocation is prospective, reconciliation is historical consequence accounting

Pinned bindings do not mean a revoked capability can continue creating new
effects forever. Before a new governed external effect, strict execution
re-checks the binding against the current site/Profile admission, active
qualification/release, provider health and exact provider version/digest.

The inverse rule is equally important: revocation cannot make an already
ambiguous consequence disappear. An Attempt that was dispatched before
revocation may still require source-of-truth reconciliation. The control plane
therefore permits consequence accounting while denying new effects. Any future
execution uses an explicit replacement binding.


## Evidence-derived readiness conditions

Work Conditions are controller projections, not client claims. The durable
control plane persists generation-scoped `ConditionEvidence` and derives
readiness from it. Positive evidence must identify a producer and concrete
semantic evidence references. A Work spec change advances generation and makes
old readiness witnesses ineligible for the new generation.

Typed producers currently bind readiness to complete Workcell resolution,
active qualification/release/site+Profile admission, valid bound Authority,
validated source-of-truth bindings and execution provenance. This closes the
otherwise dangerous path where an API client could post
`authority_satisfied=true` and bypass the actual semantic records.


## Terminal-state and event reorder semantics

Asynchronous runtime/external events may arrive late or out of order. Morn does
not let a late executor event silently move an Accepted, Rejected or Cancelled
Work back into a non-terminal phase. These phases are terminal for the current
Work generation. An explicit spec/Profile/protocol change may create a new
generation, after which readiness is evaluated again from evidence scoped to
that generation. This preserves both corrigibility and non-destructive history.


## Site/Profile conformance evidence

A SiteAdmission is downstream of two different proof chains: capability
qualification proves the capability under a context of use, while a persisted
`ProfileConformanceAttestation` proves that a concrete site/deployment context
meets the Profile guarantee floor. The attestation records exact site/Profile,
computed report, evidence refs, evaluator identity and validity. The product/API
admission path will not accept caller-supplied conformance booleans or semantic
lists as a substitute. Fixture attestations remain fixture evidence.


## Language-neutral protocol surface

The Rust crates are the reference implementation, not the protocol definition.
The repository publishes a language-neutral protocol surface under
`spec/v11.5/`:

- versioned protocol manifest;
- JSON Schema 2020-12 structural contracts;
- canonical examples for Work, Capability, Authority, ExecutionBinding,
  Attempt, external Receipt, Reconciliation, ObservedOutcome, Acceptance,
  ExecutionManifest, Profile and integration events;
- Rust conformance tests that deserialize the canonical examples.

This prevents a future Rust/Cordis/DSH implementation detail from silently
becoming a Morn semantic law. A third-party runtime may implement Morn without
importing Rust crates if it satisfies the published contract and behavioral
conformance rules.

Structural schema is deliberately not treated as sufficient conformance. The
following remain behavioral/cross-record obligations:

```text
Authority before restricted effect
Attempt pinned to exact ExecutionBinding
OUTCOME_UNKNOWN -> reconciliation before retry
external Receipt != harness ExecutionReceipt
Receipt != ObservedOutcome
ObservedOutcome != Acceptance
qualification != release != site admission
historical correction != silent overwrite
```

### Receipt taxonomy

Morn uses the word “receipt” only with an explicit scope:

- `ExecutionReceipt`: harness/runtime evidence that an executor session ran;
- `ConnectorReceipt`: external-system acknowledgement/result for a governed
  connector action;
- `ReconciliationRecord`: source-of-truth observation resolving an ambiguous
  Attempt.

A harness receipt can never satisfy a Profile requirement for external-action
truth by itself.


## Discovery and component projections

The Hub is a view over canonical Morn assets, not an independent marketplace
truth. Registry/discovery protocols are projected at the edge:

```text
Morn Capability / Release / Profile / Admission
                │
                ├─ xRegistry projection (generic metadata catalog)
                ├─ A2A Agent Card projection (Agent-kind subset)
                └─ OASF metadata projection (Agent skill/domain taxonomy)
```

Declared registry metadata remains `Declared` evidence. It does not become
`Qualified` or `Admitted` without Morn evaluation and site/Profile
conformance.

For small portable local providers, a future WIT/Wasm Component provider may
offer a typed cross-language ABI. It is deliberately optional: Cordis remains
the reference node-local composition runtime, while WIT defines one possible
capability interface/host boundary. Remote APIs, agent peers, humans, solvers
and physical executors keep their native provider protocols.


## External protocol task boundary

Modern interoperability protocols increasingly provide their own durable task
objects. Morn reuses them without collapsing the control model:

```text
Morn Work
  -> ExecutionBinding
      -> InteropBinding (exact protocol endpoint + capability)
          -> A2A Task / MCP Task
      -> WorkflowRun / HarnessSession
  -> external Receipt / task observation (executor evidence only)
  -> ObservedOutcome (source-of-truth grounded)
  -> AcceptanceDecision (independent)
```

A2A 1.0 Task/Artifact state and MCP 2026-07-28 Tasks-extension state are
provider/runtime evidence. A completed external Task never means the Work is
accepted. Provider task TTL, cancellation or deletion also cannot erase the
durable Morn Work/history.

For protocol-backed executors, the control plane persists an `InteropBinding`
before it accepts durable task observations. The binding pins the exact
`ExecutionBinding`, capability manifest and protocol endpoint. A later MCP/A2A
task observation must match that persisted endpoint and Work generation; merely
presenting a valid Work/ExecutionBinding id is insufficient. This prevents an
unrelated remote endpoint from laundering executor state into a Work evidence
trace. The binding still confers no authority and no outcome/acceptance status.

External task observations are append-only but lifecycle-monotonic. A task already
observed in a protocol terminal state cannot later reappear as working,
input-required/auth-required, submitted, or a different terminal fact. A2A 1.0
`AUTH_REQUIRED` is modeled as an interrupted, non-terminal state. MCP task
status payloads are validated against the 2026-07-28 Tasks extension: completed
requires a result, failed requires a JSON-RPC error, and non-result states may
not carry terminal result/error payloads. `input_required` additionally retains
the protocol's `inputRequests` object so interruption/recovery evidence records
what the external task is actually waiting for instead of collapsing it into a
bare status label.
