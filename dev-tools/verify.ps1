# VantaDB Pre-Flight Verification Script
# Runs all checks locally before pushing or publishing changes.
$ErrorActionPreference = "Stop"

# Auto-resolve project root to ensure it runs correctly from any CWD
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

function Write-Header($Title) {
    Write-Host "`n=== $Title ===" -ForegroundColor Cyan
}

function Run-Command($Name, [string[]]$ArgList) {
    Write-Host "`nRunning: $Name..." -ForegroundColor Yellow

    # Invoke directly so output streams through to the terminal in real time
    & $ArgList[0] ($ArgList | Select-Object -Skip 1)

    if ($LASTEXITCODE -ne 0) {
        Write-Host "`n[FAILED] $Name (exit code $LASTEXITCODE)" -ForegroundColor Red
        throw "Step '$Name' failed."
    }
    Write-Host "[PASSED] $Name" -ForegroundColor Green
}

try {
    Write-Host "=============================================" -ForegroundColor Cyan
    Write-Host "   VantaDB Pre-Flight Verification (Local)  " -ForegroundColor Cyan
    Write-Host "=============================================" -ForegroundColor Cyan

    # Force opt-level=0 to prevent MSVC stack-overflow during local validation
    $env:RUSTFLAGS = "-C opt-level=0"

    # 1. Rustfmt Check
    Write-Header "Code Formatting Check"
    Run-Command "Format Check" @("cargo", "fmt", "--all", "--", "--check")

    # 2. Cargo Check
    Write-Header "Workspace Compilation"
    Run-Command "Compilation (All Features)" @("cargo", "check", "--workspace", "--tests", "--all-features", "-j", "2")

    # 3. Clippy Lints
    Write-Header "Static Analysis (Clippy)"
    Run-Command "Clippy Lints" @("cargo", "clippy", "--workspace", "--tests", "--all-features", "-j", "2", "--", "-D", "warnings")

    # 4. Security Audit
    Write-Header "Security Auditing"
    Run-Command "Cargo Audit" @("cargo", "audit")

    # 5. Dependency Policy Check
    Write-Header "Dependency Policies"
    Run-Command "Cargo Deny Check" @("cargo", "deny", "check")

    # 6. Workspace Tests (skip long-running / benchmark tests)
    Write-Header "Unit & Integration Tests"
    Run-Command "Rust Tests" @(
        "cargo", "test", "--workspace", "--all-features", "-j", "2", "--",
        "--skip", "benchmark",
        "--skip", "competitive",
        "--skip", "recall",
        "--skip", "sift",
        "--skip", "chaos",
        "--skip", "hnsw_hard_validation",
        "--skip", "stress_protocol"
    )

    # Restore RUSTFLAGS before release build
    $env:RUSTFLAGS = $null

    # 7. Python Bindings — Maturin Build
    Write-Header "Python Bindings (Maturin)"
    Run-Command "Maturin Python SDK Build" @("maturin", "build", "--manifest-path", "./vantadb-python/Cargo.toml", "--release")

    Write-Host "`n=============================================" -ForegroundColor Green
    Write-Host "  SUCCESS: All local checks passed cleanly!  " -ForegroundColor Green
    Write-Host "  You are safe to push/publish your changes. " -ForegroundColor Green
    Write-Host "=============================================" -ForegroundColor Green
    exit 0
}
catch {
    Write-Host "`n=============================================" -ForegroundColor Red
    Write-Host "  ERROR: Verification failed during checks.  " -ForegroundColor Red
    Write-Host "  Fix the errors above before pushing.       " -ForegroundColor Red
    Write-Host "=============================================" -ForegroundColor Red
    exit 1
}
