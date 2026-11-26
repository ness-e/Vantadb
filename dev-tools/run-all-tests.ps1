#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Suite de tests de VantaDB por capas: Rust, Python, TypeScript, Node, providers e
    integraciones. Sin argumentos abre un menu interactivo con explicacion y tiempo
    estimado de cada opcion; con flags, modo no interactivo para CI/agentes.

.DESCRIPTION
    Complemento de verify.ps1 (gate local rapido) y nocturnal_suite.ps1 (pesados de
    noche). Este script encadena las suites completas, deja log por capa y un resumen
    PASS/FAIL/SKIP.

    Que es "liviano" y que es "pesado" por capa:

      RUST      liviano = cargo nextest con el default-filter de .config/nextest.toml
                          (los ~55 binarios pesados quedan fuera: wal_resilience,
                          durability_recovery, hnsw_*, sift_validation,
                          competitive_bench, stress_protocol, crash_injection, chaos_*,
                          fuzz_proptest, property_durability, snapshot_certification,
                          security_audit, benchmark_*)
                pesado  = los mismos + --ignore-default-filter y --release: TODOS los
                          binarios de integracion (77 en el core). Tarda horas.
      PYTHON    liviano = pytest sin el marcador slow (18 ficheros)
                pesado  = ademas -m slow (bulk inserts de 3k/5k, excluidos por
                          defecto en vantadb-python/pyproject.toml)
      TS        liviano = npm ci + tsc + vitest (22 ficheros)
                pesado  = ademas npm run coverage (c8 --all sobre ./src)
      NODE      una sola capa (napi build + vitest, 11 ficheros)
      PROVIDERS 3 adapters (openai, ollama, litellm) con su tests/
      INTEGRACIONES  9 adapters + integrations/test_pins.py (checks estaticos de pins)

    Prerrequisitos por capa (si faltan, la capa se marca SKIPPED, nunca PASS):
      Rust      cargo-nextest (en shells no interactivos cargo no esta en PATH; el
                script anade $env:USERPROFILE\.cargo\bin si hace falta)
      Python    .venv con `maturin develop --release` (o `python` con el binding
                instalado). El script comprueba `import vantadb` y solo compila si falta.
      TS/Node   node >= 22.19 y npm; TS necesita vantadb-wasm/pkg (el script lo
                construye si no existe; con -Suite Wasm se fuerza el rebuild)
      WASM      wasm-pack
      Adapters  la dependencia de cada adapter (langchain, crewai, mem0, dspy,
                haystack, llama_index, letta, openai, ollama, litellm) y el propio
                adapter instalado. Si falta, SKIP con el `pip install` de siempre.

    Log: target\test-runs\run-all-<timestamp>.log (target/ esta en .gitignore).
    Exit code: 0 si nada fallo, 1 si alguna capa fallo, 2 si no hay consola para el menu.

.PARAMETER Suite
    All (defecto) | Rust | Python | Ts | Node | Providers | Integrations | Wasm

.PARAMETER Weight
    Light (defecto) | Heavy. Heavy implica --release en Rust, -m slow en Python y
    coverage en TypeScript.

.PARAMETER DryRun
    Imprime el plan (comando de cada capa) y sale sin ejecutar nada.

.PARAMETER NoMenu
   Fuerza el modo flags aunque no se pase ninguno (util si el host es raro).

.EXAMPLE
    .\dev-tools\run-all-tests.ps1                       # MENU interactivo
    .\dev-tools\run-all-tests.ps1 -Suite Rust -Weight Heavy
    .\dev-tools\run-all-tests.ps1 -Suite Python -Weight Heavy
    .\dev-tools\run-all-tests.ps1 -Suite All -Weight Light
    .\dev-tools\run-all-tests.ps1 -Suite Wasm           # fuerza rebuild de wasm
    .\dev-tools\run-all-tests.ps1 -DryRun               # plan completo, exit 0

