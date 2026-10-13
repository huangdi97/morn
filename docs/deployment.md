# Morn Deployment & Operations

## Build

```powershell
cargo build --workspace --all-features
cargo build -p morn-app --bin server --all-features   # API server
cargo build -p morn-desktop                           # Tauri desktop shell
cd frontend; npm install; npm run build               # production assets -> frontend/dist
```

## Run

```text
server:  MORN_DB=<path> MORN_PORT=<port> target/debug/server(.exe)
default: http://127.0.0.1:8090, db: ./morn.db
frontend dev: cd frontend; npm run dev  (http://127.0.0.1:5173)
frontend prod: serve frontend/dist behind a proxy to /api -> server
desktop: target/debug/morn-desktop(.exe)  (WebView2; loads frontend/dist)
```

The frontend dev server proxies `/api` to the backend (see `vite.config.ts`).
For production, configure your web server to proxy `/api` to the Morn server
and serve `frontend/dist`.

## Data & persistence

- SQLite database (default `morn.db`), schema versioned (`morn-store`).
- The ledger is append-only; artifact versions and receipts are immutable.
- Secrets must not be stored in SQLite in plaintext; use the OS secret store
  or a vault and pass references.

## Attested runner requirement for live provider evidence

The manual live-provider workflows deliberately separate **identity planning** from
**authenticated execution**:

- `plan-sdk` / `plan-acp` / Pi `plan` may run on GitHub-hosted runners because
  they launch no provider process and consume no provider credential.
- DSH `smoke-sdk`, DSH `smoke-acp` and Pi `smoke` run only on a self-hosted
  runner carrying the custom label `morn-attested-provider`.
- That runner must expose a root-owned, non-symlink, non-group/world-writable
  `/etc/morn/execution-environment-ref`. Its exact text must equal the
  workflow's `execution_environment_ref` and the deployment-owned
  `ExecutionEnvironmentAttestation`.
- The workflow emits `runner-binding.json` as secret-free evidence of the
  runner/environment association. This evidence is necessary but not sufficient:
  the signed/attested environment record still owns isolation class, guarantees,
  runtime identities and validity.
- A GitHub-hosted runner plus a JSON attestation for some unrelated container or
  customer sandbox is **not** accepted as live deployment evidence.
- Provider smoke remains E0-only and cannot prove Outcome, independent
  Acceptance, customer value, production write, or G12.
- On a successful SDK/Pi smoke, the workflow also exports
  `real-runtime-evidence-claim.json`. Morn itself serializes this claim, so it
  can later be supplied through `MORN_EVIDENCE_CLAIMS_FILE` with the same
  deployment-pinned `MORN_EVIDENCE_ISSUER` without manually rebuilding the
  subject/runtime/environment/route references. This is historical
  `RealRuntime` evidence only: deployment readiness still requires a fresh
  `Healthy` lease from the currently running exact provider path.

Provision and harden the self-hosted runner outside the repository. The custom
label and local identity file are admission controls, not a substitute for
customer IAM, workload identity, host hardening or external attestation.

## Attested OCI / Sigstore / SLSA verification

B-004 has a deployment-facing verification gate in
`.github/workflows/live-supply-chain-evidence.yml`. It verifies an **already
published**, digest-pinned OCI subject. It never signs or publishes an artifact,
never creates SiteAdmission, and never authorizes production execution.

The workflow separates planning from proof:

- `plan` runs on a GitHub-hosted runner and only validates/records the intended
  OCI digest, Fulcio certificate identity, OIDC issuer and release-runner
  identity. It performs no registry request.
- `verify` runs only on a self-hosted runner with the custom label
  `morn-attested-release`.
- The release runner must expose a root-owned, non-symlink,
  non-group/world-writable `/etc/morn/release-runner-ref` equal to the explicit
  workflow input.
- The verifier is fixed at `/usr/local/bin/cosign`; the workflow hashes its
  bytes and requires an exact `sha256:<64-hex>` input. A mutable PATH lookup is
  not accepted as verifier identity.
- Private-registry credentials are optional and deployment-owned. When enabled,
  only a root-owned `/etc/morn/registry-auth/config.json` is exposed to the
  Cosign subprocess through `DOCKER_CONFIG`; repository checkout credentials
  are not persisted.
- Verification uses the deployment-pinned `certificate-identity` and
  `certificate-oidc-issuer` for both the OCI signature and the
  `slsaprovenance` attestation. The two proof axes are merged only when they
  bind the same OCI subject digest.
- A PASS exports
  `target/live-supply-chain-evidence/supply-chain-verification.json`, directly
  consumable through `MORN_SUPPLY_CHAIN_VERIFICATIONS_FILE`.

The Rust entrypoint is:

```text
cargo run -p morn-app --bin supply_chain_verify
```

with `MORN_OCI_REF`, `MORN_COSIGN_CERTIFICATE_IDENTITY`,
`MORN_COSIGN_OIDC_ISSUER` and, for private registries, the explicitly allowed
Cosign environment. The gate deliberately reports
`artifact_published_by_this_gate=false`, `site_admitted=false`,
`customer_acceptance=false` and `production_write=false`.

