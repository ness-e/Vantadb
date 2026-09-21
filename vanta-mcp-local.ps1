<#
.SYNOPSIS
  Launcher: explicit embedding env + start of the VantaDB MCP server (stdio).

.DESCRIPTION
  Sets VANTADB_EMBEDDING_PROVIDER and VANTADB_LOCAL_MODEL (absolute) in the
  session and starts `vanta-cli server --mcp --db <Db>`. The child MCP process
  inherits the session env, so it boots with the right model — verifiable via
  the `dim` field in the `embed_texts` response (384 = multilingual-e5-small).

  -Db is MANDATORY with no default: the CLI default (`./db`) must never
  silently point at (or create) a database the user did not choose.
  Secrets (e.g. VANTADB_OPENAI_API_KEY) are NEVER parameters, NEVER written
  to disk and NEVER echoed: they inherit from the session env untouched.

.EXAMPLE
  .\vanta-mcp-local.ps1 -DbPath C:\data\vanta-test
.EXAMPLE
  .\vanta-mcp-local.ps1 -DbPath C:\data\vanta-test -Provider ollama
.EXAMPLE
  # opencode.jsonc: "command": ["pwsh", "-NoProfile", "-File",
  #   "C:/Users/Eros/VantaDB Proyect/VantaDB/vanta-mcp-local.ps1",
  #   "-DbPath", "C:/Users/Eros/.vantadb"]
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, HelpMessage = 'Database directory (explicit, no default).')]
    [string]$DbPath,

    [Parameter()]
    [ValidateSet('local', 'ollama', 'openai')]
    [string]$Provider = 'local',

    [Parameter()]
    [string]$Model,

    [Parameter()]
    [string]$VantaCli,

    [Parameter()]
    [string]$ProxyConfig,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ServerArgs
)

$ErrorActionPreference = 'Stop'

# MCP stdio: stdout is the JSON-RPC channel — info MUST go to stderr.
function info([string]$m) { [System.Console]::Error.WriteLine($m) }

# --- Resolve model dir (absolute; relative default is fragile per CWD) ---
if (-not $Model) {
    $Model = Join-Path $PSScriptRoot 'embeddings/models/multilingual-e5-small/onnx'
}
$Model = [System.IO.Path]::GetFullPath($Model)
if (-not (Test-Path (Join-Path $Model 'model.onnx'))) {
    Write-Warning "Model onnx not found at '$Model' — server will fall back (see EMB-13 fallback flag)."
}

# --- Resolve vanta-cli (repo build first: it is the campaign-verified binary;
# a stale global install masks drift — same rule as skills/vantadb-mcp/scripts/test-mcp.py) ---
if (-not $VantaCli) {
    $local = Join-Path $PSScriptRoot 'target/debug/vanta-cli.exe'
    if (Test-Path $local) {
        $VantaCli = $local
    } else {
        $cmd = Get-Command 'vanta-cli' -ErrorAction SilentlyContinue
        if ($cmd) { $VantaCli = $cmd.Source }
    }
}
if (-not $VantaCli) {
    throw 'vanta-cli not found: install it on PATH or build with `cargo build --bin vanta-cli` (global install = EMB-19).'
}

# --- ORT native lib (>=1.27 required by the embed-local build; System32 1.17.1 aborts) ---
# Never downloads (that is EMB-11 setup-embeddings.ps1); only points at an existing dll.
if (-not $env:ORT_DYLIB_PATH) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'VantaDB/onnxruntime/onnxruntime.dll'),
        (Join-Path $env:TEMP 'opencode/ort130/onnxruntime-win-x64-1.30.0/lib/onnxruntime.dll')
    )
    foreach ($dll in $candidates) {
        if (Test-Path $dll) { $env:ORT_DYLIB_PATH = $dll; break }
    }
    if ($env:ORT_DYLIB_PATH) {
        info "ORT_DYLIB_PATH=$env:ORT_DYLIB_PATH"
    } else {
        Write-Warning 'No onnxruntime >=1.27 found (ORT_DYLIB_PATH unset) — local embeddings will fall back (see EMB-13 fallback flag). Run setup-embeddings.ps1 (EMB-11).'
    }
}

# --- Explicit env (inherited by the MCP child process) ---
$env:VANTADB_EMBEDDING_PROVIDER = $Provider
$env:VANTADB_LOCAL_MODEL = $Model
info "VANTADB_EMBEDDING_PROVIDER=$Provider"
info "VANTADB_LOCAL_MODEL=$Model"
info "DB=$DbPath"
info "BIN=$VantaCli"

# --- Proxy passthrough (FIND-104, SPEC Q4): info-only, nunca stdout ---
# El proxy (vanta-proxy :8096) es un proceso HTTP SEPARADO; este launcher no lo
# arranca (stdout es JSON-RPC). Si se pasa -ProxyConfig existente, se valida
# legible y se exporta VANTADB_PROXY_CONFIG para el hijo + guia a stderr.
if ($ProxyConfig) {
    if (-not (Test-Path $ProxyConfig)) { throw "proxy config no encontrada: $ProxyConfig (wizard: setup-embeddings.ps1 la crea en <Db>/vanta-proxy.toml)" }
    $env:VANTADB_PROXY_CONFIG = [System.IO.Path]::GetFullPath($ProxyConfig)
    info "VANTADB_PROXY_CONFIG=$env:VANTADB_PROXY_CONFIG"
    info 'Proxy base_url: http://127.0.0.1:8096 (arranca aparte: vanta-proxy "<config>"; apaga: Ctrl+C)'
}

# --- Start MCP server over stdio (inherits session env) ---
& $VantaCli server --mcp --db $DbPath @ServerArgs
