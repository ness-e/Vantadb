# heavy-test-lock.ps1 — Serializa pruebas pesadas entre sesiones (regla owner 2026-10-05).
# UNA prueba pesada a la vez: builds grandes, cargo nextest (workspace/crate), maturin, etc.
# Uso:
#   pwsh dev-tools/heavy-test-lock.ps1 acquire   # espera si hay otra prueba pesada corriendo
#   pwsh dev-tools/heavy-test-lock.ps1 release   # al terminar
#   pwsh dev-tools/heavy-test-lock.ps1 status    # ver estado
# TTL 45 min: si una sesión muere sin liberar, el lock expira solo (nadie queda bloqueado para siempre).
param(
  [Parameter(Mandatory)][ValidateSet('acquire', 'release', 'status')][string]$Action,
  [int]$WaitMinutes = 30,
  [int]$TtlMinutes = 45
)

$root = Split-Path $PSScriptRoot -Parent
$lockDir = Join-Path $root '.opencode\locks'
$lockFile = Join-Path $lockDir 'heavy-test.lock'
New-Item -ItemType Directory -Force -Path $lockDir | Out-Null

function Read-Lock {
  if (-not (Test-Path $lockFile)) { return $null }
  try { return (Get-Content $lockFile -Raw | ConvertFrom-Json) } catch { return $null }
}
function Is-Expired($lock) {
  if (-not $lock) { return $true }
  return ((Get-Date) -gt [datetime]$lock.expires)
}

switch ($Action) {
  'status' {
    $lock = Read-Lock
    if (-not $lock) { echo 'lock: LIBRE' }
    elseif (Is-Expired $lock) { echo "lock: EXPIRADO (de $($lock.session) desde $($lock.since)) — liberable" }
    else { echo "lock: TOMADO por $($lock.session) desde $($lock.since) (expira $($lock.expires))" }
  }
  'acquire' {
    $deadline = (Get-Date).AddMinutes($WaitMinutes)
    while ($true) {
      $lock = Read-Lock
      if (Is-Expired $lock) {
        @{ session = $env:OPENCODE_SESSION_ID; pid = $PID; since = (Get-Date -Format o); expires = ((Get-Date).AddMinutes($TtlMinutes).ToString('o')) } |
          ConvertTo-Json | Set-Content $lockFile
        echo "LOCK ADQUIRIDO (sesion $($env:OPENCODE_SESSION_ID), TTL $TtlMinutes min)"
        exit 0
      }
      if ((Get-Date) -gt $deadline) {
        echo "LOCK TIMEOUT (>${WaitMinutes}min): sigue tomado por $($lock.session)"
        exit 1
      }
      echo "lock ocupado por $($lock.session) — esperando 60s (hasta $WaitMinutes min)..."
      Start-Sleep -Seconds 60
    }
  }
  'release' {
    $lock = Read-Lock
    if (-not $lock) { echo 'lock ya estaba libre' }
    elseif ($lock.session -eq $env:OPENCODE_SESSION_ID -or (Is-Expired $lock)) {
      Remove-Item $lockFile -Force
      echo 'LOCK LIBERADO'
    }
    else { echo "lock de otra sesion ($($lock.session)) — no lo toco" }
  }
}
