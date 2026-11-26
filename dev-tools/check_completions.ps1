#!/usr/bin/env pwsh
# check_completions.ps1 - gate de frescura del snapshot `completions/`.
#
# Por que existe: el snapshot versionado solo se refresca a mano, y nada lo
# verificaba: los 4 archivos quedaron sin el subcomando `encryption`
# (anadido 2026-10-21) hasta el 2026-10-30.
#
# Mecanismo: compila vanta-cli una vez, genera los 4 scripts con el binario
# (`vanta-cli completions --shell <sh>`) a un directorio temporal y compara
# sha256 contra el snapshot. Asi el gate prueba el mismo camino que usa el
# usuario, sin variables de entorno magicas.
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
$bin = Join-Path $repo 'target/debug/vanta-cli.exe'
if (-not (Test-Path $bin)) { $bin = Join-Path $repo 'target/debug/vanta-cli' }

# Genera a un directorio temporal y compara byte a byte contra el snapshot.
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("vanta-completions-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    & cargo build -p vantadb --features cli --bin vanta-cli --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: cargo build fallo; no se puede generar el snapshot" -ForegroundColor Red
        exit 1
    }

    $shells = @('bash', 'zsh', 'fish', 'powershell')
    $names = @{
        'bash'       = 'vanta-cli.bash'
        'zsh'        = '_vanta-cli'
        'fish'       = 'vanta-cli.fish'
        'powershell' = '_vanta-cli.ps1'
    }
    foreach ($sh in $shells) {
        & $bin completions --shell $sh > (Join-Path $tmp $names[$sh])
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAIL: 'vanta-cli completions --shell $sh' fallo" -ForegroundColor Red
            exit 1
        }
    }

    $stale = @()
    foreach ($name in $names.Values) {
        $gen = Join-Path $tmp $name
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
        Write-Host "completions/ regenerado ($($names.Count) archivos) desde el binario" -ForegroundColor Green
        exit 0
    }

    if ($stale.Count -gt 0) {
        Write-Host "FAIL: completions/ desactualizado respecto a src/cli.rs:" -ForegroundColor Red
        foreach ($s in $stale) { Write-Host "  - $s" -ForegroundColor Red }
        Write-Host ""
        Write-Host "Arreglo: pwsh -NoProfile -File dev-tools/check_completions.ps1 -Refresh" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "OK: completions/ coincide con src/cli.rs ($($names.Count) archivos)" -ForegroundColor Green
    exit 0
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