Cryptographic verification proves the named OCI subject under the configured
identity policy. It does **not** prove that a capability is qualified for a
Profile/site, that a customer accepted an outcome, or that a production action
is authorized.

## Provider subprocess environment

Real DSH and Pi runtimes are launched with a **scrubbed child environment** rather than inheriting the entire Morn server process.

- DSH receives only the minimal OS/runtime environment. `MORN_DSH_HOME` is an isolated **root**, not the runtime's mutable home: every real E0 launch gets a fresh one-shot child Home under that root, so stale profile/home patches or plugins cannot silently reinterpret a binding. The route also applies a final Morn-owned global deny-tool overlay, forces a read-only DSH sandbox, treats max-token termination as non-success, and disables DSH telemetry.
- Pi receives the same minimal OS/runtime environment and the official RPC route is launched with `--no-tools --no-mcp`.
- Provider credentials or proxy variables are **not inherited implicitly**. Add only the names required by a deployment through `MORN_DSH_ENV_PASSTHROUGH` or `MORN_PI_ENV_PASSTHROUGH` (comma/semicolon separated).
- Prefer provider-managed/OS secret stores over environment credentials. Explicitly passing an API-key environment variable makes that value visible to the provider subprocess and any tools it launches, so it is a deliberate weaker boundary.
- Never include broad variables such as `GITHUB_TOKEN`, database credentials, cloud-admin secrets, or unrelated application secrets.

Example:

```powershell
$env:MORN_DSH_MODE = "real"
$env:MORN_DSH_WORKSPACE = "C:\morn\workspaces\dsh"
$env:MORN_DSH_HOME = "C:\morn\runtime\dsh-home"
$env:MORN_DSH_EXECUTION_ENVIRONMENT_REF = "env://container/dsh-runtime-a"
$env:MORN_DSH_PROFILE_CONFIGURATION_REF = "deepseek-harness-profile@<config-version>#sha256:<64-hex>"
# This secret-free, attested ref identifies the exact allowed SDK-profile composition.
# Morn creates a fresh empty child DSH_HOME per runtime; pre-existing patches/plugins
# under MORN_DSH_HOME are not loaded into that child composition.
# MORN_DSH_WORKSPACE and MORN_DSH_HOME must be disjoint directory trees.
# The environment ref must be issued/pinned by the same execution-environment
# provisioning path that produced the RuntimeContext/ExecutionBinding.
# Only when the chosen DSH profile truly requires env-based credentials:
$env:MORN_DSH_ENV_PASSTHROUGH = "DEEPSEEK_API_KEY,HTTPS_PROXY"
```

Pi uses the same opt-in model:

```powershell
$env:MORN_PI_MODE = "real"
$env:MORN_PI_WORKSPACE = "C:\\morn\\workspaces\\pi"
$env:MORN_PI_PROVIDER = "<exact-provider>"
$env:MORN_PI_MODEL = "<exact-model>"
$env:MORN_PI_EXECUTION_ENVIRONMENT_REF = "env://container/pi-runtime-a"
# The Pi launch environment ref must match the RuntimeContext/ExecutionBinding.
# Only when that provider requires environment credentials:
$env:MORN_PI_ENV_PASSTHROUGH = "<REQUIRED_API_KEY_NAME>"
```

This implements ADR-009: credentials are resolved at the execution/provider boundary and do not become general Morn process context.

A real Harness launch is rejected unless the provider configuration's
`execution_environment_ref` exactly matches the environment identity carried
by the RuntimeContext/ExecutionBinding and that context proves the required
container-or-stronger guarantee vector. Merely writing an isolation class into a
request is not sufficient. The deployment is responsible for ensuring that the
configured DSH/Pi command actually executes inside that named environment;
Morn does not infer containment from a binary name or from Docker being
installed.



## Execution-environment runtime identity attestation

A real DSH/Pi provider version or digest is **not trusted merely because an
environment variable says so**. The same deployment-owned environment
attestation used for containment must list every exact provider runtime artifact
allowed to execute there.

Use `runtime_identities` with the canonical form
`provider@version#sha256:<64-hex>`. For DSH, attest the runtime artifact,
the exact Home/Profile composition identity **and the exact semantic-route
identity emitted by `dsh_identity_plan`**. Omitting the route identity makes
the real-provider gate fail closed. The Morn real-SDK path does not consume arbitrary persistent Home/Profile overrides:
each launch starts from a fresh child Home plus the runtime-pinned official SDK profile
and Morn's final policy overlay. A permitted profile-composition change requires a new
configuration digest/attestation and therefore a new ExecutionBinding.
For example:

