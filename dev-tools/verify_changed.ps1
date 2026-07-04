# VantaDB Quick Verification — uses CodeGraph to test only affected files
# Faster than verify.ps1 during local iteration.
$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

function Write-Header($Title) {
    Write-Host "`n=== $Title ===" -ForegroundColor Cyan
}

function Run-Command($Name, [string[]]$ArgList) {
    Write-Host "`nRunning: $Name..." -ForegroundColor Yellow
    & $ArgList[0] ($ArgList | Select-Object -Skip 1)
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[FAILED] $Name (exit code $LASTEXITCODE)" -ForegroundColor Red
        throw "Step '$Name' failed."
    }
    Write-Host "[PASSED] $Name" -ForegroundColor Green
}

try {
    Write-Host "===============================================" -ForegroundColor Magenta
    Write-Host "   VantaDB Quick Verify (CodeGraph-optimized)  " -ForegroundColor Magenta
    Write-Host "===============================================" -ForegroundColor Magenta

    $env:RUSTFLAGS = "-C opt-level=0"
    $env:RUST_MIN_STACK = "16777216"

    # 1. CodeGraph Impact
    Write-Header "CodeGraph Impact"
    $codegraph = Get-Command "codegraph" -ErrorAction SilentlyContinue
    $hasIndex = Test-Path "$ProjectRoot\.codegraph\codegraph.db"
    if ($codegraph -and $hasIndex) {
        $changed = git diff --name-only HEAD 2>$null
        if ($changed) {
            $affected = $changed | & "codegraph" affected --stdin --quiet 2>$null
            if ($affected) {
                Write-Host "  $($affected.Count) test files affected:" -ForegroundColor Yellow
                $affected | ForEach-Object { Write-Host "    → $_" }
                $affected | Out-File -FilePath "$ProjectRoot\target\.affected_tests" -Encoding utf8
            } else {
                Write-Host "  No tests affected by current changes." -ForegroundColor Green
            }
        }
    } else {
        Write-Host "  CodeGraph not available — running full workspace." -ForegroundColor Yellow
    }

    # 2. Format Check
    Write-Header "Code Formatting"
    Run-Command "Format Check" @("cargo", "fmt", "--all", "--", "--check")

    # 3. Quick Compile Check
    Run-Command "Workspace Compilation" @("cargo", "check", "--workspace", "--tests", "-j", "2")

    # 4. Run affected tests only (or fallback to cargo check)
    Write-Header "Affected Tests"
    $affectedFile = "$ProjectRoot\target\.affected_tests"
    if (Test-Path $affectedFile) {
        $tests = Get-Content $affectedFile | Where-Object { $_ -match "\.rs$" -or $_ -match "\.ts$" -or $_ -match "\.py$" }
        Remove-Item $affectedFile -Force
        if ($tests) {
            Write-Host "  Running $($tests.Count) affected test files..." -ForegroundColor Yellow
            # Filter Rust test files for nextest
            $rustTests = $tests | Where-Object { $_ -match "\.rs$" } | ForEach-Object {
                $_.Replace("/", "\").Replace(".rs", "")
            }
            if ($rustTests) {
                $testList = $rustTests -join ","
                Run-Command "Rust Affected Tests" @(
                    "cargo", "nextest", "run", "--profile", "audit", "--workspace",
                    "--build-jobs", "2", "--test", $testList
                )
            }
        }
    } else {
        Write-Host "  No affected files detected — running full clippy only." -ForegroundColor Yellow
        Run-Command "Clippy" @("cargo", "clippy", "--workspace", "--tests", "-j", "2", "--", "-D", "warnings")
    }

    $env:RUSTFLAGS = $null
    $env:RUST_MIN_STACK = $null

    Write-Host "`n=============================================" -ForegroundColor Green
    Write-Host "  SUCCESS: Quick verification passed!         " -ForegroundColor Green
    Write-Host "=============================================" -ForegroundColor Green
    exit 0
}
catch {
    Write-Host "`n=============================================" -ForegroundColor Red
    Write-Host "  ERROR: Quick verification failed.           " -ForegroundColor Red
    Write-Host "  Run verify.ps1 for the full check.          " -ForegroundColor Red
    Write-Host "=============================================" -ForegroundColor Red
    exit 1
}
