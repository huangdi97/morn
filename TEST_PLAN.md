# TEST_PLAN — Morn v11.5

Updated: 2026-10-09

The test strategy follows v11.5 semantic invariants rather than the older feature-only
Goal1-5 checklist. Every regression should first preserve a reproducer, then fix the code.

## 1. Core semantic invariants

### Work identity and desired state
- Work id + generation define one desired-state authorization scope.
- Same-generation WorkSpec mutation is rejected.
- Terminal Work cannot be reopened by late/reordered evidence.
- Workspace ownership and creation identity cannot be relabeled.
- Explicit spec change increments generation and invalidates stale bindings/conditions.

### Binding and execution manifest
- ExecutionBinding matches exact Work generation/profile/site/autonomy.
- Provider/runtime version and digest are pinned.
- ExecutionManifest pins protocol invariants, source solution, runtime and authority identity.
- Provider replacement creates a new binding/migration decision; active historical bindings are not edited.

### Attempt and external effects
- ActionAttempt identity/business key/effect contract is immutable after authorization.
- Durable attempt snapshots progress only along reachable forward states.
- Stale snapshots cannot move OUTCOME_UNKNOWN back to a redispatchable state.
- Observed/Verified attempts require external reference plus evidence.
- Timeout-after-commit never triggers blind retry.
- Reconciliation must answer the exact business key and Work-owned attempt.

### World truth and acceptance
- Harness/model output is never an ObservedOutcome by itself.
- Positive source observations require deployment-attested source evidence.
- Outcome/Acceptance are exact Work-generation scoped.
- Accept requires source-grounded outcome(s), reviewer identity, authorization, reason and evidence.
- CustomerValidated value requires final acceptance of the exact assessed outcome.
- CustomerValidated additionally requires the **current** RealSite evidence claim for its exact value subject; a later Revoked or BlockedExternal claim removes current support without deleting historical assessments.
- Evidence history is append-ordered per subject/class; stale older proof cannot be replayed after revocation.
- Previous-generation outcome/acceptance does not authorize current-generation Work.

## 2. Provider Fabric contract

Run the shared contract against:
- MornNative fixture.
- DeepSeek Harness fixture.
- Pi fixture.
- Real-wire protocol fakes that exercise the exact public DSH SDK / Pi RPC shape.

Real DSH/Pi tests must verify:
- explicit configuration; no implicit credential/home discovery;
- scrubbed subprocess environment;
- exact execution-environment identity;
- E0 scope restriction and revocation;
- runtime distribution version + digest pinning;
- request/turn bounds;
- bounded UTF-8 wire frames and bounded reader queues; overflow fails closed without teardown deadlock;
- DSH JSON-RPC version/response-id/result-vs-error shape validation;
- Pi RPC response-id + command correlation; cross-request responses fail closed;
- settled success is required before Healthy lease;
- non-success/timeout degrades health;
- unsupported lifecycle methods fail closed;
- lost settlement reaps owned runtime where the wire cannot cancel safely;
- normalized audit events do not copy private reasoning, raw assistant body, tool arguments/results or secrets.

Live authenticated provider smoke is a separate external gate and must not run as an assumed CI dependency.

## 3. Execution environment and authority

- Profile-required execution class and guarantee vector enforced independently.
- Caller labels cannot manufacture RuntimeAttestation.
- Exact environment attestation identity must match ExecutionBinding and provider launch configuration.
- Credential/workload identity references stay opaque.
- E1/E2/E3 require Authority + external action permit at the enforcement boundary.
- Assist posture forbids write-like effects.
- Compensation policy cannot widen effect ceiling.

## 4. Capability supply chain

For every Artifact2Capability compiler:
- output starts as Declared candidate only;
- provenance/source refs preserved;
- no compiler path implies Qualified/Released/SiteAdmitted;
- reviewed paper executable path requires real code binding;
- repository compiler does not invent hidden entrypoints;
- model/workflow identities and versions are pinned.

Lifecycle tests:
Declared -> Observed -> Qualified -> Released -> Conformant -> SiteAdmitted.
Suspension/retirement must not erase history.

## 5. Persistence / migration / concurrency

SQLite coverage:
- schema migrations and restart persistence;
- immutable historical records reject overwrite;
- mutable projections use CAS where concurrent writers matter;
- workspace isolation;
- CloudEvents inbox dedupe by source + event id;
- outbox durability;
- controller leases/fencing;
- old schema rows migrate without silently changing semantic ownership.

## 6. Product surfaces

Frontend:
- typecheck, lint, unit/component tests, production build.
- Workbench shows canonical Work even when legacy diagnostics fail.
- Reference/fixture tools are collapsed by default.
- Studio uses one goal source and three-step Work creation flow.
- Advanced artifact import remains available without dominating primary UX.
- Console truthfully distinguishes fixture/configured/initialized/healthy/degraded/closed providers.
- Hub shows lifecycle stages without treating candidates as admitted capabilities.

Playwright:
- desktop + mobile four-surface smoke;
- Studio compile/approve/instantiate -> Workbench;
- fixture BioLab/replay actions remain reference-only;
- screenshots are visual evidence, not customer acceptance.

## 7. Zero-domain and domain-pack boundaries

- Core builds/runs with no domain pack.
- Core crates do not import BioLab/domain-specific semantics.
- BioLab reference remains a replaceable domain pack.
- domain-pack enablement never changes protocol invariants.

## 8. External-gate discipline

Tests and reports must keep these states independent:
- repository-local PASS;
- live provider evidence;
- live sandbox/environment attestation;
- real dataset evidence;
- customer/site acceptance;
- production/physical write authorization.

A fixture or fake protocol runtime may prove contract correctness only.

## 9. Exact-HEAD CI acceptance

For a commit to claim `ALL_LOCAL_GATES_PASS`, the same SHA must have:
- backend fmt/clippy/workspace all-features/zero-domain success;
- frontend success;
- Playwright E2E success;
- Cordis success;
- Windows desktop reference build success.

If any new commit is pushed, the prior PASS is historical only until the new SHA completes.
