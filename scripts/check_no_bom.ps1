# Guard: reject UTF-8 BOM in tracked text files (KF-004 class regression).
# Linux cargo/toml parsers reject BOM in Cargo.toml; keep the repo BOM-free.
$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$exts = @(".toml", ".json", ".md", ".rs", ".ts", ".tsx", ".js", ".mjs", ".ps1", ".yml", ".yaml", ".css", ".html", ".sh")
$skip = @(".git", "target", "node_modules", "dist", "src-tauri\gen")
$bad = @()
Get-ChildItem -Path $root -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
    $rel = $_.FullName.Substring($root.Path.Length)
    $ext = $_.Extension.ToLowerInvariant()
    if ($exts -notcontains $ext) { return }
    foreach ($s in $skip) { if ($rel -like "*\$s*") { return } }
    $bytes = [System.IO.File]::ReadAllBytes($_.FullName)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        $bad += $rel
    }
}
if ($bad.Count -gt 0) {
    Write-Host "UTF-8 BOM found in:"
    $bad | ForEach-Object { Write-Host "  $_" }
    throw "BOM guard failed"
}
Write-Host "BOM guard OK: no UTF-8 BOM in tracked text files"
