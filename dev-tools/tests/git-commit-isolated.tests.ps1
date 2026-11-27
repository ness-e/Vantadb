# FIND-778 test: the helper must REFUSE a commit whose tree is the canonical
# empty tree (4b825dc…), on the success path AND on the failure path, and must
# leave HEAD on the previous SHA, the worktree intact, and no private index
# behind.
#
# FIND-780 adds the structural property, and it is the reason this file exists:
# KILLING the process that commits must not move HEAD. Scenario 10 starts the
# helper with Start-Process, waits until the pre-commit hook is running, then
# `Stop-Process -Force` on the helper itself (not on a subprocess — a killed
# parent leaves the `git commit` CHILD alive on Windows, and that child is what
# baked the empty tree three times: 72643d9df, 290df23d0, 5766654ee). With the
# plumbing (`commit-tree` + `update-ref`) there is no child holding commit
# authority, so the kill leaves HEAD where it was.
#
# FIND-792/793/795/796 add scenarios 16-20, one per defect measured on
# 2026-11-26, all of them in how the check PARSED and READ its expectations:
#   16 a comma inside the expected text (blind split -> verified a fragment);
#   17 an accent: NFD vs NFC (console text), and a double-encoded expectation
#      (the blob is read as UTF-8 bytes, and the class is ENCODING, not content);
#   18 a fragment that matches twice -> AMBIGUOUS with both `file:line`s and an
#      exit code of its own, and never the unmeasured "did not carry your edit";
#   19 a `docs/dev/Backlog.md` commit that would retire ANOTHER lane's row;
#   20 blob identity (`-ExpectBlob`), the form that does not depend on text.
#
# EVERY scenario below fails against the pre-FIND-780 helper, and 16-20 also fail
# against the helper of `HEAD` at the time of writing ($env:FIND780_HELPER): that
# is the measurement this file exists to force. Reproduce it on purpose:
#   git show HEAD:dev-tools/git-commit-isolated.ps1 > $env:TEMP\old.ps1
#   $env:FIND780_HELPER = "$env:TEMP\old.ps1"; pwsh -File <this file>
#
# Self-contained: every scenario builds a throwaway git repo under $env:TEMP and
# points the child's TEMP at it, so the private index never lands in the real
# %TEMP%\opencode and the real checkout is never touched.
#
# Run: pwsh -NoProfile -File dev-tools/tests/git-commit-isolated.tests.ps1

$ErrorActionPreference = "Stop"

$SCRIPT = if ($env:FIND780_HELPER) { (Resolve-Path $env:FIND780_HELPER).Path }
else { (Resolve-Path (Join-Path $PSScriptRoot ".." "git-commit-isolated.ps1")).Path }
$EMPTY_TREE = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
$ROOT = Join-Path $env:TEMP ("f778-" + [guid]::NewGuid().ToString("N").Substring(0, 8))

$fail = 0
function Check([bool]$cond, [string]$what) {
    if ($cond) { Write-Host "  PASS  $what" -ForegroundColor Green }
    else { Write-Host "  FAIL  $what" -ForegroundColor Red; $script:fail++ }
}

function Write-Hook([string]$dir, [string]$body) {
    $hooks = Join-Path $dir ".githooks-test"
    New-Item -ItemType Directory -Force $hooks | Out-Null
    # LF endings: git runs hooks through sh.
    [IO.File]::WriteAllText((Join-Path $hooks "pre-commit"), ("#!/bin/sh`n" + $body + "`n").Replace("`r`n", "`n"))
    git -C $dir config core.hooksPath ".githooks-test"
}

function New-TestRepo([string]$name) {
    $dir = Join-Path $ROOT $name
    New-Item -ItemType Directory -Force $dir | Out-Null
    git -C $dir init -q
    if ($LASTEXITCODE -ne 0) { throw "git init failed in $dir" }
    git -C $dir config user.email "f778@test.invalid"
    git -C $dir config user.name "FIND-778"
    git -C $dir config core.autocrlf false
    git -C $dir config commit.gpgsign false
    New-Item -ItemType Directory -Force (Join-Path $dir "docs") | Out-Null
    for ($i = 0; $i -lt 60; $i++) { Set-Content -LiteralPath (Join-Path $dir "docs/file$i.txt") -Value "line $i" -NoNewline }
    git -C $dir add -A
    git -C $dir commit -qm "init"
    return $dir
}

# Runs the helper inside the repo with an isolated TEMP so the private index
# lands under the test dir. Returns the helper's exit code.
function Invoke-Helper([string]$dir, [string[]]$helperArgs) {
    $savedTemp = $env:TEMP
    $savedTmp = $env:TMP
    try {
        $env:TEMP = Join-Path $dir "tmp"
        $env:TMP = $env:TEMP
        New-Item -ItemType Directory -Force $env:TEMP | Out-Null
        # The helper resolves the repo from the CURRENT directory: it MUST be the
        # throwaway repo, never the real checkout.
        Push-Location $dir
        try {
            $out = & pwsh -NoProfile -File $SCRIPT @helperArgs 2>&1
            $code = $LASTEXITCODE
        } finally {
            Pop-Location
        }
        $out | ForEach-Object { Write-Host "    | $_" }
        return [pscustomobject]@{ Code = $code; Out = ($out -join "`n") }
    } finally {
        $env:TEMP = $savedTemp
        $env:TMP = $savedTmp
    }
}

function Get-LeftoverIndexes([string]$dir) {
    $d = Join-Path $dir "tmp" "opencode"
    if (-not (Test-Path $d)) { return @() }
    return @(Get-ChildItem $d -Filter "idx-*" -ErrorAction SilentlyContinue)
}

