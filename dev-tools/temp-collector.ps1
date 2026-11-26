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
#   3. Ficheros SUELTOS bajo <TEMP> cuyo nombre empieza por `vanta`/`vantadb`
#      (`vanta_test_wal_healing_<rand>.corrupt` 63 B, `vanta_test_sharded_<rand>.shard0.corrupt`
#      54 B, `vanta-ckpt-<pid>.jsonl`): la cuarentena de WAL escribe un hermano del
#      path del test y el `remove_file` del test no lo borra. Un fichero no tiene
#      `lock` de fjall: su señal de vida es el handle abierto.
#
# La sonda de exclusividad sirve también para ficheros sueltos — medido 2026-11-25
# (Windows 11, PS 7.6): abrir con `FileShare.None` FALLA mientras otro handle tenga
# el fichero abierto, sea cual sea su modo (None, Read, ReadWrite, ReadWrite+Delete).
# Y medido también: `Remove-Item` SÍ borra a través de un handle que permite DELETE
# (justo lo que concede `std::fs::File`), así que quien protege es la sonda, no
# `Remove-Item`.
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

function Format-Bytes([long]$bytes) {
    if ($bytes -ge 1GB) { return ('{0:N2} GB' -f ($bytes / 1GB)) }
    if ($bytes -ge 1MB) { return ('{0:N2} MB' -f ($bytes / 1MB)) }
    if ($bytes -ge 1KB) { return ('{0:N1} KB' -f ($bytes / 1KB)) }
    return ('{0:N0} B' -f $bytes)
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

# Mismo criterio de prefijos que los directorios. Coincide con el nombre de test
# literal (`vanta-ckpt-<pid>.jsonl`, `vanta_test_wal_healing_<rand>.corrupt`) y NO
# con un nombre que sólo *contenga* "vanta" (`notas-vanta.txt`): sin comodín inicial.
function Get-LeakFileKind([IO.FileInfo]$file) {
    if ($file.Name -like 'vanta*' -or $file.Name -like 'vantadb*') {
        # Los checkpoints llevan el PID en el nombre: si el proceso vive, no es residuo.
        $pidFromName = $(if ($file.Name -match '^vanta-ckpt-(\d+)') { [int]$Matches[1] } else { 0 })
        return [pscustomobject]@{ Kind = 'test-loose-file'; Pid = $pidFromName }
    }
    return $null
}

# Devuelve @{ Path; Class ('dir'|'file'); Reason; Bytes; Guard } de lo que se puede
# borrar sin riesgo. `Guard` es la ruta que hay que poder abrir en exclusiva justo
# antes de borrar ($null = sin sonda que re-verificar).
function Get-Collectable {
    $items = @()
    $now = Get-Date
    foreach ($d in @(Get-ChildItem -LiteralPath (Get-TempPrefix) -Directory -Force -ErrorAction SilentlyContinue)) {
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
        $items += [pscustomobject]@{
            Path = $d.FullName; Class = 'dir'; Reason = $reason; Bytes = $bytes
            # Re-verificado justo antes de borrar: un dir con lock pudo reabrirse.
            Guard = $(if ($lock) { $lock.FullName } else { $null })
        }
    }

    # Ficheros sueltos: mismo criterio y misma doble prueba (handle libre + inactividad).
    # Un fichero sin handle abierto que además no se tocó en N minutos está muerto:
    # ningún test vivo deja su `.corrupt` de cuarentena cerrado y quieto.
    # -Force: un residuo con atributo Hidden sigue siendo residuo; sin esto el
    # reporte diría "0 ficheros" justo en el caso que el ticket denuncia.
    foreach ($f in @(Get-ChildItem -LiteralPath (Get-TempPrefix) -File -Force -ErrorAction SilentlyContinue)) {
        $kind = Get-LeakFileKind $f
        if (-not $kind) { continue }
        # Un checkpoint de un proceso VIVO no es residuo, aunque lleve >10 min quieto
        # (lo escribe por intervalos y entre guardados no mantiene el handle abierto).
        if ($kind.Pid -gt 0 -and (Test-PidAlive $kind.Pid)) { continue }
        $idle = [int]($now - $f.LastWriteTime).TotalMinutes
        if ($idle -lt $MinAgeMinutes) { continue }
        # La sonda del fichero es el fichero: FileShare.None falla con cualquier handle vivo.
        # ponytail: techo conocido — un handle abierto con acceso CERO (sólo atributos)
        # no colisiona con FileShare.None; la inactividad de N minutos es el segundo
        # cierre para ese caso. Si aparece uno real, sondear también con FileAccess::Read.
        if (-not (Test-LockFree $f.FullName)) { continue }
        $items += [pscustomobject]@{
            Path = $f.FullName; Class = 'file'; Bytes = $f.Length; Guard = $f.FullName
            Reason = "fichero suelto de test, sin handle abierto, sin actividad hace $idle min"
        }
    }
    return $items
}

function Set-Old([string]$path, [int]$Minutes) {
    $stamp = (Get-Date).AddMinutes(-1 * ($Minutes + 1))
    Get-ChildItem -LiteralPath $path -Recurse -Force -ErrorAction SilentlyContinue |
        ForEach-Object { $_.LastWriteTime = $stamp }
    (Get-Item -LiteralPath $path -Force).LastWriteTime = $stamp
}

# Único camino de borrado: lo usan el barrido real y el self-check (si el assert
# borrara por su cuenta, el self-check pasaría sin ejercitar este código).
# Re-verifica la guarda por item: entre el inventario y el borrado una DB puede
# reabrirse y, en Windows, `Remove-Item` atraviesa un handle que permita DELETE.
function Invoke-Collect([object[]]$items) {
    $removed = 0
    $freed = [long]0
    $skipped = @()
    foreach ($i in $items) {
        if ($i.Guard -and -not (Test-LockFree $i.Guard)) {
            # Si ya no existe, otro lo borró: no es una duda, no hay nada que reportar.
            if (-not (Test-Path -LiteralPath $i.Guard)) { continue }
            $skipped += [pscustomobject]@{ Item = $i; Kind = 'duda'; Why = 'se abrió después del inventario' }
            continue
        }
        try {
            Remove-Item -LiteralPath (Assert-UnderTemp $i.Path) -Recurse -Force -ErrorAction Stop
            $freed += $i.Bytes
            $removed++
        } catch {
            # Un REFUSED de Assert-UnderTemp es "ante la duda no borrar", no un fallo.
            $kind = $(if ($_.Exception.Message -like 'REFUSED*') { 'duda' } else { 'error' })
            $skipped += [pscustomobject]@{ Item = $i; Kind = $kind; Why = $_.Exception.Message }
        }
    }
    return [pscustomobject]@{ Removed = $removed; Freed = $freed; Skipped = $skipped }
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
    # El share que concede `std::fs::File` en Windows: RW + DELETE. Es el único caso en
    # que `Remove-Item` borra a través de un handle vivo, así que es el que hay que
    # usar para que las pruebas de la guarda no pasen por el motivo equivocado.
    $holdShare = [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete

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
    $realItem = @(Get-Collectable | Where-Object { $_.Path -eq $real })[0]
    $r = Invoke-Collect @($realItem)
    Assert-SC ($r.Removed -eq 1 -and -not (Test-Path $real)) 'Invoke-Collect la borró'
    Assert-SC (-not (Test-Collectable $real)) 'no queda en la lista tras borrarla'

    # La guarda de borrado se RE-VERIFICA tras el inventario: si la DB se reabre en el
    # medio, el item ya inventariado se respeta. Sin esta prueba, borrar la guarda no
    # rompería ningún assert.
    $reopen = Assert-UnderTemp (Join-Path $temp ('.tmp' + ([guid]::NewGuid().ToString('N').Substring(0, 8).ToUpper())))
    New-Item -ItemType Directory -Force -Path (Join-Path $reopen 'data') | Out-Null
    Set-Content -LiteralPath (Join-Path $reopen 'data\lock') -Value '' -NoNewline
    Set-Old $reopen 30
    $reopenItem = @(Get-Collectable | Where-Object { $_.Path -eq $reopen })[0]
    Assert-SC ($null -ne $reopenItem) 'la DB se inventaría con el lock libre'
    # El handle concede DELETE, igual que `std::fs::File`: sin eso `Remove-Item` fallaría
    # solo y el assert pasaría aunque la guarda no existiera (misma vacuidad que en F1).
    $lockAgain = [IO.File]::Open((Join-Path $reopen 'data\lock'), [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, $holdShare)
    $r = Invoke-Collect @($reopenItem)
    Assert-SC ($r.Removed -eq 0 -and (Test-Path $reopen)) 'se reabrió tras el inventario: la guarda la respeta'
    $lockAgain.Dispose()
    $r = Invoke-Collect @($reopenItem)
    Assert-SC ($r.Removed -eq 1 -and -not (Test-Path $reopen)) 'vuelto a cerrar: el mismo item ahora sí se borra'

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

    Write-Host '=== SelfCheck: ficheros sueltos (FIND-743) ==='
    # (1) libre y viejo -> se borra. Recién escrito -> lo protege la inactividad.
    $loose = Assert-UnderTemp (Join-Path $temp ('vanta_test_loose_' + [guid]::NewGuid().ToString('N').Substring(0, 8) + '.corrupt'))
    Set-Content -LiteralPath $loose -Value 'x' -NoNewline
    Assert-SC (-not (Test-Collectable $loose)) 'fichero suelto reciente: la guarda de inactividad lo protege'
    Set-Old $loose 30
    Assert-SC (Test-Collectable $loose) 'fichero suelto viejo y con handle libre: entra en el inventario'
    $r = Invoke-Collect @(Get-Collectable | Where-Object { $_.Path -eq $loose })
    Assert-SC ($r.Removed -eq 1 -and -not (Test-Path $loose)) 'Invoke-Collect lo borró de verdad'

    # (2) abierto (handle propio) -> se respeta, incluso con un item ya inventariado.
    # El handle concede DELETE a propósito: es lo que da `std::fs::File` y es el ÚNICO
    # caso en que `Remove-Item` borra a través del handle. Con un share sin DELETE el
    # assert pasaría aunque la guarda no existiera (el propio Remove-Item fallaría).
    $heldFile = Assert-UnderTemp (Join-Path $temp ('vanta_test_held_' + [guid]::NewGuid().ToString('N').Substring(0, 8) + '.corrupt'))
    Set-Content -LiteralPath $heldFile -Value 'x' -NoNewline
    Set-Old $heldFile 30
    $heldItem = @(Get-Collectable | Where-Object { $_.Path -eq $heldFile })[0]
    Assert-SC ($null -ne $heldItem) 'el fichero se inventaría con el handle libre'
    $holdHandle = [IO.File]::Open($heldFile, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, $holdShare)
    Assert-SC (-not (Test-Collectable $heldFile)) 'fichero suelto abierto: no entra en el inventario'
    $r = Invoke-Collect @($heldItem)
    Assert-SC ($r.Removed -eq 0 -and (Test-Path $heldFile)) 'Invoke-Collect re-verifica el handle y respeta el fichero abierto'
    Assert-SC (@($r.Skipped).Count -eq 1 -and @($r.Skipped)[0].Kind -eq 'duda') 'el respeto se reporta como duda, no como fallo'

    # Control positivo: con ESTE handle, un Remove-Item directo SÍ borra. Si algún día
    # deja de borrar, el assert de arriba queda vacuo y este lo delata.
    $ctrl = Assert-UnderTemp (Join-Path $temp ('vanta_test_ctrl_' + [guid]::NewGuid().ToString('N').Substring(0, 8) + '.corrupt'))
    Set-Content -LiteralPath $ctrl -Value 'x' -NoNewline
    $ctrlHandle = [IO.File]::Open($ctrl, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, $holdShare)
    $ctrlDeleted = $false
    try { Remove-Item -LiteralPath $ctrl -Force -ErrorAction Stop; $ctrlDeleted = $true } catch { $ctrlDeleted = $false }
    $ctrlHandle.Dispose()
    Assert-SC $ctrlDeleted 'control: Remove-Item atraviesa un handle con DELETE (si no, la guarda no sería la que protege)'

    $holdHandle.Dispose()
    Remove-Item -LiteralPath $heldFile -Force -ErrorAction SilentlyContinue

    # (4) nombre ajeno (aunque contenga "vanta") -> intacto.
    foreach ($n in @(('Report_' + [guid]::NewGuid().ToString('N').Substring(0, 6) + '.txt'), ('notas-vanta_' + [guid]::NewGuid().ToString('N').Substring(0, 6) + '.txt'))) {
        $foreignFile = Assert-UnderTemp (Join-Path $temp $n)
        Set-Content -LiteralPath $foreignFile -Value 'x' -NoNewline
        Set-Old $foreignFile 30
        Assert-SC (-not (Test-Collectable $foreignFile)) ("'{0}' no matchea los prefijos: intacto" -f $n)
        Remove-Item -LiteralPath $foreignFile -Force -ErrorAction SilentlyContinue
    }
    Assert-SC ($null -eq (Get-LeakFileKind ([IO.FileInfo](Join-Path $temp 'Report.txt')))) 'un fichero cualquiera no se clasifica'

    # (5) nombre válido pero FUERA de %TEMP% -> la guarda lo rechaza end-to-end: la ruta
    # entra a Invoke-Collect como item y se convierte en skip, nunca en borrado.
    $outsidePath = Join-Path $env:SystemRoot 'vanta_test_outside.wal.corrupt'
    Assert-SC (Test-Refused $outsidePath) 'nombre válido fuera de %TEMP%: Assert-UnderTemp lo rechaza'
    $r = Invoke-Collect @([pscustomobject]@{ Path = $outsidePath; Class = 'file'; Guard = $null; Bytes = 1; Reason = 'inyectado por el self-check' })
    Assert-SC ($r.Removed -eq 0 -and @($r.Skipped | Where-Object { $_.Why -like 'REFUSED*' }).Count -eq 1) 'Invoke-Collect se niega a borrar fuera de %TEMP% y lo reporta'

    # (6) un checkpoint de un PID VIVO no se reclasifica como residuo por el mtime.
    $ownCkpt = [IO.FileInfo](Join-Path $temp ("vanta-ckpt-$PID.jsonl"))
    Assert-SC ($null -ne (Get-LeakFileKind $ownCkpt) -and (Get-LeakFileKind $ownCkpt).Pid -eq $PID) 'un `vanta-ckpt-<pid>` se clasifica con su PID'

    Write-Host ''
    Write-Host ("SELFCHECK: {0} ({1}/{2} asserts)" -f $(if ($fail -eq 0) { 'PASS' } else { 'FAIL' }), $pass, ($pass + $fail))
    exit $(if ($fail -eq 0) { 0 } else { 1 })
}

# --- Recolección ----------------------------------------------------------------
$items = @(Get-Collectable)
$dirs = @($items | Where-Object { $_.Class -eq 'dir' })
$files = @($items | Where-Object { $_.Class -eq 'file' })
$dirBytes = [double](($dirs | Measure-Object -Property Bytes -Sum).Sum)
$fileBytes = [double](($files | Measure-Object -Property Bytes -Sum).Sum)

if (-not $Quiet) {
    Write-Host ("%TEMP% = {0}" -f (Get-TempPrefix))
    foreach ($i in $dirs) { Write-Host ("  [dir ] {0,10}  {1}  [{2}]" -f (Format-Bytes $i.Bytes), $i.Path, $i.Reason) }
    foreach ($i in $files) { Write-Host ("  [file] {0,10}  {1}  [{2}]" -f (Format-Bytes $i.Bytes), $i.Path, $i.Reason) }
    # Las dos clases se reportan SIEMPRE, incluso en cero: un total único deja que
    # "limpio" signifique a la vez "no hay nada" y "no miré la mitad del inventario".
    Write-Host ("dirs : {0} carpeta(s), {1}" -f $dirs.Count, (Format-Bytes ([long]$dirBytes)))
    Write-Host ("files: {0} fichero(s), {1}" -f $files.Count, (Format-Bytes ([long]$fileBytes)))
}

if ($DryRun -or $items.Count -eq 0) {
    if (-not $Quiet -and $items.Count -gt 0) {
        Write-Host ("-DryRun: liberaría {0} en {1} carpeta(s) + {2} fichero(s). Nada se borró." -f (Format-Bytes ([long]($dirBytes + $fileBytes))), $dirs.Count, $files.Count)
    }
    exit 0
}

$result = Invoke-Collect $items

# "Ante la duda, no borrar y reportar": lo que la guarda rechazó se imprime siempre
# (también con -Quiet) — un skip silencioso es un gate ciego. Un fallo de borrado se
# distingue de una duda: son dos cosas distintas y mezclarlas esconde una de las dos.
foreach ($s in $result.Skipped) {
    $label = $(if ($s.Kind -eq 'duda') { 'respetado por duda' } else { 'NO se pudo borrar ' })
    $color = $(if ($s.Kind -eq 'duda') { 'DarkYellow' } else { 'Red' })
    Write-Host ("  {0}: {1}  [{2}]" -f $label, $s.Item.Path, $s.Why) -ForegroundColor $color
}

$dudas = @($result.Skipped | Where-Object { $_.Kind -eq 'duda' }).Count
$errores = $result.Skipped.Count - $dudas
$skippedDirs = @($result.Skipped | Where-Object { $_.Item.Class -eq 'dir' }).Count
$skippedFiles = @($result.Skipped | Where-Object { $_.Item.Class -eq 'file' }).Count
if (-not $Quiet) {
    Write-Host ("recolector: {0}/{1} carpeta(s) + {2}/{3} fichero(s), {4} liberados ({5} respetado(s) por duda, {6} fallo(s))." -f `
        ($dirs.Count - $skippedDirs), $dirs.Count, ($files.Count - $skippedFiles), $files.Count, (Format-Bytes $result.Freed), $dudas, $errores)
} elseif ($result.Removed -gt 0) {
    # -Quiet sigue avisando cuando hizo algo: un gate silencioso que libera GB es un gate ciego.
    Write-Host ("temp-collector: {0} carpeta(s) + {1} fichero(s) huérfano(s) en %TEMP%, {2} liberados." -f `
        ($dirs.Count - $skippedDirs), ($files.Count - $skippedFiles), (Format-Bytes $result.Freed))
}
exit 0
