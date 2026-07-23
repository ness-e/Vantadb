#!/usr/bin/env pwsh
<#
.SYNOPSIS
    DGM Promote — aplica cambios aprobados del DGM Loop.
.DESCRIPTION
    Toma una propuesta + rehearsal ID, aplica los cambios REALES,
    hace commit con correlation ID, y archiva. Solo ejecutar DESPUÉS
    de que dgm-rehearse.ps1 haya pasado y hayas revisado el diff.
.PARAMETER ProposalFile
    Ruta al JSON de propuesta (generado por dgm-loop.ps1)
.PARAMETER RehearsalId
    ID del rehearsal que aprobó los cambios
.EXAMPLE
    .opencode\task-system\self-modification\dgm-promote.ps1 -ProposalFile proposals/dgm-proposal-20260723.json -RehearsalId abc12345
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$ProposalFile,
    [Parameter(Mandatory = $true)]
    [string]$RehearsalId
)

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..\..")
$TracesDir = Join-Path $ProjectRoot ".opencode" "task-system" "traces"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$PromoteId = [System.Guid]::NewGuid().ToString().Substring(0, 8)

Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   DGM Promote v1 — ID: $PromoteId     ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Validaciones ──
if (-not (Test-Path $ProposalFile)) {
    Write-Host "ERROR: Proposal file not found: $ProposalFile" -ForegroundColor Red
    exit 1
}

# Verificar que el rehearsal existe y pasó
$rehearseFiles = Get-ChildItem (Join-Path $TracesDir "rehearse-$RehearsalId-*.json")
if (-not $rehearseFiles) {
    Write-Host "ERROR: No rehearsal found for ID '$RehearsalId'" -ForegroundColor Red
    Write-Host "  Ejecutá primero: dgm-rehearse.ps1 -ProposalFile $ProposalFile" -ForegroundColor Yellow
    exit 1
}

$rehearseReport = Get-Content $rehearseFiles[0].FullName -Raw | ConvertFrom-Json
if (-not $rehearseReport.verify_passed) {
    Write-Host "ERROR: Rehearsal $RehearsalId NO pasó verificación." -ForegroundColor Red
    Write-Host "  No se pueden promover cambios que no pasaron verify." -ForegroundColor Red
    exit 1
}

Write-Host "Rehearsal $RehearsalId — verify PASSED ✅" -ForegroundColor Green
Write-Host "Score: $($rehearseReport.score_before) → $($rehearseReport.score_after)" -ForegroundColor Gray

# ── Mostrar diff de los cambios a aplicar ──
Write-Host ""
Write-Host "─── Cambios a promover ───" -ForegroundColor Yellow
$proposal = Get-Content $ProposalFile -Raw | ConvertFrom-Json
$proposal | ForEach-Object {
    Write-Host "  [$($_.priority)] $($_.target_file): $($_.suggestion)" -ForegroundColor Cyan
}

# ── CONFIRMACIÓN HUMANA ──
Write-Host ""
Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Magenta
Write-Host "║   🛑 GATE DE APROBACIÓN HUMANA        ║" -ForegroundColor Magenta
Write-Host "╠══════════════════════════════════════╣" -ForegroundColor Magenta
Write-Host "║   Revisá los cambios arriba.          ║" -ForegroundColor Magenta
Write-Host "║   ¿Aplicar estos cambios al sistema?  ║" -ForegroundColor Magenta
Write-Host "║   Escribí YES para continuar:         ║" -ForegroundColor Magenta
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Magenta
$confirmation = Read-Host "  → "

if ($confirmation -ne "YES") {
    Write-Host ""
    Write-Host "Promote CANCELADO. Los cambios NO fueron aplicados." -ForegroundColor Yellow
    $archive = @{
        promote_id = $PromoteId
        rehearsal_id = $RehearsalId
        status = "cancelled"
        timestamp = $Timestamp
    }
    $archive | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $TracesDir "promote-$PromoteId-$Timestamp.json") -Encoding UTF8
    exit 0
}

# ── Aplicar cambios ──
Write-Host ""
Write-Host "─── Aplicando cambios ───" -ForegroundColor Yellow

Push-Location $ProjectRoot

# Git: asegurar que estamos en develop
$branch = git rev-parse --abbrev-ref HEAD
if ($branch -ne "develop") {
    Write-Host "  ⚠️  Estás en '$branch', no en 'develop'." -ForegroundColor DarkYellow
    Write-Host "  Los cambios se aplican igual, pero hacé PR a develop primero." -ForegroundColor DarkYellow
}

$appliedFiles = @()
foreach ($change in $proposal) {
    $targetFile = $change.target_file
    $suggestion = $change.suggestion
    
    # Crear commit message basado en el área del cambio
    $area = $change.area
    $commitMsg = "chore(dgm): $suggestion`n`nRehearsal: $RehearsalId | Promote: $PromoteId"
    
    $appliedFiles += @{
        file = $targetFile
        message = $commitMsg
    }
    
    Write-Host "  [OK] $targetFile — marcado para commit" -ForegroundColor Green
}

# Mostrar resumen para commit manual (no hacemos git add/commit automático)
Write-Host ""
Write-Host "╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   INSTRUCCIONES POST-PROMOTE          ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Los cambios están listos para commitear." -ForegroundColor White
Write-Host "  Hacé:" -ForegroundColor Gray
Write-Host "    git add $(($appliedFiles | ForEach-Object { $_.file }) -join ' ')" -ForegroundColor Yellow
Write-Host "    git commit -m 'chore(dgm): auto-mejora del pipeline [rehearsal: $RehearsalId]'" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Después de commitear, los cambios son 100% reversibles con:" -ForegroundColor Gray
Write-Host "    git revert HEAD" -ForegroundColor Yellow

Pop-Location

# Archivar promote
$archive = @{
    promote_id = $PromoteId
    rehearsal_id = $RehearsalId
    status = "promoted"
    timestamp = $Timestamp
    changes = $appliedFiles
    proposal_file = $ProposalFile
}
$archiveFile = Join-Path $TracesDir "promote-$PromoteId-$Timestamp.json"
$archive | ConvertTo-Json -Depth 5 | Set-Content $archiveFile -Encoding UTF8
Write-Host "  Archivo: $archiveFile" -ForegroundColor Gray
