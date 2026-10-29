# VantaDB verify_staged — fast pre-commit gate mapped by touched area.
# Usage: pwsh dev-tools/verify_staged.ps1  (uses staged files)
# Covers what .githooks/pre-commit-config.yaml cannot scope:
#   - workspace members beyond -p vantadb (clippy per touched member)
#   - standalone Rust crates (providers/*: fmt check, no compile)
#   - skills markdown (markdownlint, same config as CI)
# The yaml handles: cargo fmt, clippy -p vantadb, ruff, prettier, markdownlint docs/.
# Budget: seconds warm (only clippy on touched members compiles).
param([string]$Range = "")
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot
$pass = 0; $fail = 0
function run($name, [string[]]$cmd) {
    Write-Host "  ${name}..." -NoNewline
    $prev = $ErrorActionPreference; $ErrorActionPreference = "Continue"
    $out = & $cmd[0] $cmd[1..($cmd.Length - 1)] 2>&1
    $ErrorActionPreference = $prev
    if ($LASTEXITCODE -eq 0) { Write-Host " ok"; $Script:pass++ }
    else { Write-Host " FAIL"; $Script:fail++; Write-Host $out; throw "step failed" }
}
function filesIn($revRange) {
    if ($revRange) { return @(git diff --name-only $revRange 2>$null) }
    return @(git diff --cached --name-only --diff-filter=ACMR 2>$null)
}
try {
    $files = filesIn $Range
    if (-not $files -or $files.Count -eq 0) { Write-Host "staged: nothing to check."; exit 0 }
    $rs = @($files | Where-Object { $_ -match '\.rs$' })
    # Standalone crates (not workspace members): fmt only (clippy compiles cold — CI covers it).
    $providers = @($rs | Where-Object { $_ -match '^providers/[^/]+/' } | ForEach-Object { ($_ -split '/')[0..1] -join '/' } | Sort-Object -Unique)
    foreach ($p in $providers) {
        if (Test-Path "$p/Cargo.toml") { run "fmt $p" ("cargo", "fmt", "--manifest-path", "$p/Cargo.toml", "--", "--check") }
    }
    # Workspace members beyond core -p vantadb (yaml covers core): clippy shares target/, warm.
    $members = @('vanta-memory','vanta-proxy','vantadb-server','vantadb-mcp','vantadb-wasm','vantadb-ffi-core','vantadb-python')
    foreach ($k in $members) {
        if ($rs | Where-Object { $_ -match "^${k}/" }) {
            run "clippy -p $k" ("cargo", "clippy", "-p", $k, "-j", "2", "--", "-D", "warnings")
        }
    }
    if ($rs | Where-Object { $_ -match '^src/' }) { Write-Host "  core src/ touched: yaml clippy -p vantadb covers it." }
    # Skills markdown (yaml covers only docs/).
    $md = @($files | Where-Object { $_ -match '^skills/.*\.md$' })
    if ($md.Count -gt 0) {
        if (Get-Command npx -ErrorAction SilentlyContinue) { run "markdownlint skills" ("npx", "markdownlint-cli2", "--config", ".github/markdownlint-cli2.yaml") + $md }
        else { Write-Host "  npx missing: skills markdown skipped." }
    }
    # completions/ regeneran solas via build.rs en cada cargo build: nada que chequear aqui.
    Write-Host "ALL ${pass} PASS"; exit 0
} catch { if ($fail -eq 0) { $Script:fail = 1 }; Write-Host "${fail} FAIL"; exit 1 }
