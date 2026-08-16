$ErrorActionPreference = "Stop"
$env:Path = "C:\Users\Kaiser\.cargo\bin;" + $env:Path

function Stop-Port8090 {
    $conns = Get-NetTCPConnection -LocalPort 8090 -State Listen -ErrorAction SilentlyContinue
    foreach ($conn in $conns) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
    }
    Start-Sleep -Seconds 1
}
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
Run-Step "domain boundary guard" { powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check_domain_boundary.ps1 }
Run-Step "zero-domain Core build" { cargo check -p morn-app }
Run-Step "core-tests (zero-domain: migration/security/chaos/conformance/pure-core E2E)" { cargo test -p morn-core-tests }
Run-Step "developer CLI smoke" {
    cargo build -p morn-cli | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "morn-cli build failed" }
    $cli = Join-Path $root "target\debug\morn.exe"
    $cmds = @("doctor", "status", "provider", "connector list", "connector health", "plugin validate", "compat", "migrate status", "node list", "domain list", "package inspect", "conformance")
    foreach ($c in $cmds) {
        $parts = $c -split " "
        & $cli @parts | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "morn $c failed with exit code $LASTEXITCODE" }
    }
    Write-Host "CLI smoke OK: doctor/status/provider/connector/plugin/compat/migrate/node/domain/package/conformance"
}

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

# ---- Build the all-features server before smokes so they use the fresh binary ----
Run-Step "build server binary (all-features)" { cargo build -p morn-app --bin server --all-features }

# ---- Browser-level UI smoke (Playwright) ----
if (Test-Path (Join-Path $root "frontend\node_modules\playwright\package.json")) {
    $browserDir = Join-Path $env:LOCALAPPDATA "ms-playwright"
    if (Test-Path $browserDir) {
        Run-Step "frontend ui smoke (playwright)" {
            Stop-Port8090
            Remove-Item -Force (Join-Path $root "target\ui_smoke.db") -ErrorAction SilentlyContinue
            $env:MORN_DB = Join-Path $root "target\ui_smoke.db"
            $env:MORN_PORT = "8090"
            $serverExe = Join-Path $root "target\debug\server.exe"
            $server = Start-Process -FilePath $serverExe -WindowStyle Hidden -PassThru
            $fe = $null
            try {
                Start-Sleep -Seconds 2
                Push-Location (Join-Path $root "frontend")
                $fe = Start-Process -FilePath "npm.cmd" -ArgumentList "run","dev","--","--port","5173","--strictPort" -WindowStyle Hidden -PassThru
                Start-Sleep -Seconds 6
                node scripts/ui_smoke.mjs
                if ($LASTEXITCODE -ne 0) { throw "UI smoke failed with exit code $LASTEXITCODE" }
                Pop-Location
            }
            finally {
                if ($fe) { Stop-Process -Id $fe.Id -Force -ErrorAction SilentlyContinue }
                Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
                Pop-Location -ErrorAction SilentlyContinue
            }
        }
    }
    else {
        Write-Host "SKIP frontend ui smoke: playwright browsers not installed"
    }
}
else {
    Write-Host "SKIP frontend ui smoke: playwright package not installed"
}

# ---- Demo smoke: start API server, run BioLab E2E, stop ----
Run-Step "build server binary" { cargo build -p morn-app --bin server --all-features }
Run-Step "demo smoke (server + BioLab E2E)" {
            Stop-Port8090
            Remove-Item -Force (Join-Path $root "target\smoke.db") -ErrorAction SilentlyContinue
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
        $bootstrap = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/demo/bootstrap" -Method Post -TimeoutSec 10
        if ($bootstrap.objects -lt 1) { throw "demo bootstrap did not seed world objects" }
        $run = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/biolab/run" -Method Post -TimeoutSec 15
        if ($run.result.all_ok -ne $true) { throw "BioLab E2E did not pass" }
        $wb = Invoke-RestMethod -Uri "http://127.0.0.1:8090/api/workbench" -TimeoutSec 10
        if ($wb.world_objects.Count -lt 1) { throw "workbench world objects missing" }
        if ($wb.work_packages.Count -lt 1) { throw "workbench work packages missing" }
        Write-Host "demo smoke OK: health=$($health.status) bootstrap_objects=$($bootstrap.objects) e2e_steps=$($run.result.steps.Count) objects=$($wb.world_objects.Count)"
    }
    finally {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "=== Verification complete ==="


