# Morn v11.5 Convergence Closeout — 2026-10-08

Status: **LOCAL_ENGINEERING_CANDIDATE / CI_IN_PROGRESS**

This report is a repository-level implementation/evidence ledger for the
v11.5 convergence branch. It does not convert fixture evidence into real
customer, real site, real DSH/Pi deployment or production-write evidence.

## 1. Target architecture

The implemented target is:

```text
Morn Semantic Specification
        +
Durable Work Control Plane
        +
Reference Composition Runtime (Cordis)
        +
Replaceable Provider Fabric
        +
Capability Supply Chain
        +
Versioned Domain Guarantee Profiles
        +
Product/Domain surfaces
```

Morn remains Work-first, not Agent-first:

- Work = coordination unit;
- Capability = composition unit;
- Accepted Outcome = value unit;
- Agent = one executor kind.

## 2. Design-to-code closure matrix

| Design obligation | Implementation evidence | Local state |
| --- | --- | --- |
| versioned semantic constitution | `morn-kernel/src/protocol.rs` | implemented |
| explicit protocol migration | protocol compatibility + `ProtocolMigrationController` | implemented |
| non-destructive history | `morn-kernel/src/history.rs` | implemented |
| desired/observed Work | `morn-work/src/control.rs` | implemented |
| generation invalidates stale projection | `WorkResource::replace_spec` | implemented |
| durable control plane | `morn-control-plane`, `morn-store` | implemented |
| optimistic concurrency/fencing | control-plane/store CAS + leases | implemented |
| Work termination/finalizers | Work finalizers + external-action resolution | implemented |
| Work truth independent of harness | protocol invariant + conformance tests | implemented |
| node-local composition | exact-pinned Cordis host + `CompositionRuntimeProvider` contract | implemented |
| DSH provider boundary | fixture + SDK/wire seam | local contract implemented; real runtime external |
| Pi provider boundary | fixture provider + shared contract | local contract implemented; real transport external |
| provider registry | `morn-runtime/src/provider_registry.rs` | implemented |
| durable workflow provider boundary | DurableWorkflowBinding/Evidence + legacy DurableRuntime adapter | implemented |
| explicit execution environment | class + guarantee-vector provider/resolver | implemented |
| workload identity | SPIFFE-compatible provider seam | implemented; real identity infra external |
| opaque credentials | scoped credential handles | implemented; real secret backend external |
| authority separate from enforcement | AuthorityProvider + action boundary | implemented |
| MCP/A2A interoperability | `morn-integration/src/interoperability.rs` | implemented |
| binding pinned per attempt | `ExecutionBinding` | implemented |
| binding migration explicit | `BindingMigrationDecision` | implemented |
| reproducible interpretation | `ExecutionManifest` | implemented |
| unknown external outcome | Attempt + ReconciliationController | implemented |
| source-of-truth binding | integration/control-plane records | implemented |
| Outcome separate from Acceptance | world/work contracts + tests | implemented |
| Capability specification | `CapabilityManifest` | implemented |
| minimum sufficient Workcell | resolver/workcell plan | implemented |
| capability observation | assurance admission service | implemented |
| strict qualification | QualificationEvidence/Record | implemented |
| content-addressed release | CapabilityDistributionRelease | implemented |
| site + profile admission | SiteAdmission + exact admission refs | implemented |
| Profile as guarantees, not vendors | `morn-profile` | implemented |
| Profile registry/evolution | ProfileRegistry + compatibility/migration plan | implemented |
| Artifact2Capability | OpenAPI/SOP/Repo/reviewed-paper compilers | implemented |
| capability packaging | OCI-oriented descriptor + Sigstore/SLSA refs | implemented contract; real registry external |
| CloudEvents-style integration envelope | kernel event envelope | implemented |
| event semantic roles | runtime signal vs durable control/fact/observation taxonomy | implemented |
| provider-health freshness gate | ProviderRegistry health lease + ProviderGate | implemented |
| strict capability runtime eligibility | active qualification/release/site admission + live provider gate | implemented |
| Pi JSONL RPC boundary | Rust subprocess client for prompt/get_state/abort/agent_settled | implemented; real binary/model smoke external |
| atomic state + semantic outbox | CAS projection + durable event in one transaction | implemented |
| MCP 2026 resource-bound auth contract | opaque credential/resource binding | implemented |
| telemetry boundary | OTel-oriented semantic design | contract/design; backend deployment external |
| evidence-class claim ledger | categorical EvidenceClass + external blockers | implemented |
| customer value claim gate | CustomerValidated + Acceptance + explicit RealSite evidence | implemented |
| Creator | natural-language/simple draft -> existing Solution pipeline | implemented |
| SolutionPackage -> Work | explicit instantiation | implemented |
| Digital Employee | Role projection, not canonical agent truth | implemented |
| Workbench | durable Work/control/outcome surface | implemented |
| Studio | goal-first + Artifact2Capability surface | implemented |
| Console | control/trust/provider/conformance surface | implemented |
| Hub | capability/provider/runtime/compiler registry surface | implemented |
| Factory read-only wedge | CNC-17/reference integration slice | implemented fixture |
| timeout-after-commit no-blind-retry | Factory/reconciliation tests | implemented |
| provider swap continuity | DSH/Pi neutrality + binding migration tests | implemented fixture |

