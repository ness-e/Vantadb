#Requires -Version 7.0
# FIND-176 regression test — setup-embeddings.ps1 must leave ORT_DYLIB_PATH
# pointing at the onnxruntime.dll FILE, not the store DIRECTORY.
#
# Why: `ort` loads a file (src/embedding_health.rs resolve_ort_dylib_path ->
# ort::init_from); a directory fails with LoadLibraryExW and silently degrades
# embeddings to deterministic dummy vectors (fallback:true, reason:dylib).
#
# Scope: runs the REAL wizard with -NonInteractive against a TEMP store
# (LOCALAPPDATA override) + TEMP db — never touches the user's persistent store
# or ~/.vantadb. -SkipLiveTest avoids needing a vanta-cli build.
#
# Usage: pwsh -NoProfile -File dev-tools/scripts/test-find176-wizard-ort-dylib.ps1
# Exit 0 = green (or SKIP when the machine lacks a seed dll / model / python).

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$wizard = Join-Path $repoRoot 'setup-embeddings.ps1'
if (-not (Test-Path $wizard)) { throw "wizard not found: $wizard" }

function Get-DllVersion($Path) {
    if (-not (Test-Path $Path)) { return $null }
    try {
        $v = [Diagnostics.FileVersionInfo]::GetVersionInfo($Path).FileVersion
        if ($v -match '(\d+)\.(\d+)') { return [version]"$($Matches[1]).$($Matches[2])" }
        $null
    } catch { $null }
}
function Skip($why) { Write-Host "SKIP FIND-176: $why"; exit 0 }

# --- seed a real dll >= 1.27 into a temp store (same layout the wizard uses) ---
$need = [version]'1.27'
$seed = @(
    (Join-Path $env:LOCALAPPDATA 'VantaDB/onnxruntime/onnxruntime.dll'),
    (Join-Path $env:TEMP 'opencode/ort130/onnxruntime-win-x64-1.30.0/lib/onnxruntime.dll')
) | Where-Object { (Get-DllVersion $_) -ge $need } | Select-Object -First 1
if (-not $seed) { Skip 'no onnxruntime.dll >= 1.27 available to seed the temp store' }
if (-not (Get-Command python -ErrorAction SilentlyContinue)) { Skip 'python not available (wizard runs download.py --check)' }
$modelDir = Join-Path $repoRoot 'embeddings/models/multilingual-e5-small'
if (-not ((Test-Path (Join-Path $modelDir 'onnx/model.onnx')) -and (Test-Path (Join-Path $modelDir 'tokenizer.json')))) {
    Skip 'default model not on disk (wizard would download it)'
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("vantadb-find176-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
$tmpLocalAppData = Join-Path $tmp 'localappdata'
$storeDir = Join-Path $tmpLocalAppData 'VantaDB/onnxruntime'
$db = Join-Path $tmp 'db'

$pass = 0; $fail = 0
function Ok($n) { $script:pass++; Write-Host "PASS $n" }
function Bad($n, $why) { $script:fail++; Write-Host "FAIL $n -- $why" }
function Req($n, $cond, $why) { if ($cond) { Ok $n } else { Bad $n $why } }

try {
    New-Item -ItemType Directory -Force $storeDir | Out-Null
    Copy-Item $seed (Join-Path $storeDir 'onnxruntime.dll') -Force

    # --- run the real wizard against the temp store (child pwsh scopes the override) ---
    $driver = @"
Set-Location '$repoRoot'
`$env:LOCALAPPDATA = '$tmpLocalAppData'
Remove-Item Env:ORT_DYLIB_PATH -ErrorAction SilentlyContinue
& '$wizard' -NonInteractive -DbPath '$db' -SkipLiveTest -NoProxy
exit `$LASTEXITCODE
"@
    $out = & pwsh -NoProfile -Command $driver 2>&1
    $code = $LASTEXITCODE
    $out | ForEach-Object { Write-Host "  [wizard] $_" }
    Req 'run/exit-0' ($code -eq 0) "wizard exit=$code"

    # --- parse the resolved path the wizard exported ---
    $line = $out | Where-Object { "$_" -match '^\[setup\] ORT_DYLIB_PATH=' } | Select-Object -Last 1
    Req 'run/prints-ort-dylib-path' ($null -ne $line) 'wizard did not print [setup] ORT_DYLIB_PATH='

    if ($null -ne $line) {
        $value = "$line" -replace '^\[setup\] ORT_DYLIB_PATH=', '' -replace ' \(solo sesion\)$', ''

        # --- FIND-176 contract: a FILE inside the store, not the store directory ---
        Req 'path/ends-with-dll' ($value.EndsWith('onnxruntime.dll')) "expected '...\onnxruntime.dll', got '$value'"
        Req 'path/is-leaf' (Test-Path -LiteralPath $value -PathType Leaf) "not a file: '$value'"
        Req 'path/not-a-dir' (-not (Test-Path -LiteralPath $value -PathType Container)) "value is a directory: '$value'"
        Req 'path/is-store-dll' ($value -eq (Join-Path $storeDir 'onnxruntime.dll')) "expected '$(Join-Path $storeDir 'onnxruntime.dll')', got '$value'"

        Write-Host ''
        Write-Host "ORT_DYLIB_PATH resolved by wizard: $value"
    }

    # --- run #2: ORT_DYLIB_PATH pre-set to a DIRECTORY must be normalized to the
    # dll (Ensure-OrtNative pre-set branch — empty store so the branch is reached) ---
    $tmp2LocalAppData = Join-Path $tmp 'localappdata2'
    $presetDir = Join-Path $tmp 'preset-dir'
    New-Item -ItemType Directory -Force $presetDir | Out-Null
    Copy-Item $seed (Join-Path $presetDir 'onnxruntime.dll') -Force
    $db2 = Join-Path $tmp 'db2'
    $driver2 = @"
Set-Location '$repoRoot'
`$env:LOCALAPPDATA = '$tmp2LocalAppData'
`$env:ORT_DYLIB_PATH = '$presetDir'
& '$wizard' -NonInteractive -DbPath '$db2' -SkipLiveTest -NoProxy
exit `$LASTEXITCODE
"@
    $out2 = & pwsh -NoProfile -Command $driver2 2>&1
    $code2 = $LASTEXITCODE
    Req 'preset/exit-0' ($code2 -eq 0) "wizard exit=$code2"
    $line2 = $out2 | Where-Object { "$_" -match '^\[setup\] ORT_DYLIB_PATH=' } | Select-Object -Last 1
    if ($null -ne $line2) {
        $value2 = "$line2" -replace '^\[setup\] ORT_DYLIB_PATH=', '' -replace ' \(solo sesion\)$', ''
        Req 'preset/normalized-to-dll' ($value2 -eq (Join-Path $presetDir 'onnxruntime.dll')) "expected '$(Join-Path $presetDir 'onnxruntime.dll')', got '$value2'"
    } else {
        Bad 'preset/normalized-to-dll' 'run #2 did not print [setup] ORT_DYLIB_PATH='
    }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

if ($fail -gt 0) { Write-Host "FIND-176 RED: $pass PASS / $fail FAIL"; exit 1 }
Write-Host "FIND-176 GREEN: $pass PASS / 0 FAIL"
exit 0
