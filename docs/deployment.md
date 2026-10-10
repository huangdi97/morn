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
`provider@version#sha256:<64-hex>`. For DSH, attest both the runtime artifact
and the exact Home/Profile composition identity. The Morn real-SDK path does not consume arbitrary persistent Home/Profile overrides:
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
    "deepseek-harness-profile@<config-version>#sha256:<64-hex>"
  ],
  "evidence_refs": [
    "attestation://sandbox-fleet/dsh-runtime-a",
    "artifact-attestation://deepseek-harness/<digest>"
  ],
  "observed_at": "2026-10-09T00:00:00Z",
  "valid_until": "2026-10-10T00:00:00Z"
}
```

For Pi, use `pi@<version>#sha256:<digest>`. The runtime identity must match the
provider version/digest written into `ExecutionBinding` exactly. Expiry,
revocation/replacement of the environment attestation, a different digest, or a
different version blocks both new binding and later execution.

This closes a deliberate trust boundary: SDK/RPC handshakes prove protocol
behavior and liveness; they do not prove which distribution artifact the
deployment launched.

## Live provider evidence gate

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
