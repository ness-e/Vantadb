# VantaDB Heavy Nocturnal Certification Suite
# Runs all intensive, long-running validation tests locally and logs results.
$ErrorActionPreference = "Continue" # Allow the script to continue if one test fails so we have complete nocturnal telemetry

# Auto-resolve project root to ensure it runs correctly from any CWD
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$LogFile = "heavy_nocturnal_tests.log"
Set-Content -Path $LogFile -Value "=================================================="
Add-Content -Path $LogFile -Value "   VantaDB Heavy Nocturnal Certification Log      "
Add-Content -Path $LogFile -Value "=================================================="
Add-Content -Path $LogFile -Value "Fecha de Inicio : $(Get-Date)"
Add-Content -Path $LogFile -Value "Directorio Raíz  : $ProjectRoot"
Add-Content -Path $LogFile -Value "--------------------------------------------------`n"

function Write-Header($Title) {
    Write-Host "`n=== $Title ===" -ForegroundColor Cyan
    Add-Content -Path $LogFile -Value "`n=== $Title ==="
    Add-Content -Path $LogFile -Value "Hora: $(Get-Date)`n"
}

function Run-Command($Name, [string[]]$ArgList) {
    Write-Host "`nRunning: $Name..." -ForegroundColor Yellow
    Add-Content -Path $LogFile -Value "Running: $Name..."

    # Ejecuta el comando, redirige stderr a stdout, y hace un streaming simultáneo al log y a la consola
    & $ArgList[0] ($ArgList | Select-Object -Skip 1) 2>&1 | Tee-Object -FilePath $LogFile -Append

    if ($LASTEXITCODE -ne 0) {
        Write-Host "`n[FAILED] $Name (exit code $LASTEXITCODE)" -ForegroundColor Red
        Add-Content -Path $LogFile -Value "`n[FAILED] $Name (exit code $LASTEXITCODE)"
    } else {
        Write-Host "[PASSED] $Name" -ForegroundColor Green
        Add-Content -Path $LogFile -Value "[PASSED] $Name"
    }
    Add-Content -Path $LogFile -Value "--------------------------------------------------"
}

try {
    Write-Host "=============================================" -ForegroundColor Cyan
    Write-Host "   VantaDB Nocturnal Certification Suite     " -ForegroundColor Cyan
    Write-Host "=============================================" -ForegroundColor Cyan
    Write-Host "Toda la salida se transmitirá en pantalla y se guardará en '$LogFile'" -ForegroundColor Gray

    # Limpiar RUSTFLAGS para asegurar optimización release nativa y evitar stack overflows
    $env:RUSTFLAGS = $null

    # 1. Maturin Release Build (Bindings)
    Write-Header "1. Building Python SDK Bindings (Release Mode)"
    if (Get-Command "maturin" -ErrorAction SilentlyContinue) {
        Run-Command "Maturin Develop Release" @("maturin", "develop", "--manifest-path", "./vantadb-python/Cargo.toml", "--release")
    } else {
        Write-Host "WARNING: 'maturin' executable not found in PATH." -ForegroundColor Yellow
        Add-Content -Path $LogFile -Value "WARNING: 'maturin' executable not found in PATH. Skipping Python build check."
    }

    # 2. Python SDK Benchmark
    Write-Header "2. VantaDB Python SDK Benchmark (10K / 128d / 1000 q)"
    $PythonExe = ".venv\Scripts\python.exe"
    if (Test-Path $PythonExe) {
        Run-Command "Python Benchmark Suite" @($PythonExe, "benchmarks/vantadb_local_bench.py", "--size", "10000", "--dim", "128", "--queries", "1000", "--output", "benchmarks/vanta_benchmark_report.json")
    } else {
        Write-Host "WARNING: Python virtual environment not found at $PythonExe." -ForegroundColor Yellow
        Add-Content -Path $LogFile -Value "WARNING: Python venv not found at $PythonExe. Skipping Python benchmark."
    }

    # 3. Rust Stress Protocol
    Write-Header "3. Core Stress Protocol (7-Block Certification)"
    Run-Command "Stress Protocol" @("cargo", "test", "--release", "--test", "stress_protocol", "--", "--nocapture", "--test-threads=1")

    # 4. HNSW Structural Validation
    Write-Header "4. HNSW Structural Integrity & Connections"
    Run-Command "HNSW Structural Validation" @("cargo", "test", "--release", "--test", "hnsw_validation", "--", "--nocapture", "--test-threads=1")

    # 5. HNSW Recall Validation
    Write-Header "5. HNSW Recall & Accuracy Certification"
    Run-Command "HNSW Recall Certification" @("cargo", "test", "--release", "--test", "hnsw_recall_certification", "--", "--nocapture", "--test-threads=1")

    # 6. WAL Resilience Simulation
    Write-Header "6. Crash-Safe WAL Resilience Tests"
    Run-Command "WAL Resilience Tests" @("cargo", "test", "--release", "--test", "wal_resilience", "--", "--nocapture", "--test-threads=1")

    # 7. Chaos Failpoint Injections
    Write-Header "7. Chaos Integrity (Injections & Corruptions)"
    Run-Command "Chaos Integrity (Failpoints)" @("cargo", "test", "--release", "--test", "chaos_integrity", "--features", "failpoints", "--", "--nocapture", "--test-threads=1")

    # Finalización
    Write-Header "Certification Completed Successfully"
    Add-Content -Path $LogFile -Value "`nFecha de Finalización: $(Get-Date)"
    
    Write-Host "`n=============================================" -ForegroundColor Green
    Write-Host "  NOCTURNAL CERTIFICATION PIPELINE FINISHED   " -ForegroundColor Green
    Write-Host "  Review results in: $LogFile                 " -ForegroundColor Green
    Write-Host "=============================================" -ForegroundColor Green

} catch {
    Write-Host "`nAn unexpected error occurred: $_" -ForegroundColor Red
    Add-Content -Path $LogFile -Value "Fatal Script Error: $_"
    exit 1
}
