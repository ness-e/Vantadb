#Requires -Version 7.0
# FIND-192 — higiene de entorno para waves co-batch (disco + huérfanos).
#  1) Reporta disco libre + tamaño de target/ (por perfil).
#  2) Con -Clean borra lo 100% regenerable (incremental/); con -Full además debug/deps (rebuild en frío).
#  3) Lista procesos cargo/rustc/vanta-* activos — NO los mata (revisar a mano).
#
# Uso:
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1                 # solo reporte
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -Clean          # + borra incremental/ (pide confirmación)
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -Clean -Full    # + borra debug/deps (rebuild en frío)
#   pwsh -NoProfile -File dev-tools/target-cleanup.ps1 -Clean -Yes     # sin confirmación (automatización)
param(
    [switch]$Clean,
    [switch]$Full,
    [switch]$Yes
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path $PSScriptRoot -Parent
$target = Join-Path $repoRoot 'target'
if (-not (Test-Path $target)) { Write-Host "no existe target/ en $repoRoot — nada que hacer."; exit 0 }

function Get-DirSizeGB($path) {
    if (-not (Test-Path $path)) { return 0 }
    $sum = (Get-ChildItem $path -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) { return 0 }
    return [math]::Round($sum / 1GB, 2)
}

# 1) Disco + perfiles
$drive = Get-PSDrive -PSProvider FileSystem | Where-Object { $repoRoot.StartsWith($_.Root) } | Select-Object -First 1
Write-Host '=== Disco ==='
if ($drive) { Write-Host ("{0} libre: {1} GB" -f $drive.Name, [math]::Round($drive.Free / 1GB, 2)) }

Write-Host ''
Write-Host '=== target/ por perfil ==='
$profiles = Get-ChildItem $target -Directory -ErrorAction SilentlyContinue
foreach ($p in $profiles) {
    Write-Host ("  {0,-28} {1,8} GB" -f $p.Name, (Get-DirSizeGB $p.FullName))
}

# 2) Limpieza
if ($Clean) {
    if (-not $Yes) {
        $ans = Read-Host "¿Borrar incremental/$(if ($Full) { ' + debug/deps' } else { '' })? [y/N]"
        if ($ans -notmatch '^[yY]') { Write-Host 'cancelado.'; exit 0 }
    }
    Write-Host ''
    Write-Host '=== Limpieza (incremental/) ==='
    $incremental = Get-ChildItem $target -Recurse -Directory -Filter 'incremental' -ErrorAction SilentlyContinue
    foreach ($d in $incremental) {
        $gb = Get-DirSizeGB $d.FullName
        Write-Host ("  borrando {0} ({1} GB)..." -f $d.FullName.Replace($repoRoot, '.'), $gb)
        Remove-Item $d.FullName -Recurse -Force -ErrorAction Continue
    }
    if ($Full) {
        Write-Host '=== Limpieza profunda (debug/deps) ==='
        $deps = Join-Path $target 'debug/deps'
        if (Test-Path $deps) {
            $gb = Get-DirSizeGB $deps
            Write-Host ("  borrando {0} ({1} GB) — el próximo build será en frío..." -f $deps.Replace($repoRoot, '.'), $gb)
            Remove-Item $deps -Recurse -Force -ErrorAction Continue
        }
    }
    if ($drive) { Write-Host ("{0} libre ahora: {1} GB" -f $drive.Name, [math]::Round((Get-PSDrive $drive.Name).Free / 1GB, 2)) }
} else {
    Write-Host ''
    Write-Host '(-Clean para borrar incremental/; -Clean -Full además debug/deps)'
}

# 3) Procesos (solo lista; no se mata nada)
Write-Host ''
Write-Host '=== Procesos cargo/rustc/vanta-* (revisar a mano) ==='
$procs = Get-Process -Name cargo, rustc, vanta-cli, vanta-proxy -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Select-Object Id, ProcessName, @{N = 'Start'; E = { $_.StartTime } } | Format-Table -AutoSize | Out-String -Width 120 | Write-Host
} else {
    Write-Host '  sin procesos cargo/rustc/vanta-* activos.'
}