.NOTES
    Encoding: este archivo es ASCII puro, sin BOM, con CRLF. Windows PowerShell 5.1
    decodifica un .ps1 sin BOM como ANSI y los bytes multibyte rompen el parser.
    La raiz del proyecto NO se resuelve con `Split-Path -Parent $PSScriptRoot`: ese
    binding devolvio $null al ejecutar este archivo. Se usa
    $MyInvocation.MyCommand.Path con walk-up hasta encontrar dev-tools\gate-common.ps1.
#>
param(
    [ValidateSet('All', 'Rust', 'Python', 'Ts', 'Node', 'Providers', 'Integrations', 'Wasm')]
    [string]$Suite = 'All',
    [ValidateSet('Light', 'Heavy')]
    [string]$Weight = 'Light',
    [switch]$DryRun,
    [switch]$NoMenu
)

# ---------------------------------------------------------------------------
# Menu interactivo. La funcion va ANTES de la llamada (en PowerShell las
# sentencias se ejecutan en orden) y escribe en $script:* porque dentro de una
# funcion las variables del bloque param son locales.
# ---------------------------------------------------------------------------
function Invoke-Menu {
    Write-Host ''
    Write-Host '  VantaDB - suite de tests por capas' -ForegroundColor Cyan
    Write-Host '  ===================================' -ForegroundColor DarkGray
    Write-Host ''
    Write-Host '  RUST  (cargo nextest, 9 crates, 77 binarios de integracion)' -ForegroundColor White
    Write-Host '   1) Rust liviano ............ 5-15 min    gate del workspace; deja fuera' -ForegroundColor Gray
    Write-Host '                                              los ~55 binarios pesados' -ForegroundColor DarkGray
    Write-Host '   2) Rust pesado ............. 2-6 h      TODOS los binarios en --release:' -ForegroundColor Gray
    Write-Host '                                              hnsw, sift, competitive_bench,' -ForegroundColor DarkGray
    Write-Host '                                              wal_resilience, fuzz_proptest...' -ForegroundColor DarkGray
    Write-Host ''
    Write-Host '  PYTHON  (pytest; usa .venv, compila el binding si hace falta)' -ForegroundColor White
    Write-Host '   3) Python liviano .......... 2-5 min    18 ficheros, sin el marcador slow' -ForegroundColor Gray
    Write-Host '   4) Python pesado ........... 20-45 min  anade -m slow (bulk inserts 3k/5k)' -ForegroundColor Gray
    Write-Host ''
    Write-Host '  TYPESCRIPT / NODE  (vitest)' -ForegroundColor White
    Write-Host '   5) TS liviano .............. 3-8 min    npm ci + tsc + vitest (22 ficheros)' -ForegroundColor Gray
    Write-Host '   6) TS pesado ............... 10-20 min  anade coverage (c8 --all sobre ./src)' -ForegroundColor Gray
    Write-Host '   7) Node nativo ............. 3-8 min    napi build + vitest (11 ficheros)' -ForegroundColor Gray
    Write-Host ''
    Write-Host '  ADAPTERS  (cada uno se salta si falta su dependencia)' -ForegroundColor White
    Write-Host '   8) Solo providers .......... 1-3 min    openai / ollama / litellm' -ForegroundColor Gray
    Write-Host '   9) Solo integraciones ...... 2-6 min    pins + 9 adapters' -ForegroundColor Gray
    Write-Host ''
    Write-Host '  COMBINACIONES' -ForegroundColor White
    Write-Host '  10) Todo liviano ........... 20-45 min  Rust + Python + TS + Node + adapters' -ForegroundColor Gray
    Write-Host '  11) Todo pesado ............ 3-8 h      todo lo anterior en modo pesado' -ForegroundColor Gray
    Write-Host '  12) Ver el plan ............ 0 s        imprime los comandos, no ejecuta' -ForegroundColor Gray
    Write-Host '   0) Salir' -ForegroundColor Gray
    Write-Host ''

    $choice = ([string](Read-Host 'Elige opcion')).Trim()
    switch ($choice) {
        '1' { $script:Suite = 'Rust';  $script:Weight = 'Light' }
        '2' { $script:Suite = 'Rust';  $script:Weight = 'Heavy' }
        '3' { $script:Suite = 'Python'; $script:Weight = 'Light' }
        '4' { $script:Suite = 'Python'; $script:Weight = 'Heavy' }
        '5' { $script:Suite = 'Ts';    $script:Weight = 'Light' }
        '6' { $script:Suite = 'Ts';    $script:Weight = 'Heavy' }
        '7' { $script:Suite = 'Node' }
        '8' { $script:Suite = 'Providers' }
        '9' { $script:Suite = 'Integrations' }
        '10' { $script:Suite = 'All';  $script:Weight = 'Light' }
        '11' { $script:Suite = 'All';  $script:Weight = 'Heavy' }
        '12' { $script:Suite = 'All';  $script:Weight = 'Light'; $script:DryRun = $true }
        '0' { Write-Host 'Cancelado.' -ForegroundColor DarkGray; exit 0 }
        default {
            Write-Host "Opcion no valida: '$choice'" -ForegroundColor Red
            Write-Host '  1 rust-liv  2 rust-pes  3 py-liv  4 py-pes  5 ts-liv  6 ts-pes' -ForegroundColor Yellow
            Write-Host '  7 node  8 providers  9 integraciones  10 todo-liv  11 todo-pes  12 plan  0 salir' -ForegroundColor Yellow
            exit 2
        }
    }

    $confirm = Read-Host ("Ejecutar '{0}' pesado={1}? (s/N)" -f $script:Suite, $script:Weight)
    if ($confirm -notmatch '^[sSyY]') {
        Write-Host 'Cancelado (opcion 12 para ver el plan sin ejecutar).' -ForegroundColor DarkGray
        exit 0
    }
}

