#!/usr/bin/env pwsh
<#
.SYNOPSIS
  Harness externo para backlog-executor. Ejecuta tareas de un plan file
  una por una, invocando OpenCode en cada iteración.

.DESCRIPTION
  Lee docs/plans/<campaign>.md, encuentra la próxima tarea ❌,
  invoca `opencode run` con el prompt de una iteración,
  espera a que termine, verifica progreso, y repite.

  El loop vive ACÁ (en el script), no en el prompt del LLM.

.PARAMETER PlanFile
  Ruta al plan file (docs/plans/YYYY-MM-DD-<campaign>.md).

.PARAMETER Interval
  Segundos de pausa entre iteraciones (default: 5).

.PARAMETER StallThreshold
  Intentos consecutivos sin progreso antes de preguntar (default: 2).

.PARAMETER DryRun
  No ejecuta opencode, solo muestra qué haría.

.PARAMETER OpenCodePrompt
  Ruta al archivo de prompt para cada iteración.
  Default: .opencode/skills/backlog-executor/iter-prompt.md

.EXAMPLE
  .\harness-executor.ps1 -PlanFile docs\plans\2026-07-13-plan.md

.EXAMPLE
  .\harness-executor.ps1 -PlanFile docs\plans\plan.md -Interval 10
#>

param(
  [Parameter(Mandatory = $true)]
  [string]$PlanFile,

  [int]$Interval = 5,
  [int]$StallThreshold = 2,
  [switch]$DryRun,
  [string]$OpenCodePrompt = ""
)

$ErrorActionPreference = "Stop"
$planFile = Resolve-Path $PlanFile -ErrorAction Stop
$projectRoot = Split-Path $planFile -Parent | Split-Path -Parent
$planFileName = Split-Path $planFile -Leaf

if (-not (Test-Path $planFile)) {
  Write-Error "Plan file not found: $planFile"
  exit 1
}

# ---- helpers ----

function Get-NextTask {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  if (-not $content) { return $null }

  # Buscar primera tarea en ❌ o ⬜ PENDING
  $taskMatch = [regex]::Match($content, '(?s)(### Task \d+:.+?)(?=### Task|\z)')
  if (-not $taskMatch.Success) { return $null }

  $tasks = @()
  $matches = [regex]::Matches($content, '(?s)(### Task \d+:[^\n]*\n.*?)(?=### Task|\z)')
  foreach ($m in $matches) {
    $block = $m.Groups[1].Value
    if ($block -match 'Estado:\s*[❌⬜]') {
      $idMatch = [regex]::Match($block, '### Task (\d+):')
      $tasks += [PSCustomObject]@{
        Id       = if ($idMatch.Success) { $idMatch.Groups[1].Value } else { "?" }
        Block    = $block
        Raw      = $m.Groups[1].Value
      }
    }
  }
  if ($tasks.Count -eq 0) { return $null }
  return $tasks[0]
}

function Get-Recitation {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  $match = [regex]::Match($content, '(?s)=== RECITATION ===\n(.*?)=== END RECITATION ===')
  if ($match.Success) { return $match.Groups[1].Value.Trim() }
  return $null
}

function Get-StatusLine {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  $match = [regex]::Match($content, '(✅ COMPLETADO|❌ ABORTADO|⏳ EN PROGRESO)')
  if ($match.Success) { return $match.Value }
  # Contar estados en las tasks
  $completed = [regex]::Matches($content, 'Estado:\s*✅').Count
  $failed = [regex]::Matches($content, 'Estado:\s*❌ FAILED').Count
  $pending = [regex]::Matches($content, 'Estado:\s*[❌⬜]').Count
  $total = $completed + $failed + $pending
  return "$completed/$total ✅ ($failed ❌, $pending pendientes)"
}

function Get-TaskCount {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  $pending = [regex]::Matches($content, 'Estado:\s*[❌⬜]').Count
  return $pending
}

# ---- main loop ----

Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Backlog Executor Harness" -ForegroundColor Cyan
Write-Host "║  Plan: $planFileName" -ForegroundColor Cyan
Write-Host "║  Init: $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Escribir PID en el plan file para detectar doble ejecución
$pidLine = "> **Harness PID:** $PID (`$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))"
$currentContent = Get-Content $planFile -Raw
if ($currentContent -notmatch 'Harness PID') {
  $currentContent = $currentContent -replace '(# Plan de Ejecución.*?\n)', "`$1$pidLine`n"
  Set-Content $planFile $currentContent
}

$iteration = 0
$stallCount = @{}
$taskStartStates = @{}

