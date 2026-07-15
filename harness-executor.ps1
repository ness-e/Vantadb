#!/usr/bin/env pwsh
<#
.SYNOPSIS
  Harness externo para campaign-executor. Ejecuta tareas de un plan file
  una por una (o en paralelo), invocando OpenCode en cada iteración.

.DESCRIPTION
  Lee docs/plans/<campaign>.md, encuentra la próxima tarea ⬜ PENDING,
  invoca `opencode run` con el prompt de iter.md, espera que termine,
  verifica progreso, y repite.

.PARAMETER PlanFile
  Ruta al plan file (docs/plans/YYYY-MM-DD-<campaign>.md).

.PARAMETER Interval
  Segundos de pausa entre iteraciones (default: 5).

.PARAMETER StallThreshold
  Intentos consecutivos sin progreso antes de preguntar (default: 2).

.PARAMETER SingleTask
  Ejecutar solo una tarea específica (ej: "DRV-068"). Omite el resto.

.PARAMETER Timeout
  Timeout por iteración en segundos (default: 300 = 5 min, 0 = sin límite).

.PARAMETER Parallel
  Ejecuta tareas independientes en paralelo (waves concurrentes).

.PARAMETER Yes
  Auto-responde sí a todas las confirmaciones (git dirty, stall, PID conflict).
  Permite ejecución no-interactiva desde el chat con `opencode run`.

.PARAMETER DryRun
  No ejecuta opencode, solo muestra qué haría.

.EXAMPLE
  .\harness-executor.ps1 -PlanFile docs\plans\2026-07-13-plan.md
  .\harness-executor.ps1 -PlanFile docs\plans\plan.md -Interval 10 -SingleTask DRV-068
  .\harness-executor.ps1 -PlanFile docs\plans\plan.md -Parallel
#>

param(
  [Parameter(Mandatory = $true)]
  [string]$PlanFile,

  [int]$Interval = 5,
  [int]$StallThreshold = 2,
  [string]$SingleTask = "",
  [int]$Timeout = 900,
  [switch]$Parallel,
  [switch]$DryRun,
  [switch]$Yes
)

$ErrorActionPreference = "Stop"
$planFile = Resolve-Path $PlanFile -ErrorAction Stop
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$planFileName = Split-Path $planFile -Leaf

if (-not (Test-Path $planFile)) {
  Write-Error "Plan file not found: $planFile"
  exit 1
}

# ---- double PID detection ----
$thisPid = $PID
$planContent = Get-Content $planFile -Raw
$pidMatch = [regex]::Match($planContent, 'Harness PID:\s*(\d+)')
if ($pidMatch.Success) {
  $existingPid = [int]$pidMatch.Groups[1].Value
  $existingProcess = Get-Process -Id $existingPid -ErrorAction SilentlyContinue
  if ($existingProcess) {
    Write-Warning "⚠️  Harness ya ejecutándose con PID $existingPid ($($existingProcess.ProcessName))."
    Write-Warning "   Solo un harness a la vez está permitido."
    if ($Yes) {
      $existingProcess.Kill()
      Write-Host "  → Proceso $existingPid matado (-Yes)." -ForegroundColor Yellow
    } else {
      $answer = Read-Host "¿Matar el proceso existente y continuar? (s/N)"
      if ($answer -eq 's') {
        $existingProcess.Kill()
        Write-Host "  → Proceso $existingPid matado." -ForegroundColor Yellow
      } else {
        exit 6
      }
    }
  }
}

# ---- git check ----
$gitStatus = git -C $projectRoot status --porcelain 2>$null
if ($gitStatus) {
  Write-Host "⚠️  Working tree has uncommitted changes:" -ForegroundColor Yellow
  $gitStatus | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
  if (-not $Yes) {
    Write-Host "  ¿Continuar de todas formas? (s/N) " -ForegroundColor Yellow -NoNewline
    $answer = Read-Host
    if ($answer -ne 's') { exit 4 }
  }
}

# ---- helpers ----

