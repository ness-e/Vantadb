#!/usr/bin/env pwsh
<#
.SYNOPSIS
  Collects all relevant VantaDB source files into a single Markdown snapshot for AI analysis.
.DESCRIPTION
  Walks the repo, excludes non-essential dirs/files, sorts by last-modified (newest first),
  and writes a structured .md file with metadata, file tree, and full contents.
.PARAMETER OutDir
  Output directory (default: docs/operations/snapshots).
.PARAMETER MaxFileSizeMB
  Skip files larger than this (default: 1 MB).
.PARAMETER DryRun
  Preview which files would be included without writing output.
.PARAMETER NoStats
  Skip per-file line/word/token stats (faster on huge repos).
.EXAMPLE
  .\collect_code.ps1
  .\collect_code.ps1 -DryRun
  .\collect_code.ps1 -MaxFileSizeMB 2 -OutDir ./exports
#>

param(
  [string]$OutDir = "docs/operations/snapshots",
  [int]$MaxFileSizeMB = 1,
  [switch]$DryRun,
  [switch]$NoStats
)

$ErrorActionPreference = "Stop"
$MaxBytes = $MaxFileSizeMB * 1MB

$ScriptPath = $MyInvocation.MyCommand.Path
$ScriptDir = Split-Path -Parent $ScriptPath
$ProjectRoot = $ScriptDir
for ($i = 0; $i -lt 3 -and -not (Test-Path (Join-Path $ProjectRoot ".git") -PathType Container); $i++) {
  $parentDir = Split-Path -Parent $ProjectRoot
  if ($parentDir -eq $ProjectRoot) { break }
  $ProjectRoot = $parentDir
}
$OutputDir = Join-Path -Path $ProjectRoot -ChildPath $OutDir
$Date = Get-Date -Format "yyyy-MM-dd"
$OutputFile = Join-Path -Path $OutputDir -ChildPath "snapshot_$Date.md"
$ScriptName = Split-Path -Leaf $ScriptPath

if (-not $DryRun) {
  $null = New-Item -ItemType Directory -Path $OutputDir -Force
}

$ExcludedPatterns = @(
  '\.git', 'target', 'node_modules', 'venv', '\.venv', '__pycache__',
  '\.idea', '\.vscode', 'dist', 'build', '\.pytest_cache',
  'tmp', 'datasets', 'vanta_snapshots', 'vanta-web',
  'tests?_(server|graph|vector|python)_db',
  'docs[\\/]operations[\\/]snapshots', 'docs[\\/]progreso',
  'apruba', 'release_test', 'validation_logs', 'job.*_logs',
  '\.trash_docker', '\.trunk',
  'Cargo\.lock$',
  '\.wasm$', '\.png$', '\.jpg$', '\.jpeg$', '\.gif$', '\.svg$', '\.ico$',
  '\.profraw$', '\.gcda$', '\.gcno$', '\.d\.ts$',
  'vanta_certification\.json$',
  'archive[\\/]', 'packages[\\/]', 'Formula[\\/]', 'completions[\\/]',
  'package-lock\.json$',
  'vanta_benchmark_report\.json$',
  'vantadb-wasm[\\/]pkg',
  '\.cargo[\\/]', '\.devin[\\/]'
)

$IncludedExtensions = @(
  '.rs', '.toml', '.py', '.sh', '.ps1', '.bat', '.cmd',
  '.ts', '.mjs', '.cjs',
  '.md', '.json', '.yml', '.yaml',
  '.sql', '.dockerignore', '.gitignore', '.env',
  '.vanta_profile'
)

$IncludedNames = @(
  'dockerfile', 'makefile', 'license',
  'rust-toolchain.toml', '.rustfmt.toml', '.clippy.toml',
  'deny.toml'
)

$LangMap = @{
  '.rs'   = 'rust'
  '.py'   = 'python'
  '.toml' = 'toml'
  '.json' = 'json'
  '.yml'  = 'yaml'
  '.yaml' = 'yaml'
  '.sh'   = 'bash'
  '.ps1'  = 'powershell'
  '.bat'  = 'batch'
  '.cmd'  = 'batch'
  '.sql'  = 'sql'
  '.md'   = 'markdown'
  '.env'  = 'text'
}

$GitBranch = 'N/A'; $GitCommit = 'N/A'
try { $GitBranch = git rev-parse --abbrev-ref HEAD 2>$null; $GitCommit = git rev-parse --short HEAD 2>$null } catch {}

$RustVersion = 'N/A'; $CargoVersion = 'N/A'
try { $RustVersion = rustc --version 2>$null; $CargoVersion = cargo --version 2>$null } catch {}

$GitLog = 'N/A'
try { $GitLog = (git log -n 5 --oneline 2>$null) -join "`n" } catch {}