if ($PSBoundParameters.Count -eq 0 -and -not $NoMenu) {
    try {
        Invoke-Menu
    } catch {
        # Solo se traga el error de "host sin consola"; cualquier otro se relanza.
        if ($_.Exception.Message -match 'NonInteractive') {
            Write-Host ''
            Write-Host '  Sin consola interactiva: menu no disponible.' -ForegroundColor Yellow
            Write-Host '  Usa flags:  -Suite All|Rust|Python|Ts|Node|Providers|Integrations|Wasm'
            Write-Host '             [-Weight Light|Heavy] [-DryRun] [-NoMenu]'
            Write-Host '  Ejemplo:    .\dev-tools\run-all-tests.ps1 -Suite Rust -Weight Heavy'
            exit 2
        }
        throw
    }
}

# ---------------------------------------------------------------------------
# Raiz del proyecto (ver .NOTES: nada de Split-Path -Parent $PSScriptRoot).
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

function Add-Skip($Name, $Why) {
    Write-Host "  [SKIP] $Name - $Why" -ForegroundColor Yellow
    Add-Content -Path $LogFile -Value "[SKIP] $Name :: $Why"
    $script:Skip.Add($Name)
}

# Ejecuta un comando nativo: stream a consola + log, y registra PASS/FAIL/SKIP.
function Run-Step($Name, [string[]]$ArgList, $WorkDir = $ProjectRoot, $Optional = $false) {
    $exe = $ArgList[0]
    $rest = @($ArgList | Select-Object -Skip 1)
    $shown = "$exe $($rest -join ' ')"

    Write-Host "  $Name" -ForegroundColor DarkGray
    Write-Host "    > $shown" -ForegroundColor DarkGray

    if (-not (Get-Command $exe -ErrorAction SilentlyContinue)) {
        Add-Skip $Name "$exe no esta en PATH"
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

# Comprueba si un modulo de python es importable. No imprime nada.
function Test-PyImport([string[]]$Modules) {
    if (-not (Get-Command $Py -ErrorAction SilentlyContinue) -and -not (Test-Path -LiteralPath $Py)) { return $false }
    $code = ($Modules | ForEach-Object { "import $_" }) -join '; '
    if ($DryRun) { return $true }   # en plan no se puede saber: no lo marques SKIP
    & $Py -c $code *> $null
    return ($LASTEXITCODE -eq 0)
}

$jobs = if (Get-Command Get-AdaptiveJobs -ErrorAction SilentlyContinue) { Get-AdaptiveJobs } else { [math]::Min([Environment]::ProcessorCount, 4) }
$Py = if (Test-Path (Join-Path $ProjectRoot '.venv\Scripts\python.exe')) { Join-Path $ProjectRoot '.venv\Scripts\python.exe' } else { 'python' }
$tsDir = Join-Path $ProjectRoot 'vantadb-ts'
$nodeDir = Join-Path $ProjectRoot 'vantadb-node'
$wasmDir = Join-Path $ProjectRoot 'vantadb-wasm'
$wasmPkg = Join-Path $wasmDir 'pkg'

# ---------------------------------------------------------------- suites ----
function Invoke-Rust {
    Write-Head 'RUST'
    if ($Weight -eq 'Heavy') {
        Run-Step 'rust pesado (todos los binarios, --release)' @('cargo', 'nextest', 'run', '--workspace',
            '--profile', 'ci-windows', '--ignore-default-filter', '--build-jobs', '1', '--release')
    }
    else {
        Run-Step 'rust liviano (gate del workspace)' @('cargo', 'nextest', 'run', '--workspace',
            '--profile', 'ci-windows', '--build-jobs', $jobs)
    }
}

function Invoke-Python {
    Write-Head 'PYTHON'
    if (-not (Test-PyImport @('vantadb'))) {
        if ($DryRun) {
            Write-Host '  [plan] comprobaria `import vantadb` y compilaria con maturin si falta' -ForegroundColor DarkGray
            Run-Step 'py test (pytest)' @($Py, '-m', 'pytest', 'vantadb-python\tests', '-q')
            return
        }
        Write-Host '  el binding no esta instalado en el interprete; se compila con maturin' -ForegroundColor Yellow
        Run-Step 'py build (maturin develop --release)' @('maturin', 'develop', '--release',
            '--manifest-path', 'vantadb-python\Cargo.toml')
    }
    Run-Step 'py test (pytest)' @($Py, '-m', 'pytest', 'vantadb-python\tests', '-q')
    if ($Weight -eq 'Heavy') {
        Run-Step 'py test pesado (-m slow)' @($Py, '-m', 'pytest', 'vantadb-python\tests', '-q', '-m', 'slow')
    }
}

function Invoke-Ts {
    Write-Head 'TYPESCRIPT'
    $pkgReady = (Test-Path -LiteralPath $wasmPkg -PathType Container) -and
                ((Get-ChildItem $wasmPkg -ErrorAction SilentlyContinue | Measure-Object).Count -gt 0)
    if ($script:WasmBuilt) {
        Add-Skip 'wasm build' 'ya construido en esta misma corrida'
    } elseif ($pkgReady) {
        Add-Skip 'wasm build' "vantadb-wasm\pkg ya existe (usa -Suite Wasm para forzar el rebuild)"
    } else {
        Run-Step 'wasm build (wasm-pack)' @('wasm-pack', 'build', '--release') $wasmDir
    }
    Copy-WasmTypes
    Run-Step 'ts npm ci' @('npm', 'ci') $tsDir
    Run-Step 'ts build (tsc)' @('npm', 'run', 'build') $tsDir
    Run-Step 'ts test (vitest)' @('npm', 'test') $tsDir
    if ($Weight -eq 'Heavy') {
        Run-Step 'ts coverage (c8)' @('npm', 'run', 'coverage') $tsDir
    }
}

function Invoke-Node {
    Write-Head 'NODE NATIVO'
    Run-Step 'node npm ci' @('npm', 'ci') $nodeDir
    Run-Step 'node build (napi)' @('npm', 'run', 'build') $nodeDir
    Run-Step 'node test (vitest)' @('npm', 'test') $nodeDir
}

function Invoke-Wasm {
    Write-Head 'WASM'
    $script:WasmBuilt = $true
    Run-Step 'wasm build (wasm-pack --release)' @('wasm-pack', 'build', '--release') $wasmDir
    Copy-WasmTypes
}

# WSM-05: el .d.ts que genera wasm-bindgen sale casi todo `any` y NO declara
# ListOptionsInput / MemoryRecordInput / SearchRequestInput, que el SDK TS importa.
# CI reemplaza el generado por el hand-written (vantadb-wasm\src\vantadb_wasm.d.ts);
# sin este paso `tsc` falla con TS2305. Es el mismo paso que hace ci-fleet.yml.
function Copy-WasmTypes {
    $src = Join-Path $wasmDir 'src\vantadb_wasm.d.ts'
    $dst = Join-Path $wasmDir 'pkg\vantadb_wasm.d.ts'
    $name = 'wasm tipos (.d.ts hand-written, WSM-05)'
    if (-not (Test-Path -LiteralPath $src)) { Add-Skip $name "no existe $src"; return }
    if ($DryRun) {
        Write-Host "  [plan] copiaria vantadb-wasm\src\vantadb_wasm.d.ts -> vantadb-wasm\pkg\vantadb_wasm.d.ts" -ForegroundColor DarkGray
        return
    }
    if (-not (Test-Path -LiteralPath $dst)) { Add-Skip $name 'pkg/vantadb_wasm.d.ts no existe (wasm-pack no corrio?)'; return }
    Copy-Item -LiteralPath $src -Destination $dst -Force
    Write-Host "  [PASS] $name" -ForegroundColor Green
    Add-Content -Path $LogFile -Value "[PASS] $name :: Copy-Item $src -> $dst"
    $script:Pass.Add($name)
}

function Invoke-Adapters($Which) {
    $catalog = if ($Which -eq 'Providers') {
        @(
            @{ Dir = 'openai';  Mod = 'vantadb_openai';  Dep = 'openai' },
            @{ Dir = 'ollama';  Mod = 'vantadb_ollama';  Dep = 'ollama' },
            @{ Dir = 'litellm'; Mod = 'vantadb_litellm'; Dep = 'litellm' }
        )
    } else {
        @(
            @{ Dir = 'langchain';  Mod = 'vantadb_langchain';  Dep = 'langchain' },
            @{ Dir = 'llamaindex'; Mod = 'vantadb_llamaindex'; Dep = 'llama_index' },
            @{ Dir = 'mem0';       Mod = 'vantadb_mem0';       Dep = 'mem0' },
            @{ Dir = 'crewai';     Mod = 'vantadb_crewai';     Dep = 'crewai' },
            @{ Dir = 'dspy';       Mod = 'vantadb_dspy';       Dep = 'dspy' },
            @{ Dir = 'haystack';   Mod = 'vantadb_haystack';   Dep = 'haystack' },
            @{ Dir = 'letta';      Mod = 'vantadb_letta';      Dep = 'letta' },
            @{ Dir = 'ollama';     Mod = 'vantadb_ollama';     Dep = 'ollama' },
            @{ Dir = 'openai';     Mod = 'vantadb_openai';     Dep = 'openai' }
        )
    }

    Write-Head ("ADAPTERS / " + $Which.ToUpper())
    $base = Join-Path $ProjectRoot $Which.ToLower()

    if ($Which -eq 'Integrations') {
        if (Test-PyImport @('packaging')) {
            Run-Step 'integrations pins (estatico)' @($Py, '-m', 'pytest', 'integrations\test_pins.py', '-q') $ProjectRoot
        } else {
            Add-Skip 'integrations pins (estatico)' 'falta el modulo packaging (pip install packaging)'
        }
    }

    foreach ($a in $catalog) {
        $dir = Join-Path $base $a.Dir
        if (-not (Test-Path -LiteralPath $dir -PathType Container)) {
            Add-Skip "$Which/$($a.Dir)" "no existe el directorio"
            continue
        }
        if (-not (Test-PyImport @($a.Mod, $a.Dep))) {
            if ($DryRun) {
                Run-Step "$Which/$($a.Dir) test" @($Py, '-m', 'pytest', 'tests', '-q') $dir
            } else {
                Add-Skip "$Which/$($a.Dir)" "falta $($a.Mod) o $($a.Dep) -> pip install ./$($Which.ToLower())/$($a.Dir)"
            }
            continue
        }
        Run-Step "$Which/$($a.Dir) test" @($Py, '-m', 'pytest', 'tests', '-q') $dir
    }
}

# ------------------------------------------------------------------ run -----
Write-Host ''
Write-Host '=== VantaDB :: suite de tests por capas ===' -ForegroundColor Magenta
Write-Host ("    Suite={0}  Peso={1}  DryRun={2}" -f $Suite, $Weight, [bool]$DryRun) -ForegroundColor Magenta
Write-Host ("    Raiz={0}" -f $ProjectRoot) -ForegroundColor DarkGray
Write-Host ("    Log ={0}" -f $LogFile) -ForegroundColor DarkGray
Write-Host ''
"Suite=$Suite Weight=$Weight DryRun=$($DryRun.IsPresent)" | Out-File -FilePath $LogFile -Encoding utf8

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host '  aviso: cargo no esta en PATH (la capa Rust quedara SKIPPED)' -ForegroundColor Yellow
}

# Preflight de disco: el build --release de todo el workspace se come decenas de GB
# y Fjall aborta a mitad de suite con os error 112 (StorageFull). Avisar ANTES.
try {
    $drive = ([IO.Path]::GetPathRoot($ProjectRoot)) -replace '[\\:]', ''
    $freeGb = [math]::Round((Get-PSDrive $drive).Free / 1GB, 1)
    if ($freeGb -lt 20) {
        Write-Host ("  aviso: solo {0} GB libres en {1}: el build --release se quedara sin espacio" -f $freeGb, $drive) -ForegroundColor Yellow
        Write-Host '         (los tests de Fjall fallan con os error 112 / StorageFull). Libera espacio o usa -Weight Light.' -ForegroundColor Yellow
    }
} catch { }
if (-not $Py -or -not (Test-Path -LiteralPath $Py)) {
    if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
        Write-Host '  aviso: no hay .venv ni python en PATH (la capa Python quedara SKIPPED)' -ForegroundColor Yellow
    }
}

Push-Location $ProjectRoot
try {
    switch ($Suite) {
        'Rust' { Invoke-Rust }
        'Python' { Invoke-Python }
        'Ts' { Invoke-Ts }
        'Node' { Invoke-Node }
        'Wasm' { Invoke-Wasm }
        'Providers' { Invoke-Adapters 'Providers' }
        'Integrations' { Invoke-Adapters 'Integrations' }
        default {
            Invoke-Rust
            Invoke-Wasm
            Invoke-Python
            Invoke-Ts
            Invoke-Node
            Invoke-Adapters 'Providers'
            Invoke-Adapters 'Integrations'
        }
    }
} finally {
    Pop-Location
}

$summary = @"
=== RESUMEN $(Get-Date -Format 'HH:mm:ss') ===
Suite=$Suite Peso=$Weight
PASS ($($script:Pass.Count)): $($script:Pass -join ', ')
FAIL ($($script:Fail.Count)): $($script:Fail -join ', ')
SKIP ($($script:Skip.Count)): $($script:Skip -join ', ')
Log: $LogFile
"@
Write-Host "`n$summary" -ForegroundColor White
Add-Content -Path $LogFile -Value $summary

if ($DryRun) { exit 0 }
exit ([int]($script:Fail.Count -gt 0))
