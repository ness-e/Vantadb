<#
.SYNOPSIS
  Commit ONLY the paths you name, through a PRIVATE index, and prove the commit
  by CONTENT afterwards.

.DESCRIPTION
  FIND-753. With several lanes sharing one checkout, `.git/index` is a race:
  - `Unable to create .git/index.lock` on 8 consecutive attempts (4 lanes);
  - a concurrent commit baked a STALE copy of `docs/dev/Backlog.md`, so a row
    another lane had just deleted reappeared in HEAD;
  - the index held another lane's pre-staged files (`tests/cli_tests.rs`, -92
    lines): a bare `git commit -a` would have deleted committed work;
  - `MM docs/dev/Backlog.md` with a staged 6-line deletion against a working
    tree identical to HEAD.

  **The exit code proves nothing**: the commit that reverted another lane's edit
  exited 0. This script re-reads every expectation from `HEAD` after committing
  and fails if one is wrong.

  How: private index (`GIT_INDEX_FILE`) seeded with `read-tree HEAD`, `add` of
  the named paths only, `commit`. The shared index is never read nor written
  DURING the commit, so another lane's staged work cannot ride along. After the
  commit the script re-adds those same paths in the shared index — without that
  refresh the shared index still describes the previous HEAD and any lane's bare
  `git commit` would revert the commit (measured 2026-11-26).

  WHEN NOT TO USE IT: interactive staging (`git add -p`) or any commit that must
  carry what is ALREADY staged in the shared index; merge commits (MERGE_HEAD
  must be in the index, and this seeds from a single tree); `--amend` (out of
  scope by policy). Reach for plain `git commit -- <paths>` there.

.PARAMETER Message
  Commit message (conventional commits; no Co-Authored-By trailers in this repo).

.PARAMETER Paths
  Repo-relative paths to commit. Comma-separated or repeated. Nothing else is.

.PARAMETER ExpectAbsent
  `<path>::<literal>` that must NOT appear in `HEAD:<path>` after the commit.
  This is the check that catches a stale copy being baked in. Repeatable.

.PARAMETER ExpectPresent
  `<path>::<literal>` that MUST appear in `HEAD:<path>`. Repeatable.

.EXAMPLE
  pwsh dev-tools/git-commit-isolated.ps1 -Message "fix(ci): pre-push judges the range" `
      -Paths .githooks/pre-push,dev-tools/verify.ps1,docs/dev/Backlog.md `
      -ExpectAbsent docs/dev/Backlog.md::FIND-760 `
      -ExpectPresent .githooks/pre-push::skip_gate
#>
param(
    [Parameter(Mandatory = $true)][string]$Message,
    [Parameter(Mandatory = $true)][string[]]$Paths,
    [string[]]$ExpectAbsent = @(),
    [string[]]$ExpectPresent = @()
)

$ErrorActionPreference = "Stop"

$repo = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $repo) { Write-Host "ERROR: not a git repository." -ForegroundColor Red; exit 2 }
Set-Location $repo

$tmpDir = Join-Path $env:TEMP "opencode"
New-Item -ItemType Directory -Force $tmpDir | Out-Null
$idx = Join-Path $tmpDir ("idx-" + [guid]::NewGuid().ToString("N").Substring(0, 8))

$saved = $env:GIT_INDEX_FILE
$ok = $true
try {
    $env:GIT_INDEX_FILE = $idx
    & git read-tree HEAD
    if ($LASTEXITCODE -ne 0) { throw "git read-tree HEAD failed (empty repo?)" }

    & git add -- @Paths
    if ($LASTEXITCODE -ne 0) { throw "git add failed for: $($Paths -join ', ')" }

    $staged = & git diff --cached --name-status
    if (-not $staged) { throw "nothing to commit: no named path differs from HEAD" }
    Write-Host "staged in the private index:"
    $staged | ForEach-Object { Write-Host "  $_" }

    & git commit -m $Message
    if ($LASTEXITCODE -ne 0) { throw "git commit failed (exit $LASTEXITCODE)" }
} catch {
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    $ok = $false
} finally {
    if ($null -eq $saved) { Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue } else { $env:GIT_INDEX_FILE = $saved }
    Remove-Item $idx -Force -ErrorAction SilentlyContinue
}
if (-not $ok) { exit 2 }

$commit = (& git rev-parse --short HEAD).Trim()
Write-Host "committed: $commit"

# ── Refresh the SHARED index for these paths ────────────────────────────────
# Not optional and not cosmetic: the private-index commit does not update
# `.git/index`, so after it the shared index still describes the PREVIOUS HEAD.
# Measured 2026-11-26: right after a private-index commit, `git diff --cached`
# showed the whole commit as staged-for-reversion — any lane's bare `git commit`
# (or `git commit -a`) would have reverted it. Re-adding only these paths (whose
# worktree content is exactly what was committed) syncs them without touching
# another lane's staged entries.
& git add -- @Paths | Out-Null
Write-Host "shared index refreshed for: $($Paths -join ', ')"

# ── Verification BY CONTENT ─────────────────────────────────────────────────
# Deliberately not `git log`/exit-code based: a concurrent commit can bake a
# stale copy of a shared file and still exit 0. Only the bytes in HEAD answer
# "did my edit survive?".
$bad = 0
function Test-Content([string]$spec, [bool]$mustBePresent) {
    $path, $frag = $spec -split '::', 2
    if (-not $frag) { Write-Host "BAD EXPECTATION, missing '::': $spec" -ForegroundColor Red; return 1 }
    $content = ((& git show "HEAD:$path" 2>$null) -join "`n")
    if ($LASTEXITCODE -ne 0) { Write-Host "MISSING IN HEAD: $path" -ForegroundColor Red; return 1 }
    $found = $content -match [regex]::Escape($frag)
    if ($found -eq $mustBePresent) { Write-Host "ok $(if ($mustBePresent) { 'present' } else { 'absent' }): $path :: $frag" -ForegroundColor Green; return 0 }
    if ($mustBePresent) {
        Write-Host "NOT COMMITTED: '$frag' is missing from HEAD:$path." -ForegroundColor Red
    } else {
        Write-Host "REVERTED: '$frag' is STILL in HEAD:$path — the commit did not carry your edit." -ForegroundColor Red
    }
    return 1
}

foreach ($e in $ExpectAbsent) { $bad += Test-Content $e $false }
foreach ($e in $ExpectPresent) { $bad += Test-Content $e $true }

if ($bad -gt 0) {
    Write-Host "FAILED: $bad content expectation(s) unmet at $commit." -ForegroundColor Red
    Write-Host "  Inspect: git show --stat $commit ; git show $commit -- <path>"
    exit 1
}
Write-Host "OK: $commit verified by content ($($ExpectAbsent.Count + $ExpectPresent.Count) expectation(s))." -ForegroundColor Green
exit 0
