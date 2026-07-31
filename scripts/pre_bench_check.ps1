<#
.SYNOPSIS
    Pre-benchmark environment checklist for VantaDB.
    Run before any benchmark session to ensure reproducible results.

.DESCRIPTION
    Validates: CPU load, power plan, VS Code processes, disk space, RAM,
    thermal state, and build flags. Exits with code 1 if any CRITICAL
    check fails; exits 0 with warnings otherwise.

.EXAMPLE
    .\scripts\pre_bench_check.ps1
    .\scripts\pre_bench_check.ps1 -Force   # Skip interactive prompt
#>
param(
    [switch]$Force
)

$ErrorActionPreference = "Continue"
$pass = 0
$warn = 0
$fail = 0

function Write-Check {
    param([string]$Status, [string]$Message)
    switch ($Status) {
        "OK"   { Write-Host "  [OK]  $Message" -ForegroundColor Green; $script:pass++ }
        "WARN" { Write-Host "  [!!]  $Message" -ForegroundColor Yellow; $script:warn++ }
        "FAIL" { Write-Host "  [XX]  $Message" -ForegroundColor Red; $script:fail++ }
    }
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  VantaDB Pre-Benchmark Environment Check" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# -- 1. CPU Load -----------------------------------------------
$cpuLoad = (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
if ($cpuLoad -lt 30) {
    Write-Check "OK" "CPU load: ${cpuLoad}% (< 30%)"
} else {
    Write-Check "FAIL" "CPU load: ${cpuLoad}% (>= 30%) -- benchmarks will be contaminated. Close background apps."
}

# -- 2. Power Plan ---------------------------------------------
$activePlan = powercfg /getactivescheme 2>$null
$planGuid = if ($activePlan -match '([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})') { $Matches[1] } else { "" }
$highPerfGuid = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"
if ($planGuid -eq $highPerfGuid) {
    Write-Check "OK" "Power plan: Alto rendimiento (High Performance)"
} else {
    $planName = if ($activePlan -match '\((.+)\)') { $Matches[1] } else { "Unknown" }
    Write-Check "FAIL" "Power plan: $planName -- must be Alto rendimiento. Run: powercfg /setactive $highPerfGuid"
}

# -- 3. VS Code Processes -------------------------------------
$vscodeCount = (Get-Process -Name "Code" -ErrorAction SilentlyContinue | Measure-Object).Count
if ($vscodeCount -le 3) {
    Write-Check "OK" "VS Code processes: $vscodeCount (<= 3)"
} else {
    Write-Check "WARN" "VS Code processes: $vscodeCount (> 3) -- close non-essential instances"
}

# -- 4. Disk Space ---------------------------------------------
$disk = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='C:'"
$diskFreePct = [math]::Round(($disk.FreeSpace / $disk.Size) * 100, 1)
if ($diskFreePct -gt 15) {
    Write-Check "OK" "Disk C: free: ${diskFreePct}%"
} else {
    Write-Check "WARN" "Disk C: free: ${diskFreePct}% (< 15%) -- SSD performance may degrade"
}

# -- 5. RAM Available ------------------------------------------
$os = Get-CimInstance Win32_OperatingSystem
$ramFreeGB = [math]::Round($os.FreePhysicalMemory / 1MB, 1)
if ($ramFreeGB -gt 4) {
    Write-Check "OK" "RAM free: ${ramFreeGB} GB"
} else {
    Write-Check "WARN" "RAM free: ${ramFreeGB} GB (< 4 GB) -- risk of swap thrashing"
}

# -- 6. RAYON_NUM_THREADS --------------------------------------
$rayonThreads = $env:RAYON_NUM_THREADS
if ($rayonThreads) {
    Write-Check "OK" "RAYON_NUM_THREADS=$rayonThreads"
} else {
    Write-Check "WARN" "RAYON_NUM_THREADS not set (may over-subscribe cores)"
}

# -- 7. target-cpu=native in .cargo/config.toml ----------------
$cargoConfig = Join-Path $PSScriptRoot "..\.cargo\config.toml"
if (Test-Path $cargoConfig) {
    $content = Get-Content $cargoConfig -Raw
    if ($content -match 'target-cpu=native') {
        Write-Check "OK" "target-cpu=native found in .cargo/config.toml"
    } else {
        Write-Check "FAIL" "target-cpu=native NOT found in .cargo/config.toml -- SIMD disabled"
    }
} else {
    Write-Check "FAIL" ".cargo/config.toml not found -- create it with rustflags target-cpu=native"
}

# -- 8. Thermal Cooldown --------------------------------------
$recentCompile = Get-Process -Name "rustc","cargo" -ErrorAction SilentlyContinue
if ($recentCompile) {
    Write-Check "WARN" "rustc/cargo still running -- wait for compilation to finish + 60s cooldown"
} else {
    Write-Check "OK" "No active compilation detected"
}

# -- Summary ---------------------------------------------------
Write-Host ""
Write-Host "--------------------------------------------" -ForegroundColor Cyan
Write-Host "  Results: $pass passed, $warn warnings, $fail critical" -ForegroundColor $(if ($fail -gt 0) { "Red" } elseif ($warn -gt 0) { "Yellow" } else { "Green" })
Write-Host "--------------------------------------------" -ForegroundColor Cyan

if ($fail -gt 0) {
    Write-Host ""
    Write-Host "  CRITICAL issues found. Fix them before running benchmarks." -ForegroundColor Red
    if (-not $Force) {
        $response = Read-Host "  Continue anyway? [y/N]"
        if ($response -notin @("y", "Y", "yes")) {
            Write-Host "  Aborted." -ForegroundColor Red
            exit 1
        }
    }
} elseif ($warn -gt 0) {
    Write-Host ""
    Write-Host "  Warnings present. Results may have noise." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Ready to benchmark. Recommended commands:" -ForegroundColor Green
Write-Host "    python benchmarks/competitive_bench.py --dataset glove-100-angular --yes" -ForegroundColor White
Write-Host "    cargo bench --bench hnsw_recall_ef" -ForegroundColor White
Write-Host "    cargo bench --bench hnsw_pure" -ForegroundColor White
Write-Host ""
