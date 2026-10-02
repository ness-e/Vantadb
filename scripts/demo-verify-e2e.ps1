#!/usr/bin/env pwsh
#Requires -Version 5.1
<#
.SYNOPSIS
  WAL integrity (verify) E2E demo: the hash-chain catches tampering and the
  real CLI exits non-zero; a clean chain and the certified-delete roundtrip
  exit zero.

.DESCRIPTION
  Chains the VER-01/VER-02 verification story on local files:

    1. Fixtures (ignored test `wal_verify_demo_producer`): a clean chained WAL
       plus two tampered variants — one with a record extirpated (chain link
       broken) and one with a record rewritten in a decodable way plus its
       CRC32C recomputed (invisible to CRC alone; only the chain sees it).
    2. `vanta-cli verify` on the clean WAL: exit 0 (`ok: true`).
    3. `vanta-cli verify` on BOTH tampered WALs: exit != 0 with
       `status: tampered` — the negative control. If verification stops
       detecting either tamper class, this demo turns CI red.
    4. Certified delete on a real store through the CLI: `put` ->
       `delete --attest` -> `certificate verify`, all exit 0 (the certificate
       surface the WAL chain feeds; VER-02).

  Local (Windows) and CI (ubuntu via `pwsh -NoProfile -File`) run the same
  script. Deterministic and offline: local files only, no network, no tokens.
  (The certified-delete roundtrip also runs inside the privacy chain demo —
  here it is the standalone surface roundtrip; scope declared in the header.)

.PARAMETER WorkDir
  Demo directory (default: <repo>/target/wal-verify-demo). Recreated by the fixtures test.

.EXAMPLE
  pwsh -NoProfile -File scripts/demo-verify-e2e.ps1
#>
[CmdletBinding()]
param(
    [string]$WorkDir = ''
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $repoRoot 'target/wal-verify-demo'
}
$WorkDir = [System.IO.Path]::GetFullPath($WorkDir)
$env:WAL_VERIFY_DEMO_DIR = $WorkDir

Push-Location $repoRoot
try {
    Write-Host '== WAL verify (tamper-evident) demo =='
    Write-Host "   workdir: $WorkDir"

    # ── 1. Fixtures: clean + tampered WALs, handoff ─────────────────────────
    Write-Host '== [1/4] Fixtures: clean + tampered-removed + tampered-rewritten'
    & cargo nextest run -p vantadb --test wal_chain_verify --run-ignored ignored-only --no-capture --build-jobs 2
    if ($LASTEXITCODE -ne 0) {
        throw "[1/4] wal_verify_demo_producer failed (exit $LASTEXITCODE)"
    }
    $handoffPath = Join-Path $WorkDir 'handoff.json'
    if (-not (Test-Path $handoffPath)) {
        throw "[1/4] handoff not found: $handoffPath"
    }
    $handoff = ConvertFrom-Json (Get-Content -Raw $handoffPath)

    # ── 2. Clean WAL verifies green (exit 0) ────────────────────────────────
    Write-Host '== [2/4] Clean WAL: vanta-cli verify -> exit 0'
    $cleanRaw = (& cargo run -q -p vantadb --bin vanta-cli -- verify --db $handoff.clean --json | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "[2/4] clean WAL must verify green (exit $LASTEXITCODE): $cleanRaw"
    }
    $cleanReport = ConvertFrom-Json $cleanRaw
    if (-not $cleanReport.ok) {
        throw "[2/4] clean WAL reported ok:false: $cleanRaw"
    }
    $cleanStatuses = @($cleanReport.shards | ForEach-Object { $_.status })
    Write-Host "   clean: exit 0, ok:true, statuses: $($cleanStatuses -join ', ')"

    # ── 3. Tampered WALs fail (exit != 0) — negative control ────────────────
    Write-Host '== [3/4] Tampered WALs: vanta-cli verify -> exit != 0 (negative control)'
    foreach ($case in @('tampered_removed', 'tampered_rewritten')) {
        $dir = $handoff.$case
        $raw = (& cargo run -q -p vantadb --bin vanta-cli -- verify --db $dir --json | Out-String).Trim()
        $code = $LASTEXITCODE
        if ($code -eq 0) {
            throw "[3/4] $case must FAIL verification, got exit 0 — tamper not detected: $raw"
        }
        $report = ConvertFrom-Json $raw
        if ($report.ok) {
            throw "[3/4] $case reported ok:true with exit ${code}: $raw"
        }
        $statuses = @($report.shards | ForEach-Object { $_.status })
        if ($statuses -notcontains 'tampered') {
            throw "[3/4] $case statuses '$($statuses -join ', ')' (expected tampered): $raw"
        }
        Write-Host "   ${case}: exit $code, ok:false, statuses: $($statuses -join ', ') (tamper detected)"
    }

    # ── 4. Certified delete roundtrip (exit 0) ──────────────────────────────
    Write-Host '== [4/4] Certified delete: put -> delete --attest -> certificate verify'
    $certDir = Join-Path $WorkDir 'certified'
    if (Test-Path $certDir) {
        Remove-Item -Recurse -Force $certDir
    }
    & cargo run -q -p vantadb --bin vanta-cli -- put --db $certDir --namespace demo --key receipt --payload 'verify demo record'
    if ($LASTEXITCODE -ne 0) {
        throw "[4/4] put failed (exit $LASTEXITCODE)"
    }
    $cert = Join-Path $WorkDir 'certificate.json'
    & cargo run -q -p vantadb --bin vanta-cli -- delete --db $certDir --namespace demo --key receipt --attest --out $cert
    if ($LASTEXITCODE -ne 0) {
        throw "[4/4] delete --attest failed (exit $LASTEXITCODE)"
    }
    & cargo run -q -p vantadb --bin vanta-cli -- certificate verify --db $certDir --file $cert --json
    if ($LASTEXITCODE -ne 0) {
        throw "[4/4] certificate verify failed (exit $LASTEXITCODE)"
    }
    Write-Host '   put + delete --attest + certificate verify: exit 0 x3'

    Write-Host '== WAL verify demo PASSED: clean(exit 0) + removed/rewritten(tamper detected) + certified delete(exit 0) =='
}
finally {
    Pop-Location
}
exit 0
