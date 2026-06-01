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

    # 6. Workspace Tests
    # NOTE: --all-features triggers a rustc stack overflow (STATUS_STACK_BUFFER_OVERRUN) on Windows
    # when linking the test binary for hybrid_retrieval_quality. We use --features=default to run
    # all functional tests safely. The --all-features compile check in step 2 covers feature correctness.
    Write-Header "Unit & Integration Tests"
    if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
        Write-Host "cargo-nextest detected! Running accelerated tests..." -ForegroundColor Gray
        Run-Command "Rust Tests (Nextest)" @(
            "cargo", "nextest", "run", "--workspace", "-j", "2", "--",
            "--skip", "benchmark",
            "--skip", "competitive",
            "--skip", "recall",
            "--skip", "sift",
            "--skip", "chaos",
            "--skip", "hnsw_hard_validation",
            "--skip", "stress_protocol",
            "--skip", "vector_scale"
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

    # Restore env vars before release build
    $env:RUSTFLAGS = $null
    $env:RUST_MIN_STACK = $null

    # 7. Python Bindings — Maturin Build
    Write-Header "Python Bindings (Maturin)"
    if (Get-Command "maturin" -ErrorAction SilentlyContinue) {
        Run-Command "Maturin Python SDK Build" @("maturin", "build", "--manifest-path", "./vantadb-python/Cargo.toml", "--release")
    } else {
        Write-Host "WARNING: 'maturin' executable not found in PATH." -ForegroundColor Yellow
        Write-Host "Skipping local Python SDK wheel build check. (Install via 'pip install maturin' if needed)" -ForegroundColor Gray
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
