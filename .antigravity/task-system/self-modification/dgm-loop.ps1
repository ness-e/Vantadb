#!/usr/bin/env pwsh
<#
.SYNOPSIS
    DGM Loop — Diagnose → Propose → Rehearse → Promote → Archive
.DESCRIPTION
    Loop de mejora continua SIN RIESGO para el campaign executor.
    Fases:
      1. DIAGNOSE: ejecuta performance_diagnosis.py sobre el pipeline actual
      2. PROPOSE: genera propuestas de cambio concretas
      3. REHEARSE (externo): dgm-rehearse.ps1 prueba los cambios en aislamiento
      4. PROMOTE (externo): dgm-promote.ps1 aplica solo con aprobación humana
      5. ARCHIVE: registra todo para trazabilidad
    SEGURIDAD: NUNCA modifica archivos de producción directamente.
    Los cambios pasan por rehearsal (aislado, reversible) y gate humano.
.PARAMETER PipelinePath
    Ruta al pipeline a diagnosticar (default: raíz del proyecto)
.PARAMETER Quick
    Solo fase DIAGNOSE (más rápido)
.EXAMPLE
    # Diagnóstico rápido
    .antigravity\task-system\self-modification\dgm-loop.ps1 -Quick

    # Loop completo (genera propuesta)
    .antigravity\task-system\self-modification\dgm-loop.ps1

    # Probar propuesta sin riesgo
    .antigravity\task-system\self-modification\dgm-rehearse.ps1 -ProposalFile proposals/dgm-proposal-NNN.json -Quick

    # Promover (solo si rehearsal pasó)
    .antigravity\task-system\self-modification\dgm-promote.ps1 -ProposalFile proposals/dgm-proposal-NNN.json -RehearsalId <ID>
#>

param(
    [string]$PipelinePath = (Get-Location).Path,
    [switch]$Quick
)

$ReportDir = Join-Path $PipelinePath ".antigravity" "task-system" "traces"
$ProposalsDir = Join-Path $PipelinePath ".antigravity" "task-system" "self-modification" "proposals"
$DiagnosisScript = Join-Path $PipelinePath ".antigravity" "task-system" "self-modification" "performance_diagnosis.py"

# Ensure dirs exist
New-Item -ItemType Directory -Path $ReportDir -Force | Out-Null
New-Item -ItemType Directory -Path $ProposalsDir -Force | Out-Null

$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$CampaignId = [System.Guid]::NewGuid().ToString().Substring(0, 8)

Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DGM Loop v1 — Campaign $CampaignId   ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Phase 1: DIAGNOSE ──
Write-Host "─── Phase 1/5: DIAGNOSE ───" -ForegroundColor Yellow

if (-not (Test-Path $DiagnosisScript)) {
    Write-Host "⚠️  performance_diagnosis.py no encontrado en:" -ForegroundColor Red
    Write-Host "   $DiagnosisScript" -ForegroundColor Gray
    Write-Host "   El DGM Loop requiere este script. Instalá Tool 14 primero." -ForegroundColor Red
    exit 1
}

$diagnosisResult = & python $DiagnosisScript $PipelinePath 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Diagnosis falló:" -ForegroundColor Red
    Write-Host $diagnosisResult -ForegroundColor Red
    exit 1
}

$diagnosis = $diagnosisResult | Out-String | ConvertFrom-Json

$score = $diagnosis.overall_score
Write-Host "   Overall score: $score" -ForegroundColor $(
    if ($score -ge 0.7) { "Green" } elseif ($score -ge 0.5) { "Yellow" } else { "Red" }
)

$diagnosis.pipeline_health_issues | ForEach-Object { Write-Host "   ⚕️  $_" -ForegroundColor DarkYellow }
$diagnosis.budget_issues | ForEach-Object { Write-Host "   💰 $_" -ForegroundColor DarkYellow }
$diagnosis.stagnation_patterns | ForEach-Object { Write-Host "   🔄 $_" -ForegroundColor DarkYellow }
$diagnosis.error_rate_issues | ForEach-Object { Write-Host "   ❌ $_" -ForegroundColor DarkYellow }
$diagnosis.recitation_issues | ForEach-Object { Write-Host "   📝 $_" -ForegroundColor DarkYellow }

if ($Quick) {
    Write-Host ""
    Write-Host "Quick mode: solo diagnóstico completado." -ForegroundColor Gray
    Write-Host "Report: $ReportDir\diagnosis-$Timestamp.json"
    $diagnosis | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $ReportDir "diagnosis-$Timestamp.json") -Encoding UTF8
    return
}

# ── Phase 2: PROPOSE ──
Write-Host ""
Write-Host "─── Phase 2/5: PROPOSE ───" -ForegroundColor Yellow

$proposals = @()

# Health issues → proposals
foreach ($issue in $diagnosis.pipeline_health_issues) {
    $proposals += [PSCustomObject]@{
        id = [System.Guid]::NewGuid().ToString().Substring(0, 6)
        area = "pipeline_health"
        issue = $issue
        suggestion = if ($issue -match "No plan files") { "Create initial plan file from Backlog.md" } `
                     elseif ($issue -match "Low task completion") { "Review task complexity — split large tasks into smaller steps" } `
                     else { "Review pipeline structure" }
        target_file = if ($issue -match "No plan files") { "docs/plans/" } else { "iter-loop-tools.md" }
        priority = if ($issue -match "No plan files") { "high" } else { "medium" }
    }
}

