# VantaDB installer for Windows PowerShell.
# Downloads the release zip and extracts vanta-cli.exe to $HOME/.vanta/bin

$ErrorActionPreference = "Stop"

$installDir = "$HOME\.vanta\bin"
$binaryName = "vanta-cli.exe"

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
    Write-Host "⚠️ Could not fetch latest release via API. Falling back to v0.4.0" -ForegroundColor Yellow
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
Copy-Item (Join-Path $tmpDir "release\$binaryName") $installDir -Force

Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

Write-Host "✨ VantaDB CLI successfully installed to $installDir\$binaryName" -ForegroundColor Green
Write-Host ""
Write-Host "💡 To use it immediately, add it to your PATH for this session:" -ForegroundColor Cyan
Write-Host "   `$env:Path += ';$installDir'" -ForegroundColor Yellow
Write-Host ""
Write-Host "To make this change permanent for your user account, run:" -ForegroundColor Cyan
Write-Host "   [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$installDir', 'User')" -ForegroundColor Yellow
Write-Host "   (Note: You will need to restart your terminal for this permanent change to take effect)" -ForegroundColor Yellow
