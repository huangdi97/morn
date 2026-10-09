# STATUS — Morn v11.5 Control Plane Convergence

Updated: 2026-10-09  
Branch: `feat/v11.5-control-plane-convergence`  
PR: #1 (draft)

This file reports the **current v11.5 branch**, not the August v1 GA baseline.
Historical v1 status remains available in git history and older reports.

## Product truth

Morn v11.5 is a protocol-driven, outcome-oriented Work control plane.

Canonical invariants:

1. Work is the unit of coordination.
2. Capability is the unit of composition.
3. Accepted outcome is the unit of value.
4. Agent is one executor kind, not the source of Work truth.
5. Work id + generation identify one desired-state authorization scope.
6. ExecutionBinding is immutable for an execution scope.
7. Unknown external outcomes are reconciled, never blindly retried.
8. Harness/runtime success is executor evidence, not world truth or acceptance.
9. Final acceptance requires source-grounded outcomes plus independent review evidence.
10. CustomerValidated value requires the exact accepted outcome and acceptance record.

## Current implementation

### Durable Work control plane
- Work spec/status/generation/Conditions with CAS persistence.
- Same-generation spec mutation and terminal-phase rewind rejected.
- Workspace ownership and creation identity pinned.
- Generation-scoped source observations, outcomes and acceptance.
- Binding/manifest/attempt/reconciliation persistence pinned to exact Work identity.
- ActionAttempt progression is monotonic and CAS protected.
- OUTCOME_UNKNOWN / reconciliation path preserves idempotency/business key evidence.

### Execution and Provider Fabric
- MornNative deterministic reference provider.
- DeepSeek Harness fixture provider plus real official SDK stdio/JSON-RPC adapter.
- Pi fixture provider plus real JSONL RPC adapter.
- Real DSH/Pi execution is E0-only at the HarnessProvider seam.
- E1/E2/E3 effects remain governed ExternalAction operations.
- Runtime distributions pin deployment-attested version + SHA-256 digest separately from wire protocol identity.
- Child-process environments are scrubbed and explicit.
- Real runtime admission requires a runtime-attested execution environment, exact environment identity and mounted E0 scope.
- Provider health uses bounded leases after a successful settled real turn; initialized/configured alone are not Healthy.

### Trust and truth
- Deployment-backed execution-environment attestations.
- Source-of-truth bindings and deployment-attested source observations.
- Independent reviewer identity/authorization is attested and generation-scoped.
- Accepted outcome/value records cannot borrow evidence from another Work/generation/workspace.
- Harness output and assistant text never directly create ObservedOutcome or AcceptanceDecision.

### Capability supply chain
- Artifact -> candidate Capability compilers for OpenAPI, SOP/procedure, repository, reviewed paper, model and workflow manifests.
- Qualification, release, profile conformance and site admission remain distinct lifecycle stages.
- OCI/Sigstore/SLSA-oriented package descriptor/provenance model.
- Workcell resolver respects execution guarantees, effect ceilings, profile/site admission and unavailable providers.

### Product surfaces
- Workbench: canonical Work/Conditions/bindings/attempts/outcomes/acceptance first; legacy fixture diagnostics collapsed.
- Studio: goal + acceptance -> compile/approve Solution -> instantiate canonical Work; raw artifact import is advanced/optional.
- Console: provider/runtime/profile/authority/conformance and blocker visibility.
- Hub: capability supply chain, providers, packages and reusable assets.
- Four-surface Playwright smoke and Windows desktop reference build are in CI.

## Verification policy

A green reference CI may prove only repository-local engineering gates.

Required exact-HEAD CI:
- Rust fmt
- Clippy with `-D warnings`
- workspace tests, all features
- zero-domain Core build/tests
- frontend typecheck/lint/test/build
- Playwright four-surface + reference flows
- Cordis reference runtime
- Windows desktop reference build

Never carry a green conclusion from an older SHA to a newer SHA.

## External blockers

Authoritative details live in `BLOCKERS.md`.

- B-001: authenticated live official DSH runtime/model/credential smoke.
- B-002: installed/authenticated live Pi runtime/model/credential smoke.
- B-003: live deployment execution-environment attestation from a real container/microVM/Kubernetes/customer sandbox.
- G4-B-002: lawful provenance-bearing real BioLab dataset.
- G12: real customer/site source systems, IAM/policy, independent acceptance and customer evidence.
- Production/physical write: not authorized.

These are not converted to PASS by fixtures, protocol fakes, screenshots or documentation.

## Release posture

- Reference implementation: converging and continuously verified.
- PR #1 remains draft.
- ALL_LOCAL_GATES_PASS may be asserted only for a specific exact SHA after its full CI completes.
- PRODUCTION_READY / CUSTOMER_G12 / LIVE_DSH / LIVE_PI must remain false until their independent evidence exists.
