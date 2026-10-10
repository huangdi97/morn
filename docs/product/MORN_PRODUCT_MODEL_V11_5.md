# Morn v11.5 Product Model — Base, Products, Profiles and Experiences

Status: current product-model companion to the v11.5 architecture.

## 1. One-sentence definition

Morn is a **Work-first, outcome-oriented control plane and product substrate**
for composing replaceable humans, programs, solvers, models, agents, services
and machines into governed real-world work.

It is not primarily an agent framework, a digital employee, an MES, or a
smart-factory suite.

## 2. The product stack

```text
Morn Semantic Protocol
  Work / Capability / Authority / Binding / Attempt / Receipt
  Reconciliation / Outcome / Acceptance / Provenance / Profile
        |
Morn Durable Work Control Plane
  desired/observed Work + controllers + history + evidence
        |
Provider Fabric
  composition / harness / workflow / execution / policy / identity / connector
        |
Capability & Solution Supply Chain
  Artifact2Capability / qualification / release / site admission / SolutionPackage
        |
Domain Guarantee Profiles + Domain Packs
  Lite / Enterprise / Factory / Research / future packs
        |
Product Experiences
  Workbench / Studio+Creator / Console / Hub
        |
Packaged Solutions
  factory exception reviewer / research worker / QA worker / personal worker / ...
```

The stable center is the versioned semantic contract. Every implementation
below that contract is replaceable subject to conformance.

## 3. What “base + something” means

Yes, Morn is a small semantic/control **base plus composable product layers**,
but the base is not merely Cordis, DSH, Pi or another agent runtime.

- Cordis is a reference node-local composition runtime.
- DSH, Pi and other agent runtimes are HarnessProviders.
- Temporal/Dapr-class engines can be durable-workflow providers.
- OPA/Cedar/customer IAM can be AuthorityProviders.
- containers/microVMs/remote sandboxes can be ExecutionEnvironmentProviders.
- MCP/OpenAPI/A2A are interoperability protocols.

None of them owns canonical Work or accepted business outcome.

## 4. Morn Factory

**Morn Factory is a domain product**, assembled from the common Morn control
plane plus Factory profiles, mappings, capabilities, connectors, UI views and
SolutionPackages.

It is not a second kernel and its Profile is not immutable.

The first profile is deliberately:

```text
morn.factory.readonly@1.0.0
brownfield
read-first
production-write = forbidden
first wedge = outage / insert-order -> capacity -> delivery-impact review
```

Future Factory profiles may add stronger guarantees and narrowly governed
write authority, but must use new explicit profile versions and site
admissions. Installing a provider never upgrades Factory authority.

## 5. Is Morn a digital employee?

A **digital employee** is a useful product/UX metaphor for some packaged
solutions, not a canonical Morn object.

A user may perceive:

```text
“Factory Exception Analyst”
“Research QA Worker”
“Procurement Reviewer”
```

but internally each is a reviewed SolutionPackage that instantiates Work and
resolves a minimum-sufficient Workcell. That Workcell may contain zero, one or
many agents and may also contain rules, programs, solvers and humans.

This prevents persona branding from hard-coding an Agent-first architecture.

## 6. Is Morn a smart factory?

No. **Smart Factory is a target operating state/outcome**, not the identity of
the Morn base.

Morn Factory can help a brownfield plant move from connected/assisted
operations toward increasingly coordinated and governed autonomy. Existing
MES/MOM/ERP/APS/QMS/CMMS/SCADA/historians remain systems of record and
execution systems unless an explicit future profile says otherwise.

## 7. Agent Factory versus Dream Factory

### Agent Factory

The v11.5 interpretation is a **Capability Supply Chain**. It does not optimize
for producing the largest number of agents.

```text
artifact/source
 -> candidate capability
 -> observed/evaluated
 -> qualified
 -> released/content-addressed
 -> profile conformance
 -> site admitted
```

The output is a qualified capability release that may be an agent, program,
solver, model, connector or hybrid.

