<#
.SYNOPSIS
  Creates a timestamped, verified git bundle backup of this repository.
.DESCRIPTION
  Local continuity tool for the no-push workflow (AGENTS.md Regla 7: push is an
  explicit owner action). Packs every ref into one bundle, verifies it
  fail-closed with `git bundle verify`, and prunes old bundles beyond -Keep.

  The script is 100% local: it never runs `git push` and never touches remotes.
  Restore with:
    git clone <bundle> <dir>
    git fetch <bundle> develop:refs/heads/develop
.PARAMETER Dest
  Destination directory. Default: $HOME\VantaDB-Backups (outside the repo).
  Prefer an external or synced private folder: a backup on the same disk as the
  repo shares its single point of failure (plan HARD-03 pre-mortem F1).
.PARAMETER Keep
  Number of bundles to keep in Dest (newest first). Default: 7.
.EXAMPLE
  pwsh scripts\git-backup.ps1
.EXAMPLE
  pwsh scripts\git-backup.ps1 -Dest D:\VantaDB-Backups -Keep 14
#>

[CmdletBinding()]
param(
  [string]$Dest = (Join-Path $HOME 'VantaDB-Backups'),
  [int]$Keep = 7
)

$ErrorActionPreference = 'Stop'
# git writes progress to stderr; don't let an exit-0 native call abort the backup.
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
  $PSNativeCommandUseErrorActionPreference = $false
}

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$sep = [System.IO.Path]::DirectorySeparatorChar

function Stop-Backup([string]$Message) {
  Write-Host "❌ $Message" -ForegroundColor Red
  exit 1
}

# ── Preconditions ────────────────────────────────────────────────
if (-not (Get-Command git -ErrorAction SilentlyContinue)) { Stop-Backup 'git not found on PATH.' }
& git -C $root rev-parse --git-dir *> $null
if ($LASTEXITCODE -ne 0) { Stop-Backup "Not a git repository: $root" }

# ── Destination ──────────────────────────────────────────────────
try { $destFull = [System.IO.Path]::GetFullPath($Dest) }
catch { Stop-Backup "Invalid destination path: $Dest — $($_.Exception.Message)" }

# SECURITY: the bundle carries the full history — inside the repo tree only if git-ignored.
if (($destFull + $sep).StartsWith($root + $sep, [System.StringComparison]::OrdinalIgnoreCase)) {
  & git -C $root check-ignore --quiet -- $destFull
  if ($LASTEXITCODE -ne 0) {
    Stop-Backup "Destination is inside the repo and not git-ignored: $destFull — use a path outside the repo (or add it to .gitignore)."
  }
  Write-Warning "Destination is inside the repo tree (git-ignored): $destFull"
}

# Pre-mortem F1: same volume as the repo = single point of failure (non-blocking).
if ([System.IO.Path]::GetPathRoot($destFull) -eq [System.IO.Path]::GetPathRoot($root)) {
  Write-Warning "Same volume as the repo — a disk failure loses both. Prefer an external or synced private folder."
}

if (-not (Test-Path $destFull)) {
  try {
    New-Item -ItemType Directory -Path $destFull -Force | Out-Null
    Write-Host "📁 Created destination: $destFull"
  } catch {
    Stop-Backup "Cannot create destination directory: $destFull — $($_.Exception.Message)"
  }
}
if ($Keep -lt 1) { $Keep = 1 }

# ── Create (timestamped, fail-closed: never overwrite) ───────────
# ponytail: full `--all HEAD` bundle per run (simple + restorable); incremental
# bundles only if total size ever becomes a problem.
$stamp = Get-Date -Format 'yyyyMMdd-HHmm'
$bundle = Join-Path $destFull "vantadb-$stamp.bundle"
if (Test-Path $bundle) {
  Stop-Backup "Bundle already exists for this minute: $bundle — wait a minute or delete it (no silent overwrite)."
}

Write-Host "📦 Creating bundle: $bundle"
& git -C $root bundle create $bundle --all HEAD
if ($LASTEXITCODE -ne 0) { Stop-Backup "git bundle create failed (exit $LASTEXITCODE) — no backup written." }

# ── Verify (fail-closed) ─────────────────────────────────────────
$verifyOut = (& git -C $root bundle verify $bundle 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0) {
  Write-Host $verifyOut
  Stop-Backup "git bundle verify FAILED — bundle is NOT trusted: $bundle"
}

# ── Summary ──────────────────────────────────────────────────────
$sizeMB = [math]::Round((Get-Item $bundle).Length / 1MB, 2)
$refCount = (& git -C $root bundle list-heads $bundle | Measure-Object -Line).Lines
$localCommits = (& git -C $root rev-list --count origin/develop..develop 2>$null | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { $localCommits = 'n/a (origin/develop not found)' }

Write-Host ""
Write-Host "✅ Backup verified: $bundle" -ForegroundColor Green
Write-Host "   Size: $sizeMB MB · Refs: $refCount · Local commits vs origin/develop: $localCommits"
Write-Host "   Restore: git clone `"$bundle`" <dir>  |  git fetch `"$bundle`" develop:refs/heads/develop"

# ── Retention: keep the newest -Keep bundles, prune the rest ─────
Get-ChildItem -Path $destFull -Filter 'vantadb-*.bundle' -File |
  Sort-Object Name -Descending |
  Select-Object -Skip $Keep |
  ForEach-Object { Remove-Item $_.FullName -Force; Write-Host "🗑️  Pruned old bundle: $($_.Name)" }

exit 0
