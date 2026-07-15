#!/usr/bin/env pwsh
<#
.SYNOPSIS
    VantaDB audit CLI backend — mechanical checks for /audit command.
    Handles Phase 1 of the audit pipeline.

.PARAMETER Mode
    quick   → core verify (fmt + clippy + test + deny), ~2min
    ci      → full workspace (fmt + clippy + test + deny + audit), ~10min
    full    → ci + machete + bloat + docs, ~15min
    lint    → clippy pedantic + machete
    security→ audit + deny
    perf    → bloat + outdated
#>

param(
    [Parameter(Mandatory)]
    [ValidateSet('quick', 'ci', 'full', 'lint', 'security', 'perf')]
    [string]$Mode,
    [string]$ReportDir = "docs/audit-reports"
)

$ErrorActionPreference = 'Continue'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$logDir = Join-Path $ProjectRoot $ReportDir
$null = New-Item -ItemType Directory -Path $logDir -Force
$ts = Get-Date -Format 'yyyyMMdd-HHmmss'
$logFile = Join-Path $logDir "cli-$Mode-$ts.log"

function Write-Phase {
    param([string]$Name, [string]$Cmd)
    Write-Host "`n  ═══ $Name ═══" -ForegroundColor Cyan
    Write-Host "  $ $Cmd" -ForegroundColor DarkGray
    $output = Invoke-Expression $Cmd 2>&1
    $exit = $LASTEXITCODE
    $output | Out-File -FilePath $logFile -Append -Encoding utf8
    if ($exit -ne 0) {
        Write-Host "  ❌ $Name (exit $exit)" -ForegroundColor Red
        return $false
    }
    Write-Host "  ✅ $Name" -ForegroundColor Green
    return $true
}

Write-Host @"
╔══════════════════════════════════════════╗
║     VantaDB Audit — Phase 1 CLI Checks    ║
║     Mode: $Mode                             ║
╚══════════════════════════════════════════╝
"@ -ForegroundColor Magenta

$results = @{}
$core = "-p vantadb"
$lite = '--no-default-features -F "cli,fjall,sysinfo,memmap2,fs2"'

switch ($Mode) {
    'quick' {
        $results.fmt    = Write-Phase 'cargo fmt --check'             'cargo fmt --check'
        $results.clippy = Write-Phase "cargo clippy ($core)"          "cargo clippy $core $lite -- -D warnings"
        $results.test   = Write-Phase "cargo nextest ($core)"         "cargo nextest run --profile audit $core $lite --build-jobs 2"
        $results.deny   = Write-Phase 'cargo deny check'              'cargo deny check'
    }
    'ci' {
        $results.fmt    = Write-Phase 'cargo fmt --check'             'cargo fmt --check'
        $results.clippy = Write-Phase 'cargo clippy'                  'cargo clippy --workspace -- -D warnings'
        $results.test   = Write-Phase 'cargo nextest'                 'cargo nextest run --profile audit --workspace --build-jobs 2'
        $results.deny   = Write-Phase 'cargo deny check'              'cargo deny check'
        $results.audit  = Write-Phase 'cargo audit'                   'cargo audit'
    }
    'full' {
        $results.fmt    = Write-Phase 'cargo fmt --check'             'cargo fmt --check'
        $results.clippy = Write-Phase 'cargo clippy'                  'cargo clippy --workspace -- -D warnings'
        $results.test   = Write-Phase 'cargo nextest'                 'cargo nextest run --profile audit --workspace --build-jobs 2'
        $results.deny   = Write-Phase 'cargo deny check'              'cargo deny check'
        $results.audit  = Write-Phase 'cargo audit'                   'cargo audit'
        $results.machete= Write-Phase 'cargo machete'                 'cargo machete'
        $results.bloat  = Write-Phase 'cargo bloat --crates'          'cargo bloat --crates'
        $results.docs   = Write-Phase 'docs coverage'                 'pwsh -NoProfile -File scripts/validate-docs-coverage.ps1'
    }
    'lint' {
        $results.clippy  = Write-Phase "cargo clippy ($core pedantic)" "cargo clippy $core -- -D warnings -W clippy::pedantic"
        $results.machete = Write-Phase 'cargo machete'                'cargo machete'
    }
    'security' {
        $results.audit  = Write-Phase 'cargo audit'                   'cargo audit'
        $results.deny   = Write-Phase 'cargo deny check'              'cargo deny check'
    }
    'perf' {
        $results.bloat   = Write-Phase 'cargo bloat --crates'         'cargo bloat --crates'
        $results.outdated= Write-Phase 'cargo outdated'               'cargo outdated --exit-code 1'
    }
}

$passed = ($results.Values | Where-Object { $_ }).Count
$failed = ($results.Values | Where-Object { -not $_ }).Count
$total = $results.Count

$color = if ($failed -eq 0) { 'Green' } else { 'Red' }
Write-Host @"
╔══════════════════════════════════════════╗
║  CLI Phase: $passed/$total passed, $failed failed  ║
╚══════════════════════════════════════════╝
"@ -ForegroundColor $color

Write-Host "Log: $logFile" -ForegroundColor Gray
exit $(if ($failed -gt 0) { 1 } else { 0 })
