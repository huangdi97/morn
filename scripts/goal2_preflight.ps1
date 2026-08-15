$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root
Write-Host "=== Morn Goal 2 Preflight ==="
git status --short --branch
git log -8 --oneline
try { rustc --version } catch {}
try { cargo --version } catch {}
try { node --version } catch {}
try { npm --version } catch {}
try { pnpm --version } catch {}
$patterns = @("Delegation","RepresentationContract","DecisionPolicyAsset","SolutionCompiler","ProblemSpec","WorkGraph","ReplayRun","EvaluationRun","ShadowRun")
foreach ($p in $patterns) {
  Write-Host "--- $p ---"
  try { rg -n --glob '!target/**' --glob '!node_modules/**' $p . | Select-Object -First 12 } catch {}
}
Write-Host "Copy relevant findings into STATUS_GOAL2.md."
