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
