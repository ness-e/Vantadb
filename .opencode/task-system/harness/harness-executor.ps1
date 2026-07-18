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

.PARAMETER Model
  Modelo de opencode a usar (default: deepseek-v4-flash-free).

.PARAMETER TaskBase
  Ruta base para archivos de tarea (default: .opencode/skills/campaign-executor/tasks/).

.PARAMETER CampaignId
  ID de correlación para la campaña. Auto-genera UUID si no se especifica.

.PARAMETER MaxParallel
  Máximo de tareas paralelas simultáneas (default: 4).

.PARAMETER LogDir
  Directorio para logs (default: .opencode/task-system/harness/logs/).

.PARAMETER Parallel
  Ejecuta tareas independientes en paralelo (waves concurrentes).

.PARAMETER Yes
  Auto-responde sí a todas las confirmaciones (git dirty, stall, PID conflict).
  Permite ejecución no-interactiva desde el chat con `opencode run`.

.PARAMETER DryRun
  No ejecuta opencode, solo muestra qué haría.

.EXAMPLE
  .opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\2026-07-13-plan.md
  .opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\plan.md -Interval 10 -SingleTask DRV-068
  .opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\plan.md -Parallel
  .opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\plan.md -Model claude-sonnet-4-20250514 -MaxParallel 6
#>

param(
  [Parameter(Mandatory = $true)]
  [string]$PlanFile,

  [int]$Interval = 5,
  [int]$StallThreshold = 2,
  [string]$SingleTask = "",
  [int]$Timeout = 900,
  [string]$Model = "deepseek-v4-flash-free",
  [string]$TaskBase = ".opencode/skills/campaign-executor/tasks/",
  [string]$CampaignId = "",
  [int]$MaxParallel = 4,
  [string]$LogDir = "",
  [switch]$Parallel,
  [switch]$DryRun,
  [switch]$Yes
)

$ErrorActionPreference = "Stop"
$planFile = Resolve-Path $PlanFile -ErrorAction Stop
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Resolve-Path (Join-Path $scriptDir "..\..\..\")
$planFileName = Split-Path $planFile -Leaf

if (-not (Test-Path $planFile)) {
  Write-Error "Plan file not found: $planFile"
  exit 1
}

# ---- double PID detection ----
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
  param([string]$PromptBody, [int]$TimeoutSec, [switch]$DryRun, [string]$TaskId = "")
  if ($DryRun) {
    Write-Host "  [DRY RUN]" -ForegroundColor Magenta
    return $true
  }

  $tempPrompt = Join-Path $env:TEMP "campaign-iter-$(Get-Random).md"
  Set-Content $tempPrompt $PromptBody

  try {
    $opencodeCmd = (Get-Command -Name "opencode" -ErrorAction Stop).Source -replace '\.ps1$', '.cmd'

    Write-Host "  ── opencode run ($Model) ─────────────────────────" -ForegroundColor Cyan

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $opencodeCmd
    $psi.Arguments = "run -m $Model --auto --title `"Harness: $TaskId`" `"$tempPrompt`""
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    $p = [System.Diagnostics.Process]::Start($psi)

    $outTask = $p.StandardOutput.ReadToEndAsync()
    $errTask = $p.StandardError.ReadToEndAsync()

    if ($TimeoutSec -gt 0) {
      $exited = $p.WaitForExit($TimeoutSec * 1000)
      if (-not $exited) {
        $p.Kill()
        Write-Host "  ❌ Timeout ($TimeoutSec s). Matado." -ForegroundColor Red
        return $false
      }
    } else {
      $p.WaitForExit()
    }

    $stdout = $outTask.Result
    $stderr = $errTask.Result
    if ($stderr) { Write-Host $stderr -ForegroundColor DarkRed }
    if ($stdout) { Write-Host $stdout }

    Write-Host "  ─────────────────────────────────────────────────" -ForegroundColor Cyan
    return ($p.ExitCode -eq 0)
  }
  catch {
    Write-Host "  ⚠️ Error: $_" -ForegroundColor Yellow
    return $false
  }
  finally {
    Remove-Item $tempPrompt -Force -ErrorAction SilentlyContinue
  }
}

