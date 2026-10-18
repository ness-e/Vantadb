#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Ejecuta TODAS las suites de test del proyecto VantaDB (Rust + WASM + TS + Node + Python).

.DESCRIPTION
    Complemento de verify.ps1 (gate local rapido) y nocturnal_suite.ps1 (pesados de noche).
    verify.ps1 solo corre el core con un subset de features; este script encadena las
    capas completas y deja un resumen PASS/FAIL/SKIP con un log por capa.

    Capas (seleccionables; -Mode All ejecuta las tres, siempre en este orden):

      Quick     -> cargo nextest sobre el workspace con el default-filter de
                   .config/nextest.toml. Los ~55 binarios pesados quedan fuera:
                   son los de la capa Heavy.
      Heavy     -> idem pero con --ignore-default-filter, es decir TODOS los binarios,
                   incluidos wal_resilience, durability_recovery, hnsw_*, sift_validation,
                   competitive_bench, stress_protocol, crash_injection, chaos_*, fuzz_proptest,
                   property_durability, snapshot_certification, security_audit, benchmark_*.
                   Tarda horas: usalo con -Release.
                   (--ignore-default-filter es el flag real de cargo-nextest 0.9.x;
                   --no-default-filter no existe.)
      Bindings  -> wasm-pack build --release en vantadb-wasm (prerrequisito del resto),
                   npm ci / npm run build / npm test en vantadb-ts, lo mismo en
                   vantadb-node, y python -m pytest vantadb-python\tests -q
                   (con -Slow, tambien pytest -m slow).
                   Los tests Rust de los bindings ya entran via --workspace.

    Requisitos: cargo-nextest; node >= 22.19, npm, python y wasm-pack para la capa
    Bindings. Si una herramienta opcional no esta en PATH, la capa se marca SKIPPED
    con aviso, nunca PASS. Si falta cargo se avisa y se omite la capa Rust.

    En shells no interactivos (agentes, CI) cargo NO esta en PATH: el script anade
    $env:USERPROFILE\.cargo\bin al PATH si hace falta y avisa si tampoco lo encuentra.

    Log: target\test-runs\run-all-<timestamp>.log (target/ esta en .gitignore).
    Exit code: 0 si nada fallo, 1 si alguna capa fallo.

.PARAMETER Mode
    All (defecto) | Quick | Heavy | Bindings

.PARAMETER Release
    Compila en release (anade --release a cargo nextest). Recomendado, y casi
    obligatorio, para Heavy: en debug las certificaciones de SIFT / competitive_bench
    tardan horas o expiran por timeout.

.PARAMETER Slow
    Anade `pytest -m slow` (bulk inserts 3k/5k, excluidos por defecto en pyproject.toml).

.PARAMETER DryRun
    Imprime el plan (comando de cada capa) y sale sin ejecutar nada. Exit code 0.

.EXAMPLE
    .\dev-tools\run-all-tests.ps1                      # las tres capas, en debug
    .\dev-tools\run-all-tests.ps1 -Mode Quick           # solo el gate de todo el workspace
    .\dev-tools\run-all-tests.ps1 -Mode Heavy -Release  # TODOS los binarios, compilado
    .\dev-tools\run-all-tests.ps1 -Mode Bindings -Slow  # wasm + ts + node + pytest
    .\dev-tools\run-all-tests.ps1 -DryRun               # ver el plan y salir

.NOTES
    Modo de uso recomendado: arranca siempre por -DryRun para ver el plan, luego
    lanza la capa que te interese. El script nunca falla al arrancar: todo lo que
    no encuentra lo marca SKIPPED y lo dice.
#>
param(
    [ValidateSet('All', 'Quick', 'Heavy', 'Bindings')]
    [string]$Mode = 'All',
    [switch]$Release,
    [switch]$Slow,
    [switch]$DryRun
)

