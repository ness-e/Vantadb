# clean-opencode-loop.ps1 — Remove stale corrupt/tmp session files from .opencode/opencode-loop/
#
# The opencode-loop plugin (ByBrawe) writes session state atomically:
#   - writeState: writes `ses_*.json.<pid>.<ts>.tmp` then renames to `ses_*.json`
#     (if the process dies between writeFile and rename, the .tmp is left behind)
#   - readState: copies an unparseable file to `ses_*.json.corrupt-<ts>` as a backup
#
# This script removes ONLY those leftovers. Live sessions (ses_*.json), loop.log
# and goals/ are never touched. The plugin itself runs this same GC at init
# (cleanupLoopStateDir in ~/.config/opencode/plugins/opencode-loop.ts); this
# script is the manual fallback / verification tool.
#
# Usage:
#   .\dev-tools\clean-opencode-loop.ps1            # dry run (counts only)
#   .\dev-tools\clean-opencode-loop.ps1 -Apply     # actually remove
#
# Idempotent: running twice with -Apply is a no-op the second time.

param(
  [switch]$Apply
)

$dir = Join-Path $PSScriptRoot "..\.opencode\opencode-loop"
$dir = [System.IO.Path]::GetFullPath($dir)

if (-not (Test-Path -LiteralPath $dir)) {
  Write-Host "State dir not found: $dir (nothing to clean)"
  exit 0
}

$targets = Get-ChildItem -LiteralPath $dir -File | Where-Object {
  $_.Name -match '\.corrupt-[\d]+$' -or $_.Name -match '\.tmp$'
}

$count = @($targets).Count
Write-Host "corrupt/tmp found: $count"

if (-not $Apply) {
  Write-Host "Dry run — pass -Apply to remove."
  exit 0
}

foreach ($t in $targets) {
  Remove-Item -LiteralPath $t.FullName -Force
  Write-Host "removed: $($t.Name)"
}

$residual = @(Get-ChildItem -LiteralPath $dir -File | Where-Object {
  $_.Name -match '\.corrupt-[\d]+$' -or $_.Name -match '\.tmp$'
}).Count
Write-Host "residual corrupt/tmp after cleanup: $residual"
$live = @(Get-ChildItem -LiteralPath $dir -File | Where-Object { $_.Name -match '^ses_.*\.json$' }).Count
Write-Host "live sessions untouched: $live"