```json
{
  "environment_ref": "env://container/dsh-runtime-a",
  "provider": "deployment-attestor",
  "isolation": "container",
  "attested_spec": {
    "minimum_isolation": "container",
    "required_guarantees": [
      "filesystem-read-policy",
      "filesystem-write-policy",
      "process-boundary",
      "resource-limits",
      "network-egress-policy",
      "secret-indirection",
      "runtime-attestation",
      "tool-mediation"
    ],
    "network_allowlist": ["api.deepseek.com"],
    "writable_paths": ["C:\\morn\\workspaces\\dsh"],
    "secret_refs": ["secret://deepseek/provider-credential"],
    "persistence_scope": "attempt",
    "side_effect_policy": "profile-governed"
  },
  "runtime_identities": [
    "deepseek-harness@<deployment-attested-version>#sha256:<64-hex>",
    "deepseek-harness-profile@<config-version>#sha256:<64-hex>",
    "deepseek-harness-route@v1#sha256:<64-hex>"
  ],
  "evidence_refs": [
    "attestation://sandbox-fleet/dsh-runtime-a",
    "artifact-attestation://deepseek-harness/<digest>"
  ],
  "observed_at": "2026-10-09T00:00:00Z",
  "valid_until": "2026-10-10T00:00:00Z"
}
```

For ACP lifecycle control, use the ACP-specific route identity
`deepseek-harness-acp-route@v1#sha256:<64-hex>` from
`MORN_DSH_PLAN_MODE=acp cargo run -p morn-app --bin dsh_identity_plan`.
An environment may attest SDK and ACP routes together only when the deployment
has independently approved both exact compositions.

For Pi, use `pi@<version>#sha256:<digest>`. The runtime identity must match the
provider version/digest written into `ExecutionBinding` exactly. Expiry,
revocation/replacement of the environment attestation, a different digest, or a
different version blocks both new binding and later execution.

This closes a deliberate trust boundary: SDK/RPC handshakes prove protocol
behavior and liveness; they do not prove which distribution artifact the
deployment launched.

## Live provider evidence gate

Before an execution-environment attestor signs a real DSH deployment, derive the
exact **secret-free identities** that the attestation must bind:

```powershell
$env:MORN_DSH_PLAN_MODE = "sdk" # or "acp"
# configure the same MORN_DSH_* or MORN_DSH_ACP_* route/version/digest values
cargo run -p morn-app --bin dsh_identity_plan
```

The plan command never starts DSH and never requires a model credential. It
validates the real configuration and emits the exact runtime artifact identity,
profile-composition identity and hashed semantic-route identity. ACP keeps its
raw `dsh-acp-route:{...}` only for internal diagnostics; Work identity and
execution-environment attestations use
`deepseek-harness-acp-route@v1#sha256:<64-hex>`, so configuration changes
cannot silently reuse an older deployment attestation.

### Canonical DSH runtime artifact digest

For the official Python SDK distribution path, `MORN_DSH_RUNTIME_DIGEST` is
defined as the SHA-256 of the exact platform-specific
`deepseek-harness-runtime-bin` wheel whose version equals
`MORN_DSH_RUNTIME_VERSION`. This is the artifact that contains the bundled
`dsh` runtime executable used by the Python SDK. The digest must use exactly
`sha256:<64-lowercase-hex>`; alternate casing is rejected so one artifact
cannot acquire multiple semantic identities.

A version pin by itself is insufficient. The live-evidence workflow first
downloads that exact runtime-bin wheel, hashes its bytes, compares the result to
`MORN_DSH_RUNTIME_DIGEST`, validates the wheel package/version metadata, writes
a secret-free `runtime-artifact-verification.json`, and only then installs the
wheel and matching `deepseek-harness-sdk`. A mismatch stops before any model
credential is exercised. The resulting identity is therefore
`deepseek-harness@<version>#sha256:<runtime-bin-wheel-bytes>`, not a digest
copied from an environment variable without byte verification.

The same lowercase-canonical rule applies to
`deepseek-harness-profile@<version>#sha256:<digest>`. Profile and semantic-route
digests remain separate identities: hashing the runtime wheel does not prove the
Cordis/Home/Profile composition or the selected provider/model route.

For GitHub-hosted deployment verification, the repository also provides the
manual **Live DSH evidence** workflow. It has four explicit operations:
`plan-sdk`, `plan-acp`, `smoke-sdk`, and `smoke-acp`. Planning needs no
credential. Smoke modes require an explicit E0-only acknowledgement plus the
repository/environment secrets `DEEPSEEK_API_KEY` and
`MORN_EXECUTION_ATTESTATION_JSON`; the latter is materialized only on the
ephemeral runner. Configure a protected GitHub Environment or equivalent
organization approval around these secrets for real deployments. The workflow
uploads only identity/smoke reports and never promotes Outcome, Acceptance or
ProductionWrite claims.

Morn includes a one-shot deployment gate for proving the **executor transport**
against the exact configured real provider and deployment-owned execution
environment:

```powershell
cargo run -p morn-app --bin provider_smoke
```

Set `MORN_PROVIDER_SMOKE_PROVIDER` to `deepseek-harness` (or `dsh`) or
`pi`. The gate loads the normal application deployment configuration, requires
an active attestation whose `environment_ref` exactly matches the provider's
configured environment, requires that attestation to bind the exact configured
provider runtime artifact identity, mounts the provider's required E0 scope,
runs a no-tool probe, verifies the live health lease, and reaps the owned
provider runtime.
Unexpected Harness tool activity makes the gate fail closed. For DSH, a
single-use Morn overlay installs a global monotonic deny guard **before** the
model turn; for Pi, `--no-tools --no-mcp` is part of the real RPC command.
These local controls do not self-attest an arbitrary deployment, so the
execution-environment attestation must still include `tool-mediation`.

