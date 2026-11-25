#Requires -Version 7.0
# FIND-732 — reclama las bases de datos temporales que dejan los tests que mueren
# sin ejecutar destructores (nextest `slow-timeout` mata el proceso, `abort`,
# SIGKILL). `tempfile::TempDir` borra en Drop y su drop ignora errores; si el
# proceso no llega a Drop, la carpeta entera queda en %TEMP%.
#
# Qué borra (y sólo eso):
#   1. `<TEMP>\.fjall_compaction_tmp_<pid>` — nombre propio de `FjallBackend::compact`
#      (src/backends/fjall_backend.rs:512-523). Es una DB fjall: se borra si su
#      `lock` está libre, o si el PID ya no existe.
#   2. `<TEMP>\.tmp*` y `<TEMP>\vanta*`/`vantadb*` — `tempfile::TempDir` (prefijo
#      por defecto `.tmp`) y los paths fijos de los tests (`vanta_schema_test`,
#      `vantadb_ser_test_*`). Se borra SÓLO si tiene un `lock` de fjall libre, o
#      si está vacío (0 bytes: pérdida nula).
#
# Dos pruebas de abandono, ambas obligatorias:
#   - **Inactividad** (`-MinAgeMinutes`, 10 por defecto): nada escrito en el árbol
#     hace menos de N minutos. Sin esto hay tres ventanas con la DB viva y el lock
#     libre — `compact()` cierra el tmp antes del swap
#     (fjall_backend.rs:558-560 vs :628-663), un test que cierra y reabre su DB, y
#     un `.tmp*` ajeno recién creado (cualquier `tempfile` de Rust usa `.tmp`).
#   - **Lock**: fjall mantiene abierto `<db>\lock` (fjall-3.1 LOCK_FILE); abrirlo
#     con `FileShare.None` sólo funciona si nadie tiene esa DB abierta.
# Sin mtime no se adivina: se exige que ambas señales digan "nadie la usa".
#
# Uso:
#   pwsh -NoProfile -File dev-tools/temp-collector.ps1 -DryRun     # qué borraría
#   pwsh -NoProfile -File dev-tools/temp-collector.ps1             # borra
#   pwsh -NoProfile -File dev-tools/temp-collector.ps1 -Quiet      # para hooks/gates
#   pwsh -NoProfile -File dev-tools/temp-collector.ps1 -MinAgeMinutes 0   # sólo sin suites en vuelo
#   pwsh -NoProfile -File dev-tools/temp-collector.ps1 -SelfCheck  # asserts
#
# Exit codes: 0 = ok (o nada que hacer) · 1 = self-check FAIL o error inesperado
# (un rechazo de la guarda al borrar se reporta y se salta — no aborta el barrido).
param(
    [switch]$DryRun,
    [switch]$Quiet,
    [switch]$SelfCheck,
    [int]$MinAgeMinutes = 10
)

$ErrorActionPreference = 'Stop'

$sep = [IO.Path]::DirectorySeparatorChar
$script:tempRoot = $null

function Get-TempPrefix {
    if ($null -eq $script:tempRoot) {
        $full = [IO.Path]::GetFullPath($env:TEMP)
        if (-not $full.EndsWith($sep)) { $full += $sep }
        $script:tempRoot = $full
    }
    return $script:tempRoot
}

# Toda ruta a borrar pasa por acá. Rechaza %TEMP% mismo, hermanos tipo `Temp2\...`
# y cualquier escape con `..` (mismo patrón que target-cleanup.ps1:57-66).
function Assert-UnderTemp([string]$path) {
    $full = [IO.Path]::GetFullPath($path)
    $prefix = Get-TempPrefix
    if (-not $full.EndsWith($sep)) { $full += $sep }
    if ($full -eq $prefix) {
        throw "REFUSED: '$prefix' es %TEMP% mismo — el recolector no borra la raíz"
    }
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "REFUSED: '$full' no está bajo '$prefix'"
    }
    return $full.Substring(0, $full.Length - 1)
}

