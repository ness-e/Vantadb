# Coverage Report & Per-Directory Budget — HARD-02 (decisión owner (c) 2026-09-27).
#
# Vivía dentro de verify.ps1 (fast gate local); movido al nightly (nightly.yml job
# `coverage-budget`) para restaurar el fast gate <5min. El gate canónico de CI
# ADR-018 (root crate ≥80%, ci-rust.yml) NO se toca.
#
# Consumidores:
#   - nightly.yml job `coverage-budget` (enforcement diario)
#   - on-demand local: `pwsh dev-tools/coverage-budget.ps1`
#     (o `pwsh dev-tools/verify.ps1 -IncludeCoverage`, que lo corre como step)
#
# Semántica: corre llvm-cov (reporte JSON) + agrega por directorio y compara contra
# el presupuesto (ratchet: baseline − 1.0 pt). Exit 1 SOLO en violación explícita de
# un bucket con presupuesto; los buckets sin presupuesto son report-only. Si
# cargo-llvm-cov no está instalado → warning + exit 0 (nunca bloquea por tool ausente).
#
# Política + tabla: docs/dev/operations/CI_POLICY.md §"Coverage — Report & Per-Directory
# Budget". Los números del budget espejan CI_POLICY.md — actualizar ambos juntos.
# Baseline medido 2026-09-26 (Windows MSVC); el nightly corre en ubuntu-latest (delta
# de plataforma esperado <1pt — solo 24 líneas cfg(windows/unix); la primera corrida
# recalibra si un bucket difiere legítimamente — ver CI_POLICY §Coverage).

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

. (Join-Path $PSScriptRoot "gate-common.ps1")
$feats = Get-CoreFeatures
$fastGateFilter = Get-FastGateFilter

$CoverageBudget = @{
    "src (root files)" = 90.0
    "src/vector"       = 94.7
    "src/index"        = 86.4
    "src/storage"      = 88.0
    "src/sdk"          = 89.1
    "src/parser"       = 96.2
}

# Agrega el export JSON de llvm-cov por directorio.
# Buckets: src/<dir>/... → src/<dir>; src/<archivo>.rs → "src (root files)";
# archivos fuera de src/ se omiten (los budgets solo cubren src/).
function Get-CoverageByDirectory($jsonPath, $root) {
    if (-not (Test-Path $jsonPath)) { throw "coverage JSON not found: $jsonPath" }
    $data = Get-Content $jsonPath -Raw | ConvertFrom-Json
    $normRoot = ($root -replace '\\', '/')
    $agg = @{}
    foreach ($f in $data.data[0].files) {
        $rel = ($f.filename -replace '\\', '/')
        if ($rel.StartsWith($normRoot, [StringComparison]::OrdinalIgnoreCase)) { $rel = $rel.Substring($normRoot.Length) }
        $rel = $rel.TrimStart('/')
        if ($rel -notlike 'src/*') { continue }
        $seg = $rel -split '/'
        $bucket = if ($seg.Count -ge 3) { "$($seg[0])/$($seg[1])" } else { "$($seg[0]) (root files)" }
        if (-not $agg[$bucket]) { $agg[$bucket] = @{ Lines = 0; Covered = 0 } }
        $agg[$bucket].Lines += [int]$f.summary.lines.count
        $agg[$bucket].Covered += [int]$f.summary.lines.covered
    }
    $agg.GetEnumerator() | ForEach-Object {
        $pct = if ($_.Value.Lines -gt 0) { [math]::Round(100.0 * $_.Value.Covered / $_.Value.Lines, 1) } else { 0.0 }
        [pscustomobject]@{ Name = $_.Name; Lines = $_.Value.Lines; Pct = $pct }
    }
}

if (-not (Get-Command "cargo-llvm-cov" -ErrorAction SilentlyContinue)) {
    Write-Host "llvm-cov not installed - skipping coverage report" -ForegroundColor DarkYellow
    exit 0
}

# El JSON queda como artefacto de auditoría en el target dir activo.
$covDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $ProjectRoot "target" }
if (-not [IO.Path]::IsPathRooted($covDir)) { $covDir = Join-Path $ProjectRoot $covDir }
$covJson = Join-Path $covDir "coverage-report.json"

Write-Host "  coverage..." -ForegroundColor Yellow -NoNewline
if (Get-Command "cargo-nextest" -ErrorAction SilentlyContinue) {
    $cmd = @("cargo", "llvm-cov", "nextest", "--profile", "audit", "-p", "vantadb") + $feats + @("--build-jobs", "1", "-E", $fastGateFilter, "--json", "--output-path", $covJson)
} else {
    $cmd = @("cargo", "llvm-cov", "-p", "vantadb") + $feats + @("--json", "--output-path", $covJson, "-j", "1", "--", "--skip", "benchmark", "--skip", "competitive", "--skip", "recall", "--skip", "sift", "--skip", "chaos", "--skip", "hnsw_hard_validation", "--skip", "stress_protocol", "--skip", "vector_scale", "--skip", "certification", "--skip", "security_audit", "--skip", "deserialize_absurd_node_count", "--skip", "test_search_with_bizarre_text_query", "--skip", "test_malformed_payload_extremely_large")
}
& $cmd[0] $cmd[1..($cmd.Length - 1)]
if ($LASTEXITCODE -ne 0) { Write-Host " FAIL" -ForegroundColor Red; exit 1 }
Write-Host " ok" -ForegroundColor Green

$covReport = Get-CoverageByDirectory $covJson $ProjectRoot
$violations = @()
foreach ($row in $covReport | Sort-Object Name) {
    $budget = $CoverageBudget[$row.Name]
    $status = "-"; $color = "DarkGray"
    if ($null -ne $budget) {
        if ($row.Pct -lt $budget) { $status = "BUDGET VIOLATED (min $budget%)"; $color = "Red"; $violations += "$($row.Name): $($row.Pct)% < $budget%" }
        else { $status = "ok (budget $budget%)"; $color = "Green" }
    }
    Write-Host ("    {0,-18} {1,6}% lines ({2} L)  {3}" -f $row.Name, $row.Pct, $row.Lines, $status) -ForegroundColor $color
}
Write-Host "    coverage report JSON: $covJson" -ForegroundColor DarkGray
if ($violations.Count -gt 0) {
    Write-Host "  coverage budget VIOLATED: $($violations -join '; ')" -ForegroundColor Red
    exit 1
}
Write-Host "  coverage budget OK" -ForegroundColor Green
exit 0