DSH example (values are deployment-specific; never commit secret values):

```powershell
$env:MORN_PROVIDER_SMOKE_PROVIDER = "deepseek-harness"
$env:MORN_DSH_MODE = "real"
$env:MORN_DSH_WORKSPACE = "C:\morn\workspaces\dsh"
$env:MORN_DSH_HOME = "C:\morn\runtime\dsh-home"
$env:MORN_DSH_PROVIDER = "deepseek-official"
$env:MORN_DSH_MODEL = "<exact-model-route>"
$env:MORN_DSH_EXECUTION_ENVIRONMENT_REF = "env://container/dsh-runtime-a"
$env:MORN_DSH_PROFILE_CONFIGURATION_REF = "deepseek-harness-profile@<config-version>#sha256:<64-hex>"
$env:MORN_DSH_RUNTIME_VERSION = "<deployment-attested-version>"
$env:MORN_DSH_RUNTIME_DIGEST = "sha256:<64-hex>"
$env:MORN_EXECUTION_ATTESTOR = "<deployment-attestor-name>"
$env:MORN_EXECUTION_ATTESTATION_FILE = "C:\morn\trust\execution-environments.json"
# Only if this exact provider route requires an environment credential:
$env:MORN_DSH_ENV_PASSTHROUGH = "DEEPSEEK_API_KEY"
cargo run -p morn-app --bin provider_smoke
```

Pi uses the same gate with `MORN_PROVIDER_SMOKE_PROVIDER=pi`,
`MORN_PI_MODE=real`, `MORN_PI_WORKSPACE`, `MORN_PI_PROVIDER`,
`MORN_PI_MODEL`, `MORN_PI_EXECUTION_ENVIRONMENT_REF`,
`MORN_PI_RUNTIME_VERSION`, `MORN_PI_RUNTIME_DIGEST`, and optionally
`MORN_PI_COMMAND` / `MORN_PI_ENV_PASSTHROUGH`.

A successful JSON report sets `executor_live_evidence=true`. It **always**
keeps `canonical_work_outcome=false`, `customer_acceptance=false`, and
`production_write=false`: a provider/model turn is execution evidence, not
business truth. Exit code 2 means `NOT_PROVEN` rather than a fabricated pass.

## Reproducible Pi RPC runtime identity

Pi's official installer/update path owns a generated
`packages/coding-agent/install-lock/package-lock.json` for the runtime dependency
tree. Direct `npm install -g` is not a sufficient provenance boundary because
transitive dependency resolution can drift independently of Morn's
`ExecutionBinding`.

Morn's manual **Live Pi evidence** workflow therefore checks out the exact
`earendil-works/pi@v<runtime_version>` release, copies only its installer
`package.json` + `package-lock.json` into an isolated directory, and executes
`npm ci --ignore-scripts`. It then derives a stable SHA-256 over the actual
installed runtime tree: sorted relative paths, symlink targets, executable/file
modes and each regular file's SHA-256, together with the two installer metadata
files. The canonical runtime identity is:

```text
pi@<version>#sha256:<installed-runtime-tree-digest>
```

The tree digest is platform-specific by design: optional/native packages that
differ by deployment platform are part of the executable closure and must not
share an identity accidentally. The workflow's `plan` operation computes this
digest without provider credentials. A later `smoke` operation requires the
deployment-attested digest to match the freshly reconstructed runtime closure
before exposing any model credential.

The workflow then runs `pi_identity_plan` to derive the independent
`pi-rpc-route@v1#sha256:<digest>` semantic-route identity from Pi's exact
provider, model, RPC containment flags and Morn isolation policy. Runtime-tree
identity and route identity are both required by the deployment execution
attestation. The authenticated smoke reuses the ordinary `provider_smoke`
binary with `MORN_PROVIDER_SMOKE_PROVIDER=pi`; successful RPC settlement is
executor evidence only and never creates Work Outcome, Acceptance or
ProductionWrite evidence.

Secrets remain step-scoped. Configure:
- `MORN_PI_PROVIDER_API_KEY` for the provider credential;
- optional `MORN_PI_PROVIDER_BASE_URL`;
- `MORN_EXECUTION_ATTESTATION_JSON`;
- `MORN_EXECUTION_ATTESTOR`.

The workflow input `credential_env_name` selects the provider-specific
environment variable name that receives the generic credential only for the
smoke step. The child Pi process still receives only names explicitly listed in
`MORN_PI_ENV_PASSTHROUGH`.

## Capability distribution trust gate

Capability release is split into three independent deployment steps:

1. **Publish** the artifact to an OCI registry and keep the digest-pinned ORAS receipt.
2. **Verify signature** against the exact digest and expected Sigstore certificate identity/OIDC issuer.
3. **Verify SLSA provenance** independently for the same digest.

