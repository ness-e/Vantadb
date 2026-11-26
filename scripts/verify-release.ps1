# verify-release.ps1 — post-release verification for VantaDB (DIST-05).
#
# Checks that a GitHub release is actually COMPLETE: every binary asset and
# wheel is attached to the release, and every published version is live on its
# registry (crates.io / PyPI / npm). With -Smoke it also downloads the current
# platform's archive, verifies its sha256 and runs `vanta-cli --version`.
#
# Usage:
#   pwsh scripts/verify-release.ps1                    # verify the latest release
#   pwsh scripts/verify-release.ps1 -Tag v0.9.0 -Smoke # verify a tag + run the binary
#
# Exit codes: 0 = ALL GREEN, 1 = at least one artifact missing, 2 = bad input.
#
# Known gaps (documented in docs/dev/workflow/PUBLISH.md, Post-release verification):
#   - releases <= v0.7.0 ship no binary assets (pre-`RELEASE_PLZ_TOKEN` cascade);
#   - `vanta-memory` is on hold in release-plz (DIST-01) — not checked;
#   - `vantadb-node` was never published to npm — not checked.
#
# Requires PowerShell 5.1+ / 7+ (no external modules). GH_TOKEN is optional
# (raises the GitHub API rate limit; the repo is public — the token is only
# ever sent to api.github.com, never to the registries).
#
# Maintenance: the expected asset/registry list lives in
# `release-manifest.json` (single source of truth, FIND-342) — update it when
# the build matrix changes. This script renders from the manifest, never
# hardcodes the lists.
param(
  [string]$Tag,
  [switch]$Smoke,
  [string]$Repo = "ness-e/Vantadb"
)

$ErrorActionPreference = "Stop"
$script:fail = 0

function Write-Ok([string]$m)   { Write-Host "  OK   $m" -ForegroundColor Green }
function Write-Bad([string]$m)  { Write-Host "  FAIL $m" -ForegroundColor Red; $script:fail++ }
function Write-Skip([string]$m) { Write-Host "  SKIP $m" -ForegroundColor Yellow }

function Invoke-Api([string]$url) {
  $headers = @{ "User-Agent" = "vantadb-verify-release" }
  if ($env:GH_TOKEN -and $url -like "https://api.github.com/*") {
    $headers["Authorization"] = "Bearer $env:GH_TOKEN"
  }
  return Invoke-RestMethod -Uri $url -Headers $headers -UseBasicParsing
}

# --- Resolve tag -------------------------------------------------------------
if (-not $Tag) {
  try { $Tag = (Invoke-Api "https://api.github.com/repos/$Repo/releases/latest").tag_name }
  catch {
    Write-Host "FAIL could not resolve the latest release: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
  }
}
if ($Tag -notmatch '^v(\d+\.\d+\.\d+)$') {
  Write-Host "FAIL invalid tag '$Tag' (expected vX.Y.Z)" -ForegroundColor Red
  exit 2
}
$Ver = $Matches[1]
Write-Host "Verifying release $Tag (version $Ver) - $Repo" -ForegroundColor Cyan
Write-Host ""

# --- 0. Expected list: single source of truth (FIND-342) ---------------------
# release-manifest.json owns the channel->name->version map. Fail closed when
# it cannot be read: a hardcoded fallback would be a second mirrored list.
$ManifestPath = Join-Path $PSScriptRoot ".." "release-manifest.json"
try { $manifest = Get-Content -Raw $ManifestPath | ConvertFrom-Json }
catch {
  Write-Host "FAIL cannot read release-manifest.json ($ManifestPath): $($_.Exception.Message)" -ForegroundColor Red
  exit 2
}

# --- 1. GitHub release: binary assets + wheels --------------------------------
Write-Host "GitHub release assets"
try { $release = Invoke-Api "https://api.github.com/repos/$Repo/releases/tags/$Tag" }
catch {
  Write-Host "FAIL release $Tag not found: $($_.Exception.Message)" -ForegroundColor Red
  exit 1
}
$assetNames = @($release.assets | ForEach-Object { $_.name })

foreach ($b in $manifest.github.binaries) {
  $archive = "vantadb-$($b.target).$($b.ext)"
  if ($assetNames -contains $archive) { Write-Ok $archive } else { Write-Bad "$archive missing" }
  if ($assetNames -contains "$archive.sha256") { Write-Ok "$archive.sha256" } else { Write-Bad "$archive.sha256 missing" }
}
foreach ($b in $manifest.github.binaries_documented_only) {
  Write-Skip "binary $($b.target): listed in the manifest but not built by the pipeline (matrix reduced pre-pub)"
}
foreach ($p in $manifest.github.wheels.platforms) {
  $wheel = ($manifest.github.wheels.template -replace '\{version\}', $Ver) -replace '\{platform\}', $p
  if ($assetNames -contains $wheel) { Write-Ok $wheel } else { Write-Bad "$wheel missing" }
}
foreach ($d in $manifest.documented_only) { Write-Skip "$($d.label): $($d.reason)" }

