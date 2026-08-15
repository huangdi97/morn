$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root
Write-Host "=== Morn Goal4 Preflight ==="
git status --short --branch
git log -10 --oneline
Write-Host "`n=== Goal reports ==="
Get-ChildItem -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "goal[123]_final_report|tonight_final_report" } | Select-Object FullName
Write-Host "`n=== Persistence candidates ==="
Get-ChildItem -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch "target|node_modules" -and $_.Name -match "migration|schema|repository|sqlite|database|store" } | Select-Object -First 100 FullName
$patterns=@("CertificationSpec","CertifiedWorkCapability","CapabilityRelease","ManagedWorkService","DeliveryReceipt","ReplacementDecision","EvolutionCandidate","Distillation")
foreach($p in $patterns){ Write-Host "`n--- $p ---"; try { rg -n --glob '!target/**' --glob '!node_modules/**' $p . | Select-Object -First 20 } catch {} }
Write-Host "`nCopy concrete findings into STATUS_GOAL4.md."
