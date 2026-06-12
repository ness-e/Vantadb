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

    # Force opt-level=0 and increase rustc stack size to prevent MSVC STATUS_STACK_BUFFER_OVERRUN
    # when compiling a large crate with all features on Windows
    $env:RUSTFLAGS = "-C opt-level=0"
    $env:RUST_MIN_STACK = "16777216"  # 16 MB stack for rustc threads

    # 1. Rustfmt Check
    Write-Header "Code Formatting Check"
    Run-Command "Format Check" @("cargo", "fmt", "--all", "--", "--check")

    # 2. Cargo Check (compile-only; --all-features is safe here because linking is skipped)
    Run-Command "Workspace Compilation" @("cargo", "check", "--workspace", "--tests", "-j", "2")

    # 3. Clippy Lints
    Run-Command "Clippy Lints" @("cargo", "clippy", "--workspace", "--tests", "-j", "2", "--", "-D", "warnings")

    # 4. Security Audit
    Write-Header "Security Auditing"
    Run-Command "Cargo Audit" @("cargo", "audit", "--ignore", "RUSTSEC-2026-0176")

    # 5. Dependency Policy Check
    Write-Header "Dependency Policies"
    Run-Command "Cargo Deny Check" @("cargo", "deny", "check")

    # 6. Workspace Tests
    Write-Header "Unit & Integration Tests (audit profile)"
    $env:RUST_MIN_STACK = "16777216"
    if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
        Write-Host "cargo-nextest detected! Running audit profile with --build-jobs 2..." -ForegroundColor Gray
        Run-Command "Rust Tests (Nextest audit)" @(
            "cargo", "nextest", "run", "--profile", "audit", "--workspace", "--build-jobs", "2"
        )
    } else {
        Write-Host "cargo-nextest not found. Falling back to standard cargo test..." -ForegroundColor Gray
        Run-Command "Rust Tests (Standard)" @(
            "cargo", "test", "--workspace", "-j", "2", "--",
            "--skip", "benchmark",
            "--skip", "competitive",
            "--skip", "recall",
            "--skip", "sift",
            "--skip", "chaos",
            "--skip", "hnsw_hard_validation",
            "--skip", "stress_protocol",
            "--skip", "vector_scale"
        )
    }
    $env:RUST_MIN_STACK = $null

    # Restore env vars before release build
    $env:RUSTFLAGS = $null
    $env:RUST_MIN_STACK = $null

    # 7. Python Bindings — audit venv + SDK validation
    Write-Header "Python Bindings (Audit Venv)"
    $SetupScript = Join-Path $ProjectRoot "dev-tools\setup_venv.ps1"
    $ValidateScript = Join-Path $ProjectRoot "dev-tools\scripts\validate_python_sdk.ps1"
    if (Test-Path $SetupScript) {
        Run-Command "Setup Audit Venv" @("powershell", "-ExecutionPolicy", "Bypass", "-File", $SetupScript)
    }
    if (Test-Path $ValidateScript) {
        Run-Command "Python SDK Validation" @("powershell", "-ExecutionPolicy", "Bypass", "-File", $ValidateScript)
    } else {
        Write-Host "WARNING: validate_python_sdk.ps1 not found; skipping Python SDK tests." -ForegroundColor Yellow
    }

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
