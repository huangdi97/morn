$ErrorActionPreference = "Stop"

function Run-Step {
    param(
        [string]$Name,
        [scriptblock]$Action
    )
    Write-Host ""
    Write-Host "=== $Name ==="
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Write-Host "Morn full verification"
Write-Host "Root: $root"

# Rust workspace
if (Test-Path "Cargo.toml") {
    Run-Step "cargo fmt --check" { cargo fmt --all -- --check }
    Run-Step "cargo check" { cargo check --workspace --all-targets }
    Run-Step "cargo clippy" { cargo clippy --workspace --all-targets --all-features -- -D warnings }
    Run-Step "cargo test" { cargo test --workspace --all-features }
}

# Detect frontend root. Codex should refine this script after inspecting repository.
$frontendCandidates = @(".", "app", "frontend", "ui", "apps/desktop", "apps/web")
$frontend = $null
foreach ($candidate in $frontendCandidates) {
    $pkg = Join-Path $candidate "package.json"
    if (Test-Path $pkg) {
        $frontend = $candidate
        break
    }
}

if ($frontend) {
    Push-Location $frontend
    try {
        $pkgJson = Get-Content "package.json" -Raw | ConvertFrom-Json

        if ($pkgJson.scripts.typecheck) {
            Run-Step "frontend typecheck" { npm run typecheck }
        }
        if ($pkgJson.scripts.lint) {
            Run-Step "frontend lint" { npm run lint }
        }
        if ($pkgJson.scripts.test) {
            Run-Step "frontend test" { npm test -- --run }
        }
        if ($pkgJson.scripts.build) {
            Run-Step "frontend build" { npm run build }
        }
    }
    finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "=== Verification complete ==="
Write-Host "NOTE: Codex must adapt this script to the repository's actual package manager and commands."