# ---------------------------------------------------------------------------
# Raiz del proyecto.
#
# NO se usa `Split-Path -Parent $PSScriptRoot`. Ese binding devolvio $null al
# ejecutar este archivo y arrastro en cascada a Join-Path / New-Item / Add-Content
# / Out-File. La ruta se resuelve desde $MyInvocation.MyCommand.Path (fuente fiable
# y sin depender del binding automatico), con $PSCommandPath, $PSScriptRoot y el
# cwd como respaldos, y se valida subiendo de directorio hasta encontrar
# dev-tools\gate-common.ps1.
#
# Nota de encoding: este archivo es ASCII puro, sin BOM y con CRLF. Windows
# PowerShell 5.1 decodifica un .ps1 sin BOM como ANSI y los bytes multibyte
# rompen el parser, asi que ni caja de dibujo ni tildes ni flechas Unicode.
# ---------------------------------------------------------------------------
$here = $null
foreach ($cand in @($MyInvocation.MyCommand.Path, $PSCommandPath, $PSScriptRoot)) {
    if ($cand -and (Test-Path -LiteralPath $cand -PathType Leaf)) {
        $here = [System.IO.Path]::GetDirectoryName($cand)
        break
    }
}
if (-not $here) { $here = (Get-Location).Path }

$ProjectRoot = [System.IO.Path]::GetDirectoryName($here)
$GateCommon = $null
while ($ProjectRoot) {
    $candidate = Join-Path $ProjectRoot 'dev-tools\gate-common.ps1'
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        $GateCommon = $candidate
        break
    }
    $parent = [System.IO.Path]::GetDirectoryName($ProjectRoot)
    if (-not $parent -or $parent -eq $ProjectRoot) { break }
    $ProjectRoot = $parent
}

if (-not $GateCommon) {
    Write-Error "No se encuentra dev-tools\gate-common.ps1 subiendo desde '$here'. Abortando."
    exit 2
}

$ErrorActionPreference = 'Continue'

# En shells no interactivos cargo no esta en PATH; lo anadimos a mano si existe.
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path -LiteralPath $cargoBin)) {
    $env:PATH = "$cargoBin;$env:PATH"
}

. $GateCommon   # Get-AdaptiveJobs (jobs segun RAM)

$ts = Get-Date -Format 'yyyyMMdd-HHmmss'
$LogDir = Join-Path $ProjectRoot 'target\test-runs'
$null = New-Item -ItemType Directory -Path $LogDir -Force
$LogFile = Join-Path $LogDir "run-all-$ts.log"

$script:Pass = [System.Collections.Generic.List[string]]::new()
$script:Fail = [System.Collections.Generic.List[string]]::new()
$script:Skip = [System.Collections.Generic.List[string]]::new()

function Write-Head($Title) {
    Write-Host ''
    Write-Host "=== $Title ===" -ForegroundColor Cyan
    Add-Content -Path $LogFile -Value "=== $Title ===  $(Get-Date -Format 'HH:mm:ss')"
}

# Ejecuta un comando nativo: stream a consola + log, y registra PASS/FAIL/SKIP.
# Herramienta ausente -> SKIP con aviso (nunca PASS, nunca reventar).
function Run-Step($Name, [string[]]$ArgList, $WorkDir = $ProjectRoot, $Optional = $false) {
    $exe = $ArgList[0]
    $rest = @($ArgList | Select-Object -Skip 1)
    $shown = "$exe $($rest -join ' ')"

    Write-Host "  $Name" -ForegroundColor DarkGray
    Write-Host "    > $shown" -ForegroundColor DarkGray

    if (-not (Get-Command $exe -ErrorAction SilentlyContinue)) {
        $msg = if ($Optional) { "$exe no esta en PATH (herramienta opcional)" } else { "$exe no esta en PATH" }
        Write-Host "  [SKIP] $Name - $msg" -ForegroundColor Yellow
        Add-Content -Path $LogFile -Value "[SKIP] $Name :: $shown ($msg)"
        $script:Skip.Add($Name)
        return
    }

    Add-Content -Path $LogFile -Value "RUN $Name :: $shown"
    if ($DryRun) { return }

    Push-Location $WorkDir
    try {
        & $exe @rest 2>&1 | Tee-Object -FilePath $LogFile -Append | Out-Host
        $code = $LASTEXITCODE
    } finally {
        Pop-Location
    }

    if ($code -eq 0) {
        Write-Host "  [PASS] $Name" -ForegroundColor Green
        $script:Pass.Add($Name)
    } else {
        Write-Host "  [FAIL] $Name (exit $code)" -ForegroundColor Red
        $script:Fail.Add("$Name (exit $code)")
    }
}

