# Download benchmark datasets (GloVe-100)
$ErrorActionPreference = "Stop"

$dataDir = "data/benchmark"
$gloveZip = Join-Path $dataDir "glove.6B.zip"
$gloveTxt = Join-Path $dataDir "glove.6B.100d.txt"

if (!(Test-Path $dataDir)) {
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
}

# Download GloVe-100
if (!(Test-Path $gloveTxt)) {
    if (!(Test-Path $gloveZip)) {
        Write-Host "Downloading GloVe-100 (glove.6B.zip)..." -ForegroundColor Cyan
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://nlp.stanford.edu/data/glove.6B.zip" -OutFile $gloveZip -UseBasicParsing
    }
    Write-Host "Extracting glove.6B.100d.txt..." -ForegroundColor Cyan
    Expand-Archive -Path $gloveZip -DestinationPath $dataDir -Force
}

$vecCount = (Get-Content $gloveTxt | Measure-Object -Line).Lines
Write-Host "GloVe-100: $vecCount vectors (expected 400000)" -ForegroundColor Green
if ($vecCount -lt 1000) {
    Write-Host "ERROR: GloVe file too small ($vecCount lines)" -ForegroundColor Red
    exit 1
}

Write-Host "All datasets ready." -ForegroundColor Green
