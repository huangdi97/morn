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

- DSH receives only the minimal OS/runtime environment plus the explicit `DSH_HOME` selected by Morn.
- Pi receives the same minimal OS/runtime environment.
- Provider credentials or proxy variables are **not inherited implicitly**. Add only the names required by a deployment through `MORN_DSH_ENV_PASSTHROUGH` or `MORN_PI_ENV_PASSTHROUGH` (comma/semicolon separated).
- Prefer provider-managed/OS secret stores over environment credentials. Explicitly passing an API-key environment variable makes that value visible to the provider subprocess and any tools it launches, so it is a deliberate weaker boundary.
- Never include broad variables such as `GITHUB_TOKEN`, database credentials, cloud-admin secrets, or unrelated application secrets.

Example:

```powershell
$env:MORN_DSH_MODE = "real"
$env:MORN_DSH_WORKSPACE = "C:\morn\workspaces\dsh"
$env:MORN_DSH_HOME = "C:\morn\runtime\dsh-home"
$env:MORN_DSH_EXECUTION_ENVIRONMENT_REF = "env://container/dsh-runtime-a"
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