Morn intentionally does not inherit arbitrary registry credentials. The CLI
requires explicit binary paths and forwards only an optional Docker config
directory:

~~~powershell
$env:MORN_ORAS_BIN = "C:\\tools\\oras.exe"
$env:MORN_COSIGN_BIN = "C:\\tools\\cosign.exe"
$env:MORN_REGISTRY_DOCKER_CONFIG = "C:\\morn\\registry-auth"

morn package publish `
  oci://registry.example/morn/capabilities/reviewer `
  1.2.3 `
  application/vnd.morn.capability.v1+json `
  C:\\build\\reviewer.json `
  application/vnd.morn.capability.layer.v1+json
~~~

The publish command prints an ORAS-derived JSON receipt containing the
digest-pinned OCI reference. Verify that subject with the expected workload
identity:

~~~powershell
morn package verify-supply-chain `
  oci://registry.example/morn/capabilities/reviewer@sha256:<64-hex> `
  sha256:<64-hex> `
  "https://github.com/acme/morn/.github/workflows/release.yml@refs/heads/main" `
  "https://token.actions.githubusercontent.com"
~~~

The verification command executes both Cosign signature verification and SLSA
provenance verification. It outputs a merged
`SupplyChainVerificationEvidence` only when both proofs bind the same subject
digest. Save one object or an array of objects to a deployment-owned file and
load it before starting Morn:

~~~powershell
$env:MORN_SUPPLY_CHAIN_VERIFICATIONS_FILE = "C:\\morn\\trust\\capability-supply-chain.json"
~~~

The HTTP release endpoint cannot mint this trust. If a caller submits only
`signature_ref` / `provenance_ref`, Morn may record the content-addressed
distribution metadata but marks it unverified; the response points to
`deployment-supply-chain-verification`, and SiteAdmission rejects that
release. Hub and Console expose the verified/unverified state.

This local adapter/verification flow does **not** prove a real registry or
builder identity by itself. A production gate still requires the actual
authorized registry, OIDC certificate identity and provenance generated by the
deployment build system.

## Authoritative source bindings

Morn does not allow an HTTP/UI caller or a Harness to self-declare a business
source as authoritative. Deployment-reviewed source bindings are loaded at
server startup from `MORN_SOURCE_OF_TRUTH_BINDINGS_FILE`.

The file may contain one binding or an array. It contains authority metadata
and credential/query references, **not secret material**. Example:

```json
[
  {
    "id": "sot:plant-a-cmms-orders",
    "site_ref": "plant-a",
    "source_ref": "cmms://plant-a/orders",
    "authority_kind": "SystemOfRecord",
    "authoritative_fact_types": ["maintenance.order", "delivery.status"],
    "key_mapping_ref": "mapping://cmms-order-key@1",
    "query_capability_ref": "capability://cmms.read-order@1",
    "freshness_sla_ms": 30000,
    "conflict_policy": "ReconcileBeforeUse",
    "version_ref": "binding:v1",
    "created_at": "2026-10-09T00:00:00Z"
  }
]
```

At runtime the product flow is deliberately split:

1. `GET /api/v115/source-of-truth/catalog` exposes only reviewed deployment metadata.
2. `POST /api/v115/work/bind-source-of-truth` attaches a unique immutable copy to one exact Work generation and emits `SourceOfTruthBound` evidence.
3. `POST /api/v115/work/observe-outcome` accepts an observation only when its fact type and source URI are covered by that Work-scoped binding and explicit evidence references are supplied. The server stamps observation time.
4. The observation may make Work `Delivered`; it **never** makes it `Accepted`. Independent `review-outcome` remains a separate decision.

A Harness receipt, assistant message, arbitrary URL, or caller-provided
`authority_kind` cannot create this authority. Real connectors may later own
the read operation itself, but must still terminate at the same
`SourceOfTruthBinding -> ObservedOutcome` boundary.

## Deployment-owned evidence ledger

Release/production posture is not inferred from a green UI, a provider response,
or repository history. Operational deployments may load controlled claim
records through `MORN_EVIDENCE_CLAIMS_FILE`. When this file is configured,
`MORN_EVIDENCE_ISSUER` is mandatory and every claim's `issuer` must exactly
match that deployment-pinned identity; the payload cannot select its own trusted
issuer. File ownership/signature/provenance enforcement remains a deployment
responsibility and should be backed by the same protected configuration channel
used for execution attestations.

Customer/production readiness is also site-bound. Set
`MORN_DEPLOYMENT_SITE_REF` to the exact deployment/site identity. RealSite and
ProductionWrite claims for that deployment must use the subjects
`factory-customer:<site-ref>` and `factory-production-write:<site-ref>`.
Evidence for one site cannot make another site ready. Omitting the site keeps all
customer-read-only and production-write named gates blocked.

The server first loads repository baseline claims and then appends the
deployment file. Repository blockers use a historical baseline timestamp, so
genuine evidence produced before process startup can supersede them without
violating append-only claim ordering.

Example deployment trust pins:

