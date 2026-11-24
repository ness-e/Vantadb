#Requires -Version 7.0
# FIND-192 — higiene de entorno para waves co-batch (disco + huérfanos).
#  1) Reporta disco libre + tamaño de target/ (por perfil) + GB liberables por artefactos seguros.
#  2) -AutoSafe: gate pre-wave NO interactivo. Si el disco libre < -MinFreeGB (default 15) borra
#     SOLO artefactos regenerables bajo target/: {*}/incremental y *.pdb. Nunca target/ entero,
#     nunca nada fuera de target/, nunca procesos. -DryRun reporta sin tocar nada.
#  3) -Clean (interactivo) / -Clean -Full: la limpieza manual de siempre.
#  4) -SelfCheck: asserts de que la lista de objetivos vive bajo target/ y de que el validador
#     rechaza rutas fuera (incluye el truco de prefijo `target2/` y el escape con `..`).
#
# Uso:
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1                      # solo reporte
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -DryRun              # reporte + GB liberables
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -DryRun    # gate pre-wave, no toca nada
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe            # libera si < 15 GB libres
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -AutoSafe -MinFreeGB 25
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -SelfCheck           # asserts de seguridad
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -Clean [-Full] [-Yes]
#
# Exit codes: 0 = ok (reporte / dry-run / limpieza que dejó el disco sobre el umbral)
#             1 = rechazo de seguridad (ruta fuera de target/, flags incompatibles, disco
#                 indeterminable) o self-check FAIL
#             2 = AutoSafe omitido porque hay cargo/rustc en vuelo (advisory: reintentar entre builds)
#             3 = corrió y el disco SIGUE bajo el umbral (o no había nada podable) — NO es verde
param(
    [switch]$Clean,
    [switch]$Full,
    [switch]$Yes,
    [switch]$AutoSafe,
    [switch]$DryRun,
    [double]$MinFreeGB = 15,
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

function Get-FreeGB {
    $d = Get-PSDrive -Name $drive.Name -ErrorAction SilentlyContinue
    if (-not $d) { return $null }
    return [math]::Round($d.Free / 1GB, 2)
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

    $total = $script:scPass + $script:scFail
    Write-Host ''
    Write-Host ("SELFCHECK: {0} ({1}/{2} asserts)" -f $(if ($script:scFail -eq 0) { 'PASS' } else { 'FAIL' }), $script:scPass, $total)
    exit $(if ($script:scFail -eq 0) { 0 } else { 1 })
}

if (-not (Test-Path $target)) { Write-Host "no existe target/ en $repoRoot — nada que hacer."; exit 0 }

# 1) Disco + perfiles
# Ordenado por longitud de Root desc: si el repo vive bajo un PSDrive mapeado, gana el más específico.
$drive = Get-PSDrive -PSProvider FileSystem | Where-Object { $repoRoot.StartsWith($_.Root) } |
    Sort-Object { $_.Root.Length } -Descending | Select-Object -First 1
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
        exit 0
    }

    # Borrar incremental/ a mitad de build deja fingerprints apuntando a sesiones que ya no
    # existen (rustc ICE / "could not read incremental cache"). Se omite, no se rompe el wave.
    $builds = @(Get-Process -Name cargo, rustc, clippy-driver, rustdoc -ErrorAction SilentlyContinue)
    if ($builds.Count -gt 0) {
        Write-Host ("SKIP: {0} cargo/rustc en vuelo — borrar incremental/ a mitad de build corrompe la cache." -f $builds.Count)
        Write-Host 'AutoSafe: reintentar cuando terminen esos builds.'
        exit 2
    }

    $before = Get-FreeGB
    foreach ($i in $safeItems) {
        Write-Host ("  borrando {0} ({1} GB)..." -f $i.Label, (Get-ItemGB $i))
        foreach ($p in $i.Paths) {
            # Re-chequeo por ítem: el bucle puede tardar minutos sobre decenas de GB y una wave
            # co-batch arranca builds todo el tiempo. Muestrear una sola vez deja la ventana abierta.
            if (@(Get-Process -Name cargo, rustc, clippy-driver, rustdoc -ErrorAction SilentlyContinue).Count -gt 0) {
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
    $builds = @(Get-Process -Name cargo, rustc, clippy-driver, rustdoc -ErrorAction SilentlyContinue)
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