# ---- logging ----
if (-not $LogDir) { $LogDir = Join-Path $projectRoot ".opencode" "task-system" "harness" "logs" }
if (-not (Test-Path $LogDir)) { New-Item -ItemType Directory -Path $LogDir -Force | Out-Null }
$logFile = Join-Path $LogDir "harness-$CampaignId.log"
$logStream = [System.IO.StreamWriter]::new($logFile, $true)
$logStream.WriteLine("=== Harness $CampaignId started $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===")
$logStream.Flush()
Write-Host "  → Log: $logFile" -ForegroundColor DarkGray

function Write-Log {
  param([string]$Message)
  $ts = Get-Date -Format 'HH:mm:ss'
  $logStream.WriteLine("[$ts] $Message")
  $logStream.Flush()
}

function Stop-HarnessLog {
  param([int]$ExitCode = 0)
  $logStream.WriteLine("=== Harness $CampaignId exited ($ExitCode) at $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===")
  $logStream.Close()
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

# Campaign ID (auto-generar si no se pasó)
if (-not $CampaignId) { $CampaignId = [guid]::NewGuid().ToString() }

$currentContent = Get-Content $planFile -Raw

# Escribir Campaign ID si no existe
if ($currentContent -notmatch 'Campaign ID') {
  $cidLine = "> **Campaign ID:** $CampaignId"
  $currentContent = $currentContent -replace '(^>\s\*\*Inicio:\*\*)', "${cidLine}`n`$1"
  Set-Content $planFile $currentContent
} else {
  $currentContent = $currentContent -replace 'Campaign ID:\s*\S+', "Campaign ID: $CampaignId"
  Set-Content $planFile $currentContent
}

# Escribir PID en el plan file
$currentContent = Get-Content $planFile -Raw
$pidLine = "> **Harness PID:** $PID ($(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))"
if ($currentContent -notmatch 'Harness PID') {
  $currentContent = $currentContent -replace '(Campaign ID:.*?\n)', "`$1$pidLine`n"
  Set-Content $planFile $currentContent
}

$iteration = 0
$stallCount = @{}
$taskLastState = @{}

while ($true) {
  $iteration++
  $allTasks = Get-PlanTasks $planFile
  $pendingTasks = $allTasks | Where-Object { $_.State -match '[⬜⏳]' }

  if ($SingleTask) {
    $pendingTasks = $pendingTasks | Where-Object { $_.Id -eq $SingleTask -or $_.Name -match $SingleTask }
  }

  Write-Host ""
  Write-Log "Iteration $($iteration): $($pendingTasks.Count) pending"
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

    # Limpiar PID del plan file
    $content = Get-Content $planFile -Raw
    $content = $content -replace "> \*\*Harness PID:\*\*.*?\n", ""
    Set-Content $planFile $content
    Write-Host "  → PID $PID limpiado del plan file." -ForegroundColor DarkGray
    Stop-HarnessLog -ExitCode 0
    break
  }

  # ---- MODO PARALELO ----
  if ($Parallel -and $pendingTasks.Count -gt 1) {
    Write-Log "Parallel mode: $($pendingTasks.Count) tasks"
    Write-Host "  → Modo paralelo: $($pendingTasks.Count) tareas pendientes" -ForegroundColor Cyan
    $batch = $pendingTasks | Select-Object -First $MaxParallel
    $jobs = @()

    $promptPath = Join-Path $projectRoot ".opencode" "task-system" "prompts" "iter.md"
    if (-not (Test-Path $promptPath)) {
      Write-Error "Prompt not found: $promptPath"
      Stop-HarnessLog -ExitCode 1
      exit 1
    }
    $promptTemplate = Get-Content $promptPath -Raw
    $opencodeCmd = (Get-Command -Name "opencode" -ErrorAction Stop).Source -replace '\.ps1$', '.cmd'

    foreach ($t in $batch) {
      $promptBody = $promptTemplate -replace '{{PLAN_FILE}}', $planFile
      $promptBody = $promptBody -replace '{{SINGLE_TASK}}', $t.Id
      $promptBody = $promptBody -replace '{{TASK_BASE}}', $TaskBase
      $promptBody = $promptBody -replace '{{CAMPAIGN_ID}}', $CampaignId

      $promptFile = Join-Path $env:TEMP "campaign-par-$(Get-Random).md"
      Set-Content $promptFile $promptBody

      $j = Start-Job -ScriptBlock {
        param($pf, $to, $tid, $cmd, $model)
        $env:CARGO_TARGET_DIR = "$env:TEMP\cargo-target-$tid"
        $r = & $cmd run -m $model --auto --title "Harness: TASK-$tid" $pf 2>&1 | Out-String
        Remove-Item $pf -Force -ErrorAction SilentlyContinue
        return $r
      } -ArgumentList $promptFile, $Timeout, $t.Id, $opencodeCmd, $Model

      $jobs += @{ Job = $j; Task = $t }
    }

    Write-Host "  → Esperando $($jobs.Count) jobs paralelos..." -ForegroundColor Cyan
    $jobs | ForEach-Object {
      $j = $_.Job
      $j | Wait-Job -Timeout $Timeout | Out-Null
      if ($j.State -eq 'Running') {
        $j | Stop-Job
        Write-Host "  → TASK-$($_.Task.Id): TIMEOUT" -ForegroundColor Red
      } else {
        $out = Receive-Job -Job $j
        Write-Host "  → TASK-$($_.Task.Id): $($j.State)" -ForegroundColor DarkGray
      }
    }
    continue
  }

  # ---- MODO SECUENCIAL ----
  $nextTask = $pendingTasks[0]
  $taskId = "TASK-$($nextTask.Id)"

  Write-Host "  → Próxima: $taskId ($($nextTask.Name))" -ForegroundColor Cyan
  Write-Host "  → Estado: $(Get-StatusLine $planFile)" -ForegroundColor Gray
  Write-Log "Next: $taskId ($($nextTask.Name))"

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
        Stop-HarnessLog -ExitCode 2
        exit 2
      }
      Write-Host "  ❌ Stall ($StallThreshold intentos). ¿Abortar? (s/N) " -ForegroundColor Red -NoNewline
      if ((Read-Host) -eq 's') {
        $content = Get-Content $planFile -Raw
        $content = $content -replace '(Estado:\s*)⏳ EN PROGRESO', '${1}❌ ABORTADO'
        Set-Content $planFile $content
        Stop-HarnessLog -ExitCode 2
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

  $promptPath = Join-Path $projectRoot ".opencode" "task-system" "prompts" "iter.md"
  if (-not (Test-Path $promptPath)) {
    Write-Error "Prompt not found: $promptPath"
    Stop-HarnessLog -ExitCode 1
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
  $promptBody = $promptBody -replace '{{TASK_BASE}}', $TaskBase
  $promptBody = $promptBody -replace '{{CAMPAIGN_ID}}', $CampaignId

  if ($DryRun) {
    Write-Host "`n  [DRY RUN]" -ForegroundColor Magenta
  } else {
    Write-Host "  → Ejecutando opencode..." -ForegroundColor Cyan
    $env:CARGO_TARGET_DIR = "$env:TEMP\cargo-target-$($nextTask.Id)"
    $ok = Invoke-OpenCodeIteration -PromptBody $promptBody -TimeoutSec $Timeout -DryRun:$DryRun -TaskId $taskId

    if ($ok) {
      $recitation = Get-Recitation $planFile
      if (-not $recitation) {
        if ($Yes) {
          Write-Host "  → Recitation no encontrada, continuando (-Yes)." -ForegroundColor Yellow
        } else {
          Write-Host "  ⚠️ Recitation no encontrada. ¿Continuar? (s/N) " -ForegroundColor Yellow -NoNewline
          if ((Read-Host) -ne 's') { Stop-HarnessLog -ExitCode 3; exit 3 }
        }
      } elseif ($recitation -notmatch '[✅❌]') {
        if ($Yes) {
          Write-Host "  → Recitation sin ✅/❌, continuando (-Yes)." -ForegroundColor Yellow
        } else {
          Write-Host "  ⚠️ Recitation sin ✅/❌. ¿Continuar? (s/N) " -ForegroundColor Yellow -NoNewline
          if ((Read-Host) -ne 's') { Stop-HarnessLog -ExitCode 3; exit 3 }
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
