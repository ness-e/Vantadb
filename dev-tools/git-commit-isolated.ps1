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

  FIND-780 — THE COMMIT IS CREATED BY PLUMBING, NOT BY `git commit`. The three
  empty-tree commits (`72643d9df`, `290df23d0`, `5766654ee`) were made by
  `git commit` reading a private index that THIS SCRIPT's own `finally` had just
  deleted: a caller piped the helper into `Select-Object -First N`, the pipeline
  stop unwound the script, the index was removed while the `git commit` CHILD was
  still running, and git committed the canonical empty tree — with the guards
  never reached, because the process that runs them was already gone. The guards
  were not the wrong idea; they were in the wrong place. Now:
    1. the gate runs FIRST and EXPLICITLY: `git hook run pre-commit` with
       GIT_INDEX_FILE already on the private index, i.e. the same script, the
       same arguments and the same staged list git itself would have used. A
       non-zero hook aborts before any commit object exists;
    2. the tree is written from the private index AFTER the hooks, as git does
       (a hook may legitimately rewrite the index);
    3. the commit is `git commit-tree <tree> -p HEAD` — it reads OBJECTS, never
       the index — followed by a compare-and-swap `git update-ref HEAD <new> <old>`.
  A kill at any point therefore leaves HEAD at the old SHA or at the finished
  commit: after write-tree no step reads the index, and update-ref is atomic. The
  empty tree is not detected and undone, it is never created. Cost, stated: hooks
  that `git commit` would run implicitly do not run — `pre-commit` is run here
  explicitly, and `prepare-commit-msg`, `commit-msg` and `post-commit` are NOT
  (the script warns if any of them exists). `commit-tree` also does not write
  `.git/COMMIT_EDITMSG`, does not honour `commit.gpgsign`, does not clean `-m`
  messages the way `git commit` does, and writes the reflog entry with the raw
  subject instead of git's "commit: " prefix — none of which this repo relies on.

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

  FIND-792/793/795/796 — VERIFICATION BY BLOB IDENTITY, NOT BY CONSOLE TEXT.
  Five commits and five false reports on 2026-11-26 made one thing clear: every
  remaining defect lived in how the check READ and PARSED its expectations, not
  in what it proved.

    1. EXPECTATIONS ARE READ FROM THE BLOB (FIND-793). `git show HEAD:<path>`
       piped into PowerShell decodes through the console codepage, so a UTF-8
       accent can arrive as mojibake and the check fails AFTER the commit exists:
       `NOT COMMITTED: '60/60 despu\uFFFDs' is missing` with the right commit on
       HEAD. The bytes now travel git -> file -> .NET UTF-8 (`git cat-file blob
       <oid>` with an OS-level redirect), and both sides are NFC-normalised. A
       mismatch that disappears once the non-ASCII characters are folded is
       reported as an ENCODING failure (exit 8), never as missing content: the
       commit exists, and re-running it would duplicate it.
    2. NO AMBIGUOUS SEPARATOR (FIND-795). `-ExpectPresent @('a','b')` is the
       canonical form and never splits an element, so a comma inside the text is
       just text. The legacy comma-joined form (`'a::x,b::y'`) still works — for
       every lane that already uses it — but it is split ONLY when every part
       has the `<path>::` shape. Otherwise the whole argument is verified as ONE
       literal and a NOTE says so: the old code split it and could pass by
       matching a fragment that exists somewhere else.
    3. THREE DIAGNOSED STATES (FIND-796). (i) absent from the commit's blob ->
       content failure (exit 1); (ii) present but not attributable to this commit
       — it matches more than one site, or the matched line is byte-identical to
       the parent — `AMBIGUOUS EXPECTATION` with `file:line` of every site and
       exit 7; (iii) the commit or the path is not there -> says exactly that.
       The old text `REVERTED: the commit did not carry your edit` was a cause the
       script never measured, and in the measured case it was false (`git show
       <commit>:<path>` had the content, tree intact). It is gone: every claim
       printed below names the measurement behind it.
    4. FOREIGN HUNKS (FIND-792). A `-Paths docs/dev/Backlog.md` commit used to
       carry ANOTHER lane's row retirement, whose code was still uncommitted, so
       the backlog asserted a closure that is not in history (measured three
       times: `6e4119c8d`, `d15564b06`, `a476e190c`). Before creating the commit
       the helper now diffs `docs/dev/Backlog.md` between HEAD and the staged
       tree, counts removed rows (`| `ID` |`) and refuses (exit 9, nothing
       created) when a removed row's ID is not named by an expectation. The check
       is deliberately exact about Backlog rows and silent about every other
       file: refusing on any diff would block the repository.

.PARAMETER ExpectAbsent
  `<path>::<literal>` that must NOT appear in the committed blob of `<path>`.
  Repeatable. Accepts the array form (`@('a::x','b::y')`), in which an element
  is NEVER split, so the literal may contain commas. A single comma-joined
  argument is split only if every part has the `<path>::` shape.

.PARAMETER ExpectPresent
  `<path>::<literal>` that MUST appear in the committed blob of `<path>`.
  Repeatable, same rules.

