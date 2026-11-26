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

  FIND-778 — RESULT GUARDS. Two commits were created with the canonical EMPTY
  tree (`4b825dc…`) that deleted all 2365 paths, while the worktree stayed
  intact; `git commit` reported `failed (exit 1)` and the script therefore
  skipped verification. The mechanism never reproduced. The guards below fail
  CLOSED on both paths, because "the command failed" does not mean "nothing was
  created":
   1. the committed tree must not be the empty tree;
   2. the commit must not delete >= 50 paths that STILL EXIST in the worktree
      (deleting hundreds of files that are still on disk is incoherent, while a
      deliberate `git rm` removes them from disk too);
   3. the committed tree must not lose more than 200 entries relative to
      `HEAD`'s tree — measured over the last 300 commits, the largest legitimate
      drop is 43 (`960e338d0`), and the catastrophe was 2365. Known ceiling: a
      deliberate bulk delete of MORE than 200 tracked files is refused too; use
      plain `git commit` for that (FIND-778 review M2);
   4. every path staged for the commit must be present in the committed tree.
  On any violation the ref is restored to the pre-commit SHA (compare-and-swap
  `git update-ref HEAD <old> <new>`, so a concurrent update is never clobbered),
  the bad commit SHA is printed and KEPT (never deleted, never amended), and the
  script exits 4. The private index is removed on every path, including errors.
  History is never rewritten: the guards only refuse to LEAVE HEAD there.

  If the commit on HEAD is provably ANOTHER lane's — my `git commit` failed while
  they committed in the same window — it is never restored: that would revert
  their work. Identity is proved by CONTENT (the tree we staged, or every named
  path byte-identical), never by the message: two lanes using the same
  conventional subject would collide. The script reports it and exits 6. The one
  exception is the canonical EMPTY tree: it deletes the whole repository, so
  there is no work of anyone's to preserve and the ref is restored anyway.

  When `git commit` exits non-zero but HEAD did move to a commit that is provably
  ours, the commit stands and the script says so loudly; a non-zero exit with a
  commit that is NOT provably ours is reported as "nothing of yours was
  committed" and exits 6 — never 0.

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
  Neither the path nor the literal may contain a comma (the list is comma-split;
  a comma makes it two specs, and a malformed spec fails loudly, never silently).

.PARAMETER ExpectPresent
  `<path>::<literal>` that MUST appear in `HEAD:<path>`. Repeatable, same rule.

