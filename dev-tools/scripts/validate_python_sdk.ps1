#Requires -Version 5.1
<#
.SYNOPSIS
    VantaDB Python SDK Validation — Build wheel, install in clean venv, run pytest.
.PARAMETER SkipBuild
    Skip the maturin build step and use an existing wheel.
.EXAMPLE
    .\dev-tools\validate_python_sdk.ps1
    .\dev-tools\validate_python_sdk.ps1 -SkipBuild
#>
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Definition)
$WheelDir = Join-Path $RepoRoot "target\wheels"
$VenvDir  = Join-Path $RepoRoot ".validate-venv"
$PythonDir = Join-Path $RepoRoot "vantadb-python"

Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  VantaDB Python SDK Validation" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan

# ── Step 1: Build wheel ──────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "▸ Building wheel with maturin..." -ForegroundColor Yellow
    python -m maturin build `
        --manifest-path "$PythonDir\Cargo.toml" `
        --out $WheelDir `
        --release
    if ($LASTEXITCODE -ne 0) { throw "Maturin build failed." }
    Write-Host "  ✓ Wheel built successfully." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "▸ Skipping build (-SkipBuild)." -ForegroundColor DarkGray
}

# ── Step 2: Create clean venv ────────────────────────────────
Write-Host ""
Write-Host "▸ Creating clean virtual environment..." -ForegroundColor Yellow
if (Test-Path $VenvDir) { Remove-Item -Recurse -Force $VenvDir }
python -m venv $VenvDir
& "$VenvDir\Scripts\Activate.ps1"
python -m pip install --upgrade pip pytest --quiet

# ── Step 3: Install wheel ────────────────────────────────────
Write-Host ""
Write-Host "▸ Installing latest wheel..." -ForegroundColor Yellow
$Wheel = Get-ChildItem "$WheelDir\vantadb_py-*.whl" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $Wheel) {
    Write-Host "  ✗ No wheel found in $WheelDir. Run without -SkipBuild." -ForegroundColor Red
    deactivate
    Remove-Item -Recurse -Force $VenvDir -ErrorAction SilentlyContinue
    exit 1
}
python -m pip install --force-reinstall $Wheel.FullName --quiet
Write-Host "  ✓ Installed: $($Wheel.Name)" -ForegroundColor Green

# ── Step 4: Run pytest ───────────────────────────────────────
Write-Host ""
Write-Host "▸ Running Python SDK tests..." -ForegroundColor Yellow
python -m pytest "$PythonDir\tests\test_sdk.py" -v
$TestResult = $LASTEXITCODE

# ── Step 5: Cleanup ──────────────────────────────────────────
Write-Host ""
Write-Host "▸ Cleaning up..." -ForegroundColor DarkGray
deactivate
Remove-Item -Recurse -Force $VenvDir -ErrorAction SilentlyContinue

if ($TestResult -eq 0) {
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Green
    Write-Host "  ✓ Python SDK validation PASSED" -ForegroundColor Green
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Red
    Write-Host "  ✗ Python SDK validation FAILED (exit code: $TestResult)" -ForegroundColor Red
    Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Red
    exit $TestResult
}
