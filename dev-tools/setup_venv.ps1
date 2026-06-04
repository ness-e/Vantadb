#Requires -Version 5.1
<#
.SYNOPSIS
    Initialize the hermetic Python audit venv at target/audit-venv and install vantadb-python in develop mode.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File dev-tools/setup_venv.ps1
#>
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$VenvDir = Join-Path $RepoRoot "target\audit-venv"
$PythonDir = Join-Path $RepoRoot "vantadb-python"
$VenvPython = Join-Path $VenvDir "Scripts\python.exe"

Write-Host "VantaDB audit venv setup" -ForegroundColor Cyan
Write-Host "  Repo: $RepoRoot"
Write-Host "  Venv: $VenvDir"

if (-not (Test-Path $VenvDir)) {
    Write-Host "Creating virtual environment..." -ForegroundColor Yellow
    python -m venv $VenvDir
    if ($LASTEXITCODE -ne 0) { throw "Failed to create venv at $VenvDir" }
} else {
    Write-Host "Reusing existing virtual environment." -ForegroundColor DarkGray
}

Write-Host "Installing pip, wheel, maturin, pytest..." -ForegroundColor Yellow
& $VenvPython -m pip install --upgrade pip wheel maturin pytest --quiet
if ($LASTEXITCODE -ne 0) { throw "Failed to install Python tooling." }

# maturin requires VIRTUAL_ENV to install into this venv (not system Python).
$env:VIRTUAL_ENV = $VenvDir
$prevLocation = Get-Location
try {
    Set-Location $PythonDir
    Write-Host "Building and installing vantadb-python (maturin develop --release)..." -ForegroundColor Yellow
    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $VenvPython -m maturin develop --release *> $null
        if ($LASTEXITCODE -ne 0) { throw "maturin develop failed (exit $LASTEXITCODE)." }
    } finally {
        $ErrorActionPreference = $prevEap
    }
} finally {
    Set-Location $prevLocation
    Remove-Item Env:VIRTUAL_ENV -ErrorAction SilentlyContinue
}

& $VenvPython -c "import vantadb_py; print('import ok:', vantadb_py.__name__)"
if ($LASTEXITCODE -ne 0) { throw "vantadb_py import check failed after maturin develop." }

Write-Host ""
Write-Host "Audit venv ready." -ForegroundColor Green
Write-Host "  Python: $VenvPython"
Write-Host "  Activate: $($VenvDir)\Scripts\Activate.ps1"
Write-Host "  Test:     $VenvPython -m pytest $PythonDir\tests\test_sdk.py -v"
