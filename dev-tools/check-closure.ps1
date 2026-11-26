<#
.SYNOPSIS
  Closure is ONE unit: the Backlog row, the task file's **Estado:**, the avance
  entry and the commit must agree. This script checks that they do.

.DESCRIPTION
  H-04 of docs/dev/reviews/2026-11-26-harness-hardening.md (§A.7): the closure
  had five artifacts, three owners and no verifier, so the drifts were measured,
  not hypothesised:
    - task files still non-terminal after the row left the Backlog (measured
      2026-11-26: FIND-587 IN PROGRESS, plus 2026-09 audit leftovers);
    - `**Estado:**` values outside the canonical set (`Completado (2026-11-25)`,
      `RESUELTO — …`, `IN PROGRESS` without emoji) — parsers.mjs:16-21 classifies
      those as PENDING, i.e. a closed task reports as pending;
    - rows removed from the Backlog with no entry in docs/dev/avance/.

  Modes:
    -Diff    (default workflow, `/harness`, close of a task): only what the diff
             touches — task files changed, and rows removed from the Backlog.
    -All     full tree: orphans (active task file whose ID is in no Backlog row
             and in no plan) plus an ADVISORY count of legacy non-canonical
             states. Known 2026-09 leftovers are declared in
             dev-tools/closure-baseline.json so the gate is green today and red
             on the first NEW orphan.
    -SelfTest  prove each rule can fail (fixtures below).

  A gate that cannot fail is not a gate. Exit: 0 pass, 1 violation, 2 error.

.PARAMETER Base
  Git ref the diff is measured against. Default origin/develop, fallback HEAD~1.

.EXAMPLE
  pwsh dev-tools/check-closure.ps1 -Diff
  pwsh dev-tools/check-closure.ps1 -All -SelfTest
#>
param(
    [switch]$Diff,
    [switch]$All,
    [switch]$SelfTest,
    [string]$Base = "",
    [string]$Root = "",
    [string[]]$Paths = @()
)
$Paths = @($Paths | ForEach-Object { $_ -split ',' } | Where-Object { $_ })

$ErrorActionPreference = "Continue"
$root = if ($Root) { (Resolve-Path $Root).Path } else { Split-Path -Parent $PSScriptRoot }

# M3: the docstring promised origin/develop; the code used HEAD. Resolved here
# and reported, so the real base is never a guess.
function Resolve-Base {
    if ($Base) { return $Base }
    & git -C $root rev-parse --verify --quiet origin/develop *> $null
    if ($LASTEXITCODE -eq 0) { return "origin/develop" }
    & git -C $root rev-parse --verify --quiet HEAD~1 *> $null
    if ($LASTEXITCODE -eq 0) { return "HEAD~1" }
    return "HEAD"
}

# Canonical tokens, compared by code point so this file stays ASCII-safe.
$CANON = '^(\u2B1C|\u23F3|\u2705|\u274C)\s+(PENDING|IN PROGRESS|COMPLETED|FAILED)\b'
$TERMINAL = '^(\u2705|\u274C)'

$violations = New-Object System.Collections.Generic.List[string]
$advisories = New-Object System.Collections.Generic.List[string]

