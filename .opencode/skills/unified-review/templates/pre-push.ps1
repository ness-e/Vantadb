# =============================================================================
# Pre-push hook template (PowerShell) — VantaDB SIPP barrier
# =============================================================================
# SIPP = Single Iteration Pre-Push barrier. This is the FIRST line of defense
# — it runs deterministically BEFORE OpenCode is invoked. If any check fails,
# the push is blocked and OpenCode is never called (saving context budget).
#
# This template is processed by the unified-review skill at L11 (Final Report)
# when pre_push_hook.enabled=true and pre_push_hook.platform=powershell.
# Placeholders:
#   {{PROFILE_NAME}}       — e.g. vantadb
#   {{GENERATED_AT}}       — ISO timestamp
#   {{SKILL_VERSION}}      — e.g. 1.0.0
#   {{EXTRA_CHECKS}}       — additional project-specific checks (optional)
#
# Installation:
#   1. Copy this file (with placeholders filled) to .git/hooks/pre-push.ps1
#   2. Make it executable: chmod +x .git/hooks/pre-push.ps1
#   3. Wire it into .git/hooks/pre-push (the bash wrapper):
#
#        #!/bin/sh
#        exec pwsh -NoProfile -File .git/hooks/pre-push.ps1
#
#   4. Make the wrapper executable: chmod +x .git/hooks/pre-push
# =============================================================================

param(
    [string]$Mode = "certify"  # quick | certify | full
)

# -----------------------------------------------------------------------------
# Header
# -----------------------------------------------------------------------------
$ErrorActionPreference = "Continue"
$scriptStart = Get-Date

Write-Host ""
Write-Host "[SIPP] VantaDB pre-push barrier" -ForegroundColor Cyan
Write-Host "[SIPP] Profile: {{PROFILE_NAME}} | Mode: $Mode | Generated: {{GENERATED_AT}}" -ForegroundColor DarkGray
Write-Host ""

# -----------------------------------------------------------------------------
# Helper: run a command, return $LASTEXITCODE, pretty-print result
# -----------------------------------------------------------------------------
function Invoke-Barrier {
    param(
        [string]$Label,
        [scriptblock]$Block,
        [int]$MaxDurationSec = 600
    )

    $start = Get-Date
    Write-Host "[SIPP] Running: $Label" -ForegroundColor Yellow

    try {
        $job = Start-Job -ScriptBlock $Block
        if (Wait-Job $job -Timeout $MaxDurationSec) {
            $output = Receive-Job $job
            $output | ForEach-Object { Write-Host "        $_" -ForegroundColor DarkGray }
            Remove-Job $job -Force
        } else {
            Stop-Job $job
            Remove-Job $job -Force
            Write-Host "[SIPP]   ❌ TIMEOUT after ${MaxDurationSec}s" -ForegroundColor Red
            return 1
        }
    } catch {
        Write-Host "[SIPP]   ❌ EXCEPTION: $_" -ForegroundColor Red
        return 1
    }

    $duration = ((Get-Date) - $start).TotalSeconds
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host ("[SIPP]   ✅ PASS ({0:N1}s)" -f $duration) -ForegroundColor Green
    } else {
        Write-Host ("[SIPP]   ❌ FAIL ({0:N1}s, exit={1})" -f $duration, $exitCode) -ForegroundColor Red
    }
    return $exitCode
}

# -----------------------------------------------------------------------------
# Phase 1: Mechanical barrier (always runs)
# -----------------------------------------------------------------------------
$failures = 0

# Format check
$failures += Invoke-Barrier "cargo fmt --check" {
    cargo fmt --all -- --check
} -MaxDurationSec 60

# Compile check (workspace + tests)
$failures += Invoke-Barrier "cargo check --workspace --tests" {
    cargo check --workspace --tests -j 2
} -MaxDurationSec 300

# Clippy with -D warnings
$failures += Invoke-Barrier "cargo clippy -- -D warnings" {
    cargo clippy --workspace --tests -j 2 -- -D warnings
} -MaxDurationSec 300

# Tests (nextest audit profile)
$failures += Invoke-Barrier "cargo nextest run --profile audit" {
    cargo nextest run --profile audit --workspace --build-jobs 2
} -MaxDurationSec 600

# -----------------------------------------------------------------------------
# Mode-specific checks
# -----------------------------------------------------------------------------
if ($Mode -in @("certify", "full")) {
    $failures += Invoke-Barrier "cargo audit" {
        cargo audit --ignore RUSTSEC-2026-0176 --ignore RUSTSEC-2026-0177
    } -MaxDurationSec 60

    $failures += Invoke-Barrier "cargo deny check" {
        cargo deny check
    } -MaxDurationSec 60
}

if ($Mode -eq "full") {
    $failures += Invoke-Barrier "cargo machete" {
        cargo machete
    } -MaxDurationSec 60

    $failures += Invoke-Barrier "Python SDK validate" {
        pwsh -NoProfile -File dev-tools/scripts/validate_python_sdk.ps1
    } -MaxDurationSec 300

    $failures += Invoke-Barrier "Web build" {
        Set-Location web
        npm ci --ignore-scripts
        npm run lint
        npx tsc --noEmit
        npm run build
        Set-Location ..
    } -MaxDurationSec 600

    $failures += Invoke-Barrier "Docs coverage" {
        pwsh -NoProfile -File scripts/validate-docs-coverage.ps1
    } -MaxDurationSec 120
}

# -----------------------------------------------------------------------------
# Optional: project-specific extra checks (filled by skill at generation time)
# -----------------------------------------------------------------------------
{{EXTRA_CHECKS}}

# -----------------------------------------------------------------------------
# Final report
# -----------------------------------------------------------------------------
$totalDuration = ((Get-Date) - $scriptStart).TotalSeconds

Write-Host ""
if ($failures -eq 0) {
    Write-Host ("[SIPP] ✅ Barrier passed ({0:N1}s total). Push allowed." -f $totalDuration) -ForegroundColor Green
    Write-Host "[SIPP] Next: invoke /review certify --profile vantadb in OpenCode for full audit." -ForegroundColor DarkGray
    exit 0
} else {
    Write-Host ("[SIPP] ❌ Barrier FAILED ({0} check(s), {1:N1}s total). Push BLOCKED." -f $failures, $totalDuration) -ForegroundColor Red
    Write-Host "[SIPP] Fix the failures above, then retry the push." -ForegroundColor Red
    Write-Host "[SIPP] Tip: run '/review quick --profile vantadb' in OpenCode for guided fix." -ForegroundColor DarkGray
    exit 1
}
