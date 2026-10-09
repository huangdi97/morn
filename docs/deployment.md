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
# Only when the chosen DSH profile truly requires env-based credentials:
$env:MORN_DSH_ENV_PASSTHROUGH = "DEEPSEEK_API_KEY,HTTPS_PROXY"
```

This implements ADR-009: credentials are resolved at the execution/provider boundary and do not become general Morn process context.

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