while ($true) {
  $iteration++
  $pendingCount = Get-TaskCount $planFile

  Write-Host ""
  Write-Host "═══ Iteración $iteration | $pendingCount tareas pendientes ═══" -ForegroundColor Yellow

  if ($pendingCount -eq 0) {
    Write-Host ""
    Write-Host "✅ No quedan tareas pendientes. Completando campaña..." -ForegroundColor Green
    # Marcar plan como completado
    $content = Get-Content $planFile -Raw
    $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}✅ COMPLETADO'
    Set-Content $planFile $content
    Write-Host "✅ Campaña completada: $planFileName" -ForegroundColor Green
    break
  }

  $nextTask = Get-NextTask $planFile
  if (-not $nextTask) {
    Write-Host "⚠️ No se encontró próxima tarea. Estado: $(Get-StatusLine $planFile)" -ForegroundColor Yellow
    break
  }

  $taskId = "TASK-$($nextTask.Id)"

  Write-Host "  → Próxima: $taskId" -ForegroundColor Cyan
  Write-Host "  → Estado general: $(Get-StatusLine $planFile)" -ForegroundColor Gray

  # Detectar stall
  if (-not $taskStartStates.ContainsKey($taskId)) {
    $taskStartStates[$taskId] = $nextTask.Raw
    $stallCount[$taskId] = 0
  } elseif ($taskStartStates[$taskId] -eq $nextTask.Raw) {
    $stallCount[$taskId]++
    Write-Host "  ⚠️ Stall detection: intento $($stallCount[$taskId]) igual" -ForegroundColor Yellow

    if ($stallCount[$taskId] -ge $StallThreshold) {
      Write-Host "  ❌ Stall detectado ($taskId no progresa tras $StallThreshold intentos)" -ForegroundColor Red
      Write-Host ""
      Write-Host "¿Abortar campaña? (s/N)" -ForegroundColor Red
      $answer = Read-Host
      if ($answer -eq 's') {
        Write-Host "❌ Campaña abortada por stall." -ForegroundColor Red
        $content = Get-Content $planFile -Raw
        $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}❌ ABORTADO'
        Set-Content $planFile $content
        exit 2
      }
      # Reset stall counter si el usuario quiere seguir
      $stallCount[$taskId] = 0
      $taskStartStates[$taskId] = $nextTask.Raw
    }
  } else {
    # El bloque cambió → hubo progreso
    $taskStartStates[$taskId] = $nextTask.Raw
    $stallCount[$taskId] = 0
  }

  # Construir prompt para el agente
  $promptPath = if ($OpenCodePrompt) {
    $OpenCodePrompt
  } else {
    Join-Path $projectRoot ".opencode" "skills" "backlog-executor" "iter-prompt.md"
  }

  $promptBody = @"
Cargá las skills writing-plans, incremental-implementation, ponytail (full), code-review-and-quality.

Plan file: $planFile

INSTRUCCIONES — UNA SOLA ITERACIÓN:

1. Leé el plan file COMPLETO.
2. Identificá la próxima acción concreta para la tarea activa.
3. Ejecutá SOLO esa acción (gate, codegraph, implementar, verify, commit, o update).
4. Actualizá el plan file con el resultado.
5. **ESCRIBÍ EL BLOQUE RECITATION al final.**
6. Detenete. No avances a la siguiente tarea.

REGLAS:
- Ponytail activo: stdlib > reusar > dependency > desde cero
- Verify: mecánico (cargo check, nextest, tsc), nunca auto-reporte
- Si verify falla 2 veces con mismo error → marcar ❌ FAILED, escribir notas
- No cambies scope. Anotalo si encuentras algo extra, no lo implementes
"@

  if ($DryRun) {
    Write-Host ""
    Write-Host "  [DRY RUN] Inyectaría este prompt en OpenCode:" -ForegroundColor Magenta
    Write-Host $promptBody -ForegroundColor DarkGray
    Write-Host "  [DRY RUN] Luego esperaría $Interval segundos." -ForegroundColor Magenta
  } else {
    Write-Host "  → Ejecutando opencode..." -ForegroundColor Cyan

    $tempPrompt = Join-Path $env:TEMP "backlog-iter-$(Get-Random).md"
    Set-Content $tempPrompt $promptBody

    try {
      $result = opencode run $tempPrompt 2>&1
      Write-Host "  → OpenCode terminó. Código de salida: $LASTEXITCODE" -ForegroundColor Gray

      # Mostrar resumen del resultado
      $recitation = Get-Recitation $planFile
      if ($recitation) {
        Write-Host "  → Recitation: $recitation" -ForegroundColor DarkGray
      }
    }
    catch {
      Write-Host "  ⚠️ Error ejecutando opencode: $_" -ForegroundColor Yellow
    }
    finally {
      if (Test-Path $tempPrompt) { Remove-Item $tempPrompt -Force }
    }
  }

  if ($Interval -gt 0 -and (Get-TaskCount $planFile) -gt 0) {
    Write-Host "  → Esperando $Interval segundos..." -ForegroundColor DarkGray
    Start-Sleep -Seconds $Interval
  }
}

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Resumen Final" -ForegroundColor Cyan
Write-Host "║  $(Get-StatusLine $planFile)" -ForegroundColor Cyan
Write-Host "║  $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