function Test-PidAlive([int]$ProcessId) {
    if ($ProcessId -le 0) { return $false }
    return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

# $true = lock libre (nadie tiene la DB abierta) · $false = viva
function Test-LockFree([string]$lockPath) {
    try {
        $fs = [IO.File]::Open($lockPath, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
        $fs.Dispose()
        return $true
    } catch {
        return $false
    }
}

function Get-LeakKind([IO.DirectoryInfo]$dir) {
    if ($dir.Name -match '^\.fjall_compaction_tmp_(\d+)$') {
        return [pscustomobject]@{ Kind = 'fjall-compaction-tmp'; Pid = [int]$Matches[1] }
    }
    if ($dir.Name -like '.tmp*' -or $dir.Name -like 'vanta*' -or $dir.Name -like 'vantadb*') {
        return [pscustomobject]@{ Kind = 'test-tempdir'; Pid = 0 }
    }
    return $null
}

# Devuelve @{ Path; Reason; Bytes } de lo que se puede borrar sin riesgo.
function Get-Collectable {
    $items = @()
    $now = Get-Date
    foreach ($d in @(Get-ChildItem -LiteralPath (Get-TempPrefix) -Directory -ErrorAction SilentlyContinue)) {
        $kind = Get-LeakKind $d
        if (-not $kind) { continue }

        # Un solo recorrido: alimenta el lock, el tamaño y la última escritura.
        $walk = @(Get-ChildItem -LiteralPath $d.FullName -Recurse -Depth 4 -Force -ErrorAction SilentlyContinue)
        $files = @($walk | Where-Object { -not $_.PSIsContainer })
        $newest = $d.LastWriteTime
        $bytes = [long]0
        foreach ($f in $files) {
            $bytes += $f.Length
            if ($f.LastWriteTime -gt $newest) { $newest = $f.LastWriteTime }
        }
        $idle = [int]($now - $newest).TotalMinutes
        # Ventana viva (compactando / reabriendo / recién creada): no se toca.
        if ($idle -lt $MinAgeMinutes) { continue }

        $lock = $files | Where-Object { $_.Name -eq 'lock' } | Select-Object -First 1
        $reason = $null
        if ($lock) {
            if (Test-LockFree $lock.FullName) { $reason = "DB fjall con lock libre, sin actividad hace $idle min" }
        } elseif ($kind.Kind -eq 'fjall-compaction-tmp') {
            if (-not (Test-PidAlive $kind.Pid)) { $reason = "compaction tmp del PID $($kind.Pid) (muerto), sin DB abierta" }
        } elseif ($files.Count -eq 0) {
            $reason = "tempdir vacío (0 bytes), sin actividad hace $idle min"
        }
        if (-not $reason) { continue }
        $items += [pscustomobject]@{ Path = $d.FullName; Reason = $reason; Bytes = $bytes }
    }
    return $items
}

function Set-Old([string]$path, [int]$Minutes) {
    $stamp = (Get-Date).AddMinutes(-1 * ($Minutes + 1))
    Get-ChildItem -LiteralPath $path -Recurse -Force -ErrorAction SilentlyContinue |
        ForEach-Object { $_.LastWriteTime = $stamp }
    (Get-Item -LiteralPath $path -Force).LastWriteTime = $stamp
}

# --- SelfCheck (no toca residuo real; crea y borra su propio escenario) ----------
if ($SelfCheck) {
    $pass = 0; $fail = 0
    function Assert-SC([bool]$cond, [string]$label) {
        if ($cond) { Write-Host ("  PASS  {0}" -f $label); $script:pass++ }
        else { Write-Host ("  FAIL  {0}" -f $label) -ForegroundColor Red; $script:fail++ }
    }
    function Test-Refused([string]$p) {
        try { $null = Assert-UnderTemp $p; return $false } catch { return $true }
    }
    function Test-Collectable([string]$p) {
        return (@(Get-Collectable | Where-Object { $_.Path -eq $p }).Count -eq 1)
    }
    $temp = Get-TempPrefix

    Write-Host '=== SelfCheck: el recolector no puede tocar nada fuera de %TEMP% ==='
    foreach ($p in @(
            $temp,
            $env:SystemRoot,
            (Join-Path $temp '..\Windows\x'),
            (Join-Path $temp '..'),
            # hermano de %TEMP%: el prefijo con separador tiene que rechazarlo
            (((Get-TempPrefix).TrimEnd($sep)) + '2\x')
        )) {
        Assert-SC (Test-Refused $p) ("rechaza: {0}" -f $p)
    }
    Assert-SC (-not (Test-Refused (Join-Path $temp 'x\.fjall_compaction_tmp_1'))) 'acepta: <TEMP>\x\.fjall_compaction_tmp_1'
    Assert-SC ((Get-TempPrefix).EndsWith($sep)) 'el prefijo termina en separador'

    Write-Host '=== SelfCheck: sonda de vida (PID) ==='
    Assert-SC (Test-PidAlive $PID) 'el PID propio figura vivo'
    $dead = Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', 'exit' -PassThru -WindowStyle Hidden
    $dead.WaitForExit()
    Assert-SC (-not (Test-PidAlive $dead.Id)) ("PID {0} terminado figura muerto" -f $dead.Id)

    Write-Host '=== SelfCheck: el lock de fjall delata una DB viva ==='
    $root = Assert-UnderTemp (Join-Path $temp ('.vanta-tc-selftest-' + $PID))
    $db = Join-Path $root 'data'
    New-Item -ItemType Directory -Force -Path $db | Out-Null
    $lockPath = Join-Path $db 'lock'
    Set-Content -LiteralPath $lockPath -Value '' -NoNewline
    Assert-SC (Test-LockFree $lockPath) 'sin nadie abriéndola, el lock figura libre'
    $held = [IO.File]::Open($lockPath, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    Assert-SC (-not (Test-LockFree $lockPath)) 'con la DB abierta, el lock NO figura libre'
    $held.Dispose()
    Remove-Item -LiteralPath $root -Recurse -Force

    Write-Host '=== SelfCheck: el caso REAL (`.tmp*` con lock libre) se recolecta ==='
    $real = Assert-UnderTemp (Join-Path $temp ('.tmp' + ([guid]::NewGuid().ToString('N').Substring(0, 8).ToUpper())))
    New-Item -ItemType Directory -Force -Path (Join-Path $real 'data') | Out-Null
    Set-Content -LiteralPath (Join-Path $real 'data\lock') -Value '' -NoNewline
    Set-Content -LiteralPath (Join-Path $real 'data\0.jnl') -Value 'x' -NoNewline
    Assert-SC (-not (Test-Collectable $real)) 'recién escrita: la guarda de inactividad la protege'
    Set-Old $real 30
    Assert-SC (Test-Collectable $real) 'vieja y con lock libre: entra en la lista de recolección'
    foreach ($i in @(Get-Collectable | Where-Object { $_.Path -eq $real })) {
        Remove-Item -LiteralPath (Assert-UnderTemp $i.Path) -Recurse -Force -ErrorAction Continue
    }
    Assert-SC (-not (Test-Path $real)) 'el recolector la borró'
    Assert-SC (-not (Test-Collectable $real)) 'no queda en la lista tras borrarla'

    Write-Host '=== SelfCheck: lo que no es nuestro, no se toca ==='
    $foreign = Assert-UnderTemp (Join-Path $temp ('.tmpA' + ([guid]::NewGuid().ToString('N').Substring(0, 5))))
    New-Item -ItemType Directory -Force -Path $foreign | Out-Null
    Set-Content -LiteralPath (Join-Path $foreign 'other-app.dat') -Value 'x' -NoNewline
    Set-Old $foreign 30
    Assert-SC (-not (Test-Collectable $foreign)) '`.tmp*` sin lock fjall no se recolecta'

    $stale = Assert-UnderTemp (Join-Path $temp ('.fjall_compaction_tmp_' + $dead.Id))
    New-Item -ItemType Directory -Force -Path $stale | Out-Null
    Assert-SC (-not (Test-Collectable $stale)) 'compaction tmp recién creado no se recolecta'
    Set-Old $stale 30
    Assert-SC (Test-Collectable $stale) 'compaction tmp viejo de un PID muerto se recolecta'
    Assert-SC ($null -eq (Get-LeakKind ([IO.DirectoryInfo](Join-Path $temp 'no-es-nuestro')))) 'un dir cualquiera no se clasifica'

    Remove-Item -LiteralPath $foreign -Recurse -Force
    Remove-Item -LiteralPath $stale -Recurse -Force

    Write-Host ''
    Write-Host ("SELFCHECK: {0} ({1}/{2} asserts)" -f $(if ($fail -eq 0) { 'PASS' } else { 'FAIL' }), $pass, ($pass + $fail))
    exit $(if ($fail -eq 0) { 0 } else { 1 })
}

# --- Recolección ----------------------------------------------------------------
$items = @(Get-Collectable)
$totalBytes = ($items | Measure-Object -Property Bytes -Sum).Sum
if ($null -eq $totalBytes) { $totalBytes = 0 }

if (-not $Quiet) {
    Write-Host ("%TEMP% = {0}" -f (Get-TempPrefix))
    if ($items.Count -eq 0) {
        Write-Host 'sin residuos recolectables.'
    } else {
        foreach ($i in $items) {
            Write-Host ("  {0,8:N1} MB  {1}  [{2}]" -f ($i.Bytes / 1MB), $i.Path, $i.Reason)
        }
    }
}

if ($DryRun -or $items.Count -eq 0) {
    if (-not $Quiet -and $items.Count -gt 0) {
        Write-Host ("-DryRun: liberaría {0:N1} MB en {1} carpeta(s). Nada se borró." -f ($totalBytes / 1MB), $items.Count)
    }
    exit 0
}

$freed = 0
$removed = 0
foreach ($i in $items) {
    try {
        Remove-Item -LiteralPath (Assert-UnderTemp $i.Path) -Recurse -Force -ErrorAction Stop
        $freed += $i.Bytes
        $removed++
    } catch {
        if (-not $Quiet) { Write-Host ("  no se pudo borrar {0}: {1}" -f $i.Path, $_.Exception.Message) -ForegroundColor DarkYellow }
    }
}

if (-not $Quiet) {
    Write-Host ("recolector: {0}/{1} carpeta(s), {2:N1} MB liberados." -f $removed, $items.Count, ($freed / 1MB))
} elseif ($removed -gt 0) {
    # -Quiet sigue avisando cuando hizo algo: un gate silencioso que libera GB es un gate ciego.
    Write-Host ("temp-collector: {0} carpeta(s) huérfana(s) en %TEMP%, {1:N1} MB liberados." -f $removed, ($freed / 1MB))
}
exit 0
