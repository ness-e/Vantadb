#!/usr/bin/env pwsh
#Requires -Version 5.1
<#
.SYNOPSIS
  Injection governance E2E demo: budget truncates inside the cap, namespace
  ACL denials are audited, and the audit JSONL is consultable — metadata only.

.DESCRIPTION
  1. Fixtures (ignored test `governance_demo_producer`): one real proxy request
     (budget 60 tokens, ACL allowing only `persona/`) writes
     `<WorkDir>/injection-audit.jsonl` and asserts on the wire: the injected
     block stays ≤ budget with a visible `…[truncated]` cut; the ACL-denied
     scene never enters the prompt.
  2. Consult the audit: ≥1 `op:"injection"` row; ≥1 `outcome:"ok"` row with
     `truncated=true` (budget decision); ≥1 `outcome:"denied"` row with
     `acl=deny` (ACL decision).
  3. Negative control: the audit file must NOT contain the persona payload
     marker — the log is metadata-only by contract.

  Local (Windows) and CI (ubuntu via `pwsh -NoProfile -File`) run the same
  script. Deterministic and offline: in-memory store, mock upstream on
  loopback, no network, no tokens.

.PARAMETER WorkDir
  Demo directory (default: <repo>/target/injection-governance-demo). Recreated by the fixtures test.

.EXAMPLE
  pwsh -NoProfile -File scripts/demo-governance-e2e.ps1
#>
[CmdletBinding()]
param(
    [string]$WorkDir = ''
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $repoRoot 'target/injection-governance-demo'
}
$WorkDir = [System.IO.Path]::GetFullPath($WorkDir)
$env:GOVERNANCE_DEMO_DIR = $WorkDir

Push-Location $repoRoot
try {
    Write-Host '== Injection governance demo (budget / ACL / audit) =='
    Write-Host "   workdir: $WorkDir"

    # ── 1. Fixtures: one governed request through the real proxy wire ───────
    Write-Host '== [1/3] Fixtures: proxy request with budget 60 + ACL persona/ only'
    & cargo nextest run -p vanta-proxy --test ver04_governance --run-ignored ignored-only --no-capture
    if ($LASTEXITCODE -ne 0) {
        throw "[1/3] governance_demo_producer failed (exit $LASTEXITCODE)"
    }
    $handoffPath = Join-Path $WorkDir 'handoff.json'
    if (-not (Test-Path $handoffPath)) {
        throw "[1/3] handoff not found: $handoffPath"
    }
    $handoff = ConvertFrom-Json (Get-Content -Raw $handoffPath)

    # ── 2. Consult the audit (op=injection / ok+truncated / denied+acl=deny) ─
    Write-Host '== [2/3] Audit consult: op=injection rows'
    $rows = @(Select-String -Path $handoff.audit -Pattern '"op":"injection"' -SimpleMatch)
    if ($rows.Count -lt 1) {
        throw "[2/3] no op=injection rows in $($handoff.audit)"
    }
    $truncatedOk = @($rows | Where-Object { $_.Line -match '"outcome":"ok"' -and $_.Line -match 'truncated=true' })
    if ($truncatedOk.Count -lt 1) {
        throw "[2/3] no ok+truncated row (budget decision missing) in $($handoff.audit)"
    }
    $denied = @($rows | Where-Object { $_.Line -match '"outcome":"denied"' -and $_.Line -match 'acl=deny' })
    if ($denied.Count -lt 1) {
        throw "[2/3] no denied+acl=deny row (ACL decision missing) in $($handoff.audit)"
    }
    Write-Host "   audit: $($rows.Count) injection rows ($($truncatedOk.Count) ok+truncated, $($denied.Count) denied)"

    # ── 3. Negative control: metadata only, never payload ───────────────────
    Write-Host '== [3/3] Negative control: audit must not contain the payload marker'
    $leak = @(Select-String -Path $handoff.audit -Pattern 'PERSONA-MARKER' -SimpleMatch)
    if ($leak.Count -gt 0) {
        throw "[3/3] payload leaked into the audit ($($leak.Count) lines)"
    }
    Write-Host '   metadata-only: PERSONA-MARKER absent from the audit'

    Write-Host '== Injection governance demo PASSED: budget(truncated<=cap) + ACL(denied audited) + metadata-only audit =='
}
finally {
    Pop-Location
}
exit 0
