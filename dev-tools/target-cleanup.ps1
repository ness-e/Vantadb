#Requires -Version 7.0
# FIND-192 — higiene de entorno para waves co-batch (disco + huérfanos).
#  1) Reporta disco libre + tamaño de target/ (por perfil) + GB liberables por artefactos seguros.
#  2) -AutoSafe: gate pre-wave NO interactivo. Si el disco libre < -MinFreeGB (default 15) borra
#     SOLO artefactos regenerables bajo target/: {*}/incremental y *.pdb. Nunca target/ entero,
#     nunca nada fuera de target/, nunca procesos. -DryRun reporta sin tocar nada.
#     Con cargo/rustc en vuelo DEGRADA (FIND-746): poda solo *.pdb con antigüedad >= -MinAgeMinutes
#     (default 30) y no toca incremental/ (borrarlo a mitad de build corrompe la cache de rustc).
#     Los *.pdb no participan de esa cache, así que la antigüedad es prueba suficiente de abandono.
#  3) -Clean (interactivo) / -Clean -Full: la limpieza manual de siempre.
#  4) -SelfCheck: asserts de que la lista de objetivos vive bajo target/ y de que el validador
#     rechaza rutas fuera (incluye el truco de prefijo `target2/` y el escape con `..`), más el
#     caso degradado: build simulado en vuelo -> solo se poda el *.pdb viejo, incremental/ intacto.
#
# Uso:
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1                      # solo reporte
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -DryRun              # reporte + GB liberables
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -DryRun    # gate pre-wave, no toca nada
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe            # libera si < 15 GB libres
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -MinFreeGB 25
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -MinAgeMinutes 45   # poda pdb más viejos
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -SelfCheck           # asserts de seguridad
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -Clean [-Full] [-Yes]
#
# Exit codes: 0 = ok (reporte / dry-run / limpieza que dejó el disco sobre el umbral, o modo
#                 degradado que podó al menos algo)
#             1 = rechazo de seguridad (ruta fuera de target/, flags incompatibles, disco
#                 indeterminable, -MinAgeMinutes < 1 con -AutoSafe) o self-check FAIL
#             2 = AutoSafe en modo degradado que NO pudo podar nada y el disco sigue bajo
#                 umbral (advisory: reintentar entre builds) — antes era "hay builds en vuelo,
#                 no se tocó nada"; ahora solo se niega cuando no hay nada seguro que podar
#             3 = corrió y el disco SIGUE bajo el umbral (o no había nada podable) — NO es verde
param(
    [switch]$Clean,
    [switch]$Full,
    [switch]$Yes,
    [switch]$AutoSafe,
    [switch]$DryRun,
    [double]$MinFreeGB = 15,
    [int]$MinAgeMinutes = 30,
    [switch]$SelfCheck
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path $PSScriptRoot -Parent
$target = Join-Path $repoRoot 'target'
$sep = [IO.Path]::DirectorySeparatorChar
$script:targetPrefix = $null

if ($AutoSafe -and ($Clean -or $Full)) {
    Write-Host 'REFUSED: -AutoSafe nunca borra de forma profunda — usá -Clean [-Full] a mano para eso.'
    exit 1
}

# La antigüedad es la ÚNICA prueba de abandono de un *.pdb (no hay lock de compilador que lo
# cubra: el linker escribe el fichero mientras el build está "en vuelo"). 0 la anula.
if ($AutoSafe -and $MinAgeMinutes -lt 1) {
    Write-Host 'REFUSED: -MinAgeMinutes < 1 anula la única prueba de abandono de un *.pdb (un linker puede estar escribiéndolo).'
    exit 1
}

# --- Seguridad: toda ruta a borrar pasa por acá ---------------------------------
function Get-TargetPrefix {
    if ($null -eq $script:targetPrefix) {
        $full = [IO.Path]::GetFullPath($target)
        if (-not $full.EndsWith($sep)) { $full += $sep }
        $script:targetPrefix = $full
    }
    return $script:targetPrefix
}

function Assert-UnderTarget([string]$path) {
    $full = [IO.Path]::GetFullPath($path)
    $prefix = Get-TargetPrefix
    # StartsWith sobre el prefijo CON separador: rechaza hermanos tipo `target2/` y, de paso,
    # el propio `target/` (no termina en separador). Nada fuera de target/ llega a Remove-Item.
    if (-not $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "REFUSED: '$full' no está bajo '$prefix' — AutoSafe no borra fuera de target/"
    }
    return $full
}

function Get-DirSizeGB($path) {
    if (-not (Test-Path $path)) { return 0 }
    $sum = (Get-ChildItem $path -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) { return 0 }
    return [math]::Round($sum / 1GB, 2)
}

function Format-Rel([string]$path) { return $path.Replace($repoRoot + $sep, '') }

# --- Selección de artefactos regenerables (la única lista que AutoSafe puede borrar)
function Get-SafeCleanupItems {
    $items = @()
    if (-not (Test-Path $target)) { return $items }

    foreach ($d in @(Get-ChildItem -LiteralPath $target -Recurse -Directory -Filter 'incremental' -ErrorAction SilentlyContinue)) {
        if ($d.Name -cne 'incremental') { continue }
        $items += [pscustomobject]@{
            Kind  = 'dir'
            Label = (Format-Rel $d.FullName)
            Paths = @((Assert-UnderTarget $d.FullName))
            Bytes = 0
        }
    }

    $pdbs = @()
    $bytes = [long]0
    foreach ($f in @(Get-ChildItem -LiteralPath $target -Recurse -File -Filter '*.pdb' -ErrorAction SilentlyContinue)) {
        $pdbs += (Assert-UnderTarget $f.FullName)
        $bytes += $f.Length
    }
    if ($pdbs.Count -gt 0) {
        $items += [pscustomobject]@{
            Kind  = 'pdb'
            Label = ('**/*.pdb x{0}' -f $pdbs.Count)
            Paths = $pdbs
            Bytes = $bytes
        }
    }
    return $items
}

function Get-ItemGB($item) {
    if ($item.Kind -eq 'dir') { return (Get-DirSizeGB $item.Paths[0]) }
    return [math]::Round($item.Bytes / 1GB, 2)
}

function Get-RepoDrive {
    # Ordenado por longitud de Root desc: si el repo vive bajo un PSDrive mapeado, gana el más específico.
    return Get-PSDrive -PSProvider FileSystem | Where-Object { $repoRoot.StartsWith($_.Root) } |
        Sort-Object { $_.Root.Length } -Descending | Select-Object -First 1
}

function Get-FreeGB {
    # $drive se resuelve más abajo; el self-check corre antes y necesita este helper igual.
    $d = if ($drive) { $drive } else { Get-RepoDrive }
    if (-not $d) { return $null }
    return [math]::Round($d.Free / 1GB, 2)
}

# Única fuente de "hay un build en vuelo": la usan el gate, el bucle de borrado y el self-check.
function Get-InFlightBuilds {
    return @(Get-Process -Name cargo, rustc, clippy-driver, rustdoc -ErrorAction SilentlyContinue)
}

# Write-Host + registro opcional (el self-check asserta sobre el texto del modo degradado).
function Write-Note([string]$message) {
    Write-Host $message
    if ($null -ne $script:noteLog) { $script:noteLog += $message }
}

# --- Modo degradado (FIND-746): hay builds en vuelo, solo caben los *.pdb ---------------
# Borrar incremental/ a mitad de build deja fingerprints apuntando a sesiones que ya no
# existen (rustc ICE / "could not read incremental cache"). Los *.pdb NO participan de esa
# cache: con antigüedad suficiente, podarlos es seguro aunque otro carril esté compilando.
# Antes la guarda era todo-o-nada y el disco llegaba a 0 bytes con poda manual de por medio.

# Selección del modo degradado: UNA sola definición, usada por -AutoSafe y por -SelfCheck.
# Si se toca acá, el self-check lo ve (llama a esta misma función, no a una copia).
function Select-PdbItems([object[]]$Items) {
    return @($Items | Where-Object { $_.Kind -eq 'pdb' })
}

function Invoke-DegradedPrune {
    param(
        [object[]]$PdbItems = @(),
        [int]$BuildCount = 1,
        [int]$MinAgeMinutes = 30,
        [double]$MinFreeGB = 15
    )
    $script:noteLog = @()
    Write-Note ("DEGRADADO: {0} cargo/rustc en vuelo — incremental/ NO se toca (corrompe la cache de rustc)." -f $BuildCount)
    Write-Note ("DEGRADADO: solo se podan *.pdb con antigüedad >= {0} min (-MinAgeMinutes)." -f $MinAgeMinutes)

    $before = Get-FreeGB
    $cutoff = (Get-Date).AddMinutes(-[math]::Abs($MinAgeMinutes))
    $removed = 0
    $failed = 0
    $freedBytes = [long]0
    $tooYoung = 0

    foreach ($i in $PdbItems) {
        if ($i.Kind -ne 'pdb') {
            Write-Note ("  omito {0} — el modo degradado solo poda *.pdb" -f $i.Label)
            continue
        }
        foreach ($p in @($i.Paths)) {
            $file = Get-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue
            if ($null -eq $file) { continue }
            if (($file.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                Write-Note ("  omito {0} (reparse point: el destino puede estar fuera de target/)" -f (Format-Rel $p))
                continue
            }
            if ($file.LastWriteTime -gt $cutoff) { $tooYoung++; continue }
            try {
                $validated = Assert-UnderTarget $p          # última puerta antes de Remove-Item
            } catch {
                # Fallar cerrado y visible: se omite y se dice, no se borra y no se cae el gate.
                Write-Note ("  omito {0}: {1}" -f $p, $_.Exception.Message)
                continue
            }
            Write-Note ("  borrando {0} ({1} MB)..." -f (Format-Rel $validated), [math]::Round($file.Length / 1MB, 1))
            # -ErrorAction Stop dentro del try: si no, el catch es código muerto (el error de
            # "fichero en uso" es no-terminante y el operador ve un ERROR crudo del proveedor).
            try { Remove-Item -LiteralPath $validated -Force -ErrorAction Stop }
            catch { $failed++; Write-Note ("  no se pudo borrar {0}: {1}" -f (Format-Rel $validated), $_.Exception.Message) }
            # Cuenta solo si de verdad desapareció: un handle abierto (linker) hace fallar el borrado.
            if (-not (Test-Path -LiteralPath $validated)) { $removed++; $freedBytes += $file.Length }
        }
    }
    $after = Get-FreeGB

    if ($removed -gt 0) {
        Write-Note ("DEGRADADO: podados {0} *.pdb ({1} GB) con antigüedad >= {2} min." -f $removed, [math]::Round($freedBytes / 1GB, 2), $MinAgeMinutes)
        if ($failed -gt 0) { Write-Note ("DEGRADADO: {0} *.pdb no se pudieron borrar (handle abierto) — se reintentan en la próxima corrida." -f $failed) }
        if (($null -ne $before) -and ($null -ne $after)) {
            if ($after -ge $before) {
                Write-Note ("DEGRADADO: liberados {0} GB ({1} -> {2} GB libres)." -f ([math]::Round($after - $before, 2)), $before, $after)
            } else {
                # El disco es de todos: el delta puede ser negativo aunque hayamos podado.
                Write-Note ("DEGRADADO: podados {0} GB, pero el disco bajó de {1} a {2} GB libres (otros procesos)." -f [math]::Round($freedBytes / 1GB, 2), $before, $after)
            }
        }
        return [pscustomobject]@{ ExitCode = 0; Removed = $removed; Lines = @($script:noteLog) }
    }
    if (($null -ne $after) -and ($after -ge $MinFreeGB)) {
        Write-Note ("DEGRADADO: nada podable, pero quedan {0} GB >= umbral {1} GB — no hay nada que forzar." -f $after, $MinFreeGB)
        return [pscustomobject]@{ ExitCode = 0; Removed = 0; Lines = @($script:noteLog) }
    }
    Write-Note ("DEGRADADO: 0 *.pdb podables ({0} omitido(s) por antigüedad < {1} min, {2} no se pudo borrar) e incremental/ intacto; el disco sigue bajo umbral." -f $tooYoung, $MinAgeMinutes, $failed)
    Write-Note 'AutoSafe: reintentar cuando terminen esos builds, o liberá a mano con -Clean [-Full].'
    return [pscustomobject]@{ ExitCode = 2; Removed = 0; Lines = @($script:noteLog) }
}

# --- SelfCheck (no toca nada) ---------------------------------------------------
function Test-Refused([string]$p) {
    try { $null = Assert-UnderTarget $p; return $false } catch { return $true }
}

if ($SelfCheck) {
    $script:scPass = 0
    $script:scFail = 0
    function Assert-SC([bool]$cond, [string]$label) {
        if ($cond) { Write-Host ("  PASS  {0}" -f $label); $script:scPass++ }
        else { Write-Host ("  FAIL  {0}" -f $label) -ForegroundColor Red; $script:scFail++ }
    }

    Write-Host '=== SelfCheck: AutoSafe no puede tocar nada fuera de target/ ==='
    Assert-SC ((Get-TargetPrefix).EndsWith($sep)) 'el prefijo termina en separador'

    $outside = @(
        (Join-Path $repoRoot 'src/lib.rs'),
        (Join-Path $repoRoot 'dev-tools/target-cleanup.ps1'),
        (Join-Path $repoRoot 'target2/x.pdb'),
        (Join-Path $target '..\src\x.pdb'),
        (Join-Path $target '..'),
        $target
    )
    foreach ($p in $outside) { Assert-SC (Test-Refused $p) ("rechaza: {0}" -f $p) }

    foreach ($p in @((Join-Path $target 'debug/incremental'), (Join-Path $target 'debug/deps/x.pdb'))) {
        Assert-SC (-not (Test-Refused $p)) ("acepta: {0}" -f (Format-Rel $p))
    }

    # La lista REAL de objetivos: se valida entera (no una muestra) pero se reporta agregada.
    $prefix = Get-TargetPrefix
    $items = @(Get-SafeCleanupItems)
    Write-Host ("  objetivos reales: {0} grupo(s), {1} ruta(s)" -f $items.Count, (@($items | ForEach-Object { $_.Paths.Count }) | Measure-Object -Sum).Sum)
    if (-not (Test-Path $target)) {
        Write-Host '  aviso: sin target/ — solo se validaron los casos sintéticos'
    } else {
        Assert-SC ($items.Count -gt 0) 'hay al menos un objetivo real bajo target/'
    }
    foreach ($i in $items) {
        # Test-Refused usa el MISMO validador que el bucle de borrado (no una copia del StartsWith).
        $escaped = @($i.Paths | Where-Object { Test-Refused $_ })
        Assert-SC ($escaped.Count -eq 0) ("todas las rutas bajo target/: {0} ({1})" -f $i.Label, $i.Paths.Count)
        if ($i.Kind -eq 'pdb') {
            Assert-SC (@($i.Paths | Where-Object { $_ -notlike '*.pdb' }).Count -eq 0) ("todas son *.pdb: {0}" -f $i.Label)
        }
        if ($i.Kind -eq 'dir') {
            Assert-SC (@($i.Paths | Where-Object { (Split-Path $_ -Leaf) -cne 'incremental' }).Count -eq 0) ("todos los dirs son incremental/: {0}" -f $i.Label)
        }
    }
    Assert-SC ($scFail -eq 0) 'ningún objetivo ni ruta de prueba escapó de target/'

    # --- Modo degradado (FIND-746): build en vuelo -> solo *.pdb con antigüedad -----------
    Write-Host ''
    Write-Host '=== SelfCheck: modo degradado (build en vuelo -> solo *.pdb con antigüedad) ==='
    $sandbox = Join-Path ([IO.Path]::GetTempPath()) ('find746-sc-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
    $sandboxTarget = Join-Path $sandbox 'target'
    $oldPdb = Join-Path $sandboxTarget 'debug/deps/old.pdb'
    $newPdb = Join-Path $sandboxTarget 'debug/deps/new.pdb'
    $incrementalDir = Join-Path $sandboxTarget 'debug/incremental/s-abc'
    $fakeBuild = $null
    $savedPrefix = $script:targetPrefix
    try {
        $null = New-Item -ItemType Directory -Force -Path (Split-Path $oldPdb -Parent)
        $null = New-Item -ItemType Directory -Force -Path $incrementalDir
        Set-Content -LiteralPath (Join-Path $incrementalDir 'dep-graph.bin') -Value 'incremental'
        Set-Content -LiteralPath $oldPdb -Value 'pdb viejo'
        Set-Content -LiteralPath $newPdb -Value 'pdb reciente'
        (Get-Item -LiteralPath $oldPdb).LastWriteTime = (Get-Date).AddMinutes(-90)
        (Get-Item -LiteralPath $newPdb).LastWriteTime = (Get-Date).AddMinutes(-1)
        # El dir 'incremental' también es VIEJO: si la guarda de Kind desapareciera, solo ella
        # puede salvarlo (con mtime reciente el filtro de edad lo protegía por el motivo equivocado).
        (Get-Item -LiteralPath $incrementalDir).LastWriteTime = (Get-Date).AddMinutes(-120)

        # Build simulado en vuelo: un rustc.exe REAL (copia de cmd.exe) para que la detección
        # lo cuente igual que a un build de otro carril. Sin esto el assert sería vacuo.
        $fakeExe = Join-Path $sandbox 'rustc.exe'
        Copy-Item -LiteralPath $env:ComSpec -Destination $fakeExe -Force
        $fakeBuild = Start-Process -FilePath $fakeExe -ArgumentList '/c', 'ping -n 30 127.0.0.1 >nul' -PassThru -WindowStyle Hidden
        $inFlight = 0
        for ($n = 0; $n -lt 40 -and $inFlight -eq 0; $n++) {
            Start-Sleep -Milliseconds 100
            $inFlight = @(Get-InFlightBuilds).Count
        }
        # EL señuelo, no "algún build": con otros carriles compilando, count>0 sería vacuo.
        Assert-SC (@(Get-InFlightBuilds | Where-Object { $_.Id -eq $fakeBuild.Id }).Count -eq 1) ("la detección incluye el build simulado (id {0} de {1} en vuelo)" -f $fakeBuild.Id, $inFlight)

        # El prefijo pasa a ser el sandbox: se reusa el MISMO Assert-UnderTarget del bucle real.
        $script:targetPrefix = [IO.Path]::GetFullPath($sandboxTarget) + $sep
        $pdbItem = [pscustomobject]@{ Kind = 'pdb'; Label = '**/*.pdb x2'; Paths = @($oldPdb, $newPdb); Bytes = 0 }
        $dirItem = [pscustomobject]@{ Kind = 'dir'; Label = 'target/debug/incremental'; Paths = @($incrementalDir); Bytes = 0 }
        # Señuelo fuera del prefijo vigente (hermano de target/): la última puerta tiene que verlo.
        $outsidePdb = Join-Path $sandbox 'outside.pdb'
        Set-Content -LiteralPath $outsidePdb -Value 'pdb fuera de target'
        (Get-Item -LiteralPath $outsidePdb).LastWriteTime = (Get-Date).AddMinutes(-90)
        $outsideItem = [pscustomobject]@{ Kind = 'pdb'; Label = 'fuera/target'; Paths = @($outsidePdb); Bytes = 0 }
        # Junction con nombre *.pdb y mtime viejo: pasaría la edad y el prefijo, así que la única
        # guarda que lo para es el salto de reparse points (el destino puede estar fuera).
        $junctionPdb = Join-Path $sandboxTarget 'debug/deps/junction.pdb'
        $null = New-Item -ItemType Directory -Force -Path (Join-Path $sandbox 'outside-dir')
        try { $null = New-Item -ItemType Junction -Path $junctionPdb -Target (Join-Path $sandbox 'outside-dir') -ErrorAction Stop }
        catch { $junctionPdb = $null }
        $mixedItems = @($pdbItem, $dirItem, $outsideItem)
        if ($null -ne $junctionPdb) {
            (Get-Item -LiteralPath $junctionPdb -Force).LastWriteTime = (Get-Date).AddMinutes(-90)
            $mixedItems += [pscustomobject]@{ Kind = 'pdb'; Label = 'junction.pdb'; Paths = @($junctionPdb); Bytes = 0 }
        } else {
            Write-Host '  aviso: sin permisos para crear un junction — ese assert se omite'
        }

        # (1) CABLEADO REAL: se llama con la MISMA selección que usa -AutoSafe (Select-PdbItems).
        #     Si esa selección se rompe, esto falla por comportamiento, no por regex de log.
        $selected = @(Select-PdbItems $mixedItems)
        $expectedSelected = if ($null -ne $junctionPdb) { 3 } else { 2 }
        Assert-SC (($selected.Count -eq $expectedSelected) -and (@($selected | Where-Object { $_.Kind -ne 'pdb' }).Count -eq 0)) 'degradado: Select-PdbItems deja solo los grupos *.pdb (es lo que usa -AutoSafe)'
        $r1 = Invoke-DegradedPrune -PdbItems $selected -BuildCount $inFlight -MinAgeMinutes 30 -MinFreeGB 999999
        $log1 = ($r1.Lines -join "`n")
        Assert-SC ($r1.ExitCode -eq 0) 'degradado: exit 0 cuando podó algo (aunque el disco siga bajo umbral)'
        Assert-SC (-not (Test-Path -LiteralPath $oldPdb)) 'degradado: el *.pdb de 90 min se podó'
        Assert-SC (Test-Path -LiteralPath $newPdb) 'degradado: el *.pdb de 1 min se respetó (-MinAgeMinutes)'
        Assert-SC (Test-Path -LiteralPath $incrementalDir) 'degradado: incremental/ quedó intacto'
        Assert-SC ($log1 -notmatch 'borrando.*incremental') 'degradado: ni siquiera intentó borrar incremental/'
        if ($null -ne $junctionPdb) {
            Assert-SC (Test-Path -LiteralPath $junctionPdb) 'degradado: no sigue un junction (el destino puede estar fuera)'
            Assert-SC ($log1 -match 'reparse point') 'degradado: el junction omitido se reporta'
        }
        Assert-SC (Test-Path -LiteralPath $outsidePdb) 'degradado: no borra una ruta fuera del prefijo (Assert-UnderTarget sigue siendo la puerta)'
        Assert-SC ($log1 -match 'no está bajo') 'degradado: el rechazo de la ruta de fuera se reporta'
        Assert-SC ($log1 -match 'DEGRADADO') 'degradado: el output dice que está en modo degradado'
        Assert-SC ($log1 -match 'incremental/') 'degradado: el output dice por qué no toca incremental/'

        # (2) GUARDA INTERNA de Kind: se le pasa el dir directo, sin pasar por Select-PdbItems.
        $rGuard = Invoke-DegradedPrune -PdbItems @($dirItem) -BuildCount $inFlight -MinAgeMinutes 30 -MinFreeGB 999999
        $logGuard = ($rGuard.Lines -join "`n")
        Assert-SC (Test-Path -LiteralPath $incrementalDir) 'degradado: la guarda interna tampoco deja tocar incremental/'
        Assert-SC ($logGuard -match 'el modo degradado solo poda \*\.pdb') 'degradado: la guarda interna de Kind se anuncia en el output'
        Assert-SC ($logGuard -notmatch 'borrando') 'degradado: la guarda interna no llega a intentar el borrado'

        # (3) La guarda de -MinAgeMinutes < 1 es CLI: se ejerce el script real como hijo.
        $childOut = & ([Environment]::ProcessPath) -NoProfile -File $PSCommandPath -AutoSafe -MinAgeMinutes 0 2>&1 | Out-String
        $childCode = $LASTEXITCODE
        Assert-SC (($childCode -eq 1) -and ($childOut -match 'REFUSED')) 'degradado: -MinAgeMinutes 0 con -AutoSafe se rechaza (exit 1)'

        # Segunda pasada: solo queda el *.pdb reciente y el disco sigue bajo umbral (forzado).
        $r2 = Invoke-DegradedPrune -PdbItems @($pdbItem) -BuildCount $inFlight -MinAgeMinutes 30 -MinFreeGB 999999
        $log2 = ($r2.Lines -join "`n")
        Assert-SC ($r2.ExitCode -eq 2) 'degradado: exit 2 solo si no podó nada y el disco sigue bajo umbral'
        Assert-SC (Test-Path -LiteralPath $newPdb) 'degradado: la segunda pasada no borró el reciente'
        Assert-SC ($log2 -match 'DEGRADADO') 'degradado: la negativa también anuncia el modo'
        Assert-SC ($log2 -match 'reintentar') 'degradado: la negativa dice qué hacer (reintentar entre builds)'
    } catch {
        # En un disco a 0 bytes (el escenario que este gate existe para resolver) el sandbox
        # puede no crearse: que sea un FAIL del self-check, no una excepción sin mensaje.
        Assert-SC $false ("degradado: no se pudo preparar el caso simulado: {0}" -f $_.Exception.Message)
    } finally {
        $script:targetPrefix = $savedPrefix
        $script:noteLog = $null
        # Solo matar si el PID sigue siendo NUESTRO rustc.exe (un PID reciclado no se toca).
        if (($null -ne $fakeBuild) -and ((Get-Process -Id $fakeBuild.Id -ErrorAction SilentlyContinue).ProcessName -eq 'rustc')) {
            Stop-Process -Id $fakeBuild.Id -Force -ErrorAction SilentlyContinue
        }
        Remove-Item -LiteralPath $sandbox -Recurse -Force -ErrorAction SilentlyContinue
    }

    $total = $script:scPass + $script:scFail
    Write-Host ''
    Write-Host ("SELFCHECK: {0} ({1}/{2} asserts)" -f $(if ($script:scFail -eq 0) { 'PASS' } else { 'FAIL' }), $script:scPass, $total)
    exit $(if ($script:scFail -eq 0) { 0 } else { 1 })
}

if (-not (Test-Path $target)) { Write-Host "no existe target/ en $repoRoot — nada que hacer."; exit 0 }

# 1) Disco + perfiles
$drive = Get-RepoDrive
Write-Host '=== Disco ==='
$freeGB = $null
if ($drive) {
    $freeGB = Get-FreeGB
    Write-Host ("{0} libre: {1} GB" -f $drive.Name, $freeGB)
    Write-Host ("umbral AutoSafe: {0} GB libres (-MinFreeGB)" -f $MinFreeGB)
}

Write-Host ''
Write-Host '=== target/ por perfil ==='
$profiles = Get-ChildItem $target -Directory -ErrorAction SilentlyContinue
foreach ($p in $profiles) {
    Write-Host ("  {0,-28} {1,8} GB" -f $p.Name, (Get-DirSizeGB $p.FullName))
}

# 2) Artefactos seguros (siempre reportados: es lo que -DryRun promete)
$safeItems = @(Get-SafeCleanupItems)
$safeRows = @($safeItems | ForEach-Object {
        [pscustomobject]@{ Kind = $_.Kind; Label = $_.Label; GB = (Get-ItemGB $_) }
    })
$safeSum = ($safeRows | Measure-Object -Property GB -Sum).Sum
if ($null -eq $safeSum) { $safeSum = 0 }
$safeGB = [math]::Round($safeSum, 2)

Write-Host ''
Write-Host '=== artefactos seguros (AutoSafe: {*}/incremental + *.pdb) ==='
if ($safeRows.Count -eq 0) {
    Write-Host '  ninguno'
} else {
    foreach ($r in $safeRows) { Write-Host ("  {0,-40} {1,8} GB" -f $r.Label, $r.GB) }
}
Write-Host ("  GB liberables: {0}" -f $safeGB)

# 3) AutoSafe — gate pre-wave no interactivo
if ($AutoSafe) {
    Write-Host ''
    Write-Host '=== AutoSafe ==='
    if ($null -eq $freeGB) { Write-Host 'no se pudo determinar el disco de este repo — no se borra nada.'; exit 1 }

    if ($freeGB -ge $MinFreeGB) {
        Write-Host ("SKIP: {0} GB libres >= umbral {1} GB — no se toca nada." -f $freeGB, $MinFreeGB)
        exit 0
    }
    Write-Host ("BAJO UMBRAL: {0} GB libres < {1} GB." -f $freeGB, $MinFreeGB)

    if ($safeRows.Count -eq 0) {
        # Verde falso clásico: bajo umbral, sin candidatos, exit 0 → un runner arranca la wave igual.
        Write-Host 'nada podable bajo target/ (sin incremental/ ni *.pdb) y el disco sigue bajo umbral.'
        Write-Host 'AutoSafe: liberá a mano con -Clean [-Full] o llevá la wave a otro disco.'
        exit 3
    }

    if ($DryRun) {
        foreach ($i in $safeItems) {
            Write-Host ("  [dry-run] borraría {0} ({1} GB)" -f $i.Label, (Get-ItemGB $i))
        }
        Write-Host ("  [dry-run] liberaría {0} GB. Nada se borró." -f $safeGB)
        # Un dry-run durante un build no debe prometer lo que la corrida real no haría (FIND-746).
        $inFlightNow = @(Get-InFlightBuilds)
        if ($inFlightNow.Count -gt 0) {
            Write-Host ("  [dry-run] aviso: {0} cargo/rustc en vuelo — la corrida real DEGRADARÍA a *.pdb con antigüedad >= {1} min (incremental/ no se toca)." -f $inFlightNow.Count, $MinAgeMinutes)
        }
        exit 0
    }

    # Borrar incremental/ a mitad de build deja fingerprints apuntando a sesiones que ya no
    # existen (rustc ICE / "could not read incremental cache"). Antes esto abortaba la poda
    # ENTERA — incluidos los *.pdb, que no participan de esa cache — y el disco llegaba a 0
    # bytes con poda manual de por medio (FIND-746). Ahora degrada a los *.pdb con antigüedad.
    $builds = @(Get-InFlightBuilds)
    if ($builds.Count -gt 0) {
        $degraded = Invoke-DegradedPrune -PdbItems (Select-PdbItems $safeItems) `
            -BuildCount $builds.Count -MinAgeMinutes $MinAgeMinutes -MinFreeGB $MinFreeGB
        exit $degraded.ExitCode
    }

    $before = Get-FreeGB
    foreach ($i in $safeItems) {
        Write-Host ("  borrando {0} ({1} GB)..." -f $i.Label, (Get-ItemGB $i))
        foreach ($p in $i.Paths) {
            # Re-chequeo por ítem: el bucle puede tardar minutos sobre decenas de GB y una wave
            # co-batch arranca builds todo el tiempo. Muestrear una sola vez deja la ventana abierta.
            if ((Get-InFlightBuilds).Count -gt 0) {
                Write-Host 'ABORT: apareció un build en vuelo — se corta acá para no corromper la cache incremental.'
                exit 2
            }
            $validated = Assert-UnderTarget $p   # última puerta antes de Remove-Item
            # Un junction/symlink pasaría la validación de string pero su destino puede estar fuera.
            if (((Get-Item -LiteralPath $validated -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                Write-Host ("  omito {0} (reparse point: el destino puede estar fuera de target/)" -f (Format-Rel $validated))
                continue
            }
            Remove-Item -LiteralPath $validated -Recurse -Force -ErrorAction Continue
        }
    }
    $after = Get-FreeGB
    Write-Host ("AutoSafe: liberados {0} GB ({1} -> {2} GB libres)." -f ([math]::Round($after - $before, 2)), $before, $after)
    if ($after -lt $MinFreeGB) {
        Write-Host ("AVISO: quedan {0} GB < umbral {1} GB — liberá a mano con -Clean [-Full] antes de la wave." -f $after, $MinFreeGB)
        exit 3
    }
    exit 0
}

# 4) Limpieza manual (interactiva, la de siempre)
if ($Clean -and -not $DryRun) {
    if (-not $Yes) {
        $ans = Read-Host "¿Borrar incremental/$(if ($Full) { ' + debug/deps' } else { '' })? [y/N]"
        if ($ans -notmatch '^[yY]') { Write-Host 'cancelado.'; exit 0 }
    }
    # Aviso, no bloqueo: acá decidió un humano (-Yes incluido), pero el riesgo es el mismo.
    $builds = @(Get-InFlightBuilds)
    if ($builds.Count -gt 0) {
        Write-Host ("AVISO: {0} build(s) en vuelo — borrar incremental/ ahora puede corromper la cache (ver troubleshooting)." -f $builds.Count)
    }
    Write-Host ''
    Write-Host '=== Limpieza (incremental/) ==='
    foreach ($d in @($safeItems | Where-Object { $_.Kind -eq 'dir' })) {
        foreach ($p in $d.Paths) {
            Write-Host ("  borrando {0}..." -f (Format-Rel $p))
            Remove-Item -LiteralPath (Assert-UnderTarget $p) -Recurse -Force -ErrorAction Continue
        }
    }
    if ($Full) {
        Write-Host '=== Limpieza profunda (debug/deps) ==='
        $deps = Join-Path $target 'debug/deps'
        if (Test-Path $deps) {
            $gb = Get-DirSizeGB $deps
            Write-Host ("  borrando {0} ({1} GB) — el próximo build será en frío..." -f (Format-Rel $deps), $gb)
            Remove-Item -LiteralPath (Assert-UnderTarget $deps) -Recurse -Force -ErrorAction Continue
        }
    }
    if ($drive) { Write-Host ("{0} libre ahora: {1} GB" -f $drive.Name, (Get-FreeGB)) }
} elseif ($Clean -and $DryRun) {
    Write-Host ''
    Write-Host '-DryRun: -Clean se omite (nada se borró).'
} else {
    Write-Host ''
    Write-Host '(-AutoSafe [-DryRun] = gate pre-wave; -Clean [-Full] = limpieza manual)'
}

# 5) Procesos (solo lista; no se mata nada)
Write-Host ''
Write-Host '=== Procesos cargo/rustc/vanta-* (revisar a mano) ==='
$procs = Get-Process -Name cargo, rustc, vanta-cli, vanta-proxy -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Select-Object Id, ProcessName, @{N = 'Start'; E = { $_.StartTime } } | Format-Table -AutoSize | Out-String -Width 120 | Write-Host
} else {
    Write-Host '  sin procesos cargo/rustc/vanta-* activos.'
}