# FIND-793/795: `pwsh -File` cannot bind an ARRAY argument (it joins the elements
# into one string) and is the wrong channel for a fragment whose bytes matter.
# This invoker writes a launcher INSIDE the throwaway repo and runs it: the child
# builds the expectation itself (from code points, so no encoding can touch it)
# and calls the helper exactly the way the documented usage does — `& $helper`.
# The helper's `exit N` ends the launcher, so $LASTEXITCODE is the helper's code.
function Invoke-HelperScripted([string]$dir, [string]$callBody) {
    $launcher = Join-Path $dir ".launcher.ps1"
    $escapedScript = $SCRIPT -replace "'", "''"
    $src = "`$ErrorActionPreference = `"Stop`"`n" + "`$helper = '$escapedScript'`n" + $callBody + "`nexit `$LASTEXITCODE`n"
    [IO.File]::WriteAllText($launcher, $src, (New-Object Text.UTF8Encoding($false)))
    $savedTemp = $env:TEMP
    $savedTmp = $env:TMP
    try {
        $env:TEMP = Join-Path $dir "tmp"
        $env:TMP = $env:TEMP
        New-Item -ItemType Directory -Force $env:TEMP | Out-Null
        Push-Location $dir
        try {
            $out = & pwsh -NoProfile -File $launcher 2>&1
            $code = $LASTEXITCODE
        } finally {
            Pop-Location
        }
        $out | ForEach-Object { Write-Host "    | $_" }
        return [pscustomobject]@{ Code = $code; Out = ($out -join "`n") }
    } finally {
        $env:TEMP = $savedTemp
        $env:TMP = $savedTmp
    }
}

function Get-WorktreeFileCount([string]$dir) {
    return @(Get-ChildItem (Join-Path $dir "docs") -File -ErrorAction SilentlyContinue).Count
}