## 3. Version / change semantics

v11.5 no longer assumes an immutable software core.

```text
Implementation
  -> replaceable

Published protocol/profile contract
  -> versioned evolution

Running Work/ExecutionBinding
  -> pinned for its execution scope

Historical fact
  -> supersede/retract/redact/migrate explicitly
  -> never silently rewrite
```

Protocol/Profile compatibility is now machine-readable:

- same published version with changed semantics = incompatible;
- semantic-equivalent patch = compatible;
- minor strengthening/evolution = explicit reevaluation;
- protocol minor may not delete established semantic slots/invariants;
- Profile minor may not weaken required/forbidden/execution guarantees;
- downgrade is incompatible by default;
- major/cross-Profile change is incompatible by default.

## 4. Cordis / DSH / Pi boundary

Cordis is the reference **node-local composition runtime**, not Morn's business
truth layer.

Cordis owns:

- Context/Service composition;
- dependency injection;
- plugin/Fiber lifecycle;
- reversible software registration effects.

Cordis does not own:

- Work identity/generation;
- durable business status;
- Authority;
- external action truth;
- Receipt/Reconciliation;
- Outcome/Acceptance;
- capability qualification/site admission.

DSH and Pi are HarnessProviders. Their session/task success is executor evidence,
not AcceptedOutcome.

A current DSH/Cordis peer-dependency mismatch reported upstream further supports
keeping the Morn Cordis host and DSH process boundary separate rather than
forcing one shared dependency tree.

## 5. Capability supply chain

```text
Artifact
 -> compiler
 -> Declared
 -> Observed
 -> Qualified
 -> content-addressed Release
 -> Profile conformance
 -> SiteAdmission
 -> Suspended / Retired
```

No stage is collapsed:

- compilation != qualification;
- qualification != release;
- release != site admission;
- site admission is scoped to exact site + Profile.

Paper-derived executable candidates require reviewed extraction and real code
bindings; free-form paper text alone is insufficient.

## 6. Factory reference slice

The first product wedge remains:

> outage / insert-order exception -> capacity -> delivery-impact review

Reference flow:

```text
source event
 -> Work desired state
 -> Profile
 -> Capability resolution
 -> qualification/site admission
 -> Authority
 -> execution guarantee/environment selection
 -> pinned ExecutionBinding
 -> ExecutionManifest
 -> executor/solver/human Workcell
 -> external Attempt when allowed
 -> timeout-after-commit => OUTCOME_UNKNOWN
 -> reconciliation against source of truth
 -> Receipt / ObservedOutcome
 -> independent Acceptance
 -> ValueAssessment
```

The reference profile remains read-first and forbids ProductionWrite.

## 7. Evidence classes

The branch deliberately distinguishes:

- **design/spec evidence**;
- **local deterministic fixture evidence**;
- **CI/conformance evidence**;
- **real runtime evidence**;
- **real customer/site evidence**;
- **production write evidence**.

Evidence classes are categorical rather than a scalar ladder. No evidence class implicitly proves another class.

## 7.1 Value-evidence axes

Engineering/deployment evidence and value-validation evidence are orthogonal.
`EvidenceClass` records where/how a claim was proven; `ValueEvidenceClass`
records whether a ValueAssessment is fixture/simulation/shadow/operational/
customer-validated. A customer-value claim requires independent Acceptance,
explicit value evidence and an explicit RealSite evidence claim. ProductionWrite
is not required for a legitimate read-only customer value pilot.

## 7.2 Durable workflow boundary

Legacy `morn-work::DurableRuntime` and future Temporal/Dapr-class engines are
execution providers. Their workflow-run completion is runtime evidence only and
cannot directly create a Morn Outcome or Acceptance. A workflow run is tied to
the exact Work generation and ExecutionBinding through
`DurableWorkflowBinding`.

## 8. External blockers / non-claims

The following remain explicitly external or not entered:

- real DeepSeek Harness runtime/model/credentials smoke;
- real Pi transport/runtime;
- real factory/customer data;
- real factory source-system connection;
- real site IAM / OPA / Cedar;
- real SPIFFE/SPIRE or equivalent workload identity deployment;
- real OCI registry publication/signing;
- real Sigstore transparency/provenance verification;
- real microVM/DSec-class backend;
- real attribution/economics;
- controlled production write;
- physical control.

```text
REAL_PRODUCTION_WRITE = NO
CONTROLLED_REAL_WRITE = NO
REAL_FACTORY_CUSTOMER_EVIDENCE = EXTERNAL_BLOCKED
```

## 9. CI closure rule

This report may be promoted from `LOCAL_ENGINEERING_CANDIDATE` to
`ALL_LOCAL_GATES_PASS` only after the latest branch head passes the complete
GitHub Actions matrix:

- Backend fmt/clippy/tests/zero-domain;
- Frontend typecheck/lint/test/build;
- E2E server + UI smoke;
- Desktop/Tauri build;
- Cordis host tests/smoke.

A green older SHA is not sufficient for a newer branch head.

## 10. Final engineering boundary

When all local gates are green, additional work that requires only missing
customer/site/runtime credentials is not a local engineering defect. It must
remain visible as `BLOCKED_EXTERNAL` rather than being satisfied by mocks.

The branch is not authorized to claim production maturity merely because the
architecture and local conformance implementation are complete.
