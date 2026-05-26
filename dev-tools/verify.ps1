# VantaDB Pre-Flight Verification Script
# Runs all checks locally before pushing or publishing changes.
$ErrorActionPreference = "Stop"

# Auto-resolve project root to ensure it runs correctly from any CWD
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

function Write-Header($Title) {
    Write-Host "`n=== $Title ===" -ForegroundColor Cyan -FontWeight Bold
}

function Run-Command($Name, $Command, $Arguments) {
    Write-Host "Running: $Name..." -NoNewline -ForegroundColor Yellow
    
    # Run the command capturing output
    $process = Start-Process -FilePath $Command -ArgumentList $Arguments -NoNewWindow -PassThru -Wait
    
    if ($process.ExitCode -eq 0) {
        Write-Host " [PASSED]" -ForegroundColor Green
        return $true
    } else {
        Write-Host " [FAILED]" -ForegroundColor Red
        Write-Error "Step '$Name' failed with exit code $($process.ExitCode)."
        return $false
    }
}

try {
    Write-Host "=============================================" -ForegroundColor Cyan
    Write-Host "   VantaDB Pre-Flight Verification (Local)   " -ForegroundColor Cyan -FontWeight Bold
    Write-Host "=============================================" -ForegroundColor Cyan

    # 1. Rustfmt Check
    Write-Header "Code Formatting Check"
    Run-Command "Format Check" "cargo" "fmt --all -- --check"

    # 2. Cargo Check
    Write-Header "Workspace Compilation"
    Run-Command "Compilation (All Features)" "cargo" "check --workspace --all-targets --all-features"

    # 3. Clippy Lints
    Write-Header "Static Analysis (Clippy)"
    Run-Command "Clippy Lints" "cargo" "clippy --workspace --all-targets --all-features -- -D warnings"

    # 4. Security Audit
    Write-Header "Security Auditing"
    Run-Command "Cargo Audit" "cargo" "audit"

    # 5. Dependency Policy Check
    Write-Header "Dependency Policies"
    Run-Command "Cargo Deny Check" "cargo" "deny check"

    # 6. Workspace Tests
    Write-Header "Unit & Integration Tests"
    Run-Command "Rust Tests" "cargo" "test --workspace --all-targets --all-features"

    # 7. Python Bindings Maturin Build
    Write-Header "Python Bindings (Maturin)"
    Run-Command "Maturin Python SDK Build" "maturin" "build --manifest-path ./vantadb-python/Cargo.toml --release"

    Write-Host "`n=============================================" -ForegroundColor Green
    Write-Host "  SUCCESS: All local checks passed cleanly!  " -ForegroundColor Green -FontWeight Bold
    Write-Host "  You are safe to push/publish your changes. " -ForegroundColor Green
    Write-Host "=============================================" -ForegroundColor Green
    exit 0
}
catch {
    Write-Host "`n=============================================" -ForegroundColor Red
    Write-Host "  ERROR: Verification failed during checks.  " -ForegroundColor Red -FontWeight Bold
    Write-Host "  Fix the errors above before pushing.       " -ForegroundColor Red
    Write-Host "=============================================" -ForegroundColor Red
    exit 1
}
