#!/usr/bin/env pwsh
# FIND-98: the installed `vanta-cli` must have been built from this repo's HEAD.
#
# The binary embeds `<version> (commit <sha>, worktree <state>)` at build time
# (build.rs -> OUT_DIR/build_version.txt -> src/bin/vanta-cli.rs). Before that,
# a copy from two months earlier answered `vanta-cli 0.9.0` exactly like a fresh
# build, so nothing could tell an operator their CLI was stale.
#
# Exit codes: 0 fresh · 1 stale/mismatch · 2 cannot check (no binary / no repo).
[CmdletBinding()]
param(
    # Binary to check. Defaults to the first `vanta-cli` on PATH
    # (~/.cargo/bin/vanta-cli.exe on a default cargo install).
    [string]$InstalledPath,
    # Repository whose HEAD is the reference. Defaults to the one containing
    # this script.
    [string]$RepoRoot
)

$ErrorActionPreference = "Stop"

if (-not $InstalledPath) {
    $found = Get-Command vanta-cli -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $found) {
        Write-Host "FAIL: vanta-cli not found on PATH (pass -InstalledPath)."
        exit 2
    }
    $InstalledPath = $found.Source
}

if (-not $RepoRoot) {
    $RepoRoot = (& git -C $PSScriptRoot rev-parse --show-toplevel 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $RepoRoot) {
        Write-Host "FAIL: $PSScriptRoot is not inside a git checkout (pass -RepoRoot)."
        exit 2
    }
}

$headFull = (& git -C $RepoRoot rev-parse --verify HEAD 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $headFull) {
    Write-Host "FAIL: cannot resolve HEAD in $RepoRoot."
    exit 2
}
$head = (& git -C $RepoRoot rev-parse --short HEAD).Trim()

$line = (& $InstalledPath --version 2>&1 | Out-String).Trim()

if ($line -notmatch 'commit ([0-9a-fA-F]{7,40})') {
    Write-Host "FAIL: installed vanta-cli does not declare a commit - it predates FIND-98 and cannot be verified."
    Write-Host "  binary:  $InstalledPath"
    Write-Host "  reports: $line"
    Write-Host "  HEAD:    $head"
    Write-Host "  fix:     cargo install --path `"$RepoRoot`" --bin vanta-cli --force"
    exit 1
}

$declared = $Matches[1]
$declaredFull = (& git -C $RepoRoot rev-parse --verify "$declared^{commit}" 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $declaredFull) {
    Write-Host "FAIL: installed vanta-cli declares commit $declared, which is not in this repository."
    Write-Host "  binary:  $InstalledPath"
    Write-Host "  reports: $line"
    Write-Host "  HEAD:    $head in $RepoRoot"
    exit 1
}

if ($declaredFull.Trim() -ne $headFull.Trim()) {
    Write-Host "FAIL: installed vanta-cli is stale."
    Write-Host "  binary declares: $declared (built from a different commit)"
    Write-Host "  repo HEAD:       $head"
    Write-Host "  binary:          $InstalledPath"
    Write-Host "  fix:             cargo install --path `"$RepoRoot`" --bin vanta-cli --force"
    exit 1
}

if ($line -match 'worktree dirty') {
    Write-Warning "installed vanta-cli declares commit $declared but was built from a dirty tree: $line"
}

Write-Host "OK: installed vanta-cli matches HEAD ($head) - $line"
exit 0
