# dev-tools/ocr-review.ps1 — OpenCodeReview delegation spec (sin LLM, sin API key).
#
# Genera el spec determinista de OCR (file selection + rules) para que el
# host agent (OpenCode/vanta-*) ejecute la revisión cognitiva con su propio LLM.
# Advisory por defecto: exit 0 si el spec se generó, 1 solo si falla la herramienta.
# Canónico: .opencode/references/ocr-review.md

param(
  [string]$Commit = "",
  [string]$From = "",
  [string]$To = "",
  [ValidateSet("text", "json")][string]$Format = "text",
  [string]$Output = "",
  [string]$Background = "",
  [string]$BackgroundFile = "",
  [string]$Exclude = "**/target/*,**/node_modules/*,**/.venv/*,.codegraph/*",
  [string]$Repo = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot
if ($Repo -eq "") { $Repo = $ProjectRoot }

$ocr = Get-Command "ocr" -ErrorAction SilentlyContinue
if (-not $ocr) {
  Write-Host "OCR CLI no instalado. Instalá con: npm install -g @alibaba-group/open-code-review" -ForegroundColor Red
  exit 1
}

function Build-Flags {
  $f = @()
  if ($Commit -ne "") { $f += @("--commit", $Commit) }
  if ($From -ne "") { $f += @("--from", $From) }
  if ($To -ne "") { $f += @("--to", $To) }
  if ($Exclude -ne "") { $f += @("--exclude", $Exclude) }
  if ($Background -ne "") { $f += @("--background", $Background) }
  if ($BackgroundFile -ne "") { $f += @("--background-file", $BackgroundFile) }
  $f += @("--repo", $Repo)
  return $f
}

$flags = Build-Flags

if ($Format -eq "json") {
  # Un solo documento: { preview: {...}, rules: [...] } — parseable con un json.load.
  # 1. Preview — qué archivos son revisables (stderr a null: warnings benignos como el gitlink .opencode)
  $previewArgs = @("delegate", "preview", "--format", "json") + $flags
  $previewRaw = & ocr @previewArgs 2>$null
  if ($LASTEXITCODE -ne 0) { Write-Host "OCR preview falló (exit $LASTEXITCODE)" -ForegroundColor Red; exit 1 }
  $previewObj = ($previewRaw | Out-String | ConvertFrom-Json)

  # 2. Rules — checklist por archivo revisable
  $files = @(($previewObj.reviewable_files) | Where-Object { $_.path -ne ".opencode" } | ForEach-Object { $_.path })
  $groups = @()
  if ($files.Count -eq 0) {
    Write-Host "OCR: nada revisable — spec solo con preview." -ForegroundColor DarkYellow
  } else {
    Write-Host "OCR rules: $($files.Count) archivo(s)" -ForegroundColor DarkGray
    $ruleArgs = @("delegate", "rule") + $files + @("--format", "json") + $flags
    $rulesRaw = & ocr @ruleArgs 2>$null
    if ($LASTEXITCODE -ne 0) { Write-Host "OCR rule falló (exit $LASTEXITCODE)" -ForegroundColor Red; exit 1 }
    $groups = @(($rulesRaw | Out-String | ConvertFrom-Json).groups)
  }

  $spec = [ordered]@{ schema = "ocr-delegate/v1"; preview = $previewObj; rules = $groups }
  $specText = $spec | ConvertTo-Json -Depth 20
  if ($Output -ne "") { $specText | Set-Content -Path $Output -Encoding UTF8 }
  else { $specText | Write-Output }
} else {
  $previewArgs = @("delegate", "preview") + $flags
  & ocr @previewArgs
  if ($LASTEXITCODE -ne 0) { exit 1 }
  Write-Host ""
  Write-Host "Siguiente (el agente): ocr delegate rule <paths del preview> + git diff por archivo" -ForegroundColor DarkGray
}

exit 0
