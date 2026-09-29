# scripts/verify-all.ps1 — the unified verify gate (Phase 21, Goal §25).
#
# One command chaining every local gate: format check, lint, Rust tests
# (unit + integration + conformance + G-gates + reference scenario), web
# vitest + typecheck + build, and optionally e2e (SKIP_EXTERNAL by default —
# e2e needs a running dev server and the real browser channel).
#
# Usage:
#   pwsh scripts/verify-all.ps1            # local gates (SKIP_EXTERNAL for e2e)
#   pwsh scripts/verify-all.ps1 -WithE2E   # also run playwright e2e
#
# Fail-fast: any failing step exits non-zero. Steps that depend on external
# resources print SKIP_EXTERNAL explicitly and are never silently success.

param(
    [switch]$WithE2E
)

$ErrorActionPreference = "Stop"

# MSVC environment (required for cargo linking on this machine).
$msvcRoot = "D:\Code\Visual Studio\Visual Studio\VC\Tools\MSVC\14.50.35717"
$sdkRoot = "C:\Program Files (x86)\Windows Kits\10"
$env:INCLUDE = "$msvcRoot\include;$sdkRoot\Include\10.0.26100.0\ucrt;$sdkRoot\Include\10.0.26100.0\shared;$sdkRoot\Include\10.0.26100.0\um"
$env:LIB = "$msvcRoot\lib\x64;$sdkRoot\Lib\10.0.26100.0\ucrt\x64;$sdkRoot\Lib\10.0.26100.0\um\x64"

$failed = $false

function Run-Step {
    param($Name, $Command, $SkipMessage)
    if ($Command -eq $null) {
        Write-Host "== SKIP_EXTERNAL: $Name — $SkipMessage"
        return
    }
    Write-Host "== $Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "== FAIL: $Name (exit $LASTEXITCODE)"
        $script:failed = $true
    }
}

Write-Host "=== Morn v11.5-R1 verify-all ==="

# 1. Format check.
Run-Step "cargo fmt --check" { cargo fmt --check } $null

# 2. Lint (CI standard).
Run-Step "cargo clippy --lib -D warnings" { cargo clippy --lib -- -D warnings } $null

# 3. Rust tests: unit + integration + conformance + G-gates + reference
#    scenario (the workspace test set includes all of them).
Run-Step "cargo test --workspace" { cargo test --workspace } $null

# 4. Web vitest.
Push-Location web
try {
    Run-Step "web vitest" { npm run test } $null

    # 5. Web typecheck + build.
    Run-Step "web build (tsc + vite)" { npm run build } $null

    # 6. e2e: needs the dev server + real browser; SKIP_EXTERNAL by default.
    if ($WithE2E) {
        $proc = Start-Process npm.cmd -ArgumentList "run","dev" -PassThru -WindowStyle Hidden `
            -RedirectStandardOutput "$env:PI_SCRATCH_DIR\vite-verify.log" `
            -RedirectStandardError "$env:PI_SCRATCH_DIR\vite-verify-err.log"
        Start-Sleep -Seconds 8
        try {
            Run-Step "playwright e2e" { npx playwright test } $null
        } finally {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    } else {
        Write-Host "== SKIP_EXTERNAL: playwright e2e — requires dev server + real browser channel (run with -WithE2E)"
    }
} finally {
    Pop-Location
}

Write-Host "=== verify-all complete ==="
if ($failed) {
    Write-Host "RESULT: FAIL (see failing steps above)"
    exit 1
}
Write-Host "RESULT: ALL_LOCAL_GATES_PASS"
