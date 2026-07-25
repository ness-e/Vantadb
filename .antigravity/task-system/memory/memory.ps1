param(
  [Parameter(Mandatory)][ValidateSet("lessons", "decisions")][string]$File,
  [string]$Entry = "",
  [switch]$Read,
  [int]$Limit = 20
)

$memoryDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
$fp = Join-Path $memoryDir "$File.md"

if ($Read -or ![string]::IsNullOrWhiteSpace($Entry)) {
  if ($Read) {
    if (-not (Test-Path $fp)) { Write-Error "Memory file $File.md not found"; exit 1 }
    $content = Get-Content $fp -Raw
    $lines = $content -split "`n"
    $tail = $lines[-1..-($Limit+1)] -join "`n"
    Write-Output $tail
  } else {
    $date = (Get-Date -Format "yyyy-MM-dd")
    $line = "- $date | $Entry"
    Add-Content -Path $fp -Value $line -Encoding UTF8
    Write-Output "OK: $line"
  }
} else {
  Write-Output "Usage: memory.ps1 -File lessons|decisions -Read | -Entry 'text'"
}