### Dream Factory

Dream Factory is the user-facing creation loop:

```text
goal or existing artifact
 -> Creator / Artifact2Capability
 -> reviewable ProblemSpec + WorkGraph + candidate capabilities
 -> explicit acceptance + guarantee profile
 -> review/approval
 -> SolutionPackage
 -> instantiate Work
 -> resolve Workcell
 -> execute/evaluate
 -> reusable evidence-backed package
```

It is therefore “work-system creation”, not “LLM generates an autonomous
employee and deploys it immediately”.

## 8. Creator: the simplest path for a user

A basic user should not choose Cordis, DSH, Pi, A2A, container engines or
controller internals.

The minimal creation form asks only for:

1. what outcome is wanted;
2. explicit acceptance criteria;
3. guarantee profile;
4. optional site/context;
5. optional capability/data hints;
6. autonomy posture: Assist / Governed / AutonomousWithinPolicy.

Creator then translates into the existing Solution pipeline. It does not write
canonical Work until a reviewed/approved SolutionPackage is instantiated.

## 9. Minimum-sufficient Workcell

Morn never asks “how many agents should we create?” as its primary planning
question.

It asks:

> What is the minimum sufficient, qualified and authorized executor mix that
> can satisfy this Work under its profile, budget and acceptance criteria?

Examples:

```text
simple validation:
  rule + program

capacity review:
  historian reader + solver + human approver

ambiguous investigation:
  deterministic retrieval + one harness-backed agent + human reviewer

cross-organization case:
  local worker + remote A2A agent + policy gate + human owner
```

## 10. Product surfaces

- **Workbench** is the operating surface for Work, attention, Conditions,
  evidence, outcomes and acceptance.
- **Studio + Creator** is the build surface for goals, artifacts, capabilities,
  WorkGraphs and SolutionPackages.
- **Console** is the control/trust surface for providers, profiles, authority,
  bindings, runtime health, reconciliation and blockers.
- **Hub** is the reusable asset surface for capabilities, releases, compilers,
  profiles, providers and packages.

They are projections over one domain model, not four independent products.

## 11. Deployment shapes

### Morn Lite

Local/small-team composition with process-class execution and minimal durable
requirements. Useful for personal/project workers without pretending to be an
enterprise control tower.

### Morn Enterprise

Durable Work, capability qualification, governed side effects, reconciliation,
provenance, independent acceptance and customer IAM/policy integrations.

### Morn Factory

Enterprise controls plus plant source-of-truth mappings, Factory guarantee
profiles, industrial connectors, site admission and operational UI.

### Morn Research

Reproducibility/data/code/environment provenance and evidence-oriented
acceptance for research/BioLab workloads.

## 12. Positioning after the 2026 market shift

ServiceNow, UiPath and Siemens now all expose forms of AI/agent governance,
orchestration and industrial agent control. Therefore Morn should not position
“agent control plane” as a unique moat.

Morn's sharper architectural thesis is:

```text
Work truth survives executor replacement.
Authority is distinct from capability.
External effects have explicit truth states.
Unknown outcomes are reconciled, not guessed.
Business Outcome is source-grounded.
Acceptance is independent from executor success.
Value claims carry evidence class.
```

That is the layer Morn must prove empirically.

## 13. Product naming rule

Use the names consistently:

- **Morn** = common Work control substrate/product platform.
- **Morn Factory** = manufacturing domain product.
- **Morn Research/BioLab** = research domain product/routes.
- **Creator / Dream Factory** = creation experience and solution factory.
- **Capability Factory** = supply-chain mechanism formerly described loosely as
  Agent Factory.
- **Digital worker/employee** = optional packaged-solution persona.
- **Smart/Adaptive Factory** = desired operating maturity, not a kernel type.

## 14. Non-claims

The current repository may prove engineering invariants with deterministic
fixtures and CI. It does not thereby prove real customer value, real Factory
operation, production write safety, real DSH/Pi deployment, or site-specific
regulatory compliance.
