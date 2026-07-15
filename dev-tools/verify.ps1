# VantaDB Pre-Flight Verification Script
# Runs all checks locally before pushing or publishing changes.
$ErrorActionPreference = "Stop"

# Auto-resolve project root to ensure it runs correctly from any CWD
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

# ── Bootstrap MSVC toolchain (Windows only, needed for librocksdb-sys) ──
$vsBuild = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools"
$msvcVer  = Get-ChildItem "$vsBuild\VC\Tools\MSVC\*" -Directory -ErrorAction SilentlyContinue |
    Select-Object -Last 1 -ExpandProperty Name
if ($msvcVer) {
    $paths = @(
        "$vsBuild\VC\Tools\MSVC\$msvcVer\bin\HostX64\x64"
        "$vsBuild\VC\Tools\Llvm\x64\bin"
        "$vsBuild\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin"
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64"
    )
    foreach ($p in $paths) {
        $resolved = if ($p -match '\*') { Get-ChildItem $p -ErrorAction SilentlyContinue | Select-Object -Last 1 -ExpandProperty FullName } else { $p }
        if ($resolved -and (Test-Path $resolved) -and ($env:PATH -notlike "*$resolved*")) {
            $env:PATH = "$resolved;$env:PATH"
        }
    }
    $kitVer = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Include\*" -Directory -ErrorAction SilentlyContinue |
        Where-Object Name -match '^\d+\.\d+\.\d+\.\d+$' | Select-Object -Last 1 -ExpandProperty Name
    if ($kitVer) {
        $env:INCLUDE = "$vsBuild\VC\Tools\MSVC\$msvcVer\include;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\ucrt;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\um;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\shared"
        $env:LIB     = "$vsBuild\VC\Tools\MSVC\$msvcVer\lib\x64;${env:ProgramFiles(x86)}\Windows Kits\10\Lib\$kitVer\ucrt\x64;${env:ProgramFiles(x86)}\Windows Kits\10\Lib\$kitVer\um\x64"
    }
    $llvmBin = "C:\Program Files\LLVM\bin"
    if ($env:PATH -notlike "*$llvmBin*") {
        $env:PATH = "$llvmBin;$env:PATH"
    }
    $env:LIBCLANG_PATH = $llvmBin
    $cl = "$vsBuild\VC\Tools\MSVC\$msvcVer\bin\HostX64\x64\cl.exe"
    $env:CC  = $cl
    $env:CXX = $cl
}

# ── Auto-detect system resources & clamp parallelism ──
# Mirrors the LowResource / Performance / Enterprise tiers from
# src/hardware/mod.rs at the shell level, so cargo never spawns more
# rustc processes than the machine can handle.
$SysInfo = Get-CimInstance Win32_ComputerSystem -ErrorAction SilentlyContinue
$TotalRAM = if ($SysInfo.TotalPhysicalMemory) { [math]::Round($SysInfo.TotalPhysicalMemory / 1GB) } else { 2 }
$LogicalCores = if ($SysInfo.NumberOfLogicalProcessors) { $SysInfo.NumberOfLogicalProcessors } else { 1 }
if ($TotalRAM -ge 16) {
    $Script:BuildJobs = [math]::Min($LogicalCores, 4)
    $Script:TestJobs  = [math]::Min($LogicalCores, 4)
} elseif ($TotalRAM -ge 4) {
    $Script:BuildJobs = [math]::Min($LogicalCores, 2)
    $Script:TestJobs  = 2
} else {
    $Script:BuildJobs = 1
    $Script:TestJobs  = 1
}
Write-Host "  System: ${TotalRAM}GB RAM × ${LogicalCores} cores → BuildJobs=${Script:BuildJobs}, TestJobs=${Script:TestJobs}" -ForegroundColor DarkGray

function Write-Header($Title) {
    Write-Host "`n=== $Title ===" -ForegroundColor Cyan
}

