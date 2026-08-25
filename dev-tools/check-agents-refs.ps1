# check-agents-refs.ps1 — anti-drift de referencias en AGENTS.md
#
# Extrae las rutas citadas entre backticks de AGENTS.md (raíz) y .opencode/AGENTS.md
# y valida que existan en el repo. Reporta las que faltan y sale con exit 1 si hay stale.
#
# Filtro anti-falsos-positivos (solo valida rutas relativas que parecen archivos del repo):
#   - descarta whitespace (comandos/prosa), URLs (://), ~ (externo), / inicial (absoluto),
#     placeholders/globs ([<>*?#]), tokens sin separador de ruta, leaves placeholder (X.md)
#   - valida SOLO rutas con extensión de archivo (evita marcar directorios/comandos)
#
# Uso: pwsh -NoProfile dev-tools/check-agents-refs.ps1

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

$files = @("$ProjectRoot\AGENTS.md", "$ProjectRoot\.opencode\AGENTS.md")
$stale = [System.Collections.Generic.List[string]]::new()
$checked = 0

foreach ($f in $files) {
    if (-not (Test-Path -LiteralPath $f)) { continue }
    $content = Get-Content -LiteralPath $f -Raw

    foreach ($m in [regex]::Matches($content, '`([^`]+)`')) {
        $tok = $m.Groups[1].Value.Trim()
        $p = $tok -replace ':\d+.*$', ''   # quita "archivo:linea"
        $p = $p.TrimEnd('/', '\')

        if ($p -match '\s')          { continue }  # comando o prosa
        if ($p -match '^[a-z]+://')  { continue }  # URL
        if ($p -match '^~')          { continue }  # ruta externa (~/.config/...)
        if ($p -match '^/')          { continue }  # absoluta (/pipeline)
        if ($p -match '[<>*?#]')     { continue }  # placeholder o glob
        if ($p -notmatch '[/\\]')    { continue }  # no es ruta (P2-1, cargo, etc.)
        $leaf = Split-Path -Leaf $p
        if ($leaf -cmatch '^[A-Z]\.[A-Za-z0-9]+$') { continue }  # placeholder X.md (case-sensitive)
        if (-not [System.IO.Path]::GetExtension($p)) { continue } # solo archivos con extensión

        $checked++
        if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot $p))) {
            $stale.Add($p)
        }
    }
}

if ($stale.Count -gt 0) {
    Write-Host "check-agents-refs: $($stale.Count) stale ref(s) of $checked checked" -ForegroundColor Red
    $stale | Sort-Object -Unique | ForEach-Object { Write-Host "  MISSING: $_" -ForegroundColor Red }
    exit 1
}

Write-Host "check-agents-refs: OK ($checked refs)" -ForegroundColor Green
exit 0
