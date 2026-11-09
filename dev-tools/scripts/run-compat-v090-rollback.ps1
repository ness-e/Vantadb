# run-compat-v090-rollback.ps1 — ADR-0060 D5/D8b: run the gated rollback test.
#
# The compat_v090_rollback_* test needs a real v0.9.0 `vanta-cli` binary via
# VANTADB_V090_CLI. gen-fixture-v090.ps1 already builds one into the persistent
# cache %TEMP%\vanta-v090-target; this runner reuses it (or tells you to run the
# generator first) and executes the test.
#
# Usage:  pwsh dev-tools/scripts/run-compat-v090-rollback.ps1

$ErrorActionPreference = 'Stop'
$repo = (git rev-parse --show-toplevel).Trim()
Set-Location $repo

$cli = Join-Path $env:TEMP 'vanta-v090-target\debug\vanta-cli.exe'
if (-not (Test-Path $cli)) {
    throw "v0.9.0 binary not found at $cli — run dev-tools/scripts/gen-fixture-v090.ps1 first (it builds and caches it)"
}
& $cli --version | Out-Null
Write-Host "== VANTADB_V090_CLI=$cli"
$env:VANTADB_V090_CLI = $cli
cargo nextest run --profile audit -p vantadb --features encryption --locked --build-jobs 2 -E 'test(compat_v090_rollback)'
if ($LASTEXITCODE -ne 0) { throw "rollback compat test failed ($LASTEXITCODE)" }
Write-Host "== rollback test passed against the v0.9.0 binary"
