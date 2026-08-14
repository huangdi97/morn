$ErrorActionPreference = "Stop"
$env:Path = "C:\Users\Kaiser\.cargo\bin;" + $env:Path

function Run-Step {
    param([string]$Name, [scriptblock]$Action)
    Write-Host ""
    Write-Host "=== $Name ==="
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root
Write-Host "Morn full verification — root: $root"

# ---- Rust workspace ----
Run-Step "cargo fmt --check" { cargo fmt --all -- --check }
Run-Step "cargo check" { cargo check --workspace --all-targets }
Run-Step "cargo clippy (-D warnings)" { cargo clippy --workspace --all-targets --all-features -- -D warnings }
Run-Step "cargo test" { cargo test --workspace --all-features }

# ---- Frontend ----
$frontend = Join-Path $root "frontend"
if (Test-Path (Join-Path $frontend "package.json")) {
    Push-Location $frontend
    try {
        Run-Step "frontend typecheck" { npm run typecheck }
        Run-Step "frontend lint" { npm run lint }
        Run-Step "frontend test" { npm test }
        Run-Step "frontend build" { npm run build }
    }
    finally {
        Pop-Location
    }
}

# ---- Tauri desktop shell ----
if (Test-Path (Join-Path $root "src-tauri\Cargo.toml")) {
    Run-Step "tauri desktop build" { cargo build -p morn-desktop }
}

# ---- Demo smoke: start API server, run BioLab E2E, stop ----
Run-Step "build server binary" { cargo build -p morn-app --bin server }
Run-Step "demo smoke (server + BioLab E2E)" {
    $env:MORN_DB = Join-Path $root "target\smoke.db"
    $env:MORN_PORT = "8090"
    $server = Join-Path $root "target\debug\server.exe"
    if (-not (Test-Path $server)) {
        throw "server binary not found at $server"
    }
    $proc = Start-Process -FilePath $server -WindowStyle Hidden -PassThru
    try {
        Start-Sleep -Seconds 2
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/health" -TimeoutSec 10
        if ($health.status -ne "ok") { throw "health check failed" }
        $run = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/biolab/run" -Method Post -TimeoutSec 15
        if ($run.result.all_ok -ne $true) { throw "BioLab E2E did not pass" }
        $wb = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/workbench" -TimeoutSec 10
        if ($wb.world_objects.Count -lt 3) { throw "workbench world objects missing" }
        Write-Host "demo smoke OK: health=$($health.status) e2e_steps=$($run.result.steps.Count) objects=$($wb.world_objects.Count)"
    }
    finally {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "=== Verification complete ==="