function Show-CodeGraphAffected {
    $codegraph = Get-Command "codegraph" -ErrorAction SilentlyContinue
    if (-not $codegraph) { return }
    if (-not (Test-Path "$ProjectRoot\.codegraph\codegraph.db")) { return }

    Write-Host "`n  Checking CodeGraph impact..." -ForegroundColor Gray
    $changed = git diff --name-only HEAD 2>$null
    if (-not $changed) { return }

    $result = $changed | & "codegraph" affected --stdin --quiet 2>$null
    if ($result) {
        Write-Host "  CodeGraph: $($result.Count) test files affected by your changes" -ForegroundColor Magenta
        foreach ($t in $result) { Write-Host "    $t" -ForegroundColor DarkGray }
        Write-Host "  Run: dev-tools/verify_changed.ps1 for a faster targeted check`n" -ForegroundColor Yellow
    }
}

function Run-Command($Name, [string[]]$ArgList) {
    Write-Host "`nRunning: $Name..." -ForegroundColor Yellow
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

    Show-CodeGraphAffected

    # Profile.dev is opt-level=1 + debug=0. Do NOT set RUSTFLAGS opt-level=0
    # — the windows-rs crate generates monolith functions at opt-level=0 that
    # overflow rustc's stack (STATUS_STACK_BUFFER_OVERUN / 0xc0000409).
    # RUST_MIN_STACK covers rustc thread pool; BuildJobs auto-detected above.
    # WARNING: never add --all-features or default features — windows-rs OOMs
    # on low-RAM machines. Use only --no-default-features with the explicit
    # feature list below.
    $env:RUST_MIN_STACK = "33554432"

    # 1. Actionlint
    $actionlint = Get-Command "actionlint" -ErrorAction SilentlyContinue
    if ($actionlint) {
        Run-Command "Actionlint (workflows)" @($actionlint.Source)
    } else {
        Write-Host "  [SKIP] actionlint — run: winget install actionlint" -ForegroundColor DarkGray
    }

    # 2. Format
    Write-Header "Code Formatting Check"
    Run-Command "Format Check" @("cargo", "fmt", "--all", "--", "--check")

    # 3. Compile core (no default features — rocksdb/prometheus/sysinfo pull the full
    #    windows-rs crate which OOMs on low-RAM machines; fjall+cli avoids it entirely)
    Run-Command "Core Compilation" @("cargo", "check", "-p", "vantadb", "--no-default-features", "--features", "cli,fjall,memmap2,fs2", "-j", $Script:BuildJobs)

    # 4. Clippy
    Run-Command "Clippy Lints" @("cargo", "clippy", "-p", "vantadb", "--no-default-features", "--features", "cli,fjall,memmap2,fs2", "-j", $Script:BuildJobs, "--", "-D", "warnings")

    # 5. Security audit + dependency policy
    Write-Header "Security Auditing"
    Run-Command "Cargo Audit" @("cargo", "audit", "--ignore", "RUSTSEC-2026-0176", "--ignore", "RUSTSEC-2026-0177")
    Run-Command "Cargo Deny Check" @("cargo", "deny", "check")

    # 6. Core tests (no sysinfo — avoids windows-rs OOM + sysinfo-0.30 inference
    # bugs on rustc 1.94.1; memory tracking stubs to zero when feature absent)
    Write-Header "Unit & Integration Tests (audit profile)"
    $env:RUST_MIN_STACK = "33554432"
    if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
        # --build-jobs 1: one rustc at a time prevents OOM / page-file exhaustion
        # on this dev machine. Remove the cap on CI where memory is plentiful.
        Run-Command "Rust Tests (Nextest audit)" @(
            "cargo", "nextest", "run", "--profile", "audit", "-p", "vantadb", "--no-default-features", "--features", "cli,fjall,memmap2,fs2", "--build-jobs", "1", "-E", "not test(/deserialize_absurd_node_count/) and not test(/test_search_with_bizarre_text_query/) and not test(/test_malformed_payload_extremely_large/)"
        )
    } else {
        Run-Command "Rust Tests (Standard)" @(
            "cargo", "test", "-p", "vantadb", "--no-default-features", "--features", "cli,fjall,memmap2,fs2", "-j", "1", "--",
            "--skip", "benchmark",
            "--skip", "competitive",
            "--skip", "recall",
            "--skip", "sift",
            "--skip", "chaos",
            "--skip", "hnsw_hard_validation",
            "--skip", "stress_protocol",
            "--skip", "vector_scale",
            "--skip", "certification",
            "--skip", "security_audit",
            "--skip", "deserialize_absurd_node_count",
            "--skip", "test_search_with_bizarre_text_query",
            "--skip", "test_malformed_payload_extremely_large"
        )
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