.EXAMPLE
  pwsh dev-tools/git-commit-isolated.ps1 -Message "fix(ci): pre-push judges the range" `
      -Paths .githooks/pre-push,dev-tools/verify.ps1,docs/dev/Backlog.md `
      -ExpectAbsent docs/dev/Backlog.md::FIND-760 `
      -ExpectPresent .githooks/pre-push::skip_gate

.NOTES
  Exit codes: 0 ok · 1 content expectation unmet · 2 commit not created ·
  3 commit ok but the SHARED index refresh failed · 4 commit REFUSED by the
  FIND-778 result guards (HEAD restored) · 5 restore itself failed ·
  6 REFUSED but HEAD is another lane's commit (kept, not restored).
#>
param(
    [Parameter(Mandatory = $true)][string]$Message,
    [Parameter(Mandatory = $true)][string[]]$Paths,
    [string[]]$ExpectAbsent = @(),
    [string[]]$ExpectPresent = @()
)

# `pwsh -File script.ps1 -Paths a,b,c` cannot bind an array (it expands values
# into positional args), so accept the comma-joined form too — same as
# verify.ps1 -Skip. Repeatable flags work as well.
$Paths = @($Paths | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$ExpectAbsent = @($ExpectAbsent | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$ExpectPresent = @($ExpectPresent | ForEach-Object { $_ -split ',' } | Where-Object { $_ })

$ErrorActionPreference = "Stop"

$repo = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $repo) { Write-Host "ERROR: not a git repository." -ForegroundColor Red; exit 2 }
Set-Location $repo

# ── FIND-778 constants ──────────────────────────────────────────────────────
# Canonical empty tree (sha1), fixed by definition of the git object format:
# `git hash-object -t tree /dev/null`. A commit pointing here deletes everything.
$EMPTY_TREE = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"

# Entry-count floor. Measured over the last 300 commits of this repo: the
# largest legitimate drop is 43 (960e338d0); the two empty-tree commits dropped
# 2365. 200 keeps ~4.6x headroom over the largest legitimate deletion, so a mass
# `git rm` still commits, while staying an order of magnitude below the damage.
$MAX_ENTRY_DROP = 200

# A deletion whose file is still on disk is incoherent: a real `git rm` removes
# the file. One such path is a smell; this many is the incident's signature.
$MAX_DELETED_BUT_PRESENT = 50

function Get-TreeEntryCount([string]$rev) {
    $lines = @(& git ls-tree -r --name-only $rev 2>$null)
    if ($LASTEXITCODE -ne 0) { return -1 }
    # `@($null).Count` is 1, so filter empty rows: an empty tree must count 0.
    return [int](@($lines | Where-Object { $_ }).Count)
}

# Windows can keep a handle open on the private index right after git exits;
# a single Remove-Item then fails silently and leaves a ~250 KB leftover (39 of
# them measured in %TEMP%\opencode before FIND-778). Retry, then report.
function Remove-PrivateIndex([string]$path) {
    for ($i = 0; $i -lt 10; $i++) {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath ($path + ".lock") -Force -ErrorAction SilentlyContinue
        if (-not (Test-Path -LiteralPath $path)) { return $true }
        Start-Sleep -Milliseconds 150
    }
    return (-not (Test-Path -LiteralPath $path))
}

$tmpDir = Join-Path $env:TEMP "opencode"
New-Item -ItemType Directory -Force $tmpDir | Out-Null
# PID keeps two live lanes from ever sharing a private index (the incident left
# leftovers named after lanes, so a unique name is also easier to trace).
$idx = Join-Path $tmpDir ("idx-" + $PID + "-" + [guid]::NewGuid().ToString("N").Substring(0, 8))

$headBefore = (& git rev-parse --verify --quiet HEAD 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $headBefore) { $headBefore = $null } else { $headBefore = "$headBefore".Trim() }
$entriesBefore = if ($headBefore) { Get-TreeEntryCount $headBefore } else { 0 }

$treePlanned = $null      # tree we staged in the private index
$commitExit = $null       # exit code of `git commit` (null = never ran)
$preCommitError = $null   # failure before the commit (read-tree/add/guards)

$saved = $env:GIT_INDEX_FILE
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

    # FIND-778: never commit an index we did not seed. If the private index was
    # wiped (temp cleaner, killed process, wrong GIT_INDEX_FILE) `git commit`
    # happily commits an EMPTY index — that is the shape of the incident.
    if (-not (Test-Path -LiteralPath $idx)) { throw "ABORT: private index $idx vanished before commit" }
    if ((Get-Item -LiteralPath $idx).Length -le 0) { throw "ABORT: private index $idx is empty before commit" }
    if (@(& git ls-files --cached | Where-Object { $_ }).Count -lt 1) { throw "ABORT: private index lists 0 files before commit" }

    # Freeze what we are about to commit. The guards compare the committed tree
    # against this, not against a guess.
    $treePlanned = "$(& git write-tree)" -replace '\s', ''
    if ($LASTEXITCODE -ne 0 -or -not $treePlanned) { throw "git write-tree failed on the private index" }

    & git commit -m $Message
    $commitExit = $LASTEXITCODE
} catch {
    $preCommitError = $_.Exception.Message
} finally {
    # Runs on the error path too — that is the path that failed today.
    if ($null -eq $saved) { Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue } else { $env:GIT_INDEX_FILE = $saved }
    if (-not (Remove-PrivateIndex $idx)) {
        Write-Host "WARNING: could not delete the private index $idx (a process still holds it)." -ForegroundColor Yellow
        Write-Host "  delete it by hand later: Remove-Item '$idx'" -ForegroundColor Yellow
    }
}

# ══ RESULT VALIDATION — RUNS ON BOTH PATHS (FIND-778) ═══════════════════════
# Not an `if success` block: `git commit` exiting non-zero does NOT mean nothing
# was created. Both of today's empty-tree commits were reported as failures.
$headAfter = (& git rev-parse --verify --quiet HEAD 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $headAfter) { $headAfter = $null } else { $headAfter = "$headAfter".Trim() }
$moved = [bool]($headBefore -and $headAfter -and ($headAfter -ne $headBefore))

$reasons = New-Object System.Collections.Generic.List[string]
$isOurs = $false
$treeAfter = $null

if ($commitExit -eq 0 -and -not $moved) {
    $reasons.Add("git commit exited 0 but HEAD did not move ($headBefore): nothing was committed")
}

if ($moved) {
    $headAfterTree = & git rev-parse "$headAfter^{tree}"
    $treeAfter = "$headAfterTree".Trim()
    $entriesAfter = Get-TreeEntryCount $headAfter

    # Whose commit is on HEAD? If MY `git commit` failed and ANOTHER lane committed
    # in the same window, these guards would be judging THEIR commit — and
    # restoring it would revert their work. Identity is proved by CONTENT only: the
    # exact tree we staged, or every path we named landing byte-identical. The
    # commit SUBJECT is not identity: two lanes using the same conventional message
    # would collide and a restore would revert the peer's commit (P2-01 M1).

    # (1) the canonical empty tree
    if ($treeAfter -eq $EMPTY_TREE) {
        $reasons.Add("CATASTROPHIC: commit $headAfter has the canonical EMPTY tree ($EMPTY_TREE) — it deletes every tracked path")
    }

    # (2) deletions whose file is still on disk
    if ($headBefore) {
        $deleted = @(& git diff-tree -r --no-commit-id --name-only --diff-filter=D $headBefore $headAfter 2>$null)
        if ($LASTEXITCODE -ne 0) {
            # Fail CLOSED: an unreadable diff is not a clean diff (P2-01 L1).
            $reasons.Add("SUSPICIOUS: git diff-tree failed; cannot verify what commit $headAfter deletes")
        } else {
            $stillOnDisk = @($deleted | Where-Object { $_ -and (Test-Path -LiteralPath (Join-Path $repo $_)) })
            if ($stillOnDisk.Count -ge $MAX_DELETED_BUT_PRESENT) {
                $reasons.Add("CATASTROPHIC: commit $headAfter deletes $($stillOnDisk.Count) path(s) that STILL EXIST in the worktree (first: $($stillOnDisk[0]))")
            }
        }
    }

    # (3) entry-count floor
    if ($entriesBefore -lt 0 -or $entriesAfter -lt 0) {
        # Fail CLOSED: Get-TreeEntryCount returns -1 when `git ls-tree` fails, and an
        # unmeasured tree is not a small tree (P2-01 L1).
        $reasons.Add("SUSPICIOUS: cannot measure tree entries (git ls-tree failed) — refusing to report success on an unverified commit")
    } elseif ($entriesAfter -lt ($entriesBefore - $MAX_ENTRY_DROP)) {
        $reasons.Add("SUSPICIOUS: commit $headAfter has $entriesAfter tree entries, HEAD had $entriesBefore (drop exceeds the $MAX_ENTRY_DROP allowed; largest legitimate drop measured: 43)")
    }

    # (4) what we staged must be what got committed, BYTE FOR BYTE. Presence alone
    # proves nothing: an old blob of the same path exists too (P2-01 C1).
    $pathsMatch = $false
    if ($treePlanned) {
        $plannedPaths = @(& git ls-tree --name-only $treePlanned -- @Paths 2>$null | Where-Object { $_ })
        $mismatch = @()
        foreach ($p in $plannedPaths) {
            $want = "$(& git rev-parse --verify --quiet "$treePlanned`:$p" 2>$null)".Trim()
            $got = "$(& git rev-parse --verify --quiet "$headAfter`:$p" 2>$null)".Trim()
            if (-not $want -or $want -ne $got) { $mismatch += $p }
        }
        $pathsMatch = ($plannedPaths.Count -gt 0) -and ($mismatch.Count -eq 0)
        if ($mismatch.Count -gt 0) {
            $reasons.Add("CATASTROPHIC: $($mismatch.Count) staged path(s) are absent or have DIFFERENT content in commit $headAfter (first: $($mismatch[0]))")
        }
    }

    $isOurs = [bool](($treePlanned -and ($treeAfter -eq $treePlanned)) -or $pathsMatch)
}

if ($reasons.Count -gt 0) {
    Write-Host ""
    Write-Host "REFUSED (FIND-778): the commit on HEAD is not the commit you asked for." -ForegroundColor Red
    foreach ($r in $reasons) { Write-Host "  - $r" -ForegroundColor Red }
    # H1: an EMPTY tree deletes the entire repository. There is no work of anyone's
    # to preserve, so restoring is always right even when the commit is not ours.
    if ($moved -and -not $isOurs -and ($treeAfter -ne $EMPTY_TREE)) {
        Write-Host "  HEAD was moved by ANOTHER lane to $headAfter and none of the content is yours." -ForegroundColor Yellow
        Write-Host "  NOT restoring it: that would revert their commit. Nothing of yours was committed." -ForegroundColor Yellow
        Write-Host "  the commit on HEAD is KEPT as-is: $headAfter" -ForegroundColor Yellow
        exit 6
    }
    if ($moved) {
        $branch = (& git symbolic-ref -q --short HEAD 2>$null)
        if (-not $branch) { $branch = "HEAD (detached)" } else { $branch = "$branch".Trim() }
        if (-not $isOurs) {
            Write-Host "  (that commit is not ours, but an EMPTY tree deletes the whole repo: restored anyway)" -ForegroundColor Yellow
        }
        Write-Host "  restoring $branch to $headBefore (compare-and-swap, never clobbers a concurrent update)" -ForegroundColor Yellow
        & git update-ref HEAD $headBefore $headAfter
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  RESTORE FAILED: HEAD is STILL at $headAfter. Fix by hand:" -ForegroundColor Red
            Write-Host "    git update-ref HEAD $headBefore $headAfter" -ForegroundColor Red
            exit 5
        }
        Write-Host "  restored: HEAD is $((& git rev-parse --short HEAD)) (tree $((& git rev-parse --short 'HEAD^{tree}')))" -ForegroundColor Yellow
    }
    Write-Host "  the refused commit object is KEPT for the record (never amended, never deleted): $headAfter" -ForegroundColor Yellow
    Write-Host "  inspect it: git show --stat $headAfter" -ForegroundColor Yellow
    exit 4
}

if (-not $moved) {
    # Nothing was created — report the original failure, nothing to restore.
    if ($preCommitError) { Write-Host "ERROR: $preCommitError" -ForegroundColor Red }
    elseif ($null -ne $commitExit) { Write-Host "ERROR: git commit failed (exit $commitExit); HEAD did not move." -ForegroundColor Red }
    else { Write-Host "ERROR: the commit was not created." -ForegroundColor Red }
    exit 2
}

# P2-01 C1: HEAD moved, no guard tripped, but the commit is not provably ours —
# e.g. `git commit` failed and another lane committed in the same window. The
# pre-fix script exited 2 here; reporting success would tell the caller their work
# landed when it did not. Success requires identity.
if (-not $isOurs) {
    if ($preCommitError) { Write-Host "ERROR: $preCommitError" -ForegroundColor Red }
    Write-Host ""
    Write-Host "NOTHING OF YOURS WAS COMMITTED (FIND-778)." -ForegroundColor Red
    Write-Host "  HEAD moved to $headAfter, but that is not the commit you staged: different" -ForegroundColor Red
    Write-Host "  tree and the paths you named are not there byte-identical." -ForegroundColor Red
    Write-Host "  It belongs to another lane and was left untouched. Your work is still in the" -ForegroundColor Red
    Write-Host "  worktree — re-run the commit." -ForegroundColor Red
    exit 6
}

$commit = (& git rev-parse --short HEAD).Trim()
if ($null -ne $commitExit -and $commitExit -ne 0) {
    Write-Host "NOTE: git commit reported exit $commitExit but HEAD moved to $commit and the result is provably ours." -ForegroundColor Yellow
}

# ── Refresh the SHARED index for these paths ────────────────────────────────
# Not optional and not cosmetic: the private-index commit does not update
# `.git/index`, so after it the shared index still describes the PREVIOUS HEAD.
# Measured 2026-11-26: right after a private-index commit, `git diff --cached`
# showed the whole commit as staged-for-reversion — any lane's bare `git commit`
# (or `git commit -a`) would have reverted it. `git reset -- <paths>` copies
# HEAD's entries for exactly these paths into the shared index, which is correct
# for a modification, an addition AND a deletion (`git add` cannot express the
# last one: the path is gone from both worktree and index, exit 128). Other
# lanes' staged entries are not touched.
& git reset -q -- @Paths 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    # Never report success here: if the shared index keeps the OLD HEAD, any lane's
    # bare `git commit` reverts this commit (FIND-753 / `Unable to create
    # .git/index.lock`, measured 8 consecutive times with 4 lanes).
    Write-Host "commit ok but the SHARED INDEX REFRESH FAILED (git reset exit $LASTEXITCODE)." -ForegroundColor Red
    Write-Host "  fix now: git reset -- $($Paths -join ' ')" -ForegroundColor Red
    Write-Host "  until then a bare 'git commit' from any lane can revert $commit." -ForegroundColor Red
    exit 3
}
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
