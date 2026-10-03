# HARD-02 (decisión owner (c) 2026-09-27): el reporte+budget de coverage YA NO corre en
# el fast gate local por defecto — vive en el nightly (nightly.yml job `coverage-budget`,
# script dev-tools/coverage-budget.ps1). On-demand local: `pwsh dev-tools/verify.ps1 -IncludeCoverage`.
# CI canónico ADR-018 (root crate ≥80%, ci-rust.yml) intacto.
param([switch]$IncludeCoverage)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

# ── MSVC bootstrap (cached, keyed por versión de MSVC para invalidar el cache
#    al actualizar BuildTools — antes un marker genérico en TEMP congelaba
#    INCLUDE/LIB a la versión vieja) ──
$vsBuild = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools"
$msvcVer = Get-ChildItem "$vsBuild\VC\Tools\MSVC\*" -Directory -ErrorAction SilentlyContinue |
    Select-Object -Last 1 -ExpandProperty Name
$msvcMarker = "$env:TEMP\.vantadb_msvc_done$(if ($msvcVer) { "_$msvcVer" })"
if (-not (Test-Path $msvcMarker)) {
    if ($msvcVer) {
        $paths = @(
            "$vsBuild\VC\Tools\MSVC\$msvcVer\bin\HostX64\x64"
            "$vsBuild\VC\Tools\Llvm\x64\bin"
            "$vsBuild\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin"
            "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64"
        )
        foreach ($p in $paths) {
            $r = if ($p -match '\*') { Get-ChildItem $p -ErrorAction SilentlyContinue | Select-Object -Last 1 -ExpandProperty FullName } else { $p }
            if ($r -and (Test-Path $r) -and ($env:PATH -notlike "*$r*")) { $env:PATH = "$r;$env:PATH" }
        }
        $kitVer = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Include\*" -Directory -ErrorAction SilentlyContinue |
            Where-Object Name -match '^\d+\.\d+\.\d+\.\d+$' | Select-Object -Last 1 -ExpandProperty Name
        if ($kitVer) {
            $env:INCLUDE = "$vsBuild\VC\Tools\MSVC\$msvcVer\include;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\ucrt;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\um;${env:ProgramFiles(x86)}\Windows Kits\10\Include\$kitVer\shared"
            $env:LIB = "$vsBuild\VC\Tools\MSVC\$msvcVer\lib\x64;${env:ProgramFiles(x86)}\Windows Kits\10\Lib\$kitVer\ucrt\x64;${env:ProgramFiles(x86)}\Windows Kits\10\Lib\$kitVer\um\x64"
        }
        $env:PATH = "C:\Program Files\LLVM\bin;$env:PATH"
        $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
        $cl = "$vsBuild\VC\Tools\MSVC\$msvcVer\bin\HostX64\x64\cl.exe"
        $env:CC = $cl; $env:CXX = $cl
    }
    New-Item -Force $msvcMarker | Out-Null
}

$SysInfo = Get-CimInstance Win32_ComputerSystem -ErrorAction SilentlyContinue
$TotalRAM = if ($SysInfo.TotalPhysicalMemory) { [math]::Round($SysInfo.TotalPhysicalMemory / 1GB) } else { 2 }
$Cores = if ($SysInfo.NumberOfLogicalProcessors) { $SysInfo.NumberOfLogicalProcessors } else { 1 }
$Jobs = if ($TotalRAM -ge 16) { [math]::Min($Cores, 4) } elseif ($TotalRAM -ge 4) { [math]::Min($Cores, 2) } else { 1 }
Write-Host "${TotalRAM}GB ${Cores}cores j=${Jobs}" -ForegroundColor DarkGray

$env:RUST_MIN_STACK = "33554432"
# Features del core gate: definición canónica compartida (dev-tools/gate-common.ps1)
. (Join-Path $PSScriptRoot "gate-common.ps1")
$feats = Get-CoreFeatures
# Exclusiones RESOURCE-GUARD del fast gate (3 tests con inputs hostiles; trazabilidad
# Regla 2 / CI_POLICY §"Fast Gate Test Exclusions" — FIND-22). Definición canónica en
# gate-common.ps1 — compartida con coverage-budget.ps1 (nightly) para que no deriven.
$fastGateFilter = Get-FastGateFilter
$pass = 0; $fail = 0

