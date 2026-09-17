# VantaDB installer for Windows PowerShell.
# Downloads the release zip and extracts vanta-cli.exe to $HOME/.vanta/bin,
# then chains to the interactive setup wizard (FIND-104).
#
# One-liner (no clone, no rustup):
#   irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex
# Flags: -DryRun (simulate, no effects) -NoWizard (skip chain)
#        -WizardNonInteractive (wizard with defaults)
# Trust: TLS 1.2 + official repo URL + sha256 of the payload verified
# in-script when the .sha256 asset exists (warn-and-continue otherwise).
# To verify manually, download the file and compare its hash
# against the published .sha256 asset before piping to iex.
param(
  [switch]$NoWizard,
  [switch]$DryRun,
  [switch]$WizardNonInteractive
)

$ErrorActionPreference = "Stop"

$installDir = "$HOME\.vanta\bin"
$binaryName = "vanta-cli.exe"
$wizardFile = "setup-embeddings.ps1"

if ($DryRun) {
    Write-Host "[dry-run] install.ps1 -DryRun: no changes, no network." -ForegroundColor Cyan
    Write-Host "[dry-run] would: fetch latest tag (api.github.com/repos/ness-e/Vantadb/releases/latest)"
    Write-Host "[dry-run] would: download vantadb-x86_64-pc-windows-msvc.zip + .sha256, verify checksum"
    Write-Host "[dry-run] would: backup $installDir\$binaryName (.bak-<stamp>) + install"
    if (-not $NoWizard) {
        $wMode = $WizardNonInteractive ? " with -NonInteractive" : " (interactive)"
        Write-Host "[dry-run] would: chain wizard $wizardFile (<latest-tag>/$wizardFile)$wMode, else print manual next-step" -ForegroundColor Cyan
    } else {
        Write-Host "[dry-run] would: skip wizard (-NoWizard)"
    }
    Write-Host "[dry-run] chain OK: installer -> wizard (simulated, exit 0)" -ForegroundColor Green
    exit 0
}

# Create destination folder
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Write-Host "🔍 Fetching latest VantaDB release version..." -ForegroundColor Cyan

$latestRelease = $null
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/ness-e/Vantadb/releases/latest" -UseBasicParsing
    $latestRelease = $releases.tag_name
} catch {
    $latestRelease = "v0.4.0"
    Write-Host "⚠️ Could not fetch latest release via API. Falling back to v0.4.0 (pin may skew vs wizard; prefer -Version <tag> explicitly)" -ForegroundColor Yellow
}

$zipName = "vantadb-x86_64-pc-windows-msvc.zip"
$downloadUrl = "https://github.com/ness-e/Vantadb/releases/download/$latestRelease/$zipName"
$checksumUrl = "$downloadUrl.sha256"

Write-Host "📥 Downloading VantaDB CLI ($latestRelease) for Windows..." -ForegroundColor Cyan

$tmpDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
$zipPath = Join-Path $tmpDir $zipName

try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
} catch {
    Write-Host "❌ Failed to download from $downloadUrl" -ForegroundColor Red
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    Exit 1
}

# Verify checksum
try {
    $expected = (Invoke-RestMethod -Uri $checksumUrl -UseBasicParsing).Split()[0]
    $computed = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $computed) {
        Write-Host "❌ Checksum mismatch!" -ForegroundColor Red
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
        Exit 1
    }
    Write-Host "✅ Checksum verified" -ForegroundColor Green
} catch {
    Write-Host "⚠️ No checksum file at $checksumUrl — skipping verification" -ForegroundColor Yellow
}

# Extract vanta-cli.exe from zip
Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

# Idempotency (FIND-105 pre-mortem #2): back up any previous binary.
$existing = Join-Path $installDir $binaryName
if (Test-Path $existing) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    Copy-Item $existing "$existing.bak-$stamp" -Force
    Write-Host "💾 Backed up previous binary to $binaryName.bak-$stamp" -ForegroundColor Cyan
}
Copy-Item (Join-Path $tmpDir "release\$binaryName") $installDir -Force

Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

Write-Host "✨ VantaDB CLI successfully installed to $installDir\$binaryName" -ForegroundColor Green

# --- Wizard chain (FIND-105): installer -> setup-embeddings interactivo ---
$wizardUrl = "https://raw.githubusercontent.com/ness-e/Vantadb/$latestRelease/$wizardFile"
if ($NoWizard) {
    Write-Host ""
    Write-Host "⏭️  Wizard skipped (-NoWizard). Run later:" -ForegroundColor Cyan
    Write-Host "   pwsh $wizardFile -NonInteractive" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "🧙 Chaining to the interactive setup wizard (FIND-104)..." -ForegroundColor Cyan
    $wizardArgs = @()
    if ($WizardNonInteractive) { $wizardArgs += '-NonInteractive' }
    try {
        $wizardTmp = Join-Path ([System.IO.Path]::GetTempPath()) ("vanta-wizard-" + [System.Guid]::NewGuid().ToString() + ".ps1")
        Invoke-WebRequest -Uri $wizardUrl -OutFile $wizardTmp -UseBasicParsing
        & pwsh -NoProfile -File $wizardTmp @wizardArgs
        Remove-Item -Force $wizardTmp -ErrorAction SilentlyContinue
        Write-Host "✅ Wizard completed" -ForegroundColor Green
    } catch {
        Write-Host "⚠️ Wizard did not run — CLI is installed; run manually:" -ForegroundColor Yellow
        Write-Host "   pwsh $wizardFile -NonInteractive" -ForegroundColor Yellow
    }
}
Write-Host ""
Write-Host "💡 To use it immediately, add it to your PATH for this session:" -ForegroundColor Cyan
Write-Host "   `$env:Path += ';$installDir'" -ForegroundColor Yellow
Write-Host ""
Write-Host "To make this change permanent for your user account, run:" -ForegroundColor Cyan
Write-Host "   [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$installDir', 'User')" -ForegroundColor Yellow
Write-Host "   (Note: You will need to restart your terminal for this permanent change to take effect)" -ForegroundColor Yellow