.PARAMETER ExpectBlob
  `<path>::<oid>` where `<oid>` is the blob the commit must contain for that
  path — the identity form: `git rev-parse <commit>:<path>` must equal `<oid>`.
  Usable as `-ExpectBlob "docs/x.md::$((git hash-object docs/x.md))"`, which
  asserts the committed bytes are the worktree bytes. Repeatable.
  On `docs/dev/Backlog.md` a blob oid pins the whole file but declares NO row:
  hashing the working copy carries whatever another lane wrote into it, so the
  find-792 guard still needs the rows named by `-ExpectAbsent` (or
  `-AllowForeignRows` for a deliberate multi-row cleanup).

.PARAMETER AllowForeignRows
  Opt-in escape for FIND-792: allow the commit to remove Backlog rows whose ID
  is not named by an expectation (e.g. a deliberate multi-row cleanup). Without
  it the helper aborts before creating the commit.

.EXAMPLE
  $out = & ./dev-tools/git-commit-isolated.ps1 -Message "fix(ci): pre-push judges the range" `
      -Paths ".githooks/pre-push,dev-tools/verify.ps1,docs/dev/Backlog.md" `
      -ExpectAbsent 'docs/dev/Backlog.md::| `FIND-760` |' `
      -ExpectPresent @('.githooks/pre-push::skip_gate','dev-tools/verify.ps1::verificación, medida con bytes') `
      2>&1
  $out | Select-String -Pattern 'OK:|FAILED:|ERROR:'

.NOTES
  Exit codes: 0 ok · 1 content expectation unmet (measured: the text is not in
  the commit's blob, or it is there and this commit wrote that line) · 2 commit
  not created (malformed expectation, a hook refusal, or an I/O failure) · 3
  commit ok but the SHARED index refresh failed · 4 commit
  REFUSED by the FIND-778 result guards (nothing was created, or HEAD was
  restored) · 5 restore itself failed · 6 REFUSED but HEAD is another lane's
  commit — or HEAD moved while the hooks ran (kept, never clobbered) ·
  7 AMBIGUOUS expectation: the fragment is in the blob but not attributable to
  this commit (multiple sites, or the matched line is unchanged from the parent).
  The commit IS created and is NOT wrong — do not retry it · 8 EXPECTATION
  UNTRUSTWORTHY: it differs from the blob only in non-ASCII bytes (encoding
  corrupted in transport) or it is malformed. The commit IS created — do not
  retry it · 9 FOREIGN HUNK: a shared file carries an undeclared row removal
  (`docs/dev/Backlog.md`); nothing was created.
#>
param(
    [Parameter(Mandatory = $true)][string]$Message,
    [Parameter(Mandatory = $true)][string[]]$Paths,
    [string[]]$ExpectAbsent = @(),
    [string[]]$ExpectPresent = @(),
    [string[]]$ExpectBlob = @(),
    [switch]$AllowForeignRows
)

# `pwsh -File script.ps1 -Paths a,b,c` cannot bind an array (it expands values
# into positional args), so accept the comma-joined form too — same as
# verify.ps1 -Skip. Repeatable flags work as well.
$Paths = @($Paths | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })

