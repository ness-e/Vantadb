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

# Fix console encoding for Unicode box-drawing chars
$oldEncoding = [Console]::OutputEncoding
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$Host.UI.RawUI.ForegroundColor = 'Gray'

$logDir = Join-Path $ProjectRoot $ReportDir
$null = New-Item -ItemType Directory -Path $logDir -Force
$ts = Get-Date -Format 'yyyyMMdd-HHmmss'
$logFile = Join-Path $logDir "cli-$Mode-$ts.log"
"VantaDB Audit CLI — Mode: $Mode — $ts" | Out-File $logFile -Encoding utf8
"Project: $ProjectRoot" | Out-File $logFile -Append -Encoding utf8
"─" * 60 | Out-File $logFile -Append -Encoding utf8

function Write-Step {
    param([string]$Label, [string[]]$Lines)
    $color = if ($LASTEXITCODE -eq 0) { 'Green' } else { 'Red' }
    $mark  = if ($LASTEXITCODE -eq 0) { '[OK]' } else { "[EXIT $LASTEXITCODE]" }
    Write-Host "  $mark $Label" -ForegroundColor $color
    foreach ($l in $Lines) {
        if ($l.Trim()) { Write-Host "       $l" -ForegroundColor DarkGray }
    }
}

function Run-Check {
    param([string]$Name, [scriptblock]$ScriptBlock)
    Write-Host "`n  --- $Name " -NoNewline -ForegroundColor Cyan
    Write-Host "-" * ([Math]::Max(1, 60 - $Name.Length)) -ForegroundColor DarkGray
    $output = & $ScriptBlock 2>&1
    $exit = $LASTEXITCODE
    $text = $output | Out-String
    $text | Out-File $logFile -Append -Encoding utf8
    $lines = $text -split "`n" | Where-Object { $_.Trim() }
    if ($lines.Count -gt 20) {
        Write-Step $Name $lines[0..4]
        Write-Host "       ... ($($lines.Count - 10) lines truncated, see log)" -ForegroundColor DarkGray
        Write-Step '' $lines[($lines.Count-5)..($lines.Count-1)]
    } else {
        Write-Step $Name $lines
    }
    if ($exit -ne 0) {
        Write-Host "  >>> FAILED (exit $exit)" -ForegroundColor Red
        return $false
    }
    return $true
}

Write-Host "┌────────────────────────────────────────────────────────────┐" -ForegroundColor Magenta
Write-Host "│  VantaDB Audit  —  Phase 1: CLI Mechanical Checks          │" -ForegroundColor Magenta
Write-Host "│  Mode: $($Mode.PadRight(42))│" -ForegroundColor Magenta
Write-Host "│  Log:  $($Mode)-$ts.log".PadRight(60) + "│" -ForegroundColor DarkGray
Write-Host "└────────────────────────────────────────────────────────────┘" -ForegroundColor Magenta

$results = @{}
$core = '-p vantadb'
$lite = '--no-default-features -F cli,fjall,sysinfo,memmap2,fs2'

switch ($Mode) {
    'quick' {
        $results.fmt    = Run-Check 'cargo fmt --check'          { cargo fmt --check }
        $results.clippy = Run-Check "cargo clippy ($core)"       { cargo clippy $core $lite -- -D warnings }
        $results.test   = Run-Check "cargo nextest ($core)"      { cargo nextest run --profile audit $core $lite --build-jobs 2 }
        $results.deny   = Run-Check 'cargo deny check'           { cargo deny check }
    }
    'ci' {
        $results.fmt    = Run-Check 'cargo fmt --check'          { cargo fmt --check }
        $results.clippy = Run-Check 'cargo clippy (workspace)'   { cargo clippy --workspace -- -D warnings }
        $results.test   = Run-Check 'cargo nextest (workspace)'  { cargo nextest run --profile audit --workspace --build-jobs 2 }
        $results.deny   = Run-Check 'cargo deny check'           { cargo deny check }
        $results.audit  = Run-Check 'cargo audit'                { cargo audit }
    }
    'full' {
        $results.fmt    = Run-Check 'cargo fmt --check'          { cargo fmt --check }
        $results.clippy = Run-Check 'cargo clippy (workspace)'   { cargo clippy --workspace -- -D warnings }
        $results.test   = Run-Check 'cargo nextest (workspace)'  { cargo nextest run --profile audit --workspace --build-jobs 2 }
        $results.deny   = Run-Check 'cargo deny check'           { cargo deny check }
        $results.audit  = Run-Check 'cargo audit'                { cargo audit }
        $results.machete= Run-Check 'cargo machete'              { cargo machete }
        $results.bloat  = Run-Check 'cargo bloat --crates'       { cargo bloat --crates }
        $results.docs   = Run-Check 'docs coverage'              { pwsh -NoProfile -File scripts/validate-docs-coverage.ps1 }
    }
    'lint' {
        $results.clippy  = Run-Check "cargo clippy (pedantic)"    { cargo clippy $core -- -D warnings -W clippy::pedantic }
        $results.machete = Run-Check 'cargo machete'              { cargo machete }
    }
    'security' {
        $results.audit  = Run-Check 'cargo audit'                { cargo audit }
        $results.deny   = Run-Check 'cargo deny check'           { cargo deny check }
    }
    'perf' {
        $results.bloat   = Run-Check 'cargo bloat --crates'      { cargo bloat --crates }
        $results.outdated= Run-Check 'cargo outdated'            { cargo outdated --exit-code 1 }
    }
}

$passed = ($results.Values | Where-Object { $_ }).Count
$failed = ($results.Values | Where-Object { -not $_ }).Count
$total = $results.Count

$color = if ($failed -eq 0) { 'Green' } else { 'Red' }
Write-Host "`n┌────────────────────────────────────────────────────────────┐" -ForegroundColor $color
Write-Host "│  CLI Phase: $passed/$total passed, $failed failed" + ' ' * [Math]::Max(1, 30 - "$passed/$total passed, $failed failed".Length) + "│" -ForegroundColor $color
Write-Host "└────────────────────────────────────────────────────────────┘" -ForegroundColor $color
Write-Host "  Log: $logFile" -ForegroundColor Gray

# Restore encoding
[Console]::OutputEncoding = $oldEncoding

exit $(if ($failed -gt 0) { 1 } else { 0 })
