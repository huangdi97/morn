# STATUS — Morn v11.5 Control Plane Convergence

Updated: 2026-10-14  
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
- Independent reviewer identity/authorization is deployment-attested and generation-scoped.
- Terminal AcceptanceDecision pins a versioned reviewer-attestation reference plus exact one-shot review-authorization references; durable recovery revalidates the persisted trust chain before projecting Accepted/Rejected.
- Reviewer attestations are append-only/versioned by principal + observation time, so identity evidence can rotate without rewriting historical decisions.
- Accepted outcome/value records cannot borrow evidence from another Work/generation/workspace.
- Harness output and assistant text never directly create ObservedOutcome or AcceptanceDecision.
- Deployment evidence files require a deployment-pinned issuer identity; payloads cannot self-select a trusted issuer.
- Customer RealSite and ProductionWrite readiness is bound to the exact `MORN_DEPLOYMENT_SITE_REF`; evidence from one site cannot make another site ready.
- RealRuntime proof must cite the exact active DSH/Pi runtime version+digest in addition to a fresh Healthy lease.
- Evidence history is time-aware: equal-time conflicting subject/class claims are rejected, and future-dated claims are not current proof for readiness or CustomerValidated value.

### Capability supply chain
- Artifact -> candidate Capability compilers for OpenAPI, SOP/procedure, repository, reviewed paper, model and workflow manifests.
- Qualification, release, profile conformance and site admission remain distinct lifecycle stages.
- OCI/Sigstore/SLSA-oriented package descriptor/provenance model.
- Workcell resolver respects execution guarantees, effect ceilings, profile/site admission and unavailable providers.

### Product surfaces
- Workbench: canonical Work/Conditions/bindings/attempts/outcomes/acceptance first; legacy fixture diagnostics collapsed.
- Studio: goal + acceptance -> compile/approve Solution -> instantiate canonical Work; raw artifact import is advanced/optional.
- Console: evidence-aware release readiness plus provider/runtime/profile/authority/conformance and blocker visibility.
- Hub: capability supply chain, providers, packages and reusable assets.
- Four-surface Playwright smoke and Windows desktop reference build are in CI.
- Reference desktop owns a single local `server(.exe)` sidecar when needed: it verifies `morn-app` + protocol `11.5.0` health before reuse, fails closed on foreign port occupancy, binds child lifetime to stdin EOF for graceful shutdown, and enforces restrictive Tauri CSP/prototype hardening.

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

- B-001: authenticated live official DSH runtime/model/credential smoke on the attested self-hosted provider runner.
- B-002: installed/authenticated live Pi runtime/model/credential smoke on the attested self-hosted provider runner.
- B-003: live deployment execution-environment attestation from a real container/microVM/Kubernetes/customer sandbox.
- B-004: authorized OCI registry publication plus deployment-owned Sigstore/OIDC signature and SLSA provenance evidence.
- G4-B-002: lawful provenance-bearing real BioLab dataset.
- G12: real customer/site source systems, IAM/policy, independent acceptance and customer evidence.
- Production/physical write: not authorized.
- ACP Work-facing lifecycle control is implemented locally as a separate Work-bound controller, not as a fake SDK Provider feature. Authenticated official ACP lifecycle evidence remains deployment-dependent and does not count as executor-health, Outcome or Acceptance evidence.

These are not converted to PASS by fixtures, protocol fakes, screenshots or documentation.

## Latest local verification

- Verified implementation checkpoint: `5bb26a455b1aaea7daf70ab7eb6ff583b29f6c76`.
- Exact-head GitHub Actions: `38436312727` — **5/5 SUCCESS**.
- That checkpoint includes protected live-provider/supply-chain evidence workflows, attested runner binding, exact runtime/build/site identity checks and ephemeral registry credential handling.
- Later readiness-acquisition UX and customer-evidence validation changes must obtain their own exact-head CI before replacing this checkpoint; an older green run is never inherited.
- Covered gates: Rust fmt; Clippy `-D warnings`; workspace tests with all features; zero-domain Core; frontend typecheck/lint/tests/build; Playwright four-surface + reference flows; Cordis reference runtime; Windows desktop reference build.
- Release-readiness trust hardening at this checkpoint binds exact build identity, exact active provider runtime artifact, deployment-pinned evidence issuer and exact deployment site; evidence time ambiguity/future claims fail closed.
- Business-truth recovery now distinguishes observed outcome from final acceptance: Conditional/RequestMoreEvidence retain OutcomeObservation while remaining non-terminal.
- Final Accept/Reject and CustomerValidated value require the exact persisted reviewer attestation, review authorization, authorization-consumption record, Work generation and linked attested outcome.
- Governed live-provider smoke still additionally requires an exact `MORN_PROVIDER_SMOKE_OK` canary and a fresh post-turn `Healthy` lease. Process/session initialization alone is not provider-health evidence.
- DSH SDK ↔ Pi RPC adapter-wire semantic-neutrality runs through the actual subprocess adapter code with the same pinned Work/generation/ExecutionBinding/execution environment; model output is deliberately not compared.
- ACP v1 lifecycle control is Work/generation/ExecutionBinding/environment/scope/route bound. It pins exact provider/model/reasoning through official session config, re-pins on same-runtime resume, faults ambiguous settlements to prevent blind retry, and preserves out-of-band cancel/cleanup without converting lifecycle settlement into executor success.
- ACP scope ownership is revocable and snapshot-pinned: broader scopes are rejected, recreated/mutated scope state cannot take over a historical session, and one-shot DSH_HOME means cross-process resume is explicitly unavailable.
- `dsh_acp_lifecycle_smoke` is a deployment-facing gate for authenticated ACP lifecycle evidence; its PASS still reports executor/runtime-health/Outcome/Acceptance/production-write claims as false.
- Current repository-local gap audit found no remaining TODO/FIXME/unimplemented/stub item required to satisfy the v11.5 reference design. Remaining blocker classes below require external deployment/customer evidence.
- Readiness decisions now expose a machine-readable evidence-acquisition plan per blocked deployment scope. DSH/Pi point to their authenticated smoke gates; RealSite/ProductionWrite point to the protected customer/deployment evidence validation gate. These plans explain how to collect evidence without converting the external blocker into fake local PASS.

## Release posture

- **ALL_LOCAL_GATES_PASS = YES for verified implementation checkpoint `5bb26a455b1aaea7daf70ab7eb6ff583b29f6c76` and Actions `38436312727`.**
- The verified checkpoint includes the owned desktop-backend lifecycle probe and Tauri CSP hardening in addition to the v11.5 control-plane, trust, provider and UI gates.
- PR #1 remains draft because local engineering evidence is not production/customer evidence.
- PRODUCTION_READY / CUSTOMER_G12 / LIVE_DSH / LIVE_PI remain false until their independent external evidence exists.
- Any later code SHA must obtain its own full exact-head CI before inheriting `ALL_LOCAL_GATES_PASS`.
