# Performance Baseline — Morn v1 GA

Goal: `MORN-G6R-V1-GA-REFOUNDATION-GITHUB-TAKEOVER`
Date: 2026-08-16
Host: Windows, x86_64-pc-windows-msvc, Rust 1.97.1, Node 22.15.0, debug build.

## Cold start / readiness

Measured by starting `target/debug/server.exe` (all-features) fresh with a
new SQLite DB and polling `/api/health` until `status=ok`:

```text
server cold start -> /api/health ok   ~1.9 s measured
   (includes process spawn + SQLite open + schema v2 migration + first
    HTTP round trip; polled at 100 ms granularity)
```

`run_all.ps1` sleeps 2 s before the first health check — health is ready in
that window.

## Representative API latency (loopback, debug build, first hit)

```text
GET  /api/workbench                 40 ms   (first hit, cold connection)
POST /api/demo/bootstrap            29 ms
POST /api/biolab/run                22 ms   (7-step governed E2E in-process)
GET  /api/workbench (after seed)     4 ms   (warm)
```

Measured 2026-08-16 with `target/debug/server.exe` (all-features), fresh
SQLite DB in `target/perf_baseline.db`. No N+1 or full-table-scan hot paths
observed at the demo record scale. Durable/runtime and predictor paths are
in-process (no external calls).

## Work E2E

```text
BioLab E2E (dataset -> analysis -> review -> approval -> claim outcome): 7 steps,
all_ok=True, sub-second in debug build (demo smoke measured e2e_steps=7).
```

## Frontend production build size (2026-08-16)

```text
dist/index.html                0.43 kB   (gzip 0.30 kB)
dist/assets/index-*.css        2.70 kB   (gzip 1.01 kB)
dist/assets/index-*.js       187.7 kB   (gzip 59.9 kB)
38 modules transformed; build ~2.3-2.6 s
```

No obvious bundle explosion; single small JS chunk.

## Rust workspace

```text
cargo test --workspace --all-features: 218+ tests, 0 ignored, ~2-4 s runtime
   after compile; full run_all.ps1 ~4-6 min including compiles and smokes.
```

## Observations / known limits

- Debug build used for baseline; release (`lto = "thin"`) will be faster.
- No load test was run; this is a functional/order-of-magnitude baseline, not
  a capacity claim. No unbounded retries/loops or accidental huge payloads
  were found in the audit.
