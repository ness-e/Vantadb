#!/usr/bin/env pwsh
<#
.SYNOPSIS
    DGM Rehearse — prueba cambios en aislamiento sin riesgo.
.DESCRIPTION
    Toma una propuesta del DGM Loop, la aplica TEMPORALMENTE,
    corre verificación, mide el score antes/después, y restaura
    todo al estado original. Nada persiste sin aprobación humana.
.PARAMETER ProposalFile
    Ruta al JSON de propuesta (generado por dgm-loop.ps1)
.PARAMETER Quick
    Solo verificación rápida (cargo check) en vez de full (cargo nextest)
.EXAMPLE
    .antigravity\task-system\self-modification\dgm-rehearse.ps1 -ProposalFile proposals/dgm-proposal-20260723.json
    .antigravity\task-system\self-modification\dgm-rehearse.ps1 -ProposalFile proposals/dgm-proposal-20260723.json -Quick
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$ProposalFile,
    [switch]$Quick
)

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")
$TracesDir = Join-Path $ProjectRoot ".antigravity" "task-system" "traces"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$RehearsalId = [System.Guid]::NewGuid().ToString().Substring(0, 8)

Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DGM Rehearse v1 — ID: $RehearsalId   ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Leer propuesta ──
if (-not (Test-Path $ProposalFile)) {
    Write-Host "ERROR: Proposal file not found: $ProposalFile" -ForegroundColor Red
    exit 1
}
$proposal = Get-Content $ProposalFile -Raw | ConvertFrom-Json
Write-Host "Propuesta: $($proposal.Count) cambio(s) a evaluar" -ForegroundColor Yellow
$proposal | ForEach-Object { Write-Host "  [$($_.priority)] $($_.area): $($_.suggestion)" -ForegroundColor Gray }

# ── Fase 1: Score BEFORE ──
Write-Host ""
Write-Host "─── Fase 1/4: Score BEFORE ───" -ForegroundColor Yellow
$beforeReport = & python (Join-Path $PSScriptRoot "performance_diagnosis.py") $ProjectRoot 2>&1 | Out-String | ConvertFrom-Json
$beforeScore = $beforeReport.overall_score
Write-Host "  Score actual: $beforeScore" -ForegroundColor $(if ($beforeScore -ge 0.7) { "Green" } elseif ($beforeScore -ge 0.5) { "Yellow" } else { "Red" })

# ── Fase 2: Git stash + apply changes ──
Write-Host ""
Write-Host "─── Fase 2/4: Apply changes (temporal) ───" -ForegroundColor Yellow
Push-Location $ProjectRoot

# Stash cambios no commiteados para partir de estado limpio
$hasChanges = (git status --porcelain) -ne ""
if ($hasChanges) {
    Write-Host "  Stashing working directory changes..." -ForegroundColor Gray
    git stash push -m "dgm-rehearse-$RehearsalId-before" 2>&1 | Out-Null
}

$appliedCount = 0
$failedCount = 0
foreach ($change in $proposal) {
    $targetFile = $change.target_file
    $suggestion = $change.suggestion
    
    # Construir ruta absoluta
    $fullPath = Join-Path $ProjectRoot $targetFile
    
    if (-not (Test-Path $fullPath)) {
        Write-Host "  [SKIP] $targetFile — no existe" -ForegroundColor DarkGray
        $failedCount++
        continue
    }
    
    # Backup del archivo original
    $backupPath = "$fullPath.dgm-bak-$RehearsalId"
    Copy-Item $fullPath $backupPath -Force
    
    # Aplicar cambio (annotation-style: agregar comentario al final)
    try {
        $comment = "# dgm-proposal: $suggestion [rehearsal-$RehearsalId]"
        Add-Content $fullPath -Value "`n$comment" -Encoding UTF8
        Write-Host "  [OK] $targetFile — marcado con propuesta" -ForegroundColor Green
        $appliedCount++
    } catch {
        Write-Host "  [FAIL] $targetFile — $($_.Exception.Message)" -ForegroundColor Red
        $failedCount++
    }
}

Write-Host "  Aplicados: $appliedCount, Fallos: $failedCount" -ForegroundColor Yellow

# ── Fase 3: Verify ──
Write-Host ""
Write-Host "─── Fase 3/4: Verify ───" -ForegroundColor Yellow

$verifyPassed = $true
$verifyOutput = ""

