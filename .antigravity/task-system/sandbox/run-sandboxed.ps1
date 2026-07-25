param(
  [Parameter(Mandatory)][string]$Command,
  [string[]]$StageFiles = @(),
  [string]$WorkDir = "",
  [int]$TimeoutSeconds = 60,
  [switch]$BlockNetwork,
  [switch]$NoCleanup
)

$ErrorActionPreference = "Stop"
$start = Get-Date
$sandboxRoot = Join-Path $env:TEMP "vantadb-sandbox-$(Get-Random -Maximum 99999)"
$null = New-Item -ItemType Directory -Path $sandboxRoot -Force

$staged = @()
if ($StageFiles.Count -gt 0) {
  foreach ($f in $StageFiles) {
    $src = Resolve-Path $f -ErrorAction SilentlyContinue
    if (-not $src) { continue }
    $rel = $f -replace '^.*[/\\]', ''
    $dest = Join-Path $sandboxRoot $rel
    if (Test-Path -LiteralPath $src -PathType Container) {
      Copy-Item -Path $src -Destination $dest -Recurse -Force
    } else {
      $null = New-Item -ItemType Directory -Path (Split-Path $dest -Parent) -Force
      Copy-Item -Path $src -Destination $dest -Force
    }
    $staged += $f
  }
}

$useWorkDir = if ($WorkDir) { Join-Path $sandboxRoot $WorkDir } else { $sandboxRoot }
$null = New-Item -ItemType Directory -Path $useWorkDir -Force

$envBackup = @{}
$envRestore = @()
if ($BlockNetwork) {
  $envBackup.HTTP_PROXY = $env:HTTP_PROXY; $env:HTTP_PROXY = ""
  $envBackup.HTTPS_PROXY = $env:HTTPS_PROXY; $env:HTTPS_PROXY = ""
  $envBackup.NO_PROXY = $env:NO_PROXY; $env:NO_PROXY = "*"
}

try {
  $psi = New-Object System.Diagnostics.ProcessStartInfo
  if ($env:OS -match "Windows") {
    $psi.FileName = "pwsh"
    $psi.Arguments = "-NoProfile -Command `"$Command`""
  } else {
    $psi.FileName = "bash"
    $psi.Arguments = "-c `"$Command`""
  }
  $psi.WorkingDirectory = $useWorkDir
  $psi.RedirectStandardOutput = $true
  $psi.RedirectStandardError = $true
  $psi.UseShellExecute = $false
  $psi.CreateNoWindow = $true

  $proc = [System.Diagnostics.Process]::Start($psi)
  $completed = $proc.WaitForExit($TimeoutSeconds * 1000)
  if (-not $completed) {
    $proc.Kill()
    $elapsed = "{0:N1}s" -f ((Get-Date) - $start).TotalSeconds
    $result = @{ valid = $false; error = "TIMEOUT after ${TimeoutSeconds}s"; exitCode = -1; elapsed = $elapsed }
  } else {
    $stdout = $proc.StandardOutput.ReadToEnd().Trim()
    $stderr = $proc.StandardError.ReadToEnd().Trim()
    $exitCode = $proc.ExitCode
    $elapsed = "{0:N1}s" -f ((Get-Date) - $start).TotalSeconds
    $result = @{ valid = $exitCode -eq 0; exitCode = $exitCode; stdout = $stdout; stderr = $stderr; elapsed = $elapsed }
  }
} catch {
  $elapsed = "{0:N1}s" -f ((Get-Date) - $start).TotalSeconds
  $result = @{ valid = $false; error = $_.Exception.Message; exitCode = -1; elapsed = $elapsed }
} finally {
  foreach ($k in $envBackup.Keys) { Set-Item -Path "env:$k" -Value $envBackup[$k] }
  if (-not $NoCleanup) { Remove-Item -Path $sandboxRoot -Recurse -Force -ErrorAction SilentlyContinue }
}

$result.staged = $staged
$result.sandboxDir = if ($NoCleanup) { $sandboxRoot } else { $null }
$result | ConvertTo-Json -Compress