Write-Host ''
Write-Host '=== VantaDB :: todas las suites de test ===' -ForegroundColor Magenta
Write-Host ("    Mode   : {0}    Release={1}  Slow={2}  DryRun={3}" -f $Mode, [bool]$Release, [bool]$Slow, [bool]$DryRun) -ForegroundColor Magenta
Write-Host ("    Raiz   : {0}" -f $ProjectRoot) -ForegroundColor DarkGray
Write-Host ("    Log    : {0}" -f $LogFile) -ForegroundColor DarkGray
Write-Host ''
"Mode=$Mode Release=$($Release.IsPresent) Slow=$($Slow.IsPresent) DryRun=$($DryRun.IsPresent)" | Out-File -FilePath $LogFile -Encoding utf8

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host '  aviso: cargo no esta en PATH (las capas Rust se quedaran en SKIPPED)' -ForegroundColor Yellow
}

$releaseFlag = if ($Release) { @('--release') } else { @() }
$jobs = if (Get-Command Get-AdaptiveJobs -ErrorAction SilentlyContinue) { Get-AdaptiveJobs } else { [math]::Min([Environment]::ProcessorCount, 4) }
$py = if (Test-Path '.venv\Scripts\python.exe') { '.venv\Scripts\python.exe' } else { 'python' }

Push-Location $ProjectRoot
try {
    if ($Mode -in @('All', 'Quick')) {
        Write-Head 'Rust / Quick -- workspace con default-filter (excluye los pesados)'
        Run-Step 'nextest quick' (@('cargo', 'nextest', 'run', '--workspace', '--profile', 'ci-windows', '--build-jobs', $jobs) + $releaseFlag)
    }

    if ($Mode -in @('All', 'Heavy')) {
        Write-Head 'Rust / Heavy -- TODOS los binarios (--ignore-default-filter)'
        if (-not $Release) {
            Write-Host '  aviso: sin -Release los pesados tardan MUCHO mas (o expiran por timeout)' -ForegroundColor Yellow
        }
        Run-Step 'nextest heavy' (@('cargo', 'nextest', 'run', '--workspace', '--profile', 'ci-windows', '--ignore-default-filter', '--build-jobs', 1) + $releaseFlag)
    }

    if ($Mode -in @('All', 'Bindings')) {
        Write-Head 'Bindings / WASM (vantadb-wasm)'
        Run-Step 'wasm-pack build' @('wasm-pack', 'build', '--release') (Join-Path $ProjectRoot 'vantadb-wasm') $true

        Write-Head 'Bindings / TypeScript (vantadb-ts)'
        $tsDir = Join-Path $ProjectRoot 'vantadb-ts'
        Run-Step 'ts npm ci' @('npm', 'ci') $tsDir $true
        Run-Step 'ts build' @('npm', 'run', 'build') $tsDir $true
        Run-Step 'ts test (vitest)' @('npm', 'test') $tsDir $true

        Write-Head 'Bindings / Node nativo (vantadb-node)'
        $nodeDir = Join-Path $ProjectRoot 'vantadb-node'
        Run-Step 'node npm ci' @('npm', 'ci') $nodeDir $true
        Run-Step 'node build (napi)' @('npm', 'run', 'build') $nodeDir $true
        Run-Step 'node test (vitest)' @('npm', 'test') $nodeDir $true

        Write-Head 'Bindings / Python (vantadb-python)'
        Run-Step 'py test (pytest)' @($py, '-m', 'pytest', 'vantadb-python\tests', '-q') $ProjectRoot
        if ($Slow) {
            Run-Step 'py test slow (pytest -m slow)' @($py, '-m', 'pytest', 'vantadb-python\tests', '-q', '-m', 'slow') $ProjectRoot
        }
    }
} finally {
    Pop-Location
}

$summary = @"
=== RESUMEN $(Get-Date -Format 'HH:mm:ss') ===
PASS ($($script:Pass.Count)): $($script:Pass -join ', ')
FAIL ($($script:Fail.Count)): $($script:Fail -join ', ')
SKIP ($($script:Skip.Count)): $($script:Skip -join ', ')
Log: $LogFile
"@
Write-Host ''
Write-Host $summary -ForegroundColor White
Add-Content -Path $LogFile -Value $summary

if ($DryRun) { exit 0 }
exit ([int]($script:Fail.Count -gt 0))