```powershell
$env:MORN_EVIDENCE_CLAIMS_FILE = "C:\morn\trust\release-evidence.json"
$env:MORN_EVIDENCE_ISSUER = "deployment-attestor-prod-a"
$env:MORN_DEPLOYMENT_SITE_REF = "plant-a"
```

The running deployment must also set an immutable build identity reference, for
example:

```powershell
$env:MORN_BUILD_IDENTITY_REF = "git://huangdi97/morn/bdc9e924f962f436a0dd52a4ac43c4ad5a83774c"
```

A CI claim is current only when one of its `evidence_refs` exactly equals
`MORN_BUILD_IDENTITY_REF`. A proof for another SHA is reported as
`identity-mismatch`; omitting the build identity keeps CI readiness
`missing-evidence` even when a historical CI claim exists.

Example exact-head CI proof:

```json
[
  {
    "id": "evidence-claim:ci-conformance-2026-10-10",
    "subject": "morn-v11.5-ci-conformance",
    "class": "ci-conformance",
    "state": "proven",
    "evidence_refs": [
      "github-actions://huangdi97/morn/runs/38064799438",
      "git://huangdi97/morn/bdc9e924f962f436a0dd52a4ac43c4ad5a83774c"
    ],
    "issuer": "deployment-attestor-prod-a",
    "reason": "exact-head configured repository gates passed",
    "observed_at": "2026-10-10T15:46:00Z"
  }
]
```

A RealRuntime proof must cite the **whole active execution identity**, not only the
provider executable. For DSH that means the exact runtime artifact, exact
execution-environment identity, exact Home/Profile composition identity, and
exact provider/model route attestation.
The live `provider_smoke` report emits `required_real_runtime_claim_refs`
so the deployment attestor does not have to reconstruct this set manually:

```json
{
  "id": "evidence-claim:dsh-live-2026-10-13",
  "subject": "deepseek-harness",
  "class": "real-runtime",
  "state": "proven",
  "evidence_refs": [
    "runtime://deepseek-harness/<version>#sha256:<64-hex>",
    "env://container/dsh-runtime-a",
    "deepseek-harness-profile@<config-version>#sha256:<64-hex>",
    "deepseek-harness-route@v1#sha256:<64-hex>",
    "attestation://provider-smoke/dsh-2026-10-13"
  ],
  "issuer": "deployment-attestor-prod-a",
  "reason": "authenticated official runtime smoke passed for the exact pinned DSH semantic route",
  "observed_at": "2026-10-13T03:30:00Z"
}
```

Pi uses the same rule with
`runtime://pi/<version>#sha256:<64-hex>`, its exact
`MORN_PI_EXECUTION_ENVIRONMENT_REF`, and
`pi-rpc-route@v1#sha256:<64-hex>`.

Real-runtime, real-site and production-write claims use the same append-only
ledger but remain separate evidence classes. A later `blocked-external` or
`revoked` record for the same subject/class invalidates a previous proof until
a fresh `proven` record is appended. Claims for the same subject/class must
have strictly increasing `observed_at` values; equal-time contradictory claims
are rejected so JSON/file order cannot choose readiness. A future-dated current
claim is projected as `not-yet-valid` rather than proof. The Console release-readiness projection
also requires a **fresh Healthy provider lease** in addition to a
`real-runtime` claim for DSH/Pi, and rechecks every currently configured
execution/semantic identity. A smoke proof from a different environment, or for
an older runtime, profile or route, cannot be combined with health from the
currently configured provider.

Canonical subjects used by the top-level readiness projection are:

- `morn-v11.5-architecture` / `design-spec`;
- `factory-readonly-wedge` / `local-fixture`;
- `morn-v11.5-ci-conformance` / `ci-conformance`;
- `deepseek-harness` and `pi-harness` / `real-runtime`, with evidence
  referencing the exact active runtime artifact **and** every active semantic
  route identity required by that provider;
- `factory-customer:<MORN_DEPLOYMENT_SITE_REF>` / `real-site`;
- `factory-production-write:<MORN_DEPLOYMENT_SITE_REF>` / `production-write`.

These are evidence statements, **not Authority grants**. In particular, a
`production-write` evidence claim cannot authorize a Work action. Profile,
Work generation, ExecutionBinding, AuthorityDecision and ExternalActionPermit
checks remain mandatory at the action boundary. HTTP clients cannot append
evidence claims; the file is deployment-owned.

The status API also evaluates server-defined deployment scopes rather than one
ambiguous global "ready" boolean:

- `local-reference`: architecture + local reference slice + exact-build CI;
- `deepseek-read-only` / `pi-read-only`: local-reference + the selected live runtime;
- `customer-read-only-*`: selected live runtime + explicit RealSite evidence;
- `production-write-*`: customer scope + ProductionWrite evidence **and** an
  independently permissive Profile/Authority posture.

The current Factory Read-Only Profile fails the final authority condition even
if a historical production-write evidence claim exists. Clients cannot remove
required axes from these named scopes.

## Independent acceptance reviewers

An acceptance role is not trusted because a browser submits the string
`independent-reviewer`. Reviewer principals and allowed roles are deployment
identity evidence loaded from `MORN_ACCEPTANCE_REVIEWERS_FILE`.