# --- 2. Registries -------------------------------------------------------------
Write-Host ""
Write-Host "Registries"
function Test-Artifact([string]$label, [string]$url) {
  try { Invoke-Api $url | Out-Null; Write-Ok "$label $Ver is live" }
  catch { Write-Bad "$label $Ver NOT live ($url)" }
}
foreach ($r in $manifest.registries) {
  Test-Artifact $r.label ($r.url_template -replace '\{version\}', $Ver)
}

# --- 3. Smoke: current platform's archive -> sha256 -> run ----------------------
if ($Smoke) {
  Write-Host ""
  Write-Host "Smoke (current platform)"

  $isWin = $true; $isLin = $false; $isMac = $false
  if (Get-Variable IsWindows -ErrorAction SilentlyContinue) {
    $isWin = $IsWindows; $isLin = $IsLinux; $isMac = $IsMacOS
  }
  $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
  $target = $null
  if ($isWin -and $arch -eq "X64") { $target = "x86_64-pc-windows-msvc" }
  elseif ($isLin -and $arch -eq "X64") { $target = "x86_64-unknown-linux-gnu" }
  elseif ($isLin -and $arch -eq "Arm64") { $target = "aarch64-unknown-linux-gnu" }
  elseif ($isMac -and $arch -eq "X64") { $target = "x86_64-apple-darwin" }
  elseif ($isMac -and $arch -eq "Arm64") { $target = "aarch64-apple-darwin" }

  if (-not $target) {
    Write-Skip "no released asset for this platform (win=$isWin lin=$isLin mac=$isMac arch=$arch)"
  } else {
    $isZip = $false; $ext = "tar.gz"
    if ($isWin) { $isZip = $true; $ext = "zip" }
    $assetFile = "vantadb-$target.$ext"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("vanta-verify-" + [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    try {
      $dlBase = "https://github.com/$Repo/releases/download/$Tag"
      Invoke-WebRequest -Uri "$dlBase/$assetFile" -OutFile (Join-Path $tmp $assetFile) -UseBasicParsing
      $expectedRaw = (Invoke-WebRequest -Uri "$dlBase/$assetFile.sha256" -UseBasicParsing).Content
      if ($expectedRaw -is [byte[]]) { $expectedRaw = [System.Text.Encoding]::UTF8.GetString($expectedRaw) }
      $expected = ("$expectedRaw" -split '\s+')[0].Trim().ToLower()
      $computed = (Get-FileHash (Join-Path $tmp $assetFile) -Algorithm SHA256).Hash.ToLower()
      $hashOk = $false
      if ($expected -eq $computed) { Write-Ok "sha256 verified ($assetFile)"; $hashOk = $true }
      else { Write-Bad "sha256 mismatch ($assetFile): expected $expected, got $computed" }

      if ($hashOk) {
        if ($isZip) {
          Expand-Archive -Path (Join-Path $tmp $assetFile) -DestinationPath $tmp -Force
          $binName = "vanta-cli.exe"
        } else {
          & tar -xzf (Join-Path $tmp $assetFile) -C $tmp
          $binName = "vanta-cli"
        }
        # The archive layout is flat today (vanta-cli at the root); older/newer
        # packagers may nest it under release/ — resolve either way.
        $binItem = Get-ChildItem -Path $tmp -Recurse -Filter $binName -File | Select-Object -First 1
        if (-not $binItem) { throw "binary $binName not found in $assetFile (unexpected archive layout)" }
        $bin = $binItem.FullName
        if (-not $isWin) { & chmod +x $bin }
        # Never hand the API token to the child process (defense in depth).
        $savedToken = $env:GH_TOKEN
        $env:GH_TOKEN = $null
        try { $out = (& $bin --version 2>&1 | Out-String).Trim() }
        finally { $env:GH_TOKEN = $savedToken }
        if ($out -match [regex]::Escape($Ver)) { Write-Ok "vanta-cli --version -> $out" }
        else { Write-Bad "vanta-cli --version does not report $Ver (got: $out)" }
      } else {
        Write-Skip "execution skipped (checksum mismatch - refusing to run the binary)"
      }
    } catch {
      Write-Bad "smoke failed: $($_.Exception.Message)"
    } finally {
      Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
  }
}

# --- Summary --------------------------------------------------------------------
Write-Host ""
if ($script:fail -gt 0) {
  Write-Host "$($script:fail) check(s) FAILED - release $Tag is incomplete." -ForegroundColor Red
  Write-Host "Remediation: docs/dev/workflow/PUBLISH.md (Post-release verification)." -ForegroundColor Yellow
  exit 1
}
Write-Host "ALL GREEN - release $Tag verified (version $Ver)." -ForegroundColor Green
exit 0