function Test-Binary {
  param([string]$Path)
  try {
    $buffer = [byte[]]::new(8192)
    $stream = [System.IO.File]::OpenRead($Path)
    $read = $stream.Read($buffer, 0, $buffer.Length)
    $stream.Close()
    for ($i = 0; $i -lt $read; $i++) { if ($buffer[$i] -eq 0) { return $true } }
    return $false
  } catch { return $true }
}

Write-Host ''
Write-Host ('=' * 55) -ForegroundColor Cyan
Write-Host '  VANTA DB - Code Snapshot Collector' -ForegroundColor White -BackgroundColor DarkBlue
Write-Host ('=' * 55) -ForegroundColor Cyan
Write-Host "  Branch: $GitBranch  |  Commit: $GitCommit" -ForegroundColor Gray
Write-Host "  Max file size: ${MaxFileSizeMB}MB  |  Dry-run: $DryRun" -ForegroundColor Gray
Write-Host ('=' * 55) -ForegroundColor Cyan

$AllFiles = Get-ChildItem -File -Recurse -LiteralPath $ProjectRoot | Where-Object {
  $RelPath = $_.FullName.Substring($ProjectRoot.Length).TrimStart('\', '/')
  $Ext = $_.Extension.ToLower()
  $Name = $_.Name.ToLower()

  foreach ($P in $ExcludedPatterns) { if ($RelPath -match $P) { return $false } }
  if ($RelPath -eq $ScriptName -or $RelPath -eq "docs/operations/snapshots/snapshot_$Date.md") { return $false }
  if ($_.Length -gt $MaxBytes) { return $false }
  if (Test-Binary $_.FullName) { return $false }
  if ($IncludedExtensions -contains $Ext) { return $true }
  if ($IncludedNames -contains $Name) { return $true }

  return $false
} | Sort-Object LastWriteTime -Descending

$TotalFiles = $AllFiles.Count
if ($TotalFiles -eq 0) { Write-Host 'No files matched. Check exclusions.' -ForegroundColor Yellow; return }

# ---- DRY RUN ----
if ($DryRun) {
  $totalWordsDry = 0
  Write-Host "`n[DRY-RUN] $TotalFiles files would be included:" -ForegroundColor Cyan
  foreach ($F in $AllFiles) {
    $Rel = $F.FullName.Substring($ProjectRoot.Length).TrimStart('\', '/')
    $Size = if ($F.Length -ge 1MB) { "{0:N2} MB" -f ($F.Length/1MB) } else { "{0:N1} KB" -f ($F.Length/1KB) }
    Write-Host ('  ' + $Rel + ' (' + $Size + ', modified ' + $F.LastWriteTime.ToString('yyyy-MM-dd HH:mm') + ')') -ForegroundColor Gray
    $c = Get-Content $F.FullName -Raw -ErrorAction SilentlyContinue
    if ($c) { $totalWordsDry += ($c -split '\s+').Count }
  }
  $EstTokens = [Math]::Round($totalWordsDry * 1.35)
  Write-Host "`nEstimated: ~$EstTokens tokens" -ForegroundColor Yellow
  return
}

# ---- BUILD OUTPUT ----
$Sb = [System.Text.StringBuilder]::new()
$Nl = [System.Environment]::NewLine

$Null = $Sb.AppendLine("# VANTA DB - Project Context Snapshot")
$Null = $Sb.AppendLine("Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')")
$Null = $Sb.AppendLine("Git: branch [$GitBranch] commit [$GitCommit]")
$Null = $Sb.AppendLine("Rust: $RustVersion")
$Null = $Sb.AppendLine("Cargo: $CargoVersion")
$Null = $Sb.AppendLine("")
$Null = $Sb.AppendLine("## Recent History")
$Null = $Sb.AppendLine('```text'); $Null = $Sb.AppendLine($GitLog); $Null = $Sb.AppendLine('```')
$Null = $Sb.AppendLine("")
$Null = $Sb.AppendLine("## AI Instructions")
$Null = $Sb.AppendLine("This file is a consolidated snapshot of VantaDB for AI analysis.")
$Null = $Sb.AppendLine("1. Files are ordered by last modified (newest first).")
$Null = $Sb.AppendLine("2. Use the file tree below to understand the project structure.")
$Null = $Sb.AppendLine("3. Look for '--- START OF FILE ---' and '--- END OF FILE ---' delimiters.")
$Null = $Sb.AppendLine("")

$Null = $Sb.AppendLine("## Project Structure")
$Null = $Sb.AppendLine('```text')
foreach ($F in $AllFiles) {
  $Rel = $F.FullName.Substring($ProjectRoot.Length).TrimStart('\', '/')
  $Null = $Sb.AppendLine('  ' + $Rel)
}
$Null = $Sb.AppendLine('```')
$Null = $Sb.AppendLine("")
$Null = $Sb.AppendLine('=' * 80)

$TotalLines = 0
$TotalWords = 0
$LangStats = @{}
$Current = 0

foreach ($File in $AllFiles) {
  $Current++
  $Rel = $File.FullName.Substring($ProjectRoot.Length).TrimStart('\', '/')
  $Ext = $File.Extension.ToLower()
  $LangStats[$Ext] = ($LangStats[$Ext] + 1)

  $SizeStr = if ($File.Length -ge 1MB) { "{0:N2} MB" -f ($File.Length/1MB) } else { "{0:N1} KB" -f ($File.Length/1KB) }
  $Modified = $File.LastWriteTime.ToString("yyyy-MM-dd HH:mm")

  Write-Progress -Activity "Collecting" -Status $Rel -PercentComplete (($Current / $TotalFiles) * 100)

  try {
    $Content = Get-Content $File.FullName -Raw -ErrorAction Stop
    $Lines = ($Content -split '\r?\n').Count
    $Words = ($Content -split '\s+').Count
    $TotalLines += $Lines; $TotalWords += $Words

    $Lang = if ($LangMap.ContainsKey($Ext)) { $LangMap[$Ext] } else { 'text' }
    $Fence = '```'

    $Null = $Sb.AppendLine($Nl + ('=' * 80))
    $Null = $Sb.AppendLine("--- START OF FILE: $Rel ($SizeStr, modified $Modified) ---")
    $Null = $Sb.AppendLine('=' * 80)
    $Null = $Sb.AppendLine("${Fence}$Lang")
    $Trimmed = $Content.TrimEnd("`r", "`n")
    $Null = $Sb.Append($Trimmed)
    $Null = $Sb.AppendLine("")
    $Null = $Sb.AppendLine($Fence)
    $Null = $Sb.AppendLine('=' * 80)
    $Null = $Sb.AppendLine("--- END OF FILE: $Rel ---")
    $Null = $Sb.AppendLine(('=' * 80) + $Nl)

    Write-Host ("[$Current/$TotalFiles] ") -NoNewline -ForegroundColor Cyan
    Write-Host 'v ' -NoNewline -ForegroundColor Green
    Write-Host "$Rel " -NoNewline -ForegroundColor Gray
    Write-Host "($SizeStr)" -ForegroundColor DarkGray
  } catch {
    $Null = $Sb.AppendLine($Nl + ('=' * 80))
    $Null = $Sb.AppendLine("--- START OF FILE: $Rel (read error) ---")
    $Null = $Sb.AppendLine('=' * 80)
    $Null = $Sb.AppendLine("[Error: $($_.Exception.Message)]")
    $Null = $Sb.AppendLine(('=' * 80) + $Nl)
    $Null = $Sb.AppendLine("--- END OF FILE: $Rel ---")
    $Null = $Sb.AppendLine(('=' * 80) + $Nl)
    Write-Host ("[$Current/$TotalFiles] x $Rel [Error]") -ForegroundColor Red
  }
}

Write-Progress -Activity "Collecting" -Completed

$EstTokens = [Math]::Round($TotalWords * 1.35)

$Null = $Sb.AppendLine("---")
$Null = $Sb.AppendLine("## Collection Summary")
$Null = $Sb.AppendLine("- **Total files**: $TotalFiles")
$Null = $Sb.AppendLine("- **Total lines**: $TotalLines")
$Null = $Sb.AppendLine("- **Estimated tokens (AI)**: ~$EstTokens")
$Null = $Sb.AppendLine("")
$Null = $Sb.AppendLine("### Per language")
foreach ($KV in $LangStats.GetEnumerator() | Sort-Object Name) {
  $Null = $Sb.AppendLine("- $($KV.Key): $($KV.Value) files")
}
$Null = $Sb.AppendLine("")
$Null = $Sb.AppendLine("---")
$Null = $Sb.AppendLine("END OF SNAPSHOT")
$Null = $Sb.AppendLine("")

[System.IO.File]::WriteAllText($OutputFile, $Sb.ToString(), [System.Text.UTF8Encoding]::new($false))
$OutputSize = (Get-Item $OutputFile).Length
$OutputSizeStr = if ($OutputSize -ge 1MB) { "{0:N2} MB" -f ($OutputSize/1MB) } else { "{0:N1} KB" -f ($OutputSize/1KB) }

Write-Host "`n$(Get-Date -Format 'HH:mm:ss') - Snapshot written:" -ForegroundColor Cyan
Write-Host "  $OutputFile" -ForegroundColor White
Write-Host "  Size: $OutputSizeStr  |  Files: $TotalFiles  |  Lines: $TotalLines  |  ~${EstTokens} tokens" -ForegroundColor Yellow
Write-Host "  Max file size: ${MaxFileSizeMB}MB (larger files skipped)" -ForegroundColor DarkGray

if ($EstTokens -gt 200000) {
  Write-Host ""
  Write-Host "WARNING: ~${EstTokens} tokens exceeds most AI context windows." -ForegroundColor Red
  Write-Host "  Consider: -MaxFileSizeMB 0.5 or manually prune the snapshot." -ForegroundColor Yellow
}
