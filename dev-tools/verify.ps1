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
# P2-06: prudent initial llvm-cov line-coverage floor. Raise after first runs (see CI_POLICY.md).
$CoverageThreshold = 60
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
        # Exclusiones del fast gate (trazabilidad Regla 2 / CI_POLICY):
        #   - deserialize_absurd_node_count      → CATEGORY: RESOURCE-GUARD (input de
        #     tamaño absurdo diseñado para OOM el runner; no es flaky, es bomba de memoria)
        #   - test_search_with_bizarre_text_query / test_malformed_payload_extremely_large
        #     → CATEGORY: RESOURCE-GUARD (inputs malformados gigantes; cubiertos por
        #     fuzzing dedicado, no por el fast gate)
        # TODO(FIND): formalizar estas 3 exclusiones en docs/operations/CI_POLICY.md
        run "nextest" (("cargo", "nextest", "run", "--profile", "audit", "-p", "vantadb") + $feats + @("--build-jobs", "1", "-E", "not test(/deserialize_absurd_node_count/) and not test(/test_search_with_bizarre_text_query/) and not test(/test_malformed_payload_extremely_large/)"))
    } else {
        run "test" (("cargo", "test", "-p", "vantadb") + $feats + @("-j", "1", "--", "--skip", "benchmark", "--skip", "competitive", "--skip", "recall", "--skip", "sift", "--skip", "chaos", "--skip", "hnsw_hard_validation", "--skip", "stress_protocol", "--skip", "vector_scale", "--skip", "certification", "--skip", "security_audit", "--skip", "deserialize_absurd_node_count", "--skip", "test_search_with_bizarre_text_query", "--skip", "test_malformed_payload_extremely_large"))
    }
    if (Get-Command "cargo-llvm-cov" -ErrorAction SilentlyContinue) {
        if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
            run "coverage" (("cargo", "llvm-cov", "nextest", "run", "--profile", "audit", "-p", "vantadb", "--fail-under-lines", "$CoverageThreshold") + $feats + @("--build-jobs", "1", "-E", "not test(/deserialize_absurd_node_count/) and not test(/test_search_with_bizarre_text_query/) and not test(/test_malformed_payload_extremely_large/)"))
        } else {
            run "coverage" (("cargo", "llvm-cov", "-p", "vantadb", "--fail-under-lines", "$CoverageThreshold") + $feats + @("-j", "1", "--", "--skip", "benchmark", "--skip", "competitive", "--skip", "recall", "--skip", "sift", "--skip", "chaos", "--skip", "hnsw_hard_validation", "--skip", "stress_protocol", "--skip", "vector_scale", "--skip", "certification", "--skip", "security_audit", "--skip", "deserialize_absurd_node_count", "--skip", "test_search_with_bizarre_text_query", "--skip", "test_malformed_payload_extremely_large"))
        }
    } else {
        Write-Host "  llvm-cov not installed - skipping coverage gate" -ForegroundColor DarkYellow
    }
    if (Test-Path "$ProjectRoot\scripts\validate-docs-coverage.ps1") {
        run "docs-coverage" ("pwsh", "-NoProfile", "$ProjectRoot\scripts\validate-docs-coverage.ps1")
    } else {
        Write-Host "  docs-coverage script not installed - skipping docs gate" -ForegroundColor DarkYellow
    }
    Write-Host "ALL ${pass} PASS" -ForegroundColor Green; exit 0
} catch {
    if ($fail -eq 0) { $Script:fail = 1 }
    Write-Host "${fail} FAIL" -ForegroundColor Red; exit 1
}
