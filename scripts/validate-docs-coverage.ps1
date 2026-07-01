<#
.SYNOPSIS
  Valida que los métodos públicos del código fuente estén documentados.
.DESCRIPTION
  Compara funciones públicas de src/sdk.rs, campos de src/config.rs,
  variantes de src/error.rs y comandos CLI contra sus respectivos docs.
  Emite errores si encuentra métodos sin documentar.
.EXAMPLE
  pwsh scripts\validate-docs-coverage.ps1
  pwsh scripts\validate-docs-coverage.ps1 -ReportOnly
#>

param(
  [switch]$ReportOnly
)

$exitCode = 0
$root = Resolve-Path "$PSScriptRoot\.."

# ─── helpers ───
function Test-InDoc {
  param([string]$DocText, [string]$Name)
  return ($DocText -match "``$Name``" -or                      # `Name`
          $DocText -match "``$Name\b" -or                        # `Name(
          $DocText -match "$Name\(" -or                          # Name(
          $DocText -match "(?m)^#{2,4}\s+$Name\b" -or            # ## Name
          $DocText -match "VantaError::$Name\b" -or               # VantaError::Name
          $DocText -match "``$Name\b" -or                        # `name (lowercase match)
          $DocText -match "\b$Name\b")                            # bare word boundary
}

function Check-Methods {
  param(
    [string]$Label,
    [string[]]$Methods,
    [string]$DocRelPath,
    [string]$DocLabel,
    [string[]]$Exclude
  )
  $docPath = Join-Path $root $DocRelPath
  if (-not (Test-Path $docPath)) {
    Write-Host "⚠️  $Label — DOC NOT FOUND: $DocRelPath" -ForegroundColor Red
    if (-not $ReportOnly) { $script:exitCode = 1 }
    return
  }
  $doc = Get-Content $docPath -Raw
  $undocumented = @()
  foreach ($m in $Methods) {
    if ($m -in $Exclude) { continue }
    if (-not (Test-InDoc -DocText $doc -Name $m)) { $undocumented += $m }
  }
  if ($undocumented.Count -gt 0) {
    Write-Host "⚠️  $Label ($($undocumented.Count)/$($Methods.Count) gaps en $DocLabel):" -ForegroundColor Yellow
    foreach ($u in $undocumented) { Write-Host "    - $u" }
    if (-not $ReportOnly) { $script:exitCode = 1 }
  } else {
    Write-Host "✅ $Label — $($Methods.Count) items ok en $DocLabel" -ForegroundColor Green
  }
}

# ═══════════════════════════════════════
#  1. SDK
# ═══════════════════════════════════════
$sdkAll = Select-String -Path "$root\src\sdk.rs" -Pattern '^\s*pub (unsafe )?(async )?fn (\w+)' |
  ForEach-Object { $_.Matches[0].Groups[3].Value } | Sort-Object -Unique

$sdkNormal = $sdkAll | Where-Object { $_ -notlike 'debug_*' }
$sdkDebug  = $sdkAll | Where-Object { $_ -like 'debug_*' }

Check-Methods -Label "src/sdk.rs (públicos)" -Methods $sdkNormal -DocRelPath "docs\api\EMBEDDED_SDK.md" -DocLabel "EMBEDDED_SDK.md"
Check-Methods -Label "src/sdk.rs (debug_*)" -Methods $sdkDebug -DocRelPath "docs\api\EMBEDDED_SDK.md" -DocLabel "EMBEDDED_SDK.md" -Exclude @(
  'debug_clear_derived_indexes_for_tests',
  'debug_clear_text_index_for_tests',
  'debug_corrupt_derived_index_state_for_tests',
  'debug_corrupt_text_index_doc_stats_for_tests',
  'debug_corrupt_text_index_posting_positions_for_tests',
  'debug_corrupt_text_index_posting_tf_for_tests',
  'debug_corrupt_text_index_state_for_tests',
  'debug_corrupt_text_index_term_stats_for_tests',
  'debug_memory_search_plan_for_tests',
  'debug_text_index_audit_for_tests',
  'debug_text_index_posting_for_tests',
  'debug_text_index_posting_keys_for_tests'
)

# ═══════════════════════════════════════
#  2. Config
# ═══════════════════════════════════════
$configFields = Select-String -Path "$root\src\config.rs" -Pattern '^\s+pub (\w+):' |
  ForEach-Object { $_.Matches[0].Groups[1].Value } | Sort-Object -Unique

