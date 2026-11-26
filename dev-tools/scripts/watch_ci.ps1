#!/usr/bin/env pwsh
param([int]$Timeout = 600)
$elapsed = 0
while ($elapsed -lt $Timeout) {
    $runs = gh run list --branch main --limit 5 --json workflowName,conclusion,status,number | ConvertFrom-Json
    $inProgress = $runs | Where-Object { $_.status -eq 'in_progress' }
    Clear-Host
    Write-Host "=== CI Workflow Monitor ($elapsed s) ==="
    Write-Host ""
    foreach ($run in $runs) {
        $icon = if ($run.status -eq 'in_progress') { '...' } elseif ($run.conclusion -eq 'success') { 'OK' } elseif ($run.conclusion -eq 'failure') { 'FAIL' } else { $run.conclusion }
        Write-Host "[$icon] $($run.workflowName) #$($run.number)"
    }
    if ($inProgress.Length -eq 0) {
        Write-Host ""
        Write-Host "All workflows completed!"
        exit 0
    }
    Start-Sleep -Seconds 15
    $elapsed += 15
}
Write-Host "Timeout reached ($Timeout s)"
exit 1