try {
    # ── Scenario 1: SUCCESS path, tree ends up empty ────────────────────────
    # The hook wipes the private index after write-tree; `git commit` then
    # commits the empty index and exits 0 — the incident's payload, clean exit.
    Write-Host "`n[1] success path, empty tree (hook empties the private index)" -ForegroundColor Cyan
    $r1 = New-TestRepo "s1"
    Write-Hook $r1 @"
# FIND-778: sabotage the private index, exactly the shape the incident produced
git read-tree --empty
exit 0
"@
    $treeBefore = "$(git -C $r1 rev-parse 'HEAD^{tree}')".Trim()
    $headBefore1 = "$(git -C $r1 rev-parse HEAD)".Trim()
    Set-Content -LiteralPath (Join-Path $r1 "docs/file0.txt") -Value "changed" -NoNewline

    $res = Invoke-Helper $r1 @("-Message", "test: empty tree", "-Paths", "docs/file0.txt")
    Check ($res.Code -ne 0) "exits non-zero (got $($res.Code))"
    Check ($res.Code -eq 4) "exit code is 4 = refused by result guards"
    Check ($res.Out -match "REFUSED") "output says REFUSED"
    Check ($res.Out -match "EMPTY tree") "output names the empty tree"
    Check ("$(git -C $r1 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD tree is NOT the empty tree"
    Check ("$(git -C $r1 rev-parse HEAD)".Trim() -eq $headBefore1) "HEAD restored to the previous SHA"
    Check (Get-WorktreeFileCount $r1 -eq 60) "worktree intact (60 files)"
    Check ((Get-LeftoverIndexes $r1).Count -eq 0) "no private index left behind"

    # ── Scenario 2: FAILURE path, HEAD moved anyway ────────────────────────
    # The hook creates the empty-tree commit itself and then exits 1: `git
    # commit` reports failure while HEAD has already moved. This is the exact
    # gap that made today's incident unrecoverable by exit code.
    Write-Host "`n[2] failure path (commit exists, git commit exit != 0)" -ForegroundColor Cyan
    $r2 = New-TestRepo "s2"
    Write-Hook $r2 @"
git read-tree --empty
bad=`$(git commit-tree "`$(git write-tree)" -p HEAD -m "test: empty tree via failure")
git update-ref HEAD "`$bad"
echo "`$bad" > "$($r2 -replace '\\','/')/.bad-sha"
exit 1
"@
    $headBefore2 = "$(git -C $r2 rev-parse HEAD)".Trim()
    Set-Content -LiteralPath (Join-Path $r2 "docs/file0.txt") -Value "changed" -NoNewline

    $res2 = Invoke-Helper $r2 @("-Message", "test: empty tree via failure", "-Paths", "docs/file0.txt")
    $badSha = (Get-Content (Join-Path $r2 ".bad-sha") -ErrorAction SilentlyContinue) -join ""
    $badSha = $badSha.Trim()
    Check ($res2.Code -eq 4) "exit code is 4 (got $($res2.Code))"
    Check ($badSha.Length -eq 40) "hook did create a commit ($badSha)"
    Check ("$(git -C $r2 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD tree is NOT the empty tree"
    Check ("$(git -C $r2 rev-parse HEAD)".Trim() -eq $headBefore2) "HEAD restored to the previous SHA"
    Check ($res2.Out -match [regex]::Escape($badSha)) "output names the refused commit SHA"
    if ($badSha.Length -eq 40) {
        & git -C $r2 cat-file -e "$badSha^{commit}" 2>$null
        Check ($LASTEXITCODE -eq 0) "refused commit object kept for the record"
    }
    Check ((Get-LeftoverIndexes $r2).Count -eq 0) "no private index left behind"

    # ── Scenario 3: pre-commit aborts -> nothing created, still cleaned ────
    Write-Host "`n[3] hook aborts before the commit (no commit created)" -ForegroundColor Cyan
    $r3 = New-TestRepo "s3"
    Write-Hook $r3 "exit 1"
    $headBefore3 = "$(git -C $r3 rev-parse HEAD)".Trim()
    Set-Content -LiteralPath (Join-Path $r3 "docs/file0.txt") -Value "changed" -NoNewline

    $res3 = Invoke-Helper $r3 @("-Message", "test: aborted", "-Paths", "docs/file0.txt")
    Check ($res3.Code -eq 2) "exit code is 2 = commit not created (got $($res3.Code))"
    Check ($res3.Out -match "pre-commit hook FAILED") "output names the failing gate (FIND-780)"
    Check ($res3.Out -notmatch "OK:") "never prints OK"
    Check ("$(git -C $r3 rev-parse HEAD)".Trim() -eq $headBefore3) "HEAD did not move"
    Check ((Get-LeftoverIndexes $r3).Count -eq 0) "no private index left behind"

    # ── Scenario 4: happy path still works ─────────────────────────────────
    Write-Host "`n[4] happy path (real commit, content verified)" -ForegroundColor Cyan
    $r4 = New-TestRepo "s4"
    Set-Content -LiteralPath (Join-Path $r4 "docs/file0.txt") -Value "MARKER-778 changed" -NoNewline
    $headBefore4 = "$(git -C $r4 rev-parse HEAD)".Trim()

    $res4 = Invoke-Helper $r4 @("-Message", "test: happy path", "-Paths", "docs/file0.txt", "-ExpectPresent", "docs/file0.txt::MARKER-778")
    Check ($res4.Code -eq 0) "exit code is 0 (got $($res4.Code))"
    Check ("$(git -C $r4 rev-parse HEAD)".Trim() -ne $headBefore4) "HEAD moved"
    Check ("$(git -C $r4 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD tree is not empty"
    Check ("$(git -C $r4 log -1 --pretty=%s)".Trim() -eq "test: happy path") "commit message correct"
    Check ((Get-WorktreeFileCount $r4) -eq 60) "worktree intact (60 files)"
    Check ((Get-LeftoverIndexes $r4).Count -eq 0) "no private index left behind"

    # ── Scenario 5: legitimate deletion is NOT blocked (pre-mortem a) ──────
    # A single `git rm` must still commit: the guards only fire on mass
    # deletions whose files are still on disk, on the empty tree, or on an
    # entry-count collapse.
    Write-Host "`n[5] a legitimate deletion still commits" -ForegroundColor Cyan
    $r5 = New-TestRepo "s5"
    Remove-Item -LiteralPath (Join-Path $r5 "docs/file5.txt") -Force
    git -C $r5 rm -q --cached -- "docs/file5.txt" | Out-Null
    $headBefore5 = "$(git -C $r5 rev-parse HEAD)".Trim()
    $res5 = Invoke-Helper $r5 @("-Message", "test: delete one file", "-Paths", "docs/file5.txt")
    Check ($res5.Code -eq 0) "exit code is 0 (got $($res5.Code))"
    Check ("$(git -C $r5 rev-parse HEAD)".Trim() -ne $headBefore5) "HEAD moved"
    Check ($res5.Out -match "refreshed") "shared index refreshed"
    # ── Scenario 6: another lane's VALID commit is never reverted, never "OK" ──
    # The hook commits a DIFFERENT file on its own private index and exits 1. HEAD
    # then holds a perfectly valid commit that is not ours: the script must not
    # restore it, must not print OK (P2-01 C1: pre-fix it printed OK and exit 0),
    # and must exit 6.
    Write-Host "`n[6] another lane's valid commit: refused, not reverted, no OK" -ForegroundColor Cyan
    $r6 = New-TestRepo "s6"
    Write-Hook $r6 @"
export GIT_INDEX_FILE="`$(git rev-parse --git-dir)/peer-index"
git read-tree HEAD
echo peer > docs/peer.txt
git add docs/peer.txt
peer_tree=`$(git write-tree)
bad=`$(git commit-tree "`$peer_tree" -p HEAD -m "another lane: unrelated work")
git update-ref HEAD "`$bad"
exit 1
"@
    Set-Content -LiteralPath (Join-Path $r6 "docs/file0.txt") -Value "changed" -NoNewline
    $headBefore6 = "$(git -C $r6 rev-parse HEAD)".Trim()
    $res6 = Invoke-Helper $r6 @("-Message", "test: someone else's commit", "-Paths", "docs/file0.txt")
    $headAfter6 = "$(git -C $r6 rev-parse HEAD)".Trim()
    Check ($res6.Code -eq 6) "exit code is 6 = nothing of yours, peer commit kept (got $($res6.Code))"
    Check ($res6.Out -match "ANOTHER lane") "output names the other lane"
    Check ($headAfter6 -ne $headBefore6) "the other lane's commit was NOT reverted"
    Check ($res6.Out -notmatch "OK:") "never prints OK"
    Check ($res6.Out -match [regex]::Escape($headAfter6)) "their commit SHA is reported"

    # ── Scenario 7: an EMPTY tree is restored even when it is not ours ──────
    # An empty tree deletes the whole repository: there is no work of anyone's to
    # preserve, so the ref is restored anyway (P2-01 H1).
    Write-Host "`n[7] another lane's EMPTY-tree commit: restored anyway" -ForegroundColor Cyan
    $r7 = New-TestRepo "s7"
    Write-Hook $r7 @"
git read-tree --empty
bad=`$(git commit-tree "`$(git write-tree)" -p HEAD -m "another lane: wipes the repo")
git update-ref HEAD "`$bad"
exit 1
"@
    Set-Content -LiteralPath (Join-Path $r7 "docs/file0.txt") -Value "changed" -NoNewline
    $headBefore7 = "$(git -C $r7 rev-parse HEAD)".Trim()
    $res7 = Invoke-Helper $r7 @("-Message", "test: someone else's empty tree", "-Paths", "docs/file0.txt")
    Check ($res7.Code -eq 4) "exit code is 4 = refused and restored (got $($res7.Code))"
    Check ("$(git -C $r7 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD is not the empty tree"
    Check ("$(git -C $r7 rev-parse HEAD)".Trim() -eq $headBefore7) "HEAD restored despite the commit not being ours"
    Check ($res7.Out -match "restored anyway") "output explains the empty-tree exception"
    Check ((Get-WorktreeFileCount $r7) -eq 60) "worktree intact (60 files)"

    # ── Scenario 8: the GATE still runs, and sees the paths being committed ──
    # FIND-780 replaced `git commit` (which ran the hook) with plumbing (which does
    # not). The hook is now invoked explicitly; this proves it ran AND that its
    # `git diff --staged` resolves to the PRIVATE index — the same file set git
    # would have handed it. Without this the fix would trade a data-loss bug for a
    # silent loss of every gate.
    Write-Host "`n[8] the pre-commit gate runs and sees the private index" -ForegroundColor Cyan
    $r8 = New-TestRepo "s8"
    $r8u = $r8 -replace '\\', '/'
    Write-Hook $r8 @"
git diff --cached --name-only > "$r8u/.hook-sees"
echo ran > "$r8u/.hookran"
exit 0
"@
    Set-Content -LiteralPath (Join-Path $r8 "docs/file0.txt") -Value "MARKER-780 changed" -NoNewline
    $headBefore8 = "$(git -C $r8 rev-parse HEAD)".Trim()

    $res8 = Invoke-Helper $r8 @("-Message", "test: gate ran", "-Paths", "docs/file0.txt", "-ExpectPresent", "docs/file0.txt::MARKER-780")
    Check ($res8.Code -eq 0) "exit code is 0 (got $($res8.Code))"
    Check (Test-Path (Join-Path $r8 ".hookran")) "the pre-commit hook EXECUTED (its own side effect, not our log line)"
    $hookSees = (Get-Content (Join-Path $r8 ".hook-sees") -ErrorAction SilentlyContinue) -join "`n"
    Check ($hookSees -match "docs/file0\.txt") "the hook's 'git diff --staged' resolved to the private index"
    Check ($hookSees -notmatch "file1\.txt") "the hook did not see paths outside the commit"
    Check ("$(git -C $r8 rev-parse HEAD)".Trim() -ne $headBefore8) "HEAD moved"
    Check ("$(git -C $r8 log -1 --pretty=%s)".Trim() -eq "test: gate ran") "commit message correct"
    Check ($res8.Out -match "pre-commit hook passed") "the helper reports the gate result"
    Check ((Get-LeftoverIndexes $r8).Count -eq 0) "no private index left behind"

    # ── Scenario 9: a peer commits WHILE the hooks run -> compare-and-swap ──
    # The hook passes (exit 0) but moves HEAD to another lane's commit. Our tree was
    # built on the OLD HEAD: attaching it would revert their work. update-ref's
    # expected-old makes that impossible instead of merely unlikely.
    Write-Host "`n[9] HEAD moves while the hooks run: CAS refuses, peer commit kept" -ForegroundColor Cyan
    $r9 = New-TestRepo "s9"
    $r9u = $r9 -replace '\\', '/'
    Write-Hook $r9 @"
export GIT_INDEX_FILE="`$(git rev-parse --git-dir)/peer-index"
git read-tree HEAD
echo peer > docs/peer.txt
git add docs/peer.txt
peer_tree=`$(git write-tree)
peer=`$(git commit-tree "`$peer_tree" -p HEAD -m "peer during hooks")
git update-ref HEAD "`$peer"
echo "`$peer" > "$r9u/.peer-sha"
exit 0
"@
    Set-Content -LiteralPath (Join-Path $r9 "docs/file0.txt") -Value "MARKER-780 mine" -NoNewline
    $headBefore9 = "$(git -C $r9 rev-parse HEAD)".Trim()

    $res9 = Invoke-Helper $r9 @("-Message", "test: should not land", "-Paths", "docs/file0.txt")
    $peerSha = ((Get-Content (Join-Path $r9 ".peer-sha") -ErrorAction SilentlyContinue) -join "").Trim()
    $headAfter9 = "$(git -C $r9 rev-parse HEAD)".Trim()
    Check ($res9.Code -eq 6) "exit code is 6 = nothing of yours, HEAD not clobbered (got $($res9.Code))"
    Check ($peerSha.Length -eq 40) "the peer commit exists ($peerSha)"
    Check ($headAfter9 -eq $peerSha) "the peer's commit is STILL on HEAD (never reverted)"
    Check ($res9.Out -match "another lane moved HEAD") "output says another lane moved HEAD"
    Check ($res9.Out -notmatch "OK:") "never prints OK"
    Check ("$(git -C $r9 log -1 --pretty=%s)".Trim() -eq "peer during hooks") "the message on HEAD is the peer's"
    Check (((Get-Content (Join-Path $r9 "docs/file0.txt") -Raw) -match "MARKER-780 mine")) "our edit is still in the worktree, uncommitted"
    Check ((Get-LeftoverIndexes $r9).Count -eq 0) "no private index left behind"

    # ── Scenario 10: KILL the process that commits (FIND-780) ───────────────
    # The incident, reproduced: the hook empties the private index (the payload),
    # signals, and sleeps. The test kills the HELPER — the process that commits —
    # with Stop-Process -Force while the commit is in flight. Pre-FIND-780 the
    # `git commit` CHILD survived the kill, read the emptied index and committed
    # the canonical empty tree (measured: 60/60 paths deleted, HEAD moved). With
    # `commit-tree` + `update-ref` the only step that moves HEAD is executed by the
    # killed process itself, and it never got there.
    Write-Host "`n[10] kill the committing process mid-flight: HEAD must not move" -ForegroundColor Cyan
    $r10 = New-TestRepo "s10"
    $r10u = $r10 -replace '\\', '/'
    Write-Hook $r10 @"
git read-tree --empty
echo ready > "$r10u/.sentinel"
sleep 3
exit 0
"@
    Set-Content -LiteralPath (Join-Path $r10 "docs/file0.txt") -Value "MARKER-780 killed" -NoNewline
    $headBefore10 = "$(git -C $r10 rev-parse HEAD)".Trim()
    $treeBefore10 = "$(git -C $r10 rev-parse 'HEAD^{tree}')".Trim()
    $countBefore10 = [int](git -C $r10 rev-list --count HEAD)
    $sentinel10 = Join-Path $r10 ".sentinel"
    $log10 = Join-Path $r10 ".helper.out"

    $savedTemp = $env:TEMP
    $savedTmp = $env:TMP
    $proc = $null
    try {
        $env:TEMP = Join-Path $r10 "tmp"
        $env:TMP = $env:TEMP
        New-Item -ItemType Directory -Force $env:TEMP | Out-Null
        # Start-Process joins -ArgumentList with spaces and does NOT quote: the
        # script path contains "VantaDB Proyect", so it must be quoted here or pwsh
        # prints its usage help and no commit is ever attempted.
        $proc = Start-Process pwsh -PassThru -NoNewWindow -WorkingDirectory $r10 `
            -ArgumentList @("-NoProfile", "-File", "`"$SCRIPT`"", "-Message", "killtest", "-Paths", "docs/file0.txt") `
            -RedirectStandardOutput $log10 -RedirectStandardError ($log10 + ".err")
    } finally {
        $env:TEMP = $savedTemp
        $env:TMP = $savedTmp
    }

    $deadline = (Get-Date).AddSeconds(90)
    while (-not (Test-Path $sentinel10) -and (Get-Date) -lt $deadline -and -not $proc.HasExited) {
        Start-Sleep -Milliseconds 100
    }
    Check (Test-Path $sentinel10) "the commit reached the hook (kill lands mid-flight)"
    if (Test-Path $sentinel10) {
        Stop-Process -Id $proc.Id -Force
        # A killed parent leaves its native children alive on Windows. Wait long
        # enough that a surviving `git commit` would have finished its work, so
        # "nothing was created" is MEASURED, not assumed.
        Start-Sleep -Seconds 8
    }
    Get-Content $log10 -ErrorAction SilentlyContinue | ForEach-Object { Write-Host "    | $_" }

    $headAfter10 = "$(git -C $r10 rev-parse HEAD)".Trim()
    $treeAfter10 = "$(git -C $r10 rev-parse 'HEAD^{tree}')".Trim()
    Check ($headAfter10 -eq $headBefore10) "HEAD did not move (kill left the previous SHA: $($headBefore10.Substring(0,8)))"
    Check ($treeAfter10 -ne $EMPTY_TREE) "HEAD tree is NOT the empty tree (4b825dc...)"
    Check ($treeAfter10 -eq $treeBefore10) "HEAD tree is unchanged"
    Check ([int](git -C $r10 rev-list --count HEAD) -eq $countBefore10) "no commit was created at all"
    Check ((@(git -C $r10 ls-tree -r --name-only HEAD)).Count -eq 60) "HEAD still lists 60 paths"
    Check ((Get-WorktreeFileCount $r10) -eq 60) "worktree intact (60 files)"

    # ── Scenario 11: the helper must not contain a `git commit` invocation ──
    # Scenario 10 kills while the hook runs, which is BEFORE any `git commit` child
    # could exist. A hybrid regression — explicit `git hook run` back in place AND
    # `& git commit` restored — would keep passing it. This is the cheap static
    # guard that closes that hole (P2-01 LOW-1).
    Write-Host "`n[11] the helper commits with plumbing only (no `git commit` call)" -ForegroundColor Cyan
    $src = Get-Content -LiteralPath $SCRIPT -Raw
    Check ($src -match '&\s*git\s+commit-tree\s') "the commit is created with 'git commit-tree'"
    Check ($src -match '&\s*git\s+update-ref\s') "HEAD is moved with 'git update-ref' (compare-and-swap)"
    Check ($src -notmatch '&\s*git\s+commit\s') "no invocation of 'git commit' (the kill window cannot come back)"

    # ── Scenarios 12/13: a hook that commits the EMPTY tree and PASSES the gate ──
    # P2-01 HIGH-1: an `exit` inside the try would terminate the script before the
    # FIND-778 restore policy, leaving the hook's empty-tree commit on HEAD while the
    # message claimed "HEAD did not move". The refusal must go THROUGH the block.
    Write-Host "`n[12] hook commits the empty tree and exits 0 (index intact): restored" -ForegroundColor Cyan
    $r12 = New-TestRepo "s12"
    Write-Hook $r12 @"
old=`$(git write-tree)
git read-tree --empty
bad=`$(git commit-tree "`$(git write-tree)" -p HEAD -m "hook: empty tree, index intact")
git update-ref HEAD "`$bad"
git read-tree "`$old"
exit 0
"@
    Set-Content -LiteralPath (Join-Path $r12 "docs/file0.txt") -Value "changed" -NoNewline
    $headBefore12 = "$(git -C $r12 rev-parse HEAD)".Trim()
    $res12 = Invoke-Helper $r12 @("-Message", "test: hook empty tree", "-Paths", "docs/file0.txt")
    Check ($res12.Code -eq 4) "exit code is 4 (got $($res12.Code))"
    Check ("$(git -C $r12 rev-parse HEAD)".Trim() -eq $headBefore12) "HEAD restored to the previous SHA"
    Check ("$(git -C $r12 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD is NOT the empty tree"
    Check ($res12.Out -match "restored") "output says it restored"
    Check ((Get-LeftoverIndexes $r12).Count -eq 0) "no private index left behind"

    Write-Host "`n[13] hook empties the index AND commits the empty tree: restored" -ForegroundColor Cyan
    $r13 = New-TestRepo "s13"
    Write-Hook $r13 @"
git read-tree --empty
bad=`$(git commit-tree "`$(git write-tree)" -p HEAD -m "hook: empty tree, index emptied")
git update-ref HEAD "`$bad"
exit 0
"@
    Set-Content -LiteralPath (Join-Path $r13 "docs/file0.txt") -Value "changed" -NoNewline
    $headBefore13 = "$(git -C $r13 rev-parse HEAD)".Trim()
    $res13 = Invoke-Helper $r13 @("-Message", "test: hook empty tree and index", "-Paths", "docs/file0.txt")
    Check ($res13.Code -eq 4) "exit code is 4 (got $($res13.Code))"
    Check ("$(git -C $r13 rev-parse HEAD)".Trim() -eq $headBefore13) "HEAD restored to the previous SHA"
    Check ("$(git -C $r13 rev-parse 'HEAD^{tree}')".Trim() -ne $EMPTY_TREE) "HEAD is NOT the empty tree"
    Check ($res13.Out -match "REFUSED") "output says REFUSED"
    Check ((Get-WorktreeFileCount $r13) -eq 60) "worktree intact (60 files)"

    # ── Scenario 14: the hooks that do NOT run are named (P2-01 M2) ─────────
    Write-Host "`n[14] skipped commit-time hooks are named in a warning" -ForegroundColor Cyan
    $r14 = New-TestRepo "s14"
    $hooks14 = Join-Path $r14 ".githooks-test"
    New-Item -ItemType Directory -Force $hooks14 | Out-Null
    foreach ($h in @("pre-commit", "commit-msg")) {
        [IO.File]::WriteAllText((Join-Path $hooks14 $h), "#!/bin/sh`nexit 0`n")
    }
    git -C $r14 config core.hooksPath ".githooks-test"
    Set-Content -LiteralPath (Join-Path $r14 "docs/file0.txt") -Value "MARKER-780 hook" -NoNewline
    $res14 = Invoke-Helper $r14 @("-Message", "test: skipped hook warning", "-Paths", "docs/file0.txt")
    Check ($res14.Code -eq 0) "exit code is 0 (got $($res14.Code))"
    Check ($res14.Out -match "does NOT run: commit-msg") "the warning NAMES the hook git would have run"
    Check ($res14.Out -match "pre-commit hook passed") "the gate that does exist still ran"

    # ── Scenario 15: the ref cannot be locked -> exit 2, nothing created ────
    # `update-ref` failing with HEAD unchanged used to be reported as "another lane
    # committed"; the helper now re-reads HEAD and names the real cause.
    Write-Host "`n[15] a held ref lock: exit 2, nothing created, honest message" -ForegroundColor Cyan
    $r15 = New-TestRepo "s15"
    Set-Content -LiteralPath (Join-Path $r15 "docs/file0.txt") -Value "changed" -NoNewline
    $headBefore15 = "$(git -C $r15 rev-parse HEAD)".Trim()
    $branch15 = "$(git -C $r15 symbolic-ref --short HEAD)".Trim()
    $lock15 = Join-Path $r15 ".git" "refs" "heads" "$branch15.lock"
    New-Item -ItemType File -Force -Path $lock15 | Out-Null
    $res15 = Invoke-Helper $r15 @("-Message", "test: locked ref", "-Paths", "docs/file0.txt")
    Remove-Item -LiteralPath $lock15 -Force -ErrorAction SilentlyContinue
    Check ($res15.Code -eq 2) "exit code is 2 = nothing created (got $($res15.Code))"
    Check ("$(git -C $r15 rev-parse HEAD)".Trim() -eq $headBefore15) "HEAD did not move"
    Check ($res15.Out -match "could not lock HEAD") "the message names the LOCK, not a phantom peer"
    Check ((Get-LeftoverIndexes $r15).Count -eq 0) "no private index left behind"

    $utf8 = New-Object Text.UTF8Encoding($false)

    # ── Scenario 16: a comma in the CONTENT is text, not a separator (FIND-795) ──
    # The measured case: `-ExpectAbsent '...verbatim, 1428 chars'` was split, the
    # fragment `verbatim` existed elsewhere and the check either passed by
    # measuring another line or reported BAD EXPECTATION. A separator that can
    # appear in the content cannot be the list separator: the array form is used
    # here, and one element is never split.
    Write-Host "`n[16] a comma inside the expected text is TEXT (FIND-795)" -ForegroundColor Cyan
    $r16 = New-TestRepo "s16"
    [IO.File]::WriteAllText((Join-Path $r16 "docs/file0.txt"),
        "verificacion, medida con bytes`nsolo verificacion aqui`n", $utf8)
    $call16 = @'
$specs = @("docs/file0.txt::verificacion, medida con bytes")
& $helper -Message "test: comma in text" -Paths "docs/file0.txt" -ExpectPresent $specs
'@
    $res16 = Invoke-HelperScripted $r16 $call16
    Check ($res16.Code -eq 0) "exit 0: the WHOLE phrase is verified (got $($res16.Code))"
    Check ($res16.Out -match "ok present") "reports the declared phrase as present"
    Check ($res16.Out -notmatch "BAD EXPECTATION") "never splits it into a fragment"
    $blob16 = "$(git -C $r16 rev-parse 'HEAD:docs/file0.txt')".Trim()
    $disk16 = "$(git -C $r16 hash-object -- 'docs/file0.txt')".Trim()
    Check ($blob16 -eq $disk16) "the committed blob is the worktree's bytes (identity, not text)"

    # The dangerous half: a comma-joined argument that is NOT a spec list must be
    # verified as ONE literal — the old split matched `verificacion` and printed
    # `ok present` for a phrase it never checked.
    Write-Host "`n[16b] a broken list is NOT verified by one of its fragments" -ForegroundColor Cyan
    $r16b = New-TestRepo "s16b"
    [IO.File]::WriteAllText((Join-Path $r16b "docs/file0.txt"),
        "verificacion, medida con bytes`nsolo verificacion aqui`n", $utf8)
    $res16b = Invoke-Helper $r16b @("-Message", "test: comma literal", "-Paths", "docs/file0.txt",
        "-ExpectPresent", "docs/file0.txt::verificacion, medida con bytes XX")
    Check ($res16b.Code -eq 1) "exit 1 = content failure (got $($res16b.Code))"
    Check ($res16b.Out -notmatch "ok present") "never passed by matching the fragment 'verificacion'"
    Check ($res16b.Out -match "ONE literal") "says it treated the comma argument as one literal"

    # ── Scenario 17: the accent lives in the BLOB, not in the console (FIND-793) ──
    # (a) the file holds NFC `después` and the expectation arrives in NFD: the text
    # is the same, so the check must pass. Old: ordinal compare on console text,
    # exit 1 with the right commit on HEAD.
    Write-Host "`n[17] accents: NFC/NFD matches, mojibake is ENCODING, not content (FIND-793)" -ForegroundColor Cyan
    $r17 = New-TestRepo "s17"
    $e17 = [char]0x00E9
    [IO.File]::WriteAllText((Join-Path $r17 "docs/file0.txt"), "medido: 60/60 despu$($e17)s del fix`n", $utf8)
    $call17 = @'
$nfd = "despue" + [char]0x0301 + "s"
& $helper -Message "test: nfd expectation" -Paths "docs/file0.txt" -ExpectPresent "docs/file0.txt::$nfd"
'@
    $res17 = Invoke-HelperScripted $r17 $call17
    Check ($res17.Code -eq 0) "exit 0: an NFD expectation matches the NFC blob (got $($res17.Code))"
    Check ($res17.Out -match "ok present") "reports it present"

    # (b) the caller's bytes were double-encoded (`despuÃ©s`): the blob is fine and
    # the class must say ENCODING — the old exit 1 invited a duplicate commit.
    [IO.File]::WriteAllText((Join-Path $r17 "docs/file0.txt"), "medido: 60/60 despu$($e17)s del fix (r2)`n", $utf8)
    $call17b = @'
$bad = "despu" + [char]0x00C3 + [char]0x00A9 + "s"
& $helper -Message "test: mojibake expectation" -Paths "docs/file0.txt" -ExpectPresent "docs/file0.txt::$bad"
'@
    $res17b = Invoke-HelperScripted $r17 $call17b
    Check ($res17b.Code -eq 8) "exit 8 = ENCODING class, distinct from content (got $($res17b.Code))"
    Check ($res17b.Out -match "ENCODING") "names the encoding class"
    Check ($res17b.Out -match "do NOT re-run") "tells the caller the commit exists and must not be duplicated"
    Check ($res17b.Out -notmatch "NOT COMMITTED") "does NOT report missing content"

    # (c) the control: a truly absent fragment is still a CONTENT failure (exit 1),
    # so the encoding class did not swallow the real diagnosis.
    [IO.File]::WriteAllText((Join-Path $r17 "docs/file0.txt"), "medido: 60/60 despu$($e17)s del fix (r3)`n", $utf8)
    $res17c = Invoke-Helper $r17 @("-Message", "test: truly absent", "-Paths", "docs/file0.txt",
        "-ExpectPresent", "docs/file0.txt::texto que no existe")
    Check ($res17c.Code -eq 1) "exit 1 = content failure (got $($res17c.Code))"
    Check ($res17c.Out -match "NOT COMMITTED") "says the content is not in the blob"

    # (d) the diacritic-free projection: a fragment whose accents differ still
    # names the SITE (the line must be real: an index into the projection is not
    # an index into the text).
    Write-Host "`n[17d] accents differ: ENCODING names the real line" -ForegroundColor Cyan
    $r17d = New-TestRepo "s17d"
    [IO.File]::WriteAllText((Join-Path $r17d "docs/file0.txt"),
        ("linea uno`nmedido: canci" + [char]0x00F3 + "n completada`n"), $utf8)
    $call17d = @'
$odd = "can" + [char]0x00E7 + "i" + [char]0x00F3 + "n"
& $helper -Message "t: skeleton" -Paths "docs/file0.txt" -ExpectPresent "docs/file0.txt::$odd"
'@
    $res17d = Invoke-HelperScripted $r17d $call17d
    Check ($res17d.Code -eq 8) "exit 8 = ENCODING (got $($res17d.Code))"
    Check ($res17d.Out -match "docs/file0\.txt:2") "names the line where the projection matched (2, not an offset)"

    # ── Scenario 18: three states, and no invented cause (FIND-796) ────────────
    # `REVERTED: the commit did not carry your edit` was a cause the helper never
    # measured; in the measured case the commit DID carry the edit. A fragment
    # matching two sites must now say AMBIGUOUS, name both `file:line`, and exit 7.
    Write-Host "`n[18] a fragment matching twice is AMBIGUOUS, with both sites (FIND-796)" -ForegroundColor Cyan
    $r18 = New-TestRepo "s18"
    [IO.File]::WriteAllText((Join-Path $r18 "docs/file0.txt"),
        "status: completed`nprose explaining the change`nstatus: completed again in the notes`n", $utf8)
    $res18 = Invoke-Helper $r18 @("-Message", "test: ambiguous", "-Paths", "docs/file0.txt",
        "-ExpectPresent", "docs/file0.txt::status: completed")
    Check ($res18.Code -eq 7) "exit 7 = AMBIGUOUS (got $($res18.Code))"
    Check ($res18.Out -match "AMBIGUOUS EXPECTATION") "names the class"
    Check ($res18.Out -match "docs/file0\.txt:1") "names the first site (file:line)"
    Check ($res18.Out -match "docs/file0\.txt:3") "names the second site (file:line)"
    Check ($res18.Out -notmatch "did not carry your edit") "the unmeasured cause is gone"
    Check ($res18.Out -match "commit exists") "says the commit exists and is not the problem"

    # ── Scenario 19: a foreign Backlog row aborts BEFORE the commit (FIND-792) ──
    # Measured three times (`6e4119c8d`, `d15564b06`, `a476e190c`): a `-Paths
    # docs/dev/Backlog.md` commit carried ANOTHER lane's row retirement, whose code
    # was still uncommitted. The helper's own expectation passed because it checked
    # its own row.
    Write-Host "`n[19] a commit that retires another lane's Backlog row aborts (FIND-792)" -ForegroundColor Cyan
    $r19 = New-TestRepo "s19"
    New-Item -ItemType Directory -Force (Join-Path $r19 "docs/dev") | Out-Null
    $bl19 = @"
| ID | Prioridad |
|---|---|
| ``FIND-101`` | alta |
| ``FIND-102`` | alta |
"@
    [IO.File]::WriteAllText((Join-Path $r19 "docs/dev/Backlog.md"), $bl19, $utf8)
    git -C $r19 add -A | Out-Null
    git -C $r19 commit -qm "backlog seed"
    # FIND-102 is another lane's: its code is NOT in this commit, its row is gone
    # from the working copy because that lane has not committed yet.
    [IO.File]::WriteAllText((Join-Path $r19 "docs/dev/Backlog.md"), "| ID | Prioridad |`n|---|---|`n", $utf8)
    $headBefore19 = "$(git -C $r19 rev-parse HEAD)".Trim()
    $countBefore19 = [int](git -C $r19 rev-list --count HEAD)
    $call19 = @'
$specs = @('docs/dev/Backlog.md::| `FIND-101` |')
& $helper -Message "test: foreign row" -Paths "docs/dev/Backlog.md" -ExpectAbsent $specs
'@
    $res19 = Invoke-HelperScripted $r19 $call19
    Check ($res19.Code -eq 9) "exit 9 = FOREIGN HUNK (got $($res19.Code))"
    Check ($res19.Out -match "FOREIGN HUNK") "names the class"
    Check ($res19.Out -match "FIND-102") "names the foreign row"
    Check ("$(git -C $r19 rev-parse HEAD)".Trim() -eq $headBefore19) "HEAD did not move"
    Check ([int](git -C $r19 rev-list --count HEAD) -eq $countBefore19) "no commit was created"
    $blInHead = ((& git -C $r19 cat-file -p 'HEAD:docs/dev/Backlog.md') -join "`n")
    Check ($blInHead -match "FIND-102") "the other lane's row is still in HEAD"

    # Precision: the same shape with ONLY its own row removed still commits. The
    # guard must not block normal Backlog work (pre-mortem d).
    Write-Host "`n[19b] a commit that retires only its own row still commits" -ForegroundColor Cyan
    $r19b = New-TestRepo "s19b"
    New-Item -ItemType Directory -Force (Join-Path $r19b "docs/dev") | Out-Null
    [IO.File]::WriteAllText((Join-Path $r19b "docs/dev/Backlog.md"), $bl19, $utf8)
    git -C $r19b add -A | Out-Null
    git -C $r19b commit -qm "backlog seed"
    $onlyOwn = @"
| ID | Prioridad |
|---|---|
| ``FIND-102`` | alta |
"@
    [IO.File]::WriteAllText((Join-Path $r19b "docs/dev/Backlog.md"), $onlyOwn, $utf8)
    $headBefore19b = "$(git -C $r19b rev-parse HEAD)".Trim()
    $res19b = Invoke-Helper $r19b @("-Message", "test: own row", "-Paths", "docs/dev/Backlog.md",
        "-ExpectAbsent", 'docs/dev/Backlog.md::| `FIND-101` |')
    Check ($res19b.Code -eq 0) "exit 0 (got $($res19b.Code))"
    Check ("$(git -C $r19b rev-parse HEAD)".Trim() -ne $headBefore19b) "HEAD moved"
    Check ($res19b.Out -match "ok absent") "its own retirement is verified"

    # ── Scenario 20: identity by blob oid, not by text (FIND-793/792) ──────────
    # `git rev-parse <commit>:<path>` == the declared oid is the form that cannot
    # be corrupted by a transport: it compares object identity, not characters.
    Write-Host "`n[20] verification by blob identity (-ExpectBlob)" -ForegroundColor Cyan
    $r20 = New-TestRepo "s20"
    [IO.File]::WriteAllText((Join-Path $r20 "docs/file0.txt"), "identity marker`n", $utf8)
    $oid20 = "$(git -C $r20 hash-object -- 'docs/file0.txt')".Trim()
    $res20 = Invoke-Helper $r20 @("-Message", "test: blob identity", "-Paths", "docs/file0.txt",
        "-ExpectBlob", "docs/file0.txt::$oid20")
    Check ($res20.Code -eq 0) "exit 0 when the committed blob is the declared oid (got $($res20.Code))"
    Check ($res20.Out -match "ok blob") "reports the blob identity"
    Check ("$(git -C $r20 rev-parse 'HEAD:docs/file0.txt')".Trim() -eq $oid20) "HEAD carries exactly that blob"

    $head20 = "$(git -C $r20 rev-parse HEAD)".Trim()
    [IO.File]::WriteAllText((Join-Path $r20 "docs/file0.txt"), "identity marker, changed`n", $utf8)
    $res20b = Invoke-Helper $r20 @("-Message", "test: blob mismatch", "-Paths", "docs/file0.txt",
        "-ExpectBlob", "docs/file0.txt::$oid20")
    Check ($res20b.Code -eq 1) "exit 1 on a blob that is not the declared one (got $($res20b.Code))"
    Check ($res20b.Out -match "BLOB MISMATCH") "names the mismatch with both oids"
    Check ("$(git -C $r20 rev-parse 'HEAD:docs/file0.txt')".Trim() -ne $oid20) "the commit carries the new bytes"
}
finally {
    Remove-Item -LiteralPath $ROOT -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
if ($fail -gt 0) { Write-Host "FAILED: $fail assertion(s)." -ForegroundColor Red; exit 1 }
Write-Host "OK: git-commit-isolated guards hold." -ForegroundColor Green
exit 0
