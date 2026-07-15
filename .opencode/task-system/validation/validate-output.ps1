param(
  [Parameter(Mandatory, Position=0)][string]$Content,
  [Parameter(Position=1)][ValidateSet("shell","file_path","python","code","sql","html","text")][string]$Type = "text",
  [string]$Workspace = ""
)

$dangerousCmd = @(
  '\brm\s+-rf?\b', '\bformat\s+\w:?\b', '\brd\s+/s\s+/q\b',
  '\bmkfs\.\w+\b', '\bdd\s+if=', '\bchmod\s+777\s+/',
  ':\(\)\{', '>\s*/dev/(sda|sdb|sdc|null)'
)
$pipedShell = @('\|\s*(bash|sh|zsh|pwsh|powershell|cmd)\b')
$sysDirs = @('/etc', '/bin', '/sbin', '/usr', '/boot', '/dev', '/proc', '/sys',
  'C:\Windows', 'C:\System32', 'C:\Program Files')
$dangerousPy = @('import os', 'import subprocess', 'import sys', 'eval(', 'exec(', '__import__(')

$errors = @()
$warnings = @()
$checks = @()
$valid = $true
$risk = "safe"

switch ($Type) {
  "shell" {
    if ([string]::IsNullOrWhiteSpace($Content)) { $errors += "Empty command"; $valid = $false; break }
    foreach ($pat in $dangerousCmd) { if ($Content -match $pat) { $errors += "Dangerous pattern: $pat" } }
    foreach ($pat in $pipedShell) { if ($Content -match $pat) { $warnings += "Piped to shell: $pat" } }
    $checks += "Shell command checked"
  }
  "file_path" {
    if ([string]::IsNullOrWhiteSpace($Content)) { $errors += "Empty path"; $valid = $false; break }
    if ($Content -match '\.\.') { $errors += "Path traversal detected" }
    if ($Workspace) {
      $resolved = [System.IO.Path]::GetFullPath($Content)
      $ws = [System.IO.Path]::GetFullPath($Workspace)
      if (-not $resolved.StartsWith($ws, [StringComparison]::OrdinalIgnoreCase)) { $errors += "Path escapes workspace" }
    }
    foreach ($d in $sysDirs) { if ($Content -match [regex]::Escape($d)) { $errors += "Writes to system directory: $d" } }
    $checks += "File path checked"
  }
  { $_ -in "python", "code" } {
    foreach ($d in $dangerousPy) { if ($Content -match [regex]::Escape($d)) { $warnings += "Contains: $d" } }
    $checks += "Python code checked (dangerous imports)"
  }
  "sql" {
    if ($Content -match '\b(drop|truncate|alter|create|grant|revoke)\s+') { $warnings += "SQL contains DDL/DCL keyword" }
    $checks += "SQL checked"
  }
  "html" {
    if ($Content -match '<script') { $warnings += "HTML contains <script> — XSS risk" }
    $checks += "HTML checked"
  }
  default { $checks += "Text validated" }
}

if ($errors.Count -gt 0) { $risk = "dangerous"; $valid = $false }
elseif ($warnings.Count -gt 0) { $risk = "moderate" }

$result = @{
  valid = $valid
  riskLevel = $risk
  errors = $errors
  warnings = $warnings
  checksPassed = $checks
}

$result | ConvertTo-Json -Compress
