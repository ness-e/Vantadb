# VantaDB installer for Windows PowerShell.
# Downloads the precompiled vanta-cli binary and puts it in $HOME/.vanta/bin

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
    # Resolve security protocol issues for old PowerShell versions
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/ness-e/Vantadb/releases/latest" -UseBasicParsing
    $latestRelease = $releases.tag_name
} catch {
    $latestRelease = "v0.1.4"
    Write-Host "⚠️ Could not fetch latest release via API. Falling back to default version $latestRelease" -ForegroundColor Yellow
}

$downloadUrl = "https://github.com/ness-e/Vantadb/releases/download/$latestRelease/vanta-cli-windows-amd64.exe"
$destPath = Join-Path $installDir $binaryName

Write-Host "📥 Downloading VantaDB CLI ($latestRelease) for Windows..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing
} catch {
    Write-Host "❌ Failed to download binary from $downloadUrl" -ForegroundColor Red
    Exit 1
}

Write-Host "✨ VantaDB CLI successfully installed to $destPath" -ForegroundColor Green
Write-Host ""
Write-Host "💡 To use it immediately, add it to your PATH for this session:" -ForegroundColor Cyan
Write-Host "   `$env:Path += ';$installDir'" -ForegroundColor Yellow
Write-Host ""
Write-Host "To make this change permanent for your user account, run:" -ForegroundColor Cyan
Write-Host "   [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$installDir', 'User')" -ForegroundColor Yellow
Write-Host "   (Note: You will need to restart your terminal for this permanent change to take effect)" -ForegroundColor Yellow
