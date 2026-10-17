# STATUS — Morn v11.5 Control Plane Convergence

Updated: 2026-10-17  
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

- Verified implementation checkpoint: `49023ac444d824ae74694d61cfc0d44ac87afe50`.
- Exact-head GitHub Actions: `39026643224` — **5/5 SUCCESS**.
- Covered gates: Rust fmt; Clippy `-D warnings`; workspace tests with all features; domain-boundary guard; zero-domain Core build/tests; developer CLI smoke; frontend typecheck/lint/tests/build; Playwright four-surface + reference flows; Cordis reference runtime; Windows desktop reference build and owned-backend lifecycle smoke.
- The external-effect recovery path is crash-consistent: a write-like `Authorized` ActionAttempt and its one-shot ExternalActionPermit consumption commit atomically; dispatch is persisted before connector execution; an authoritative reconciliation atomically advances the Attempt and appends its immutable ReconciliationRecord.
- The Factory read-only vertical slice follows that exact durable ordering through `Authorized -> Dispatched -> OutcomeUnknown -> Reconciliation -> Observed -> Verified`; ambiguous effects cannot be blindly redispatched.
- ExecutionBinding/Work/business-key/resource/site/authority/permit/effect identity remains immutable across persisted Attempt snapshots, with CAS preventing stale projection overwrite.
- DSH SDK/ACP integration has been re-checked against official DeepSeek Harness `0.2.1-rc.2`. Production configs require that exact qualified distribution plus deployment-attested digest/profile/route/environment identity. The only qualification bypass is the private `#[cfg(test)]` protocol-fixture transport and is not serializable or available to production builds.
- SDK `session/resume`, `session/wait` and working-directory methods are treated as executor-transport controls only. Morn does not widen HarnessProvider lifecycle semantics or reconstruct Work ownership from a DSH session id.
- For real providers without per-session close, the Morn E0 execution boundary constructs and persists a Work/generation/binding/scope/runtime-pinned `ExecutionReceipt` after each turn. A lost settlement or policy escape becomes an unsettled `outcome-unknown` receipt and requires explicit reconciliation; it never becomes an ObservedOutcome or AcceptanceDecision.
- Protected live-provider, supply-chain, execution-environment, real BioLab and customer evidence handoffs remain independently external. No fixture, protocol fake, screenshot or local CI result converts those blockers to PASS.
- Current repository-local required-design gap audit remains closed for the v11.5 reference scope at this implementation checkpoint; any later code SHA requires its own exact-head CI before inheriting this conclusion.

## Release posture

- **ALL_LOCAL_GATES_PASS = YES for verified implementation checkpoint `49023ac444d824ae74694d61cfc0d44ac87afe50` and Actions `39026643224`.**
- The verified checkpoint includes the Work/acceptance trust chain, real DSH/Pi adapter boundaries, exact DSH distribution qualification, owned desktop-backend lifecycle, Tauri security hardening, four-surface UI smoke and zero-domain gates.
- PR #1 remains draft because repository-local engineering evidence is not authenticated provider/customer/production evidence.
- PRODUCTION_READY / CUSTOMER_G12 / LIVE_DSH / LIVE_PI remain false until their independent external evidence exists.
- Any later code SHA must obtain its own full exact-head CI before inheriting `ALL_LOCAL_GATES_PASS`; documentation-only commits do not inherit the prior SHA's green status automatically.