Example:

```json
[
  {
    "principal_id": "prc:quality-reviewer-a",
    "acting_roles": ["independent-reviewer"],
    "evidence_refs": ["iam://quality/reviewers/a"],
    "observed_at": "2026-10-09T00:00:00Z",
    "valid_until": "2027-01-01T00:00:00Z"
  }
]
```

The Workbench may select one of these reviewers, but cannot invent a principal
or role. The review API rejects expired/unattested principal-role pairs and
rejects the workspace owner as the independent acceptance reviewer. Identity
attestation evidence is copied into the immutable `AcceptanceDecision`
alongside the review's own evidence.

This is a separation-of-duties control, not proof of customer acceptance by
itself. Production deployments should source these attestations from the
customer IAM/governance system and independently audit who was allowed to
approve which acceptance contract.

## Customer-validated value attestations

A real-site evidence claim proves that a Work episode has deployment-owned
site evidence. It does **not** prove arbitrary ROI/KPI numbers typed later into
the Workbench. Values presented as `CustomerValidated` therefore require a
second, exact deployment artifact loaded through
`MORN_CUSTOMER_VALUE_ATTESTATIONS_FILE`.

Each attestation binds one workspace, Work generation, source-grounded Outcome
and final Acceptance decision to the baseline and KPI values that the customer
or deployment governance process actually reviewed:

```json
[
  {
    "attestation_id": "customer-value-42",
    "workspace_id": "ws:customer-a",
    "work_package_id": "work:delivery-review-42",
    "work_generation": 1,
    "outcome_id": "out:delivery-review-42",
    "acceptance_id": "adec:delivery-review-42",
    "baseline_ref": "baseline://customer-approved/42",
    "kpis": [
      { "name": "human_minutes_saved", "value": 12.5 },
      { "name": "late_minutes_delta", "value": -12.0 }
    ],
    "evidence_refs": [
      "customer://signed/value-review/42"
    ],
    "issuer": "customer-value-governance",
    "observed_at": "2026-10-10T00:00:00Z",
    "valid_until": "2027-01-01T00:00:00Z"
  }
]
```

The `POST /api/v115/work/assess-value` contract is deliberately asymmetric:

- Fixture/Simulation/Shadow/ObservedOperational assessments may submit their
  own evidence references and optional baseline/KPI data, and remain labeled as
  that evidence class.
- CustomerValidated accepts only the exact
  `customer_value_attestation_id`; callers may not submit `evidence_refs`,
  `baseline_ref` or `kpis` for that class.
- The attestation must match the current Work generation, Outcome and final
  Acceptance, be time-valid, and the exact value subject must also have a
  current deployment-owned `RealSite` evidence claim.
- A customer-value attestation is consumed once. Replay is rejected rather than
  creating several apparently independent customer validations from one signed
  review.
- Historical assessments remain immutable. Current support is shown separately
  and becomes unsupported when the RealSite evidence is revoked/blocked or the
  exact customer-value attestation is no longer active.

The Workbench requests the out-of-band attestation ID in CustomerValidated
mode and renders the signed KPI/baseline copied from deployment evidence. It
does not offer editable customer KPI fields in that mode.

## Migration / upgrade

- Migrations are versioned with preflight, dry-run, apply, verify, and
  restore-plan support (`morn-cli migrate`, `morn-core-tests` migration
  security suite).
- Downgrade without a restore plan is rejected; migration failures do not
  corrupt data.
- Back up the SQLite file before applying a destructive migration.

## Troubleshooting

| Symptom | Likely cause / fix |
| --- | --- |
| `frontend test` fails with `spawn EPERM` | esbuild native spawn blocked by a restricted shell/sandbox; run outside the sandbox or via CI. |
| `/api/biolab/*` returns 404 | server was built without the `domain-biolab` feature; build `--all-features` (zero-domain core intentionally has no domain routes). |
| Playwright missing | `cd frontend; npx playwright install chromium` |
| Server won't start on a port | port in use; set `MORN_PORT`. |
| DB locked/busy | close other processes on the same SQLite file; SQLite is single-writer. |
| Tauri GUI won't launch in a restricted shell | WebView2/launch token denied (environment boundary); run outside the restricted token. |

## Verification

```powershell
powershell -ExecutionPolicy Bypass -File scripts/run_all.ps1
```

Exit 0 = fmt/check/clippy/tests + frontend + Tauri + UI smoke + demo smoke all
pass. CI runs the same gates on GitHub Actions.


## DeepSeek ACP lifecycle control gate

Morn keeps the official DSH SDK execution provider and the ACP lifecycle/control
transport as separate evidence channels. SDK settled turns may contribute to
provider runtime-health evidence when all SDK gates pass. ACP never does:
upstream ACP intentionally maps several underlying Harness endings to the same
`end_turn` stop reason, so an ACP prompt result cannot prove executor success,
ObservedOutcome, Acceptance or production authorization.

The Work-bound ACP controller requires all of the following before it creates a
session:

- exact Work id/generation and `ExecutionBinding` in `RuntimeContext`;
- a currently mounted **ExecutionRun** scope with `morn.effects<=E0`;
- deployment-attested container-or-stronger execution guarantees;
- exact execution-environment identity;
- exact DSH runtime version + SHA-256 digest;
- exact Home/Profile configuration identity;
- exact provider/model route and optional reasoning effort.

After `session/new`, Morn immediately applies official ACP
`session/set_config_option` for the configured model route and verifies the
server-reported `currentValue` before the first prompt. Resume repeats this
route pin. A revoked/recreated scope with a changed snapshot cannot take over an
existing session. Ambiguous/unknown prompt settlement faults the session and
forbids blind prompt retry; cancellation/close/shutdown remain available for
quiescent cleanup.

Configure ACP independently from the SDK provider. For the deployment-only smoke gate,
the ACP variables below are sufficient. To expose the same governed lifecycle
through the normal Morn server/Workbench, also set
`MORN_DSH_ACP_MODE=real`. If the mode is absent or `disabled`, the product
API reports ACP as unconfigured and does not spawn or accept lifecycle sessions.

~~~powershell
$env:MORN_DSH_ACP_MODE = "real"
$env:MORN_DSH_ACP_WORKSPACE = "C:\morn\workspaces\dsh-acp"
$env:MORN_DSH_ACP_HOME = "C:\morn\runtime\dsh-acp-home-root"
$env:MORN_DSH_ACP_PROVIDER = "deepseek-official"
$env:MORN_DSH_ACP_MODEL = "<exact-model-route>"
# Optional, only when the exact route declares this effort:
$env:MORN_DSH_ACP_REASONING_EFFORT = "<exact-effort>"
$env:MORN_DSH_ACP_EXECUTION_ENVIRONMENT_REF = "env://container/dsh-runtime-a"
$env:MORN_DSH_ACP_PROFILE_CONFIGURATION_REF = "deepseek-harness-profile@<config-version>#sha256:<64-hex>"
$env:MORN_DSH_ACP_RUNTIME_VERSION = "<deployment-attested-version>"
$env:MORN_DSH_ACP_RUNTIME_DIGEST = "sha256:<64-hex>"
$env:MORN_EXECUTION_ATTESTOR = "<deployment-attestor-name>"
$env:MORN_EXECUTION_ATTESTATION_FILE = "C:\morn\trust\execution-environments.json"
# Only if the exact provider route needs environment credentials:
$env:MORN_DSH_ACP_ENV_PASSTHROUGH = "DEEPSEEK_API_KEY"
cargo run -p morn-app --bin dsh_acp_lifecycle_smoke
~~~

Optional controls are `MORN_DSH_ACP_COMMAND`,
`MORN_DSH_ACP_REQUEST_TIMEOUT_MS`, `MORN_DSH_ACP_PROMPT_TIMEOUT_MS` and
`MORN_DSH_ACP_SMOKE_DB`.

### Work-bound product orchestration

The product path deliberately uses a **separate immutable lifecycle Binding**.
Create it through `POST /api/v115/work/bind-attested-e0` with
`transport: "acp-lifecycle"`. That Binding pins the exact Work generation,
capability, attested execution environment, DSH runtime version/digest and ACP
route. The ordinary `/work/execute-e0` path rejects/excludes this Binding; it
is not an SDK executor Binding.

Workbench and the HTTP API then expose:

- `GET /api/v115/work/acp-lifecycle/sessions` — non-secret configured route
  identity plus Work-bound lifecycle snapshots;
- `POST .../open` — mount one E0 ExecutionRun scope and create one session;
- `POST .../prompt` — execute only while the Work generation, capability
  admission, environment and exact Binding route remain current;
- `POST .../cancel` — send a session-scoped out-of-band cancellation request
  without acquiring the blocking prompt/controller mutex;
- `POST .../close` — quiesce the session but retain its exact scope for
  same-process resume;
- `POST .../resume` — continue only after revalidating the original
  Work/generation/Binding/provider route/environment/scope snapshot;
- `POST .../release` — permanently release the lifecycle scope and remove the
  reachable cancel handle.

Cancel/close/release remain usable for a historical generation so operators can
quiesce stale activity safely. New prompt/resume is fail-closed after the Work
generation changes. None of these endpoints creates `ObservedOutcome`,
`AcceptanceDecision`, executor-health evidence or production-write authority.

A `MORN_LIVE_DSH_ACP_LIFECYCLE_SMOKE = PASS` result proves only that the
deployment-attested ACP runtime could create a Work-bound E0 session, pin the
exact route, reach protocol settlement, return the lifecycle canary, close the
session, revoke its scope and clean up the transport. The report deliberately
keeps `executor_success=false`, `provider_runtime_health=false`,
`canonical_work_outcome=false`, `customer_acceptance=false` and
`production_write=false`.

Production ACP still uses a one-shot DSH_HOME. Therefore same-process
`session/close -> session/resume` is supported and revalidated, while
cross-process resume is intentionally fail-closed: the session store was
reaped with the one-shot Home. Cross-process recovery would require a separate
deployment-owned, integrity-pinned durable session store design and evidence.
