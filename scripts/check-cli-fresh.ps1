#!/usr/bin/env pwsh
# FIND-98: the installed `vanta-cli` must have been built from this repo's HEAD.
#
# The binary embeds `<version> (commit <sha>)` at build time (build.rs ->
# OUT_DIR/build_version.txt -> src/bin/vanta-cli.rs). Before that, a copy from
# two months earlier answered `vanta-cli 0.9.0` exactly like a fresh build, so
# nothing could tell an operator their CLI was stale.
#
# Exit codes: 0 fresh · 1 stale / unverifiable · 2 cannot check (no binary,
# unreadable binary, not a git checkout). "Cannot check" is never reported as
# "stale": a crash, a missing DLL or a wrong-arch binary are not a stale build.
[CmdletBinding()]
param(
    # Binary to check. Defaults to the first `vanta-cli` on PATH
    # (~/.cargo/bin/vanta-cli.exe on a default cargo install).
    [string]$InstalledPath,
    # Repository whose HEAD is the reference. Defaults to the one containing
    # this script.
    [string]$RepoRoot,
    # FIND-754: report findings, exit 0, and say nothing at all when there is no
    # installed binary or when it already matches HEAD. This is the pre-push
    # mode: the verdict of a push must not depend on a binary the pusher may
    # never have installed. Every exit route below goes through Exit-Check; a
    # TERMINATING error that throws before reaching one (e.g. `git` missing from
    # PATH) still exits !=0 and is the caller's to absorb — the pre-push hook
    # does exactly that with `|| true`. Without this switch the 0/1/2 matrix
    # above is unchanged.
    [switch]$Advisory
)

$ErrorActionPreference = "Stop"

# Every exit goes through here so -Advisory can neutralize the code in one
# place: the messages (what to fix, both commits) are the value, the code is
# what a caller that wants a verdict uses.
function Exit-Check([int]$Code) {
    if ($Advisory) { exit 0 }
    exit $Code
}

if (-not $InstalledPath) {
    $found = Get-Command vanta-cli -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $found) {
        # No installed CLI is the normal case (CI runners, fresh clones, lanes
        # that only touch docs): in -Advisory it is silence, never a complaint.
        if (-not $Advisory) {
            Write-Host "CANNOT CHECK: vanta-cli not found on PATH (pass -InstalledPath)."
        }
        Exit-Check 2
    }
    $InstalledPath = $found.Source
}

if (-not (Test-Path -LiteralPath $InstalledPath -PathType Leaf)) {
    Write-Host "CANNOT CHECK: $InstalledPath does not exist or is not a file."
    Exit-Check 2
}

if (-not $RepoRoot) {
    $RepoRoot = (& git -C $PSScriptRoot rev-parse --show-toplevel 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $RepoRoot) {
        Write-Host "CANNOT CHECK: $PSScriptRoot is not inside a git checkout (pass -RepoRoot)."
        Exit-Check 2
    }
}

$headFull = (& git -C $RepoRoot rev-parse --verify HEAD 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $headFull) {
    Write-Host "CANNOT CHECK: cannot resolve HEAD in $RepoRoot."
    Exit-Check 2
}
$head = (& git -C $RepoRoot rev-parse --short HEAD)

$line = $null
try {
    $line = (& $InstalledPath --version 2>$null | Out-String)
} catch {
    Write-Host "CANNOT CHECK: $InstalledPath failed to run: $($_.Exception.Message)"
    Exit-Check 2
}
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($line)) {
    Write-Host "CANNOT CHECK: $InstalledPath --version exited $LASTEXITCODE without output."
    Exit-Check 2
}
$line = $line.Trim()

if ($line -match 'commit unknown') {
    Write-Host "UNVERIFIABLE: the installed vanta-cli declares no commit (built outside a git checkout)."
    Write-Host "  binary:  $InstalledPath"
    Write-Host "  reports: $line"
    Write-Host "  HEAD:    $head"
    Write-Host "  fix:     build from a checkout (or set VANTADB_BUILD_COMMIT) and reinstall:"
    Write-Host "           cargo install --path `"$RepoRoot`" --bin vanta-cli --force"
    Exit-Check 1
}

if ($line -notmatch 'commit ([0-9a-fA-F]{7,40})') {
    Write-Host "STALE: the installed vanta-cli does not declare a commit - it predates FIND-98 and cannot be verified."
    Write-Host "  binary:  $InstalledPath"
    Write-Host "  reports: $line"
    Write-Host "  HEAD:    $head"
    Write-Host "  fix:     cargo install --path `"$RepoRoot`" --bin vanta-cli --force"
    Exit-Check 1
}

$declared = $Matches[1]
$declaredFull = (& git -C $RepoRoot rev-parse --verify "$declared^{commit}" 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $declaredFull) {
    Write-Host "STALE: the installed vanta-cli declares commit $declared, which is not in this repository."
    Write-Host "  binary:  $InstalledPath"
    Write-Host "  reports: $line"
    Write-Host "  HEAD:    $head in $RepoRoot"
    Exit-Check 1
}

if ($declaredFull.Trim() -ne $headFull.Trim()) {
    Write-Host "STALE: the installed vanta-cli is behind HEAD."
    Write-Host "  binary declares: $declared (a different commit)"
    Write-Host "  repo HEAD:       $head"
    Write-Host "  binary:          $InstalledPath"
    Write-Host "  fix:             cargo install --path `"$RepoRoot`" --bin vanta-cli --force"
    Exit-Check 1
}

# Fresh (and the dirty-worktree note) is the only case with nothing to act on:
# in -Advisory it prints NOTHING, so the pre-push stays silent for the lanes
# that did what the message asked. The dirty note is tree-wide and this repo is
# always dirty with 4 lanes — printing it on every push would be the permanent
# false positive that makes an advisory ignorable (FIND-754 review, measured).
if (-not $Advisory) {
    # The identity is the base commit; a build made with uncommitted changes still
    # declares that commit. Say so instead of pretending the artifact is exactly HEAD.
    $dirty = (& git -C $RepoRoot status --porcelain --untracked-files=no)
    if ($LASTEXITCODE -eq 0 -and $dirty) {
        Write-Warning "the repository has uncommitted changes: an artifact built now would carry $head plus local edits."
    }

    Write-Host "OK: installed vanta-cli matches HEAD ($head) - $line"
}
Exit-Check 0