Check-Methods -Label "src/config.rs" -Methods $configFields -DocRelPath "docs\operations\CONFIGURATION.md" -DocLabel "CONFIGURATION.md" -Exclude @(
  'llm_model','llm_url','llm_summarize_model'
)

# ═══════════════════════════════════════
#  3. Error variants
# ═══════════════════════════════════════
$errorText = Get-Content "$root\src\error.rs" -Raw
$errorEnumBody = $errorText -split '(?<=pub enum VantaError \{)' | Select-Object -Skip 1 -First 1
$allErrors = if ($errorEnumBody) {
  [regex]::Matches($errorEnumBody, '^\s{4}(\w+)', 'Multiline') |
    ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique
} else { @() }

Check-Methods -Label "src/error.rs (VantaError)" -Methods $allErrors -DocRelPath "docs\api\EMBEDDED_SDK.md" -DocLabel "EMBEDDED_SDK.md"

# ═══════════════════════════════════════
#  4. CLI commands
# ═══════════════════════════════════════
$cliText = Get-Content "$root\src\cli.rs" -Raw
$cliBody = $cliText -split '(?<=pub enum Commands \{)' | Select-Object -Skip 1 -First 1
$cliCommands = if ($cliBody) {
  [regex]::Matches($cliBody, '^\s{4}(\w+)', 'Multiline') |
    ForEach-Object { $_.Groups[1].Value }
} else { @() }

$nsBody = $cliText -split '(?<=pub enum NamespaceCommand \{)' | Select-Object -Skip 1 -First 1
$cliSub = if ($nsBody) {
  [regex]::Matches($nsBody, '^\s{4}(\w+)', 'Multiline') |
    ForEach-Object { $_.Groups[1].Value }
} else { @() }

$allCli = ($cliCommands + $cliSub) | Sort-Object -Unique | Where-Object { $_ -ne 'fn' }
# Los comandos CLI se documentan en kebab-case (ej: AuditIndex → audit-index)
$cliKebab = $allCli | ForEach-Object {
  # PascalCase → kebab-case: "AuditIndex" → "audit-index"
  $_ -creplace '(?<=[a-z])(?=[A-Z])', '-' -creplace '(?<=[A-Z])(?=[A-Z][a-z])', '-' | ForEach-Object { $_.ToLower() }
}

Check-Methods -Label "src/cli.rs (comandos)" -Methods $cliKebab -DocRelPath "docs\operations\CONFIGURATION.md" -DocLabel "CONFIGURATION.md (sección CLI)" -Exclude @(
  'info','list','bash','zsh','fish','power-shell'
)

# ═══════════════════════════════════════
#  5. Python bindings
# ═══════════════════════════════════════
$pyLib = "$root\vantadb-python\src\lib.rs"
if (Test-Path $pyLib) {
  $pyInternals = @(
    'py_any_to_value','extract_vector','set_python_value',
    'runtime_profile_label','tier_label','node_to_pydict',
    'format_query_result','capabilities_to_pydict',
    'memory_record_to_pydict','bm25_term_to_pydict',
    'explanation_hit_to_pydict','hybrid_fusion_report_to_pydict',
    'search_explanation_to_pydict','memory_hit_to_pydict',
    'rebuild_report_to_pydict','export_report_to_pydict',
    'import_report_to_pydict','text_index_repair_report_to_pydict',
    'text_index_audit_report_to_pydict','operational_metrics_to_pydict',
    'py_dict_to_metadata','search_batch','__repr__'
  )
  $pyAll = Select-String -Path $pyLib -Pattern '^\s{4}fn (\w+)' |
    ForEach-Object { $_.Matches[0].Groups[1].Value } |
    Where-Object { $_ -notin $pyInternals } | Sort-Object -Unique

  Check-Methods -Label "vantadb-python (métodos expuestos)" -Methods $pyAll -DocRelPath "docs\api\PYTHON_SDK.md" -DocLabel "PYTHON_SDK.md"
}

# ═══════════════════════════════════════
#  Resumen
# ═══════════════════════════════════════
if ($exitCode -eq 0) {
  Write-Host "`n✅ Validación de cobertura completada — 0 gaps" -ForegroundColor Green
} else {
  Write-Host "`n⚠️  Hay métodos sin documentar. Edita el doc correspondiente y vuelve a ejecutar." -ForegroundColor Yellow
}

exit $exitCode