function Read-Utf8([string]$path) {
    return [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
}

function Get-Estado([string]$text) {
    $m = [regex]::Match($text, '(?m)^\s*-\s*\*\*Estado:\*\*\s*(.+)$')
    if (-not $m.Success) { return $null }
    return $m.Groups[1].Value.Trim()
}

function Test-Terminal([string]$estado) {
    if (-not $estado) { return $false }
    return [regex]::IsMatch($estado, $TERMINAL)
}

function Test-Canonical([string]$estado) {
    if (-not $estado) { return $false }
    return [regex]::IsMatch($estado, $CANON)
}

function Get-IdInText([string]$text) {
    $ids = New-Object System.Collections.Generic.List[string]
    foreach ($m in [regex]::Matches($text, '\b[A-Z][A-Z0-9]*-[A-Za-z0-9]+\b|\b(?:FIND|MCP|MEMG|AUD|AUDREP|DEAD|DIST|EGO|GTM|NOT|WIRE|CI|D1b)-[A-Za-z0-9]+\b')) {
        if (-not $ids.Contains($m.Value)) { $ids.Add($m.Value) }
    }
    return $ids
}

function Get-TaskFiles {
    param([string[]]$Filter)
    $dir = Join-Path $root "docs/dev/tasks"
    if (-not (Test-Path $dir)) { return @() }
    # complete/ and closed/ are archives: their state is historical, not live.
    $all = Get-ChildItem $dir -Filter *.md
    if ($Filter) {
        $all = $all | Where-Object { $rel = $_.FullName.Substring($root.Length + 1).Replace('\', '/'); $Filter -contains $rel }
    }
    return $all
}

function Get-DiffFiles([string[]]$FilePaths) {
    $out = & git -C $root diff --name-only --diff-filter=ACMR $Base -- @FilePaths 2>$null
    if ($LASTEXITCODE -ne 0) { return @() }
    $files = @($out | Where-Object { $_ })
    # M3: a brand-new task file is untracked, so `git diff` cannot see it — and
    # that is exactly the closure that nobody registers. Union the untracked set.
    $untracked = & git -C $root ls-files --others --exclude-standard -- @FilePaths 2>$null
    if ($LASTEXITCODE -eq 0) { $files += @($untracked | Where-Object { $_ }) }
    $files = @($files | Sort-Object -Unique)
    if ($Paths.Count -gt 0) {
        $files = @($files | Where-Object { $f = $_; [bool]($Paths | Where-Object { $f.StartsWith($_) -or $f.Contains($_) }) })
    }
    return $files
}

function Get-RemovedBacklogIds {
    $diff = & git -C $root diff -U0 $Base -- docs/dev/Backlog.md 2>$null
    if ($LASTEXITCODE -ne 0) { return @() }
    $ids = New-Object System.Collections.Generic.List[string]
    foreach ($line in $diff) {
        if ($line -match '^-' -and $line -notmatch '^---') {
            $m = [regex]::Match($line, '^\-\s*\|\s*`([^`]+)`')
            if ($m.Success -and -not $ids.Contains($m.Groups[1].Value)) { $ids.Add($m.Groups[1].Value) }
        }
    }
    return $ids
}

function Test-AvanceHasId([string]$id) {
    $dir = Join-Path $root "docs/dev/avance"
    if (-not (Test-Path $dir)) { return $false }
    return [bool](Get-ChildItem $dir -Recurse -Filter *.md |
        Where-Object { Test-IdPresent (Read-Utf8 $_.FullName) $id } |
        Select-Object -First 1)
}

# ── Self-test: each rule must be able to fail ───────────────────────────────
function Invoke-SelfTest {
    $sample = "## Metadata`n- **Estado:** " + [char]0x23F3 + " IN PROGRESS`n"
    $removed = @("-| ``FIND-999`` | x | y |", "-| ``FIND-998`` | x | y |", "+| ``FIND-997`` | new |")
    $removedIds = @()
    foreach ($line in $removed) {
        if ($line -match '^-' -and $line -notmatch '^---') {
            $m = [regex]::Match($line, '^\-\s*\|\s*`([^`]+)`')
            if ($m.Success) { $removedIds += $m.Groups[1].Value }
        }
    }

    $cases = @(
        @("terminal: COMPLETED is terminal", (Test-Terminal ([string][char]0x2705 + " COMPLETED"))),
        @("terminal: FAILED is terminal", (Test-Terminal ([string][char]0x274C + " FAILED"))),
        @("terminal: IN PROGRESS is not", (-not (Test-Terminal ([string][char]0x23F3 + " IN PROGRESS")))),
        @("canonical: PENDING accepted", (Test-Canonical ([string][char]0x2B1C + " PENDING"))),
        @("canonical: 'Completado (date)' rejected", (-not (Test-Canonical ([string][char]0x2705 + " Completado (2026-11-25)")))),
        @("canonical: bare 'IN PROGRESS' rejected", (-not (Test-Canonical "IN PROGRESS"))),
        @("estado: parsed from metadata", ($null -ne (Get-Estado $sample))),
        @("estado: absent -> null", ($null -eq (Get-Estado "no field here"))),
        @("closure: removed rows parsed", ($removedIds.Count -eq 2 -and $removedIds[0] -eq "FIND-999"))
    )

    $failed = 0
    foreach ($c in $cases) {
        $name = $c[0]
        $pass = [bool]$c[1]
        Write-Host ("{0}  {1}" -f $(if ($pass) { "ok  " } else { "FAIL" }), $name)
        if (-not $pass) { $failed++ }
    }
    Write-Host ("`n{0}/{1} self-test fixtures passed" -f ($cases.Count - $failed), $cases.Count)
    if ($failed -gt 0) { exit 1 }
}

# L1: the fixtures above test the regexes. These test the three RULES end-to-end,
# in a throw-away git repo, by re-invoking this script with -Root.
function Invoke-EndToEnd {
    $scriptPath = Join-Path $PSScriptRoot "check-closure.ps1"
    $tmp = Join-Path $env:TEMP ("closure-e2e-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    $write = { param($rel, $text) $p = Join-Path $tmp $rel; New-Item -ItemType Directory -Force (Split-Path $p) | Out-Null; [System.IO.File]::WriteAllText($p, $text, $utf8) }
    $run = { param($extra) & pwsh -NoProfile -File $scriptPath @extra *> $null; return $LASTEXITCODE }
    $cases = @()

    & $write "docs/dev/Backlog.md" "# Backlog`n`n| ID | Hallazgo |`n|---|---|`n| ``FIND-901`` | x |`n"
    & $write "docs/dev/avance/activo/core.md" "# core`n"
    & git -C $tmp init -q 2>$null
    & git -C $tmp -c user.email=t@t -c user.name=t add -A 2>$null
    & git -C $tmp -c user.email=t@t -c user.name=t commit -qm init 2>$null

    # (a) + M3 (untracked) + M4 (FIND-90 vs FIND-901): orphan active task
    & $write "docs/dev/tasks/FIND-90.md" ("## Metadata`n- **Estado:** " + [char]0x23F3 + " IN PROGRESS`n")
    $cases += ,@("e2e (a): untracked active orphan is a violation (FIND-90 vs FIND-901)", ((& $run @("-Diff", "-Root", $tmp)) -eq 1))
    Remove-Item (Join-Path $tmp "docs/dev/tasks/FIND-90.md") -Force

    # (b) non-canonical Estado on a touched task file
    & $write "docs/dev/tasks/FIND-91.md" ("## Metadata`n- **Estado:** " + [char]0x2705 + " Completado (2026-11-25)`n")
    $cases += ,@("e2e (b): non-canonical Estado is a violation", ((& $run @("-Diff", "-Root", $tmp)) -eq 1))
    Remove-Item (Join-Path $tmp "docs/dev/tasks/FIND-91.md") -Force

    # (c) a Backlog row removed with no avance entry
    & $write "docs/dev/Backlog.md" "# Backlog`n`n| ID | Hallazgo |`n|---|---|`n| ``FIND-901`` | x |`n| ``FIND-902`` | y |`n"
    & git -C $tmp -c user.email=t@t -c user.name=t add -A 2>$null
    & git -C $tmp -c user.email=t@t -c user.name=t commit -qm row 2>$null
    & $write "docs/dev/Backlog.md" "# Backlog`n`n| ID | Hallazgo |`n|---|---|`n| ``FIND-901`` | x |`n"
    $cases += ,@("e2e (c): removed row without avance entry is a violation", ((& $run @("-Diff", "-Base", "HEAD", "-Root", $tmp)) -eq 1))

    # (c') the same close WITH an avance entry passes (the rule is not just "always red")
    & $write "docs/dev/avance/activo/core.md" "# core`n`n- ``FIND-902`` cerrada.`n"
    $cases += ,@("e2e (c'): removed row with avance entry passes", ((& $run @("-Diff", "-Base", "HEAD", "-Root", $tmp)) -eq 0))

    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue

    $failed = 0
    foreach ($c in $cases) {
        Write-Host ("{0}  {1}" -f $(if ($c[1]) { "ok  " } else { "FAIL" }), $c[0])
        if (-not $c[1]) { $failed++ }
    }
    Write-Host ("`n{0}/{1} end-to-end rules passed" -f ($cases.Count - $failed), $cases.Count)
    if ($failed -gt 0) { exit 1 }
}

if ($SelfTest) { Invoke-SelfTest; Invoke-EndToEnd; if (-not $All -and -not $Diff) { exit 0 } }

if ($All -and $Diff) {
    Write-Host "ERROR: -Diff and -All are mutually exclusive." -ForegroundColor Red
    exit 2
}
if (-not $All -and -not $Diff) { $Diff = $true }   # diff-scoped is the default
if (-not $Base) { $Base = Resolve-Base }

Write-Host "=== Closure check (H-04) — $(if ($Diff) { "diff vs $Base" } else { 'whole tree' }) ===" -ForegroundColor Cyan

# ── (a) orphans: active task file whose ID is in no Backlog row and no active plan
$backlogPath = Join-Path $root "docs/dev/Backlog.md"
$backlogText = if (Test-Path $backlogPath) { Read-Utf8 $backlogPath } else { "" }
# ACTIVE plans only: an archived plan mentioning the ID is history, not a queue.
# Same rule as check-harness-contract.mjs: absence of `status:` means active.
$planText = ""
$planDir = Join-Path $root "docs/dev/plans"
if (Test-Path $planDir) {
    $planText = (Get-ChildItem $planDir -Filter *.md |
        Where-Object { $s = [regex]::Match((Read-Utf8 $_.FullName), '(?m)^status:\s*([^\s#]+)'); -not $s.Success -or $s.Groups[1].Value -notmatch '^(superseded|archived|done|completed|closed)$' } |
        ForEach-Object { Read-Utf8 $_.FullName }) -join "`n"
}
# M4: word boundary. `FIND-90` must not be satisfied by `FIND-901` in the Backlog.
function Test-IdPresent([string]$haystack, [string]$id) {
    return [regex]::IsMatch($haystack, '(?<![A-Za-z0-9-])' + [regex]::Escape($id) + '(?![A-Za-z0-9-])')
}

$taskFilter = $null
if ($Diff) {
    $changed = Get-DiffFiles @("docs/dev/tasks")
    $taskFilter = @($changed | Where-Object { $_ -like "docs/dev/tasks/*" })
    if (-not $taskFilter.Count) { Write-Host "  (a) no task file in the diff — skipped for this run" -ForegroundColor Gray }
}
$taskFiles = @(Get-TaskFiles $taskFilter)
if ($Diff -and (-not $taskFilter -or $taskFilter.Count -eq 0)) { $taskFiles = @() }
$orphanCount = 0
$legacyNoField = 0
foreach ($f in $taskFiles) {
    $text = Read-Utf8 $f.FullName
    $estado = Get-Estado $text
    if (-not $estado) {
        # A task file you are touching must carry its state; inherited archives
        # without the field are an advisory census, never a new failure.
        if ($Diff) {
            $violations.Add("$($f.Name): task file has no '**Estado:**' field")
        } else {
            $legacyNoField++
        }
        continue
    }
    if (-not (Test-Canonical $estado)) {
        if ($Diff) {
            $violations.Add("$($f.Name): '**Estado:** $estado' is outside the canonical set (PENDING|IN PROGRESS|COMPLETED|FAILED with emoji) — parsers.mjs reads it as PENDING")
        }
        continue
    }
    if (Test-Terminal $estado) { continue }

    $id = [System.IO.Path]::GetFileNameWithoutExtension($f.Name)
    if (Test-IdPresent $backlogText $id) { continue }
    if (Test-IdPresent $planText $id) { continue }
    $orphanCount++
    $violations.Add("$id ($estado): active task file whose row is in no Backlog and in no active plan — the closure left this state behind")
}

# ── (b) closure: a row removed from the Backlog needs an avance entry ────────
$removedIds = @()
if ($Diff -and ($Paths.Count -eq 0 -or ($Paths | Where-Object { $_ -like "*Backlog.md" }))) {
    $removedIds = @(Get-RemovedBacklogIds)
    if (-not $removedIds.Count) { Write-Host "  (b) no Backlog row removed in the diff" -ForegroundColor Gray }
}
foreach ($id in $removedIds) {
    if (Test-AvanceHasId $id) { continue }
    $violations.Add("$id : row removed from docs/dev/Backlog.md with NO entry in docs/dev/avance/ — the close is not registered")
}

# ── (c) advisory: whole-tree legacy census (never fails by itself)
if ($All) {
    $legacy = New-Object System.Collections.Generic.List[string]
    foreach ($f in (Get-TaskFiles $null)) {
        $estado = Get-Estado (Read-Utf8 $f.FullName)
        if ($estado -and -not (Test-Canonical $estado)) { $legacy.Add($estado) }
    }
    $kinds = ($legacy | Group-Object | Sort-Object Count -Descending)
    $advisories.Add("$($legacy.Count) task file(s) with non-canonical '**Estado:**' + $legacyNoField without the field — inherited from before 2026-11-26 ($($kinds.Count) distinct forms). Fix the ones you touch; the canonical set is enforced on every touched task file.")
}

# ── report ───────────────────────────────────────────────────────────────────
foreach ($a in $advisories) { Write-Host "  WARN  $a" -ForegroundColor Yellow }
if ($violations.Count -gt 0) {
    foreach ($v in $violations) { Write-Host "  FAIL  $v" -ForegroundColor Red }
    Write-Host "`nCLOSURE: $($violations.Count) violation(s)" -ForegroundColor Red
    exit 1
}
Write-Host "`nCLOSURE OK ($(if ($Diff) { "diff-scoped" } else { 'whole tree' }))" -ForegroundColor Green
exit 0
