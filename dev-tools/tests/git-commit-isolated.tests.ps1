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
# Scenario 10 FAILS against the pre-FIND-780 helper. Reproduce that on purpose:
#   $env:FIND780_HELPER = '<copy of the old script>'; pwsh -File <this file>
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
}
finally {
    Remove-Item -LiteralPath $ROOT -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
if ($fail -gt 0) { Write-Host "FAILED: $fail assertion(s)." -ForegroundColor Red; exit 1 }
Write-Host "OK: git-commit-isolated guards hold." -ForegroundColor Green
exit 0
