$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Write-Host "=== Morn preflight ==="
Write-Host "Root: $root"
Write-Host ""

Write-Host "--- git ---"
git status --short --branch
git log -5 --oneline

Write-Host ""
Write-Host "--- tools ---"
try { rustc --version } catch {}
try { cargo --version } catch {}
try { node --version } catch {}
try { npm --version } catch {}
try { pnpm --version } catch {}
try { yarn --version } catch {}
try { bun --version } catch {}

Write-Host ""
Write-Host "--- key files ---"
Get-ChildItem -Force | Select-Object Name, Mode

Write-Host ""
Write-Host "Codex: copy relevant findings into STATUS.md before refactoring."
