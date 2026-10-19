# STATUS — Morn v11.5 Control Plane Convergence

Updated: 2026-10-19  
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
- DeepSeek Harness fixture provider plus real official SDK stdio/JSON-RPC adapter, re-qualified against the exact official `0.2.1-rc.2` distribution and current pre-release wire surface.
- Pi fixture provider plus real JSONL RPC adapter.
- Real DSH/Pi execution is E0-only at the HarnessProvider seam.
- E1/E2/E3 effects remain governed ExternalAction operations.
- Runtime distributions pin deployment-attested version + SHA-256 digest separately from wire protocol identity.
- Child-process environments are scrubbed and explicit.
- Real runtime admission requires a runtime-attested execution environment, exact environment identity and mounted E0 scope.
- Provider health uses bounded leases after a successful settled real turn; initialized/configured alone are not Healthy.
- Official DSH distribution identity is independent of `serverInfo.version`: live launch is fail-closed unless the deployment-attested distribution version equals the currently qualified `0.2.1-rc.2`; test-only protocol fixtures cannot weaken that production gate.
- DSH SDK `session/resume` / `session/wait` / working-directory methods are used only within their documented transport semantics; Work-facing durable lifecycle ownership remains generation/binding-scoped and is not inferred from a DSH session id.
- DSH SDK, DSH ACP and Pi provider streams are minimized at transport ingress: private reasoning/transcript bodies, raw tool arguments/results and free-form tool names/titles are discarded before Morn buffering; only visible assistant output plus correlation/settlement/policy facts needed by the governed E0 boundary are retained.

### Trust and truth
- Deployment-backed execution-environment attestations.
- Source-of-truth bindings and deployment-attested source observations.
- Independent reviewer identity/authorization is deployment-attested and generation-scoped.
- Terminal AcceptanceDecision pins a versioned reviewer-attestation reference plus exact one-shot review-authorization references; durable recovery revalidates the persisted trust chain before projecting Accepted/Rejected.
- Reviewer attestations are append-only/versioned by principal + observation time, so identity evidence can rotate without rewriting historical decisions.
- Accepted outcome/value records cannot borrow evidence from another Work/generation/workspace.
- Harness output and assistant text never directly create ObservedOutcome or AcceptanceDecision.
- Deployment evidence files require a deployment-pinned issuer identity; payloads cannot self-select a trusted issuer.
- Customer RealSite and ProductionWrite facts remain append-only evidence history, but **current deployment readiness** additionally requires the exact `MORN_DEPLOYMENT_SITE_REF` and exact `MORN_BUILD_IDENTITY_REF`; evidence from another site or older build cannot be spliced into a different running deployment.
- RealRuntime proof must cite the exact active DSH/Pi runtime artifact plus profile/route/environment identities in addition to a fresh Healthy lease; replacement-provider health cannot reuse an old smoke claim.
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

- Verified implementation checkpoint: `d1c8bf11e373ae04b56c21d166fcb45f226ac254`.
- Exact-head GitHub Actions: `39252110666` — **5/5 SUCCESS**.
- Covered gates: privileged-workflow and external-evidence-entrypoint audits; full Cargo.lock resolution; Rust fmt; Clippy `-D warnings`; workspace all-features tests; domain-boundary guard; zero-domain Core build/tests; developer CLI smoke; frontend typecheck/lint/test/build; Playwright/BioLab/zero-domain browser E2E; Cordis reference runtime; and the Windows desktop reference build with owned-backend, foreign-port/protocol-mismatch and single-instance-lock probes.
- Provider evidence minimization is now provider-neutral across qualified DSH SDK, DSH ACP and Pi paths: private reasoning and raw tool payload/free-form tool metadata are removed at ingress without weakening tool-activity fail-closed detection or settlement correlation.
- Deployment-readiness coherence now rejects cross-epoch evidence splicing: a current RealSite or ProductionWrite readiness axis must use the exact site subject **and** cite the running build identity. Historical evidence remains auditable rather than rewritten.
- The protected customer/production evidence handoff requires `build_identity_ref`, the verifier requires the same exact build in the claim's evidence refs, the normalized report exposes the verified build identity, and the repository entrypoint audit fails if that contract is removed.
- ProductionWrite evidence remains distinct from action authority: no readiness claim can grant Profile/Authority permission or replace the exact Work-generation/action/resource/site ExternalActionPermit path.
- RealRuntime readiness remains independently pinned to the active provider artifact + profile/route/environment identities plus fresh Healthy lease; provider replacement does not reinterpret old Work/binding/receipt history.
- Protected live-provider, supply-chain, execution-environment, real BioLab and customer evidence remain independently external. No local gate above converts B-001/B-002/B-003/B-004/G4-B-002/G12 or production/physical write to PASS.

## Release posture

- **ALL_LOCAL_GATES_PASS = YES for verified implementation checkpoint `d1c8bf11e373ae04b56c21d166fcb45f226ac254` and Actions `39252110666`.**
- The verified checkpoint includes the Work/acceptance trust chain, real DSH/Pi adapter boundaries, provider-ingress evidence minimization, exact DSH distribution qualification, deployment-coherent customer/write readiness, owned desktop-backend lifecycle, Tauri security hardening, four-surface UI smoke and zero-domain gates.
- PR #1 remains draft because repository-local engineering evidence is not authenticated provider/customer/production evidence.
- PRODUCTION_READY / CUSTOMER_G12 / LIVE_DSH / LIVE_PI remain false until their independent external evidence exists.
- Any later code SHA must obtain its own full exact-head CI before inheriting `ALL_LOCAL_GATES_PASS`; documentation-only commits do not inherit the prior SHA's green status automatically.