# Budget issues → proposals
foreach ($issue in $diagnosis.budget_issues) {
    $proposals += [PSCustomObject]@{
        id = [System.Guid]::NewGuid().ToString().Substring(0, 6)
        area = "budget"
        issue = $issue
        suggestion = if ($issue -match "No budget tracking") { "Enable budget tracking — add budget.json to .campaign/" } `
                     elseif ($issue -match "exhausted") { "Increase budget limit or reduce per-task tool calls" } `
                     else { "Review budget allocation" }
        target_file = if ($issue -match "No budget tracking") { "campaign-server.mjs" } else { "RULES.md" }
        priority = "high"
    }
}

# Error rate → proposals
foreach ($issue in $diagnosis.error_rate_issues) {
    $proposals += [PSCustomObject]@{
        id = [System.Guid]::NewGuid().ToString().Substring(0, 6)
        area = "error_handling"
        issue = $issue
        suggestion = "Add retry with backoff — see Rule 19 in RULES.md"
        target_file = "RULES.md"
        priority = "medium"
    }
}

# Stagnation → proposals
foreach ($issue in $diagnosis.stagnation_patterns) {
    $proposals += [PSCustomObject]@{
        id = [System.Guid]::NewGuid().ToString().Substring(0, 6)
        area = "stagnation"
        issue = $issue
        suggestion = "Review MoM ladder — escalate to stronger model if stuck"
        target_file = "iter-loop-tools.md"
        priority = "high"
    }
}

if ($proposals.Count -eq 0) {
    Write-Host "   ✅ No se detectaron issues — el pipeline está saludable." -ForegroundColor Green
} else {
    Write-Host "   $($proposals.Count) propuesta(s) generadas:" -ForegroundColor Cyan
    $proposals | ForEach-Object {
        Write-Host "   [$($_.id)] $($_.area): $($_.suggestion)" -ForegroundColor Yellow
    }
}

# ── Phase 3: PROPOSE (guarda propuesta para rehearsal) ──
Write-Host ""
Write-Host "─── Phase 3/5: PROPOSE ───" -ForegroundColor Yellow

$proposalFile = Join-Path $ProposalsDir "dgm-proposal-$Timestamp.json"
$proposals | ConvertTo-Json -Depth 5 | Set-Content $proposalFile -Encoding UTF8
Write-Host "   Propuesta guardada en: $proposalFile" -ForegroundColor Gray

# ── Phase 4: EVALUATE ──
Write-Host ""
Write-Host "─── Phase 4/5: EVALUATE ───" -ForegroundColor Yellow

$evalResult = @{
    campaign_id = $CampaignId
    timestamp = $Timestamp
    score = $diagnosis.overall_score
    high_priority = $diagnosis.high_priority_areas
    proposal_count = $proposals.Count
    suggested_improvement = if ($diagnosis.overall_score -lt 0.5) { "Critical — restructure needed" } `
                            elseif ($diagnosis.overall_score -lt 0.7) { "Needs improvement — review proposals" } `
                            else { "Healthy — minor optimizations available" }
}

Write-Host "   Score: $score" -ForegroundColor $(
    if ($score -ge 0.7) { "Green" } elseif ($score -ge 0.5) { "Yellow" } else { "Red" }
)
Write-Host "   Verdict: $($evalResult.suggested_improvement)" -ForegroundColor $(
    if ($score -ge 0.7) { "Green" } elseif ($score -ge 0.5) { "Yellow" } else { "Red" }
)

# ── Phase 5: ARCHIVE ──
Write-Host ""
Write-Host "─── Phase 5/5: ARCHIVE ───" -ForegroundColor Yellow

$archiveFile = Join-Path $ReportDir "dgm-$CampaignId-$Timestamp.json"
$archive = @{
    campaign_id = $CampaignId
    timestamp = $Timestamp
    model = "performance_diagnosis.py + dgm-loop.ps1 v1"
    phases = @(
        @{ phase = "diagnose"; status = "completed"; output = "score: $score" },
        @{ phase = "propose"; status = "completed"; proposals = $proposals.Count },
        @{ phase = "rehearse"; status = "pending (run dgm-rehearse.ps1)" },
        @{ phase = "evaluate"; status = "completed"; verdict = $evalResult.suggested_improvement },
        @{ phase = "archive"; status = "completed" }
    )
    diagnosis = $diagnosis
    evaluation = $evalResult
    proposals = $proposals
}
$archive | ConvertTo-Json -Depth 10 | Set-Content $archiveFile -Encoding UTF8
Write-Host "   Archivo: $archiveFile" -ForegroundColor Gray

Write-Host ""
Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DGM Loop v1 — COMPLETED            ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "  ▶ Próximo paso seguro (sin riesgo):" -ForegroundColor Green
Write-Host "    .antigravity\task-system\self-modification\dgm-rehearse.ps1 -ProposalFile $proposalFile -Quick" -ForegroundColor Yellow
Write-Host ""
Write-Host "  ▶ Si el rehearsal pasa y querés promover:" -ForegroundColor Cyan
Write-Host "    .antigravity\task-system\self-modification\dgm-promote.ps1 -ProposalFile $proposalFile -RehearsalId <ID>" -ForegroundColor Yellow
Write-Host ""
Write-Host "  ════════════════════════════════════════" -ForegroundColor DarkGray
Write-Host "  Flujo completo de mejora continua SEGURA:" -ForegroundColor White
Write-Host "  1. dgm-loop.ps1           → diagnosticar + proponer" -ForegroundColor Gray
Write-Host "  2. dgm-rehearse.ps1       → probar en aislamiento (0 riesgo)" -ForegroundColor Gray
Write-Host "  3. Revisar el reporte     → ¿score mejoró? ¿verify pasó?" -ForegroundColor Gray
Write-Host "  4. dgm-promote.ps1        → solo si aprobás (gate humano)" -ForegroundColor Gray
Write-Host "  5. git revert HEAD        → si algo sale mal (rollback total)" -ForegroundColor Gray
Write-Host "  ════════════════════════════════════════" -ForegroundColor DarkGray