function Get-PlanTasks {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  $tasks = @()
  $matches = [regex]::Matches($content, '(?s)(### Task \d+:[^\n]*\n.*?)(?=### Task|\z)')
  foreach ($m in $matches) {
    $block = $m.Groups[1].Value
    $idMatch = [regex]::Match($block, '### Task (\d+):\s*(.+)')
    $nameMatch = [regex]::Match($block, '### Task \d+:\s*(.+)')
    $stateMatch = [regex]::Match($block, 'Estado:\s*([⬜⏳✅❌])')
    $fileMatch = [regex]::Match($block, 'Task file:\s*`?([^`\n]+)')
    $depMatch = [regex]::Match($block, 'Dependencias?:?\s*(.+)')
    $tasks += [PSCustomObject]@{
      Id        = if ($idMatch.Success) { $idMatch.Groups[1].Value } else { "?" }
      Name      = if ($nameMatch.Success) { $nameMatch.Groups[1].Value.Trim() } else { "?" }
      State     = if ($stateMatch.Success) { $stateMatch.Groups[1].Value } else { "⬜" }
      TaskFile  = if ($fileMatch.Success) { $fileMatch.Groups[1].Value.Trim() } else { $null }
      Deps      = if ($depMatch.Success) { $depMatch.Groups[1].Value.Trim() } else { "" }
      Block     = $block
    }
  }
  return $tasks
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
  $completed = [regex]::Matches($content, 'Estado:\s*✅').Count
  $failed = [regex]::Matches($content, 'Estado:\s*❌').Count
  $pending = [regex]::Matches($content, 'Estado:\s*[⬜⏳]').Count
  $total = $completed + $failed + $pending
  return "$completed/$total ✅ ($failed ❌, $pending pendientes)"
}

function Get-TaskCounts {
  param([string]$Path)
  $content = Get-Content $Path -Raw
  $do = [regex]::Matches($content, '✅ DO').Count
  $defer = [regex]::Matches($content, '🟡 DEFER').Count
  $skip = [regex]::Matches($content, '❌ SKIP').Count
  $bloqueado = [regex]::Matches($content, '🔴 BLOQUEADO').Count
  return @{ DO = $do; DEFER = $defer; SKIP = $skip; BLOQUEADO = $bloqueado }
}

function Sync-LastSynced {
  param([string]$PlanPath)
  $now = Get-Date -Format "yyyy-MM-ddTHH:mm"
  $content = Get-Content $PlanPath -Raw
  if ($content -match 'last-synced:\s*') {
    $content = $content -replace 'last-synced:\s*\S+', "last-synced: $now"
  }
  Set-Content $PlanPath $content

  $tasks = Get-PlanTasks $PlanPath
  foreach ($t in $tasks) {
    if ($t.TaskFile) {
      $tf = Join-Path $projectRoot $t.TaskFile
      if (Test-Path $tf) {
        $tc = Get-Content $tf -Raw
        if ($tc -match 'last-synced:\s*') {
          $tc = $tc -replace 'last-synced:\s*\S+', "last-synced: $now"
          Set-Content $tf $tc
        }
      }
    }
  }
}

function Invoke-OpenCodeIteration {
  param([string]$PromptBody, [int]$TimeoutSec, [switch]$DryRun)
  if ($DryRun) {
    Write-Host "  [DRY RUN]" -ForegroundColor Magenta
    return $true
  }

  $tempPrompt = Join-Path $env:TEMP "campaign-iter-$(Get-Random).md"
  Set-Content $tempPrompt $PromptBody

  try {
    $opencodeCmd = (Get-Command -Name "opencode" -ErrorAction Stop).Source -replace '\.ps1$', '.cmd'

    Write-Host "  ── opencode run (deepseek-v4-flash-free) ─────────" -ForegroundColor Cyan
    $ps = Start-Process -FilePath $opencodeCmd -ArgumentList "run -m deepseek-v4-flash-free --auto --title `"Harness: $taskId`" `"$tempPrompt`"" -NoNewWindow -PassThru

    if ($TimeoutSec -gt 0) {
      $waited = $ps.WaitForExit($TimeoutSec * 1000)
      if (-not $waited) {
        Write-Host "`n  ❌ Timeout ($TimeoutSec s). Matando..." -ForegroundColor Red
        $ps.Kill()
        return $false
      }
    } else {
      $ps.WaitForExit()
    }
    Write-Host "  ─────────────────────────────────────────────────" -ForegroundColor Cyan

    return $ps.ExitCode -eq 0
  }
  catch {
    Write-Host "  ⚠️ Error: $_" -ForegroundColor Yellow
    return $false
  }
  finally {
    Remove-Item $tempPrompt -Force -ErrorAction SilentlyContinue
  }
}