# FIND-795: expectations are NOT split blindly. `@('a','b')` is one spec per
# element and an element is never split, so a comma inside the text is text. The
# legacy comma-joined argument is still accepted for the lanes that use it, but
# only when EVERY part has the `<path>::` shape; otherwise the whole argument is
# one literal and a NOTE says so (the old blind split verified a fragment of the
# declared phrase — a check that passed by measuring something else).
function Resolve-ExpectationList {
    param([string[]]$Raw, [string]$Name)
    $out = New-Object System.Collections.Generic.List[string]
    foreach ($item in @($Raw)) {
        if (-not $item) { continue }
        if ($item -notmatch ',') { $out.Add($item); continue }
        $parts = @($item -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
        $allSpecShaped = ($parts.Count -gt 1)
        foreach ($p in $parts) { if ($p -notmatch '^\S+::') { $allSpecShaped = $false; break } }
        if ($allSpecShaped) {
            foreach ($p in $parts) { $out.Add($p) }
        } else {
            Write-Host "NOTE: $Name '$item' contains a comma but is not a list of '<path>::<text>' specs — treating it as ONE literal. Use @('a','b') to pass a list." -ForegroundColor Yellow
            $out.Add($item)
        }
    }
    return ,$out.ToArray()
}

$ExpectAbsent = Resolve-ExpectationList $ExpectAbsent "ExpectAbsent"
$ExpectPresent = Resolve-ExpectationList $ExpectPresent "ExpectPresent"
$ExpectBlob = @($ExpectBlob | ForEach-Object { $_ } | Where-Object { $_ })

# A spec that cannot be judged is rejected BEFORE any git work: a commit created
# with an unverifiable expectation is the FIND-793/796 shape (exit 1 with the
# right commit on HEAD). Exit 2 = nothing was created, so retrying with fixed
# arguments is safe — which is exactly what exit 8 (below) does NOT mean.
foreach ($s in (@($ExpectAbsent) + @($ExpectPresent))) {
    if ($s -notmatch '^[^:]+::.+$') {
        Write-Host "ERROR: malformed expectation (needs '<path>::<text>'): $s" -ForegroundColor Red
        Write-Host "  nothing was created; fix the argument and re-run." -ForegroundColor Yellow
        exit 2
    }
}
foreach ($s in $ExpectBlob) {
    if ($s -notmatch '^[^:]+::[0-9a-fA-F]{7,40}$') {
        Write-Host "ERROR: malformed ExpectBlob (needs '<path>::<hex blob oid>'): $s" -ForegroundColor Red
        Write-Host "  nothing was created; fix the argument and re-run." -ForegroundColor Yellow
        exit 2
    }
}


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

$treeSeeded = $null      # tree of the private index BEFORE the hooks
$treePlanned = $null      # tree this commit must carry (the one we create)
$preCommitError = $null   # failure before a commit object exists
# Set when we refuse BEFORE creating a commit. Never `exit` from inside the try
# to report it: `exit` terminates the script, so the FIND-778 restore policy below
# never runs — and a hook that moved HEAD to the empty tree would be left there
# (P2-01 FIND-780 HIGH-1, reproduced).
$preCommitRefusal = $null
# Which exit code the refusal reports. 4 = the FIND-778 result guards; 9 = a
# foreign row in a shared file (FIND-792). Both mean "this script created no
# commit"; the class is what tells an orchestrator whether to retry.
$preCommitExitCode = 4
$preCommitTitle = "REFUSED (FIND-780): no commit was created."

# ══ READING A BLOB WITHOUT THE CONSOLE (FIND-793) ═══════════════════════════
# `git show HEAD:<path>` piped into PowerShell decodes the bytes with the console
# codepage, so a UTF-8 accent can arrive as mojibake and the check fails AFTER the
# commit exists. Here the bytes go git -> FILE (OS-level redirect) -> .NET UTF-8
# decoder. Nothing on this path depends on how the console encodes text.
function Get-BlobText([string]$Rev, [string]$Path) {
    $spec = "$Rev`:$Path"
    $oid = "$(& git rev-parse --verify --quiet --end-of-options $spec 2>$null)".Trim()
    if ($LASTEXITCODE -ne 0 -or -not $oid) { return $null }   # path not at that rev
    $tmp = Join-Path $tmpDir ("obj-" + [guid]::NewGuid().ToString("N").Substring(0, 10) + ".bin")
    try {
        $p = Start-Process -FilePath "git" -ArgumentList @("cat-file", "blob", $oid) `
            -NoNewWindow -Wait -PassThru -RedirectStandardOutput $tmp -RedirectStandardError ($tmp + ".err")
        if ($p.ExitCode -ne 0) { return $null }
        $bytes = [IO.File]::ReadAllBytes($tmp)
        $text = [Text.Encoding]::UTF8.GetString($bytes)
        if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) { $text = $text.Substring(1) }
        return $text
    } finally {
        Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath ($tmp + ".err") -Force -ErrorAction SilentlyContinue
    }
}

function ConvertTo-Nfc([string]$s) {
    if ($null -eq $s) { return "" }
    return $s.Normalize([Text.NormalizationForm]::FormC)
}

# Diacritic-free, non-ASCII-free projection: what survives a transport that drops
# or re-encodes accents. `después` and `despues` share the skeleton `despues`; so
# does the NFD form. A difference that disappears here is an ENCODING failure.
function Get-AsciiSkeleton([string]$s) {
    $decomposed = (ConvertTo-Nfc $s).Normalize([Text.NormalizationForm]::FormD)
    $sb = New-Object System.Text.StringBuilder
    foreach ($ch in $decomposed.ToCharArray()) {
        if ([Globalization.CharUnicodeInfo]::GetUnicodeCategory($ch) -eq [Globalization.UnicodeCategory]::NonSpacingMark) { continue }
        if ([int]$ch -lt 128) { [void]$sb.Append($ch) }
    }
    return $sb.ToString()
}

# The classic double encoding: the caller's UTF-8 bytes were read as Latin-1 and
# re-encoded (`después` -> `despuÃ©s`). Undo it; the caller is not asked to retype.
function Repair-DoubleEncoding([string]$s) {
    try {
        $latin1 = [Text.Encoding]::GetEncoding(28591)
        return [Text.Encoding]::UTF8.GetString($latin1.GetBytes($s))
    } catch { return $s }
}

# Offsets of every occurrence, with ordinal semantics: no culture rules, no regex.
function Find-Fragment([string]$Text, [string]$Fragment) {
    $hits = New-Object System.Collections.Generic.List[int]
    if (-not $Fragment) { return ,$hits.ToArray() }
    $at = $Text.IndexOf($Fragment, [StringComparison]::Ordinal)
    while ($at -ge 0) {
        $hits.Add($at)
        if ($at -ge ($Text.Length - 1)) { break }
        $at = $Text.IndexOf($Fragment, $at + 1, [StringComparison]::Ordinal)
    }
    return ,$hits.ToArray()
}

function Get-LineOf([string]$Text, [int]$Index) {
    return ($Text.Substring(0, $Index) -split "`n").Count
}

# Names WHERE the fragment matched: the whole point of FIND-796 is that the
# operator must see file:line instead of an invented cause.
function Write-MatchSite([string]$Path, [string]$Text, [int]$Index) {
    $line = Get-LineOf $Text $Index
    $lines = @($Text -split "`n")
    $raw = if ($line -le $lines.Count) { $lines[$line - 1] } else { "" }
    $shown = $raw.Trim()
    if ($shown.Length -gt 160) { $shown = $shown.Substring(0, 157) + "..." }
    Write-Host "    $Path`:$line  $shown" -ForegroundColor Yellow
    return $line
}

# Same, but for a fragment located on the DIACRITIC-FREE projection: the offsets
# there are not offsets in the original text (the projection is shorter), so the
# line is found per line instead of by index. A wrong line number in the file
# that exists to stop wrong diagnoses would be the same defect again.
function Write-SkeletonSites([string]$Path, [string]$Text, [string]$SkeletonFragment) {
    $lines = @($Text -split "`n")
    $found = 0
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ((Get-AsciiSkeleton $lines[$i]).IndexOf($SkeletonFragment, [StringComparison]::Ordinal) -lt 0) { continue }
        $found++
        $shown = $lines[$i].Trim()
        if ($shown.Length -gt 160) { $shown = $shown.Substring(0, 157) + "..." }
        Write-Host "    $Path`:$($i + 1)  $shown" -ForegroundColor Yellow
    }
    return $found
}

# ══ FOREIGN ROWS IN A SHARED FILE (FIND-792) ════════════════════════════════
# `-Paths docs/dev/Backlog.md` commits the WHOLE working copy of that file, so a
# row another lane retired (whose code is still uncommitted) rides along and the
# backlog asserts a closure that is not in history — measured three times
# (`6e4119c8d` took FIND-775, `d15564b06` took FIND-788, `a476e190c` took
# FIND-774). The helper's own expectation could not see it: it checked ITS row.
# The check is exact about Backlog rows and silent about every other file: a
# general "hunks must match declarations" rule would refuse normal commits.
$BACKLOG_PATH = "docs/dev/Backlog.md"

function Get-BacklogRowIds([string]$Text) {
    $ids = New-Object System.Collections.Generic.List[string]
    if ($null -eq $Text) { return ,$ids.ToArray() }
    foreach ($m in [regex]::Matches($Text, '(?m)^\|\s*`(?<id>[A-Za-z][A-Za-z0-9]*-\d+)`\s*\|')) {
        $ids.Add($m.Groups['id'].Value)
    }
    return ,$ids.ToArray()
}

# IDs named by ANY expectation spec. That is the declaration: "this row is mine".
function Get-DeclaredIds([string[]]$Specs) {
    $ids = New-Object System.Collections.Generic.List[string]
    foreach ($s in $Specs) {
        foreach ($m in [regex]::Matches($s, '[A-Za-z][A-Za-z0-9]*-\d+')) { $ids.Add($m.Value) }
    }
    return ,$ids.ToArray()
}


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
    $treeSeeded = "$(& git write-tree)" -replace '\s', ''
    if ($LASTEXITCODE -ne 0 -or -not $treeSeeded) { throw "git write-tree failed on the private index" }
    # On the hook-failure path this is what the guards must judge identity against
    # (the tree we staged), exactly as before FIND-780.
    $treePlanned = $treeSeeded

    # ══ THE GATE, RUN EXPLICITLY (FIND-780) ══════════════════════════════════
    # `git commit` was what ran .githooks/pre-commit, and `git commit` is what this
    # script no longer calls (see the plumbing below). The gate must not be lost,
    # so it runs here through the SAME entry point git uses: `git hook run`
    # resolves core.hooksPath, and GIT_INDEX_FILE is already the private index, so
    # the hook's `git diff --staged` sees exactly the paths this commit carries.
    #
    # Running the HOOK — not `pre-commit run --config ... --files` — is the whole
    # point: the shim also enforces DOC-04 junk files, the FIND-726 unmerged and
    # unstaged-config guards, verify_staged.ps1 and actionlint. Re-listing its
    # gates here would drop them silently the day the shim changes.
    $hookDir = "$(& git rev-parse --git-path hooks)".Trim()
    $hookPresent = [bool]($hookDir -and (Test-Path -LiteralPath (Join-Path $hookDir "pre-commit")))
    if (-not $hookPresent) {
        Write-Host "WARNING: no pre-commit hook under '$hookDir' — committing WITHOUT the gate." -ForegroundColor Yellow
    }
    # `git commit` would also run these; plumbing does not. A warning is the only
    # honest tripwire that can exist for a hook someone adds later (P2-01 M2):
    # their absence today is what makes the omission safe.
    $skippedHooks = @(@("prepare-commit-msg", "commit-msg", "post-commit") |
        Where-Object { $hookDir -and (Test-Path -LiteralPath (Join-Path $hookDir $_)) })
    if ($skippedHooks.Count -gt 0) {
        Write-Host "WARNING: this script does NOT run: $($skippedHooks -join ', ') — 'git commit' would." -ForegroundColor Yellow
        Write-Host "  Wire them into dev-tools/git-commit-isolated.ps1 before relying on them." -ForegroundColor Yellow
    }
    & git hook run --ignore-missing pre-commit
    $hookExit = $LASTEXITCODE
    if ($hookExit -ne 0) { throw "pre-commit hook FAILED (exit $hookExit): commit blocked, nothing was created" }
    if ($hookPresent) { Write-Host "pre-commit hook passed." }

    # Written AFTER the hooks, as `git commit` does: a hook may rewrite the index.
    $treePlanned = "$(& git write-tree)" -replace '\s', ''
    if ($LASTEXITCODE -ne 0 -or -not $treePlanned) { throw "git write-tree failed after the pre-commit hook" }
    if ($treePlanned -ne $treeSeeded) {
        Write-Host "NOTE: the pre-commit hook changed the private index ($($treeSeeded.Substring(0, 8)) -> $($treePlanned.Substring(0, 8)))." -ForegroundColor Yellow
    }

    # Refuse the catastrophe BEFORE it is a commit. The result guards below still
    # run (a hook can move HEAD itself), but the empty tree is never created by us.
    # Do NOT exit here: the refusal is REPORTED through the validation block, which
    # is the only code that restores HEAD when a hook moved it (HIGH-1).
    if ($treePlanned -eq $EMPTY_TREE) {
        $preCommitRefusal = "CATASTROPHIC: the tree staged for this commit is the canonical EMPTY tree ($EMPTY_TREE) — committing it would delete every tracked path. A hook emptied the private index, or it vanished."
        throw "REFUSED (FIND-780): the private index is EMPTY after the hooks; no commit was created"
    }
    $plannedEntries = Get-TreeEntryCount $treePlanned
    if ($plannedEntries -lt 0 -or $plannedEntries -lt ($entriesBefore - $MAX_ENTRY_DROP)) {
        $preCommitRefusal = "SUSPICIOUS: the staged tree has $plannedEntries entries; HEAD had $entriesBefore (largest legitimate drop measured: 43, allowed: $MAX_ENTRY_DROP). For a deliberate bulk delete use plain 'git commit'."
        throw "REFUSED (FIND-780): the staged tree lost too many paths; no commit was created"
    }

    if (-not $headBefore) { throw "HEAD could not be resolved; refusing to create a parentless commit" }

    # ══ FOREIGN ROWS IN A SHARED FILE (FIND-792) ════════════════════════════
    # Runs BEFORE commit-tree, so the abort creates nothing and needs no restore.
    # Declared = any ID named by an expectation (that is the caller saying "this
    # row is mine"). A removed Backlog row whose ID nobody declared belongs to
    # another lane whose code is still uncommitted: committing it closes a row
    # that has no fix in history. Silence for every other path is deliberate:
    # "hunks must match declarations" on a normal source file refuses real work.
    #
    # WHICH paths? The STAGED set from the private index, never the arguments
    # (P2-01 round 1): `-Paths docs/dev` stages the Backlog and does not spell it,
    # so a `$Paths -contains` test was a silent bypass of the whole guard — the
    # incident it exists to stop, with `OK` at the end.
    $stagedPaths = @(& git diff --cached --name-only 2>$null | Where-Object { $_ })
    if (-not $AllowForeignRows -and ($stagedPaths -contains $BACKLOG_PATH)) {
        $blBase = Get-BlobText $headBefore $BACKLOG_PATH
        $blNew = Get-BlobText $treePlanned $BACKLOG_PATH
        if ($null -ne $blBase -and $null -ne $blNew) {
            $baseIds = Get-BacklogRowIds $blBase
            $newIds = Get-BacklogRowIds $blNew
            $removed = @($baseIds | Where-Object { $newIds -notcontains $_ } | Select-Object -Unique)
            # A row whose ID is still ANYWHERE in the new blob was reformatted or
            # MOVED, not retired. Without this, `| `FIND-102` (blocked) |` aborted
            # the commit and accused another lane of a row nobody removed, and a
            # pure row move (the real `FIND-787` case) was reported as a removal.
            if ($removed.Count -gt 0) {
                $removed = @($removed | Where-Object { -not $blNew.Contains('`' + $_ + '`') })
            }
            if ($removed.Count -gt 0) {
                $declared = Get-DeclaredIds (@($ExpectAbsent) + @($ExpectPresent) + @($ExpectBlob))
                $foreign = @($removed | Where-Object { $declared -notcontains $_ })
                if ($foreign.Count -gt 0) {
                    $preCommitExitCode = 9
                    $preCommitTitle = "REFUSED (FIND-792): this commit would retire rows that are not yours."
                    $preCommitRefusal = "FOREIGN HUNK: removing $($removed.Count) Backlog row(s) [$($removed -join ', ')], but only [$((@($removed | Where-Object { $declared -contains $_ })) -join ', ')] were declared. Undeclared: [$($foreign -join ', ')]. Their code is not in this commit — the backlog would say 'closed' about a file that is not in history. Declare each row with -ExpectAbsent 'docs/dev/Backlog.md::| ``ID`` |', commit that lane's work first, or pass -AllowForeignRows if the cleanup is deliberate."
                    throw "REFUSED (FIND-792): foreign Backlog row(s) in the staged tree: $($foreign -join ', ')"
                }
            }
        }
    }

    # ══ CREATE THE COMMIT WITH PLUMBING (FIND-780) ═══════════════════════════
    # `git commit-tree` reads the TREE OBJECT, not the index: from here on the
    # private index may be deleted by anything (a kill unwinding this script's
    # `finally`, a temp cleaner) and the commit is unaffected.
    $newCommit = "$(& git commit-tree $treePlanned -p $headBefore -m $Message)" -replace '\s', ''
    if ($LASTEXITCODE -ne 0 -or -not $newCommit) { throw "git commit-tree failed; no commit object was created" }

    # Compare-and-swap: only move HEAD if it is still the commit we seeded from.
    # A peer that committed while the hooks ran is NEVER clobbered or reverted.
    # (git 2.51 checks this inside `git commit` too — measured — so this is not a
    # new guarantee, it is the same one made explicit and independent of git's
    # internals, with a message that names which of the two causes fired.)
    & git update-ref -m $Message HEAD $newCommit $headBefore
    if ($LASTEXITCODE -ne 0) {
        $headNow = "$(& git rev-parse --verify --quiet HEAD 2>$null)".Trim()
        if ($headNow -and $headNow -ne $headBefore) {
            throw "REFUSED (FIND-780): another lane moved HEAD to $headNow while the hooks ran; our commit $newCommit was NOT attached (it would have reverted their work)"
        }
        throw "REFUSED (FIND-780): git update-ref could not lock HEAD (a peer's lock or an I/O error); our commit $newCommit was NOT attached"
    }
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
# Not an `if success` block: a hook moving HEAD, or a step exiting non-zero, does
# NOT mean nothing was created. Both of today's empty-tree commits were reported
# as failures. The commit step itself is now plumbing that cannot half-exist
# (FIND-780), but the hook still runs arbitrary code on HEAD — so this stays.
$headAfter = (& git rev-parse --verify --quiet HEAD 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $headAfter) { $headAfter = $null } else { $headAfter = "$headAfter".Trim() }
$moved = [bool]($headBefore -and $headAfter -and ($headAfter -ne $headBefore))

$reasons = New-Object System.Collections.Generic.List[string]
$isOurs = $false
$treeAfter = $null

if ($moved) {
    $headAfterTree = & git rev-parse "$headAfter^{tree}"
    $treeAfter = "$headAfterTree".Trim()
    $entriesAfter = Get-TreeEntryCount $headAfter

    # Whose commit is on HEAD? If MY commit step failed (a hook rejecting it, a lost
    # compare-and-swap) and ANOTHER lane committed in the same window, these guards
    # would be judging THEIR commit — and
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
        if ($preCommitError) { Write-Host "  $preCommitError" -ForegroundColor Yellow }
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
    if ($preCommitRefusal) {
        # A refusal that PREVENTED the commit: the empty/tiny tree family (exit 4,
        # FIND-778/780) or a foreign Backlog row (exit 9, FIND-792). Either way this
        # script created no commit — the class tells an orchestrator what to do.
        Write-Host ""
        Write-Host $preCommitTitle -ForegroundColor Red
        Write-Host "  - $preCommitRefusal" -ForegroundColor Red
        if ($headAfter -and $headAfter -eq $headBefore) {
            Write-Host "  HEAD did not move: it is still $headBefore." -ForegroundColor Yellow
        } else {
            # `$moved` is false here, so HEAD is either unchanged or gone (a hook
            # may delete the ref). Claiming "still <sha>" would be a lie (P2-01 LOW-B).
            Write-Host "  HEAD is no longer resolvable — a hook deleted the ref; check the hook output above." -ForegroundColor Yellow
        }
        Write-Host "  this script created no commit; check the hook output above." -ForegroundColor Yellow
        exit $preCommitExitCode
    }
    if ($preCommitError) { Write-Host "ERROR: $preCommitError" -ForegroundColor Red }
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
if ($preCommitError) {
    # Only reachable when a hook moved HEAD to a commit that IS provably ours and
    # then failed the commit. The work landed, so report it — but never as green.
    Write-Host "NOTE: $preCommitError" -ForegroundColor Yellow
    Write-Host "NOTE: HEAD moved to $commit and the result is provably ours, but the GATE FAILED. Do not treat this as a green commit." -ForegroundColor Yellow
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

# ── Verification BY BLOB IDENTITY (FIND-792/793/795/796) ────────────────────
# Deliberately not `git log`/exit-code based: a concurrent commit can bake a stale
# copy of a shared file and still exit 0. Only the bytes in the commit's blobs
# answer "did my edit survive?" — and they are read from the OBJECT DATABASE, not
# from a PowerShell pipe, so the console codepage cannot corrupt them.
#
# Three states, not one (FIND-796). Every line printed below names the measurement
# behind it: never "the commit did not carry your edit" without having measured it.
$commitSha = (& git rev-parse HEAD).Trim()
$failCount = 0
# Exit class, with precedence by RANK (a later, more specific class wins): content
# (1) < ambiguous (7) < encoding/transport (8). Fixed order, not "last wins":
# an ExpectPresent failure must not mask an ExpectAbsent ambiguity.
$failExit = 0
function Set-FailClass([int]$code) {
    if ($code -eq 0) { return }
    $script:failCount++
    $rank = @{ 1 = 1; 7 = 2; 8 = 3 }
    $current = if ($rank.ContainsKey([int]$script:failExit)) { $rank[[int]$script:failExit] } else { 0 }
    if ($rank.ContainsKey([int]$code) -and $rank[[int]$code] -gt $current) { $script:failExit = $code }
}

function Test-Expectation([string]$spec, [bool]$mustBePresent) {
    $parts = $spec -split '::', 2
    $path = $parts[0]
    $frag = if ($parts.Count -gt 1) { $parts[1] } else { $null }
    if (-not $frag) {
        Write-Host "MALFORMED EXPECTATION (no '::' or empty text): $spec" -ForegroundColor Red
        return 8
    }

    $oid = "$(& git rev-parse --verify --quiet --end-of-options "$commitSha`:$path" 2>$null)".Trim()
    if ($LASTEXITCODE -ne 0 -or -not $oid) {
        # State (iii): measured, and it says exactly that.
        Write-Host "NOT IN COMMIT: '$path' does not exist in $commitSha, so '$frag' cannot be judged." -ForegroundColor Red
        return 1
    }
    $text = Get-BlobText $commitSha $path
    if ($null -eq $text) {
        Write-Host "UNREADABLE BLOB: $commitSha`:$path resolved to $oid but git cat-file failed." -ForegroundColor Red
        return 1
    }

    $nText = ConvertTo-Nfc $text
    $nFrag = ConvertTo-Nfc $frag
    $hits = Find-Fragment $nText $nFrag

    if ($mustBePresent) {
        if ($hits.Count -eq 1) {
            $line = Get-LineOf $nText $hits[0]
            Write-Host "ok present: $path`:$line :: $frag" -ForegroundColor Green
            return 0
        }
        if ($hits.Count -gt 1) {
            Write-Host "AMBIGUOUS EXPECTATION: '$frag' matches $($hits.Count) times in $path — a passing check would not say WHICH one you meant:" -ForegroundColor Red
            foreach ($h in $hits) { Write-MatchSite $path $nText $h | Out-Null }
            Write-Host "  narrow the expectation (include the surrounding structure) or pass each site as its own spec." -ForegroundColor Yellow
            return 7
        }
        # Zero hits. Is the difference the TRANSPORT or the CONTENT? Measured:
        # (a) accents/structure folded away, (b) the classic Latin-1 double decode.
        $skeletonFrag = Get-AsciiSkeleton $nFrag
        $skeletonText = Get-AsciiSkeleton $nText
        if ($skeletonFrag -and ($skeletonText -ne $nText) -and $skeletonFrag -ne $nFrag) {
            $sh = Find-Fragment $skeletonText $skeletonFrag
            if ($sh.Count -ge 1) {
                # Measured: the difference between the fragment and the blob is
                # confined to characters outside ASCII. That is COMPATIBLE with a
                # transport that dropped or re-encoded them AND with a different
                # spelling — the helper must not choose (P2-01 round 1). It states
                # both and lets the operator look at the line.
                Write-Host "ENCODING?: '$frag' does not match, but its accent-free projection ('$skeletonFrag') does — the two differ only outside ASCII." -ForegroundColor Red
                Write-SkeletonSites $path $nText $skeletonFrag | Out-Null
                Write-Host "  measured: the blob is UTF-8, read from the object database. Two hypotheses fit and this check cannot separate them:" -ForegroundColor Yellow
                Write-Host "    (a) the expectation's non-ASCII bytes did not survive the command line (transport);" -ForegroundColor Yellow
                Write-Host "    (b) the blob spells those characters differently from what you typed." -ForegroundColor Yellow
                Write-Host "  the commit EXISTS and is not wrong: do NOT re-run it (that duplicates it)." -ForegroundColor Yellow
                return 8
            }
        }
        $repaired = ConvertTo-Nfc (Repair-DoubleEncoding $nFrag)
        if ($repaired -and $repaired -ne $nFrag) {
            $rh = Find-Fragment $nText $repaired
            if ($rh.Count -ge 1) {
                Write-Host "ENCODING: '$frag' was DOUBLE-ENCODED in transport; decoded once it is '$repaired', which IS in the blob." -ForegroundColor Red
                foreach ($h in $rh) { Write-MatchSite $path $nText $h | Out-Null }
                Write-Host "  the commit EXISTS and is not wrong: do NOT re-run it. Pass the expectation with real UTF-8 bytes." -ForegroundColor Yellow
                return 8
            }
        }
        Write-Host "NOT COMMITTED: '$frag' is not in the blob of $path at $commitSha." -ForegroundColor Red
        Write-Host "  measured: read $($text.Length) chars from git cat-file (NFC-normalised on both sides), 0 occurrences." -ForegroundColor Yellow
        if ($text.Length -ne (Get-AsciiSkeleton $text).Length) {
            Write-Host "  the blob DOES contain non-ASCII characters: if the expectation was typed with accents, check that they reached this process intact (an ASCII-only fragment cannot be diagnosed as a transport problem — it is a different string)." -ForegroundColor Yellow
        }
        return 1
    }

    # ExpectAbsent
    if ($hits.Count -eq 0) { Write-Host "ok absent: $path :: $frag" -ForegroundColor Green; return 0 }
    if ($hits.Count -gt 1) {
        Write-Host "AMBIGUOUS EXPECTATION: '$frag' matches $($hits.Count) times in $path — 'absent' cannot be proven about the phrase you meant:" -ForegroundColor Red
        foreach ($h in $hits) { Write-MatchSite $path $nText $h | Out-Null }
        Write-Host "  the commit EXISTS; nothing here says it reverted your edit. Narrow the expectation to one site." -ForegroundColor Yellow
        return 7
    }
    $siteLine = Get-LineOf $nText $hits[0]
    $parentText = if ($headBefore) { Get-BlobText $headBefore $path } else { $null }
    # Does the FRAGMENT exist in the parent? Asking by line index was wrong
    # (P2-01 round 1): a commit that deletes line 1 shifts every later line, so a
    # byte-identical line looked "changed" and the helper printed `this commit
    # CHANGED that line` about a line it never touched — the FIND-796 defect,
    # one layer down.
    $sameAsParent = [bool]($null -ne $parentText -and (Find-Fragment (ConvertTo-Nfc $parentText) $nFrag).Count -ge 1)

    Write-Host "PRESENT: '$frag' is still in $path`:$siteLine at $commitSha — that is the measured fact." -ForegroundColor Red
    Write-MatchSite $path $nText $hits[0] | Out-Null
    if ($sameAsParent) {
        # The string is in the parent too. Two measured-consistent causes, and the
        # helper cannot tell them apart — so it does not choose:
        # (a) pre-existing text (prose, another row) the fragment also matches;
        # (b) a removal that did not land (stale copy baked in, FIND-753).
        Write-Host "  the same string exists in the PARENT commit, so this commit did not introduce it." -ForegroundColor Yellow
        Write-Host "  consistent with (a) pre-existing text your fragment also matches — the expectation lacks a structural anchor —" -ForegroundColor Yellow
        Write-Host "  or (b) a removal you declared that did not land. The commit EXISTS; no reversion is claimed here." -ForegroundColor Yellow
        return 7
    }
    Write-Host "  the string is NOT in the parent commit: this commit introduced it (or baked in a copy that did)." -ForegroundColor Red
    Write-Host "  content failure — nothing here says the commit reverted your edit: re-inspect the file." -ForegroundColor Yellow
    return 1
}

function Test-BlobIdentity([string]$spec) {
    $parts = $spec -split '::', 2
    $path = $parts[0]
    $want = if ($parts.Count -gt 1) { $parts[1].Trim().ToLowerInvariant() } else { "" }
    if ($want -notmatch '^[0-9a-f]{7,40}$') {
        Write-Host "MALFORMED ExpectBlob (needs '<path>::<hex oid>'): $spec" -ForegroundColor Red
        return 8
    }
    $wantFull = "$(& git rev-parse --verify --quiet "$want^{blob}" 2>$null)".Trim()
    if ($LASTEXITCODE -ne 0 -or -not $wantFull) {
        Write-Host "BAD ExpectBlob: '$want' is not a blob in this object database." -ForegroundColor Red
        return 8
    }
    $actual = "$(& git rev-parse --verify --quiet --end-of-options "$commitSha`:$path" 2>$null)".Trim()
    if ($LASTEXITCODE -ne 0 -or -not $actual) {
        Write-Host "NOT IN COMMIT: '$path' does not exist in $commitSha (ExpectBlob)." -ForegroundColor Red
        return 1
    }
    if ($actual -ne $wantFull) {
        Write-Host "BLOB MISMATCH: $path at $commitSha is $actual, expected $wantFull." -ForegroundColor Red
        return 1
    }
    $onDisk = "$(& git hash-object -- $path 2>$null)".Trim()
    if ($LASTEXITCODE -eq 0 -and $onDisk -and $onDisk -ne $actual) {
        Write-Host "WARNING: $path in the worktree is $onDisk, the commit carries $actual — the commit is correct, the worktree has moved on." -ForegroundColor Yellow
    }
    Write-Host "ok blob: $path :: $actual" -ForegroundColor Green
    return 0
}

# An uncaught I/O error inside a check would surface as a non-zero exit with the
# commit already created — and `1` reads as "content", which invites a duplicate.
# Measured cost of not catching it: the class was right by luck.
try {
    foreach ($e in $ExpectAbsent) { Set-FailClass (Test-Expectation $e $false) }
    foreach ($e in $ExpectPresent) { Set-FailClass (Test-Expectation $e $true) }
    foreach ($e in $ExpectBlob) { Set-FailClass (Test-BlobIdentity $e) }
} catch {
    Write-Host "ERROR: the verification could not run: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "  the commit EXISTS ($commitSha) and no expectation was judged. Do not re-run it." -ForegroundColor Yellow
    exit 8
}

$expectCount = $ExpectAbsent.Count + $ExpectPresent.Count + $ExpectBlob.Count
if ($failCount -gt 0) {
    Write-Host "FAILED: $failCount expectation(s) unmet at $commit (exit class $failExit)." -ForegroundColor Red
    Write-Host "  Inspect the BLOB, not the console: git cat-file -p $commit`:<path>"
    if ($failExit -eq 7) { Write-Host "  class 7 = AMBIGUOUS: the commit exists and is not wrong; the expectation matched text it does not own." -ForegroundColor Yellow }
    if ($failExit -eq 8) { Write-Host "  class 8 = EXPECTATION UNTRUSTWORTHY: the commit exists and is not wrong; do not re-run the commit." -ForegroundColor Yellow }
    exit $failExit
}
Write-Host "OK: $commit verified against the commit's blobs ($expectCount expectation(s))." -ForegroundColor Green
exit 0
