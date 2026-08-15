$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root
$fail = 0

Write-Host "=== Domain Boundary Guard ==="
# 1) No Core crate may depend on a domain pack (optional feature-gated deps are allowed).
Get-ChildItem crates -Directory | ForEach-Object {
    $cargo = Join-Path $_.FullName "Cargo.toml"
    if (Test-Path $cargo) {
        $lines = Get-Content $cargo
        $inDeps = $false
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -match '^\[dependencies') { $inDeps = $true }
            elseif ($lines[$i] -match '^\[') { $inDeps = $false }
            if ($inDeps -and $lines[$i] -match 'morn-biolab-reference|domain-packs') {
                # skip if this dep line is optional (feature-gated)
                if ($lines[$i] -notmatch 'optional\s*=\s*true') {
                    Write-Host "FAIL: core crate $($_.Name) has a non-optional domain pack dependency"
                    $fail = 1
                }
            }
        }
    }
}
# 2) Core lib sources must not import a concrete domain crate (feature-gated imports allowed).
$files = Get-ChildItem crates -Recurse -Filter *.rs | Where-Object { $_.FullName -notmatch '\\tests\\' }
foreach ($f in $files) {
    $lines = Get-Content $f.FullName
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match 'use morn_biolab(_reference)?::') {
            $prev = if ($i -gt 0) { $lines[$i-1] } else { "" }
            if ($prev -notmatch 'cfg\(feature = "domain-') {
                Write-Host "FAIL: core import -> $($f.FullName):$($i+1)"
                $fail = 1
            }
        }
    }
}
# 3) Core must not match on concrete domain words (compiler-heuristic class leak).
$coreFiles = Get-ChildItem crates -Recurse -Filter *.rs | Where-Object { $_.FullName -notmatch "\\tests\\" }
foreach ($f in $coreFiles) {
    $text = Get-Content $f.FullName -Raw
    foreach ($w in @("biolab", "claim", "hypothesis", "scientific", "aging_pilot")) {
        if ($text -match "contains\(`"$w`"\)|`"$w`".*contains\(") {
            Write-Host "FAIL: core domain-word heuristic -> $($f.FullName) ($w)"
            $fail = 1
        }
    }
}
# 4) Kernel must not define concrete domain IDs (exact identifiers).
$kernelIds = Get-Content crates/morn-kernel/src/ids.rs -Raw
foreach ($word in @('DatasetTag','SampleTag','ScientificClaimTag','AnalysisRunTag','QCResultTag')) {
    if ($kernelIds -match "\b$word\b") { Write-Host "FAIL: kernel defines domain ID $word"; $fail = 1 }
}
if ($fail -eq 0) { Write-Host "OK: no Core -> domain dependency; kernel domain-free" } else { throw "Domain boundary guard failed" }
