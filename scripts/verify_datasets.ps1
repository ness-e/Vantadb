# verify_datasets.ps1 — Pre-test gate for heavy-certification workflow (Windows).
#
# PowerShell mirror of scripts/verify_datasets.sh. Same contract: exit 0 when
# all 5 canonical datasets present, exit 1 when any is missing.
#
# Tested in TBH-01 (2026-08-30). See .opencode/skills/campaign-executor/tasks/TBH-01.md.
#
# Usage:
#   pwsh scripts/verify_datasets.ps1         # human-readable table, exit 0/1
#   pwsh scripts/verify_datasets.ps1 -Json   # machine-readable JSON

[CmdletBinding()]
param(
    [switch]$Json
)

$ErrorActionPreference = "Stop"

# Dataset definitions: name | source-script | paths
$DATASETS = @(
    @{ Name = "SIFT-1M"; Source = "dev-tools/scripts/download_sift.py"; Paths = @(
        "datasets/sift/sift_base.fvecs",
        "datasets/sift/sift_query.fvecs",
        "datasets/sift/sift_groundtruth.ivecs"
    ) },
    @{ Name = "GloVe-100"; Source = "scripts/download_benchmark_datasets.sh"; Paths = @(
        "data/benchmark/glove.6B.100d.txt"
    ) },
    @{ Name = "GloVe-300"; Source = "scripts/download_benchmark_datasets.sh"; Paths = @(
        "data/benchmark/glove.6B.300d.txt"
    ) },
    @{ Name = "SIFT-128 euclidean subset"; Source = "scripts/download_ground_truth.py"; Paths = @(
        "data/benchmark/sift-128/train.f32",
        "data/benchmark/sift-128/test.f32",
        "data/benchmark/sift-128/test_neighbors.u64",
        "data/benchmark/sift-128/meta.json"
    ) },
    @{ Name = "GloVe-100 angular subset"; Source = "scripts/download_ground_truth.py"; Paths = @(
        "data/benchmark/glove-100-angular/train.f32",
        "data/benchmark/glove-100-angular/test.f32",
        "data/benchmark/glove-100-angular/test_neighbors.u64",
        "data/benchmark/glove-100-angular/meta.json"
    ) }
)

$results = @()
$missing = 0

foreach ($ds in $DATASETS) {
    $missingPaths = @()
    foreach ($p in $ds.Paths) {
        if (-not (Test-Path -LiteralPath $p)) {
            $missingPaths += $p
        }
    }
    if ($missingPaths.Count -eq 0) {
        $results += @{ name = $ds.Name; status = "OK"; action = "none" }
    } else {
        $missing++
        $action = "run $($ds.Source) (missing: $($missingPaths -join ', '))"
        $results += @{ name = $ds.Name; status = "MISSING"; action = $action }
    }
}

if ($Json) {
    $payload = @{ missing = $missing; datasets = $results }
    $payload | ConvertTo-Json -Compress
}
else {
    Write-Host "=== verify_datasets: $missing missing / $($DATASETS.Count) expected ==="
    $nameWidth = 26
    $statusWidth = 10
    $ruleName = [string]::new("-", $nameWidth + 2)
    $ruleStatus = [string]::new("-", $statusWidth + 2)
    $ruleAction = [string]::new("-", 60)
    Write-Host ("| {0,-$nameWidth} | {1,-$statusWidth} | {2}" -f "dataset", "status", "action")
    Write-Host ("|{0}|{1}|{2}" -f $ruleName, $ruleStatus, $ruleAction)
    foreach ($r in $results) {
        Write-Host ("| {0,-$nameWidth} | {1,-$statusWidth} | {2}" -f $r.name, $r.status, $r.action)
    }
}

if ($missing -gt 0) {
    exit 1
}
exit 0
