# audit-tokens.ps1 — Swiss+Neubrutalism design token linter for VantaDB
# Usage: pwsh scripts/audit-tokens.ps1
#
# Checks:
#  1. Inline styles (style={{...}}) in TSX
#  2. Non-zero border-radius in CSS
#  3. Blur shadows in CSS
#  4. Hardcoded hex/rgb colors in CSS (not using vars)
#  5. Decorative gradients in CSS
#  6. Colored border-left in CSS (AI slop pattern)

$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$WEB = Join-Path $ROOT "web\src"
$VIOLATIONS = 0

function Write-Header($label) {
  Write-Host "─── $label ───" -ForegroundColor Cyan
}

function Test-InlineStyles {
  Write-Header "1/6  Inline styles (style={{...}})"
  $matches = Get-ChildItem -Recurse -Filter "*.tsx" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' } | `
    Select-String -Pattern "style={{" -SimpleMatch
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - No inline styles found" -ForegroundColor Green
  return $true
}

function Test-BorderRadius {
  Write-Header "2/6  Non-zero border-radius"
  $matches = Get-ChildItem -Recurse -Filter "*.css" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' } | `
    Select-String -Pattern "border-radius" | `
    Where-Object { $_.Line -notmatch '(0px|0;)' }
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)  $($_.Line.Trim())" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - All border-radius are 0px" -ForegroundColor Green
  return $true
}

function Test-Shadows {
  Write-Header "3/6  Blur shadows (non-token box-shadow)"
  $matches = Get-ChildItem -Recurse -Filter "*.css" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' } | `
    Select-String -Pattern "box-shadow" | `
    Where-Object { $_.Line -notmatch '(0px 0px|--shadow-)' }
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)  $($_.Line.Trim())" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - All box-shadows use --shadow-* tokens" -ForegroundColor Green
  return $true
}

function Test-HardcodedColors {
  Write-Header "4/6  Hardcoded colors in CSS (outside tokens.css)"
  $tokensPath = Join-Path $WEB "styles\tokens.css"
  $matches = Get-ChildItem -Recurse -Filter "*.css" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' -and $_.FullName -ne $tokensPath } | `
    Select-String -Pattern "#[0-9a-fA-F]{3,}|rgba\(|rgb\(" | `
    Where-Object { $_.Line -notmatch '(--.*:|noise-overlay|scanline|::-)' }
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)  $($_.Line.Trim())" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - No hardcoded colors outside tokens.css" -ForegroundColor Green
  return $true
}

function Test-Gradients {
  Write-Header "5/6  Decorative gradients"
  $matches = Get-ChildItem -Recurse -Filter "*.css" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' } | `
    Select-String -Pattern "linear-gradient|radial-gradient|conic-gradient" | `
    Where-Object { $_.Line -notmatch 'scanline' }
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)  $($_.Line.Trim())" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - No decorative gradients" -ForegroundColor Green
  return $true
}

function Test-BorderLeft {
  Write-Header "6/6  Colored border-left (AI slop pattern)"
  $matches = Get-ChildItem -Recurse -Filter "*.css" $WEB | `
    Where-Object { $_.FullName -notmatch '\\node_modules\\' } | `
    Select-String -Pattern "border-left" -SimpleMatch
  if ($matches) {
    $matches | ForEach-Object { Write-Host "  $($_.Filename):$($_.LineNumber)  $($_.Line.Trim())" }
    $script:VIOLATIONS++
    return $false
  }
  Write-Host "  OK - No border-left patterns" -ForegroundColor Green
  return $true
}

# Run checks
$results = @(
  Test-InlineStyles
  Test-BorderRadius
  Test-Shadows
  Test-HardcodedColors
  Test-Gradients
  Test-BorderLeft
)

Write-Host ""
Write-Host "═══════════════════════════" -ForegroundColor Cyan
if ($VIOLATIONS -eq 0) {
  Write-Host "  ALL CHECKS PASSED" -ForegroundColor Green
  exit 0
} else {
  Write-Host "  $VIOLATIONS violation groups found" -ForegroundColor Red
  exit 1
}