function run($name, [string[]]$cmd) {
    Write-Host "  ${name}..." -ForegroundColor Yellow -NoNewline
    $output = & $cmd[0] $cmd[1..($cmd.Length - 1)] 2>&1
    if ($LASTEXITCODE -eq 0) { Write-Host " ok" -ForegroundColor Green; $Script:pass++ }
    else { Write-Host " FAIL" -ForegroundColor Red; $Script:fail++; Write-Host $output -ForegroundColor Red; throw "step failed" }
}

try {
    run "fmt" ("cargo", "fmt", "--all", "--", "--check")
    run "check" (("cargo", "check", "-p", "vantadb", "-j", "$Jobs") + $feats)
    run "clippy" (("cargo", "clippy", "-p", "vantadb", "-j", "$Jobs") + $feats + @("--", "-D", "warnings"))
    run "audit" ("cargo", "audit")   # ignores managed in .cargo/audit.toml
    run "deny" ("cargo", "deny", "check")
    if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
        # exclusiones RESOURCE-GUARD: ver $fastGateFilter arriba + CI_POLICY §"Fast Gate Test Exclusions"
        run "nextest" (("cargo", "nextest", "run", "--profile", "audit", "-p", "vantadb") + $feats + @("--build-jobs", "1", "-E", $fastGateFilter))
    } else {
        run "test" (("cargo", "test", "-p", "vantadb") + $feats + @("-j", "1", "--", "--skip", "benchmark", "--skip", "competitive", "--skip", "recall", "--skip", "sift", "--skip", "chaos", "--skip", "hnsw_hard_validation", "--skip", "stress_protocol", "--skip", "vector_scale", "--skip", "certification", "--skip", "security_audit", "--skip", "deserialize_absurd_node_count", "--skip", "test_search_with_bizarre_text_query", "--skip", "test_malformed_payload_extremely_large"))
    }
    if ($IncludeCoverage) {
        # On-demand: mismo script que corre el nightly (reporte JSON + presupuesto por directorio).
        run "coverage-budget" ("pwsh", "-NoProfile", "$PSScriptRoot\coverage-budget.ps1")
    } else {
        # HARD-02 (decisión owner (c) 2026-09-27): coverage vive en el nightly (nightly.yml job
        # `coverage-budget`); el fast gate local no lo corre por defecto (restaura el budget <5min).
        # On-demand: -IncludeCoverage. CI canónico ADR-018 intacto (ci-rust.yml).
        Write-Host "  coverage report+budget moved to nightly - run with -IncludeCoverage" -ForegroundColor DarkYellow
    }
    if (Test-Path "$ProjectRoot\scripts\validate-docs-coverage.ps1") {
        run "docs-coverage" ("pwsh", "-NoProfile", "$ProjectRoot\scripts\validate-docs-coverage.ps1")
    } else {
        Write-Host "  docs-coverage script not installed - skipping docs gate" -ForegroundColor DarkYellow
    }
    # GOV-A3 probe CLI reales (doctor/backup/restore) — ponytail 1-2 líneas: reuse cargo run --bin vanta-cli
    run "cli-probes" ("cargo", "run", "-p", "vantadb", "--bin", "vanta-cli", "--", "--help")
    # GOV-B3 consumo guard — anti-regresión compile gate (ponytail: solo --no-run, sin timed bench en fast gate)
    run "consumo guard" ("cargo", "bench", "-p", "vantadb", "--bench", "canonical_p99", "--no-run")
    # GOV-C3 Daily Backup Verification — ponytail: validates runbook §3.1 exists (docs-only), no heavy restore in fast gate
    # Full restore+doctor verification lives in DISASTER_RECOVERY_RUNBOOK.md §3.1; fast gate only checks doc anchor exists
    run "daily backup verification" ("pwsh", "-NoProfile", "-Command", "if ((Select-String -Path 'docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md' -Pattern 'Daily Backup Verification' | Measure-Object).Count -ge 1) { exit 0 } else { Write-Host 'Daily Backup Verification missing in runbook' -ForegroundColor Red; exit 1 }")
    Write-Host "ALL ${pass} PASS" -ForegroundColor Green; exit 0
} catch {
    if ($fail -eq 0) { $Script:fail = 1 }
    Write-Host "${fail} FAIL" -ForegroundColor Red; exit 1
}
