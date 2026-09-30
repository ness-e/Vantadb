#!/usr/bin/env pwsh
#Requires -Version 5.1
<#
.SYNOPSIS
  ICP-02 privacy chain E2E demo: capture -> PII audit -> certified forget ->
  injection audit consult.

.DESCRIPTION
  Chains the three F4 guarantees of the privacy track on ONE on-disk store:

    1. Rust E2E (vanta-proxy test, `#[ignore]` + `--features vantadb/fjall`):
       captures synthetic PII through the real proxy wire, audits
       store+indexes+export with the versioned `vanta-pii-audit` binary
       (0 cleartext; a planted leak must fail the audit), exercises the
       declared Disarmed degradation (no key -> never cleartext) and writes
       <WorkDir>/handoff.json.
    2. Certified forget with the real CLI, exit 0 verified twice:
       `vanta-cli delete --attest` + `vanta-cli certificate verify` for both
       records of the captured turn (proxy-turns + l1).
    3. Post-forget PII audit (still 0 cleartext).
    4. Injection audit consult: rows matching `"op":"injection"` (+ outcome ok)
       via Select-String.

  Local (Windows) and CI (ubuntu via `pwsh -NoProfile -File`) run the same
  script. PII is synthetic; the encryption key is a test key, process-local.

.PARAMETER WorkDir
  Demo directory (default: <repo>/target/icp02-demo). Recreated by the test.

.EXAMPLE
  pwsh -NoProfile -File scripts/demo-privacy-e2e.ps1
#>
[CmdletBinding()]
param(
    [string]$WorkDir = ''
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $repoRoot 'target/icp02-demo'
}
$WorkDir = [System.IO.Path]::GetFullPath($WorkDir)
$env:ICP02_DEMO_DIR = $WorkDir

Push-Location $repoRoot
try {
    Write-Host '== ICP-02 privacy chain demo =='
    Write-Host "   workdir: $WorkDir"

    # ── 1. Rust E2E: capture + PII audit + Disarmed + handoff ───────────────
    Write-Host '== [1/4] Rust E2E: capture with synthetic PII + PII audit + Disarmed'
    & cargo nextest run -p vanta-proxy --features vantadb/fjall --test icp02_privacy_demo --run-ignored ignored-only --no-capture
    if ($LASTEXITCODE -ne 0) {
        throw "[1/4] icp02_privacy_demo failed (exit $LASTEXITCODE)"
    }

    $handoffPath = Join-Path $WorkDir 'handoff.json'
    if (-not (Test-Path $handoffPath)) {
        throw "[1/4] handoff not found: $handoffPath"
    }
    $handoff = ConvertFrom-Json (Get-Content -Raw $handoffPath)
    Write-Host "   db: $($handoff.db_dir)"

    # ── 2. Certified forget (real CLI, exit 0) ──────────────────────────────
    Write-Host '== [2/4] Certified forget: vanta-cli delete --attest + certificate verify'
    $i = 0
    foreach ($rec in $handoff.records) {
        $i++
        $cert = Join-Path $WorkDir "certificate-$i.json"
        & cargo run -q -p vantadb --bin vanta-cli -- delete --db $handoff.db_dir --namespace $rec.namespace --key $rec.key --attest --out $cert
        if ($LASTEXITCODE -ne 0) {
            throw "[2/4] delete --attest failed for $($rec.namespace)/$($rec.key) (exit $LASTEXITCODE)"
        }
        & cargo run -q -p vantadb --bin vanta-cli -- certificate verify --db $handoff.db_dir --file $cert --json
        if ($LASTEXITCODE -ne 0) {
            throw "[2/4] certificate verify failed for $cert (exit $LASTEXITCODE)"
        }
        Write-Host "   purged + verified: $($rec.namespace)/$($rec.key)"
    }

    # ── 3. Post-forget PII audit (still 0 cleartext) ────────────────────────
    Write-Host '== [3/4] Post-forget PII audit (store + indexes + export)'
    $auditRaw = & cargo run -q -p vanta-proxy --features vantadb/fjall --bin vanta-pii-audit -- --json $handoff.db_dir $handoff.export
    if ($LASTEXITCODE -ne 0) {
        throw "[3/4] PII audit failed (exit $LASTEXITCODE)"
    }
    $audit = ConvertFrom-Json (($auditRaw | Out-String).Trim())
    if (-not $audit.ok) {
        throw "[3/4] PII audit reported findings: $((($auditRaw | Out-String).Trim()))"
    }
    Write-Host "   audit clean: files=$($audit.audited_files) bytes=$($audit.audited_bytes)"

    # ── 4. Injection audit consult (op=injection) ───────────────────────────
    Write-Host '== [4/4] Injection audit consult ("op":"injection")'
    $rows = @(Select-String -Path $handoff.injection_audit -Pattern '"op":"injection"' -SimpleMatch)
    if ($rows.Count -lt 1) {
        throw "[4/4] no op=injection rows in $($handoff.injection_audit)"
    }
    $okRows = @($rows | Where-Object { $_.Line -match '"outcome":"ok"' })
    if ($okRows.Count -lt 1) {
        throw "[4/4] no outcome=ok injection rows in $($handoff.injection_audit)"
    }
    Write-Host "   injection events: $($rows.Count) total, $($okRows.Count) ok (metadata only)"

    Write-Host '== ICP-02 demo PASSED: capture -> audit(0) -> forget(certified) -> injection audit =='
}
finally {
    Pop-Location
}
exit 0