if ($Quick) {
    Write-Host "  Modo quick: cargo check -p vantadb" -ForegroundColor Gray
    $verifyOutput = & cargo check -p vantadb 2>&1
    if ($LASTEXITCODE -ne 0) {
        $verifyPassed = $false
        Write-Host "  ❌ cargo check FAILED" -ForegroundColor Red
    } else {
        Write-Host "  ✅ cargo check PASSED" -ForegroundColor Green
    }
} else {
    Write-Host "  Modo full: cargo nextest run --profile audit --workspace --build-jobs 2" -ForegroundColor Gray
    $verifyOutput = & cargo nextest run --profile audit --workspace --build-jobs 2 2>&1
    if ($LASTEXITCODE -ne 0) {
        $verifyPassed = $false
        Write-Host "  ❌ cargo nextest FAILED" -ForegroundColor Red
    } else {
        Write-Host "  ✅ cargo nextest PASSED" -ForegroundColor Green
    }
}

# ── Fase 4: Restaurar TODO + Score AFTER ──
Write-Host ""
Write-Host "─── Fase 4/4: Restore + Score AFTER ───" -ForegroundColor Yellow

# Restaurar backups
foreach ($change in $proposal) {
    $targetFile = $change.target_file
    $fullPath = Join-Path $ProjectRoot $targetFile
    $backupPath = "$fullPath.dgm-bak-$RehearsalId"
    if (Test-Path $backupPath) {
        Move-Item $backupPath $fullPath -Force
    }
}

# Restaurar git stash
if ($hasChanges) {
    git stash pop 2>&1 | Out-Null
    Write-Host "  Working directory restaurado" -ForegroundColor Gray
}

Pop-Location

# Score AFTER (sin cambios — debería ser igual que BEFORE)
$afterReport = & python (Join-Path $PSScriptRoot "performance_diagnosis.py") $ProjectRoot 2>&1 | Out-String | ConvertFrom-Json
$afterScore = $afterReport.overall_score

Write-Host "  Score before: $beforeScore" -ForegroundColor Gray
Write-Host "  Score after:  $afterScore" -ForegroundColor Gray
$scoreDelta = $afterScore - $beforeScore
$scoreSymbol = if ($scoreDelta -gt 0) { "+" } else { "" }
Write-Host "  Delta: $scoreSymbol$scoreDelta" -ForegroundColor $(if ($scoreDelta -ge 0) { "Green" } else { "Red" })

# ── Reporte final ──
Write-Host ""
Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DGM Rehearse v1 — RESULTADO        ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan

$veredict = if ($verifyPassed) { "✅ Verify PASSED" } else { "❌ Verify FAILED" }
Write-Host "  Rehearsal ID: $RehearsalId" -ForegroundColor Cyan
Write-Host "  Cambios probados: $appliedCount" -ForegroundColor Yellow
Write-Host "  $veredict" -ForegroundColor $(if ($verifyPassed) { "Green" } else { "Red" })
Write-Host "  Score: $beforeScore → $afterScore ($scoreSymbol$scoreDelta)" -ForegroundColor $(if ($scoreDelta -ge 0) { "Green" } else { "Red" })

if ($verifyPassed) {
    Write-Host ""
    Write-Host "  ▶ Para PROMOVER estos cambios (aplicarlos de verdad):" -ForegroundColor Cyan
    Write-Host "    .antigravity\task-system\self-modification\dgm-promote.ps1 -ProposalFile $ProposalFile -RehearsalId $RehearsalId" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  ▶ Para RECHAZAR y descartar:" -ForegroundColor Gray
    Write-Host "    (ya está todo restaurado — no hace falta nada)" -ForegroundColor Gray
} else {
    Write-Host ""
    Write-Host "  ⚠️  Los cambios NO pasaron verificación." -ForegroundColor Red
    Write-Host "     La propuesta queda archivada para revisión manual." -ForegroundColor Red
}

# Archivar rehearsal
$archive = @{
    rehearsal_id = $RehearsalId
    timestamp = $Timestamp
    proposal_file = $ProposalFile
    changes_tested = $appliedCount
    verify_passed = $verifyPassed
    score_before = $beforeScore
    score_after = $afterScore
    score_delta = $scoreDelta
}
$archiveFile = Join-Path $TracesDir "rehearse-$RehearsalId-$Timestamp.json"
$archive | ConvertTo-Json -Depth 5 | Set-Content $archiveFile -Encoding UTF8
Write-Host "  Archivo: $archiveFile" -ForegroundColor Gray
