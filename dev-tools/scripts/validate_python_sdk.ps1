#Requires -Version 5.1
<#
.SYNOPSIS
    VantaDB Python SDK Validation - Build wheel, install in audit venv, run pytest.
.PARAMETER SkipBuild
    Skip the maturin build step and use the editable install from setup_venv.ps1.
.EXAMPLE
    .\dev-tools\scripts\validate_python_sdk.ps1
    .\dev-tools\scripts\validate_python_sdk.ps1 -SkipBuild
#>
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$WheelDir = Join-Path $RepoRoot "target\wheels"
$VenvDir = Join-Path $RepoRoot "target\audit-venv"
$PythonDir = Join-Path $RepoRoot "vantadb-python"
$VenvPython = Join-Path $VenvDir "Scripts\python.exe"
$SetupScript = Join-Path $RepoRoot "dev-tools\setup_venv.ps1"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  VantaDB Python SDK Validation" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

Write-Host ""
Write-Host "> Ensuring audit venv (setup_venv.ps1)..." -ForegroundColor Yellow
& $SetupScript
if ($LASTEXITCODE -ne 0) { throw "setup_venv.ps1 failed." }

if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "> Building wheel with maturin..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $WheelDir | Out-Null
    $env:VIRTUAL_ENV = $VenvDir
    try {
        Push-Location $PythonDir
        & $VenvPython -m maturin build --out $WheelDir --release
        if ($LASTEXITCODE -ne 0) { throw "Maturin build failed." }
    } finally {
        Pop-Location
        Remove-Item Env:VIRTUAL_ENV -ErrorAction SilentlyContinue
    }
    Write-Host "  [OK] Wheel built successfully." -ForegroundColor Green

    Write-Host ""
    Write-Host "> Installing wheel into audit venv..." -ForegroundColor Yellow
    $Wheel = Get-ChildItem "$WheelDir\vantadb_py-*.whl" -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $Wheel) {
        throw "No wheel found in $WheelDir after build."
    }
    & $VenvPython -m pip install --force-reinstall $Wheel.FullName --quiet
    Write-Host "  [OK] Installed: $($Wheel.Name)" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "> Skipping wheel build (-SkipBuild); using editable install." -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "> Running Python SDK tests..." -ForegroundColor Yellow
& $VenvPython -c "import vantadb_py"
if ($LASTEXITCODE -ne 0) { throw "vantadb_py import check failed." }

& $VenvPython -m pytest "$PythonDir\tests\test_sdk.py" -v
$TestResult = $LASTEXITCODE

if ($TestResult -eq 0) {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host "  [OK] Python SDK validation PASSED" -ForegroundColor Green
    Write-Host "============================================================" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Red
    Write-Host "  [FAIL] Python SDK validation FAILED (exit code: $TestResult)" -ForegroundColor Red
    Write-Host "============================================================" -ForegroundColor Red
    exit $TestResult
}
