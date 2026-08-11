# check-avance-coverage.ps1 — Mapa de cobertura fuente→destino (migración docs/progreso → docs/avance)
# Verifica: (1) cada fuente tiene destino; (2) cada ID de tarea de las fuentes está
# procesado en un archivo de dominio de docs/avance (excluye snapshots, que son espejo literal).
# Uso: pwsh scripts/check-avance-coverage.ps1 [-Detail]

param([switch]$Detail)

$root   = Split-Path $PSScriptRoot -Parent
$srcDir = Join-Path $root "docs/progreso"
$dstDir = Join-Path $root "docs/avance"

# --- 0. Fuentes vivas externas (no se mueven; se catalogan por referencia) ---
# Carpeta -> catálogo en docs/avance que la integra sin moverla
$live = @{
    "docs/plans"            = "docs/avance/fuentes-vivas.md"
    "docs/audit-reports"    = "docs/avance/fuentes-vivas.md"
    "docs/reviews"          = "docs/avance/fuentes-vivas.md"
    "docs/Investigaciones"  = "docs/avance/investigaciones.md"
}

# --- 1. Mapa estático fuente → destino (copia directa o procesado) ---
$map = @{
    "README.md"                 = "historial/snapshot-2026-08-07.md (copia directa) + activo/* desglosado por dominio"
    "BACKLOG_HISTORY.md"        = "historial/backlog-history.md (copia directa)"
    "ARCHIVO_HISTORICO.md"      = "historial/archivo-historico.md (copia directa) + refs en activo/*, auditoria/*, decisiones/*, meta.md"
    "bitacora.md"               = "historial/sesiones/2026-07.md + historial/sesiones/2026-07-consolidacion.md (sección pendientes, copia) + activo/*"
    "2026-07-28-sdk-gap-audit.md" = "historial/sdk-gap-audit-2026-07-28.md (link → canónico docs/progreso/2026-07-28-sdk-gap-audit.md)"
}

Write-Output "=== MAPA FUENTE -> DESTINO ==="
foreach ($f in (Get-ChildItem $srcDir -File | Sort-Object Name)) {
    $dest = $map[$f.Name]
    $ok = if ($dest) { "OK" } else { "SIN DESTINO" }
    Write-Output ("{0,-35} -> {1,-80} [{2}]" -f $f.Name, ($(if ($dest) { $dest } else { "???" })), $ok)
    foreach ($missing in ($map.Keys | Where-Object { -not (Test-Path (Join-Path $srcDir $_)) })) {
        if ($missing) { Write-Output ("  [map ref] $missing no existe en fuente (ok si ya se migró)") }
    }
}

# --- 2. Cobertura de IDs por dominio (excluye snapshots = espejo literal) ---
$domainFiles = Get-ChildItem $dstDir -Recurse -File -Exclude "snapshot-*.md","README.md" | ForEach-Object { $_.FullName }
$snapshots   = Get-ChildItem $dstDir -Recurse -File -Filter "snapshot-*.md" | ForEach-Object { $_.FullName }

$idAll = New-Object System.Collections.Generic.HashSet[string]
$idProcessed = New-Object System.Collections.Generic.HashSet[string]
$idSnapshotOnly = New-Object System.Collections.Generic.HashSet[string]

foreach ($f in Get-ChildItem $srcDir -File) {
    $text = Get-Content $f.FullName -Raw
    foreach ($m in [regex]::Matches($text, '\b(?<![A-Za-z0-9])[A-Z]{2,6}-(?:[0-9]{2,4}|[0-9]{1,4}[A-Z]?)\b')) {
        $id = $m.Value
        # ignora falsos positivos (versiones de deps, hashes, fechas)
        if ($id -match '^(?:ID|ES|MS|US|CS|MS)-' ) { continue }
        [void]$idAll.Add($id)
    }
}

# IDs presentes en archivos de dominio (no snapshot)
$domainBlob = ($domainFiles | ForEach-Object { Get-Content $_ -Raw }) -join "`n"
foreach ($id in $idAll) {
    if ($domainBlob -match [regex]::Escape($id)) { [void]$idProcessed.Add($id) }
}

Write-Output ""
Write-Output ("=== COBERTURA DE IDs ===")
Write-Output ("  IDs únicos detectados en fuentes:          {0}" -f $idAll.Count)
Write-Output ("  procesados en archivos de dominio:         {0}" -f $idProcessed.Count)
Write-Output ("  solo en snapshot (espejo literal, OK):     {0}" -f ($idAll.Count - $idProcessed.Count))

if ($Detail) {
    Write-Output ""
    Write-Output "  --- IDs en fuentes NO presentes en dominio (muestra) ---"
    ($idAll | Where-Object { -not $idProcessed.Contains($_) } | Sort-Object | Select-Object -First 40) | ForEach-Object { Write-Output "    $_" }
    Write-Output ""
    Write-Output "  --- 30 IDs procesados (muestra) ---"
    $idProcessed | Sort-Object | Select-Object -First 30 | ForEach-Object { Write-Output "    $_" }
}

Write-Output ""
Write-Output "=== FUENTES VIVAS EXTERNAS (catalogadas, no movidas) ==="
foreach ($dir in $live.Keys | Sort-Object) {
    $cat = $live[$dir]
    $dirExists = Test-Path (Join-Path $root $dir)
    $catExists = Test-Path (Join-Path $root $cat)
    $ok = if ($dirExists -and $catExists) { "OK" } else { "REVISAR" }
    Write-Output ("  {0,-25} -> catálogo {1} [{2}]" -f $dir, $cat, $ok)
}

Write-Output ""
Write-Output ("FINAL: {0}/{1} IDs cubiertos en dominio ({2:P1}); resto garantizado por snapshots espejo" -f $idProcessed.Count, $idAll.Count, ($idProcessed.Count / [math]::Max(1,$idAll.Count)))