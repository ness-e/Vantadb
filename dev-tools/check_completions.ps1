#!/usr/bin/env pwsh
# check_completions.ps1 - gate de frescura del snapshot `completions/`.
#
# Por que existe: build.rs escribe en OUT_DIR (publish-safe), NO en el repo. El
# snapshot versionado solo se refresca a mano con VANTA_OUT_DIR=completions, y
# nada lo verificaba: los 4 archivos quedaron sin el subcomando `encryption`
# (anadido 2026-10-21) hasta el 2026-10-30.
#
# Uso:
#   pwsh -NoProfile -File dev-tools/check_completions.ps1           # verifica
#   pwsh -NoProfile -File dev-tools/check_completions.ps1 -Refresh  # regenera
#
# Exit 0 = snapshot al dia; exit 1 = stale (o fallo de build).
param(
    [switch]$Refresh
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$target = Join-Path $repo 'completions'

# Genera a un directorio temporal y compara byte a byte contra el snapshot.
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("vanta-completions-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    $env:VANTA_OUT_DIR = $tmp
    & cargo build -p vantadb --features cli --bin vanta-cli --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: cargo build fallo; no se puede generar el snapshot" -ForegroundColor Red
        exit 1
    }

    $expected = @('vanta-cli.bash', 'vanta-cli.fish', '_vanta-cli', '_vanta-cli.ps1')
    $stale = @()

    foreach ($name in $expected) {
        $gen = Join-Path $tmp $name
        if (-not (Test-Path $gen)) {
            $stale += "$name (no generado)"
            continue
        }
        $cur = Join-Path $target $name
        if (-not (Test-Path $cur)) {
            $stale += "$name (falta en completions/)"
            continue
        }
        $a = (Get-FileHash $gen -Algorithm SHA256).Hash
        $b = (Get-FileHash $cur -Algorithm SHA256).Hash
        if ($a -ne $b) { $stale += "$name (desactualizado)" }
    }

    if ($Refresh) {
        Copy-Item (Join-Path $tmp '*') $target -Force
        Write-Host "completions/ regenerado ($($expected.Count) archivos) desde src/cli.rs" -ForegroundColor Green
        exit 0
    }

    if ($stale.Count -gt 0) {
        Write-Host "FAIL: completions/ desactualizado respecto a src/cli.rs:" -ForegroundColor Red
        foreach ($s in $stale) { Write-Host "  - $s" -ForegroundColor Red }
        Write-Host ""
        Write-Host "Arreglo: pwsh -NoProfile -File dev-tools/check_completions.ps1 -Refresh" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "OK: completions/ coincide con src/cli.rs ($($expected.Count) archivos)" -ForegroundColor Green
    exit 0
}
finally {
    Remove-Item Env:\VANTA_OUT_DIR -ErrorAction SilentlyContinue
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
