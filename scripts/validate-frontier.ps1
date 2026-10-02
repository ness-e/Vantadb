<#
.SYNOPSIS
  Valida la frontera de producto declarada en EXPERIMENTAL_FEATURES.md contra el repo real (offline).
.DESCRIPTION
  Checks mecánicos:
    a) cada fila de la sección Production-Facing resuelve a features/paths existentes;
    b) una feature marcada Deferred que sigue viva en Cargo.toml lleva marca de estado vivo;
    c) toda feature/workspace-member real está clasificada en el doc (o en la allowlist interna).
  Mensajes accionables `archivo:línea [row] -> qué falta`. Exit 0 si todo resuelve; 1 si no.
.EXAMPLE
  pwsh -NoProfile -File scripts/validate-frontier.ps1
  pwsh -NoProfile -File scripts/validate-frontier.ps1 -DocPath $env:TEMP/perturbed.md   # dry-run RED
#>
param(
  [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path,
  [string]$DocPath = '',
  [string]$CargoPath = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $DocPath) { $DocPath = Join-Path $Root 'docs/user/operations/EXPERIMENTAL_FEATURES.md' }
if (-not $CargoPath) { $CargoPath = Join-Path $Root 'Cargo.toml' }
$docLabel = if ($DocPath.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase)) {
  $DocPath.Substring($Root.Length).TrimStart('\', '/')
} else { $DocPath }

$script:exitCode = 0
function Fail {
  param([string]$Where, [string]$What)
  Write-Host "❌ $Where → $What" -ForegroundColor Red
  $script:exitCode = 1
}

# Features internas (backends/allocators/CI/test toggles) fuera de la frontera de producto.
# Una feature NUEVA en Cargo.toml hace fallar este gate hasta clasificarla: en el doc
# (evidencia `feature:X`) o en esta allowlist explícita (decisión DEF-03 Spec #2).
$internalFeatures = @(
  'default', 'async-ingestion', 'async-io', 'arrow', 'rocksdb', 'fjall', 'roaring',
  'failpoints', 'python_sdk', 'custom-allocator', 'jemalloc', 'wasm', 'opentelemetry',
  'hot-reload', 'encryption', 'bayesian_decay', 'tls', 'prometheus', 'rayon', 'sysinfo', 'tui'
)
$aliveMarker = '(?i)(shipped|exists|opt-in|partial|pending)'
$pathLike = '\.(rs|md|toml|json|ya?ml|py|mjs|sh|ts|tsx|js)$'

$cargoText = Get-Content -LiteralPath $CargoPath -Raw
$featBlock = [regex]::Match($cargoText, '(?s)\[features\]\r?\n(.*?)\r?\n\[')
if (-not $featBlock.Success) { Write-Host "❌ $CargoPath → no [features] section found" -ForegroundColor Red; exit 1 }
$cargoFeatures = @([regex]::Matches($featBlock.Groups[1].Value, '(?m)^([\w-]+)\s*=') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)

$membersBlock = [regex]::Match($cargoText, '(?s)(?m)\[workspace\].*?^members\s*=\s*\[(.*?)\]')
if (-not $membersBlock.Success) { Write-Host "❌ $CargoPath → no [workspace].members found" -ForegroundColor Red; exit 1 }
$members = @([regex]::Matches($membersBlock.Groups[1].Value, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)

$docText = Get-Content -LiteralPath $DocPath -Raw
$docLines = Get-Content -LiteralPath $DocPath

# (1) doc → Cargo: toda feature citada en el doc debe existir.
$docFeatures = @([regex]::Matches($docText, 'feature:([\w-]+)') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
foreach ($f in $docFeatures) {
  if ($f -notin $cargoFeatures) {
    Fail "$docLabel (doc-wide)" "feature '$f' is referenced in the doc but missing from Cargo.toml [features]"
  }
}

# (2) Cargo → doc: toda feature real debe estar clasificada.
foreach ($f in $cargoFeatures) {
  if (($f -notin $docFeatures) -and ($f -notin $internalFeatures)) {
    Fail $docLabel "Cargo feature '$f' is neither listed in the doc (feature:$f) nor in the internal allowlist — classify it"
  }
}

# (3) workspace members: existen en disco y están clasificados en el doc.
foreach ($mb in $members) {
  if ($mb -eq '.') { continue }
  if (-not (Test-Path -LiteralPath (Join-Path $Root $mb) -PathType Container)) {
    Fail $CargoPath "workspace member '$mb' directory not found"
  }
  if (-not $docText.Contains($mb)) {
    Fail $docLabel "workspace member '$mb' is not classified in the frontier doc (add a row or note it)"
  }
}

# (4) filas: Production-Facing → evidencia existe; Deferred con feature viva → marca de estado.
$section = ''
$prodRows = 0
for ($i = 0; $i -lt $docLines.Count; $i++) {
  $line = $docLines[$i]
  if ($line -match '^##\s+(.+)$') { $section = $Matches[1].Trim(); continue }
  if (-not $line.StartsWith('|')) { continue }
  $cells = $line.Split('|')
  if ($cells.Count -lt 5) { continue }
  $row = $cells[1].Trim()
  if (($row -eq 'Area') -or ($row -match '^-+$')) { continue }
  # Evidence = 3ª columna de las tablas 3-col (Production-Facing MVP / Deferred); las tablas
  # de 4 columnas (Optional/Utilities) no se path-checkean — ver check (4) en .DESCRIPTION.
  $evidence = $cells[3]
  $where = "$docLabel`:$($i + 1) [row '$row']"
  if ($section -eq 'Production-Facing MVP') { $prodRows++ }
  $checkable = 0
  $tokens = @([regex]::Matches($evidence, '`([^`]+)`') | ForEach-Object { $_.Groups[1].Value })
  foreach ($tkn in $tokens) {
    if ($tkn -like 'feature:*') {
      $checkable++
      $fname = $tkn.Substring(8)
      if ($section -eq 'Production-Facing MVP') {
        if ($fname -notin $cargoFeatures) { Fail $where "feature '$fname' not found in Cargo.toml [features]" }
      } elseif (($section -eq 'Deferred') -and ($fname -in $cargoFeatures) -and ($line -notmatch $aliveMarker)) {
        Fail $where "feature '$fname' is alive in Cargo.toml but the Deferred row carries no alive marker (shipped/exists/opt-in/partial/pending)"
      }
    } elseif ($section -eq 'Production-Facing MVP') {
      $p = $tkn -replace ':\d+$', ''
      if (($p -match '/') -or ($p -match $pathLike)) {
        $checkable++
        if (-not (Test-Path -LiteralPath (Join-Path $Root $p))) { Fail $where "path '$tkn' does not exist in the repo" }
      }
    }
  }
  if (($section -eq 'Production-Facing MVP') -and ($checkable -eq 0)) {
    Fail $where "row has no machine-checkable evidence (feature:X or repo path)"
  }
}

$namedMembers = @($members | Where-Object { $_ -ne '.' })
if ($prodRows -eq 0) {
  Fail $docLabel "no Production-Facing MVP rows found — section renamed, table reformatted, or doc truncated"
}
if ($script:exitCode -eq 0) {
  Write-Host "✅ validate-frontier — $prodRows production rows · $($docFeatures.Count) doc features · $($namedMembers.Count) members — all resolved" -ForegroundColor Green
} else {
  Write-Host "`n❌ validate-frontier — frontier drift detected (see ❌ lines above); doc: $docLabel" -ForegroundColor Yellow
}
exit $script:exitCode