# ---- main ----

Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Campaign Executor Harness" -ForegroundColor Cyan
Write-Host "║  PID: $PID" -ForegroundColor Cyan
Write-Host "║  Plan: $planFileName" -ForegroundColor Cyan
if ($SingleTask) { Write-Host "║  SingleTask: $SingleTask" -ForegroundColor Cyan }
if ($Parallel) { Write-Host "║  Mode: PARALLEL" -ForegroundColor Cyan }
Write-Host "║  Init: $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan

# Escribir PID en el plan file
$pidLine = "> **Harness PID:** $PID (`$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))"
$currentContent = Get-Content $planFile -Raw
if ($currentContent -notmatch 'Harness PID') {
  $currentContent = $currentContent -replace '(# Plan de Ejecución.*?\n)', "`$1$pidLine`n"
  Set-Content $planFile $currentContent
}

$iteration = 0
$stallCount = @{}
$taskLastState = @{}
$stats = @{ completed = 0; failed = 0; pending = 0 }

while ($true) {
  $iteration++
  $allTasks = Get-PlanTasks $planFile
  $pendingTasks = $allTasks | Where-Object { $_.State -match '[⬜⏳]' }

  if ($SingleTask) {
    $pendingTasks = $pendingTasks | Where-Object { $_.Id -eq $SingleTask -or $_.Name -match $SingleTask }
  }

  Write-Host ""
  Write-Host "═══ Iteración $iteration | $($pendingTasks.Count) pendientes ═══" -ForegroundColor Yellow

  if ($pendingTasks.Count -eq 0) {
    Write-Host "✅ No quedan tareas pendientes." -ForegroundColor Green
    $content = Get-Content $planFile -Raw
    $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}✅ COMPLETADO'
    Set-Content $planFile $content

    # Resumen final con desglose
    $counts = Get-TaskCounts $planFile
    $completed = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*✅').Count
    $failed = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*❌').Count
    $pending = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*[⬜⏳]').Count
    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  Resumen Final" -ForegroundColor Green
    Write-Host "║  ✅ Completadas: $completed" -ForegroundColor Green
    Write-Host "║  ❌ Fallidas:    $failed" -ForegroundColor Red
    Write-Host "║  ⬜ Pendientes:  $pending" -ForegroundColor Yellow
    Write-Host "║  ────────────────────────────" -ForegroundColor Gray
    Write-Host "║  Gate: $($counts.DO) DO · $($counts.DEFER) DEFER · $($counts.SKIP) SKIP · $($counts.BLOQUEADO) BLOQUEADO" -ForegroundColor Gray
    Write-Host "║  $(Get-Date -Format 'yyyy-MM-dd HH:mm')" -ForegroundColor Gray
    Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Green
    break
  }

  # ---- MODO PARALELO ----
  if ($Parallel -and $pendingTasks.Count -gt 1) {
    Write-Host "  → Modo paralelo: $($pendingTasks.Count) tareas pendientes" -ForegroundColor Cyan
    $batch = $pendingTasks | Select-Object -First 4
    $jobs = @()

    foreach ($t in $batch) {
      $promptPath = Join-Path $projectRoot ".opencode" "prompts" "iter.md"
      $promptTemplate = Get-Content $promptPath -Raw
      $promptBody = $promptTemplate -replace '{{PLAN_FILE}}', $planFile
      $promptBody = $promptBody -replace '{{SINGLE_TASK}}', $t.Id
      $promptBody = $promptBody -replace '{{TASK_BASE}}', '.opencode/skills/campaign-executor/tasks/'

      $promptFile = Join-Path $env:TEMP "campaign-par-$(Get-Random).md"
      Set-Content $promptFile $promptBody

      $j = Start-Job -ScriptBlock {
        param($pf, $to, $taskId)
        $env:CARGO_TARGET_DIR = "$env:TEMP\cargo-target-$taskId"
        $r = opencode run -m deepseek-v4-flash-free --auto --title "Harness: TASK-$($taskId)" $pf 2>&1 | Out-String
        Remove-Item $pf -Force -ErrorAction SilentlyContinue
        return $r
      } -ArgumentList $promptFile, $Timeout, $t.Id

      $jobs += @{ Job = $j; Task = $t }
    }

    Write-Host "  → Esperando $($jobs.Count) jobs paralelos..." -ForegroundColor Cyan
    $jobs | ForEach-Object { $_.Job | Wait-Job -Timeout $Timeout | Out-Null }
    $jobs | ForEach-Object {
      $out = Receive-Job -Job $_.Job
      Write-Host "  → TASK-$($_.Task.Id): $($_.Job.State)" -ForegroundColor DarkGray
    }
    continue
  }

  # ---- MODO SECUENCIAL ----
  $nextTask = $pendingTasks[0]
  $taskId = "TASK-$($nextTask.Id)"

  Write-Host "  → Próxima: $taskId ($($nextTask.Name))" -ForegroundColor Cyan
  Write-Host "  → Estado: $(Get-StatusLine $planFile)" -ForegroundColor Gray

  # Detectar stall (compara solo estado, no bloque completo)
  $currentState = $nextTask.State
  if (-not $taskLastState.ContainsKey($taskId)) {
    $taskLastState[$taskId] = $currentState
    $stallCount[$taskId] = 0
  } elseif ($taskLastState[$taskId] -eq $currentState) {
    $stallCount[$taskId]++
    Write-Host "  ⚠️ Stall: intento $($stallCount[$taskId]) sin cambios de estado" -ForegroundColor Yellow
    if ($stallCount[$taskId] -ge $StallThreshold) {
      if ($Yes) {
        Write-Host "  → Stall, abortando (-Yes)." -ForegroundColor Red
        $content = Get-Content $planFile -Raw
        $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}❌ ABORTADO'
        Set-Content $planFile $content
        exit 2
      }
      Write-Host "  ❌ Stall ($StallThreshold intentos). ¿Abortar? (s/N) " -ForegroundColor Red -NoNewline
      if ((Read-Host) -eq 's') {
        $content = Get-Content $planFile -Raw
        $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}❌ ABORTADO'
        Set-Content $planFile $content
        exit 2
      }
      $stallCount[$taskId] = 0
      $taskLastState[$taskId] = $currentState
    }
  } else {
    $taskLastState[$taskId] = $currentState
    $stallCount[$taskId] = 0
  }

  Sync-LastSynced $planFile

  $promptPath = Join-Path $projectRoot ".opencode" "prompts" "iter.md"
  if (-not (Test-Path $promptPath)) {
    Write-Error "Prompt not found: $promptPath"
    exit 1
  }

  $counts = Get-TaskCounts $planFile
  $completed = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*✅').Count
  $failed = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*❌').Count
  $pending = [regex]::Matches((Get-Content $planFile -Raw), 'Estado:\s*[⬜⏳]').Count
  $summaryLine = "Resumen: $completed/$($counts.DO) ✅ · $failed ❌ · $pending pendientes"

  $promptTemplate = Get-Content $promptPath -Raw
  $promptBody = $promptTemplate -replace '{{PLAN_FILE}}', $planFile
  $promptBody = $promptBody -replace '{{SINGLE_TASK}}', $SingleTask
  $promptBody = $promptBody -replace '{{SUMMARY}}', $summaryLine
  $promptBody = $promptBody -replace '{{TASK_BASE}}', '.opencode/skills/campaign-executor/tasks/'

  if ($DryRun) {
    Write-Host "`n  [DRY RUN]" -ForegroundColor Magenta
  } else {
    Write-Host "  → Ejecutando opencode..." -ForegroundColor Cyan
    $ok = Invoke-OpenCodeIteration -PromptBody $promptBody -TimeoutSec $Timeout -DryRun:$DryRun

    if ($ok) {
      $recitation = Get-Recitation $planFile
      if (-not $recitation) {
        if ($Yes) {
          Write-Host "  → Recitation no encontrada, continuando (-Yes)." -ForegroundColor Yellow
        } else {
          Write-Host "  ⚠️ Recitation no encontrada. ¿Continuar? (s/N) " -ForegroundColor Yellow -NoNewline
          if ((Read-Host) -ne 's') { exit 3 }
        }
      } elseif ($recitation -notmatch '[✅❌]') {
        if ($Yes) {
          Write-Host "  → Recitation sin ✅/❌, continuando (-Yes)." -ForegroundColor Yellow
        } else {
          Write-Host "  ⚠️ Recitation sin ✅/❌. ¿Continuar? (s/N) " -ForegroundColor Yellow -NoNewline
          if ((Read-Host) -ne 's') { exit 3 }
        }
      } else {
        Write-Host "  → Recitation ok" -ForegroundColor DarkGray
      }
    }
  }

  if ($Interval -gt 0 -and (Get-PlanTasks $planFile | Where-Object { $_.State -match '[⬜⏳]' }).Count -gt 0) {
    Write-Host "  → Esperando $Interval s..." -ForegroundColor DarkGray
    Start-Sleep -Seconds $Interval
  }
}
