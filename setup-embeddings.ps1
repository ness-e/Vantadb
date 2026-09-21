#Requires -Version 7.0
<#
.SYNOPSIS
  Asistente de instalacion de embeddings locales VantaDB (EMB-11, Q1/Q2 owner).

.DESCRIPTION
  Deja modelo local funcionando con Enter-Enter. `-NonInteractive` aplica todo
  por default (provider `local`, modelo default del manifest) sin preguntar.
  Modo interactivo: cada prompt muestra su default y Enter = auto.

  ONNX RUNTIME NATIVO (restriccion EMB-10): `ort` compila ORT_API_VERSION=27 y
  exige onnxruntime nativo >= 1.27. El de System32 (1.17.1) ABORTA el proceso
  (FIND-100) y el de Temp/ es efimero. Este script garantiza un nativo >= 1.27
  en ubicacion persistente y setea ORT_DYLIB_PATH en sesion.

  SECRETS: este script NUNCA escribe secrets a disco. Las keys solo se leen del
  env de sesion ($env:VANTADB_OPENAI_API_KEY / $env:OPENAI_API_KEY) para chequear
  presencia, se muestran maskeadas y jamas se persisten ni loguean en claro.

.EXAMPLE
  pwsh -NoProfile -File setup-embeddings.ps1 -NonInteractive
.EXAMPLE
  pwsh -NoProfile -File setup-embeddings.ps1
#>
[CmdletBinding()]
param(
  [switch]$NonInteractive,
  [switch]$Help,
  [string]$DbPath,
  [switch]$NoProxy,
  [switch]$SkipLiveTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:onnxOverride = $null

function Get-RepoRoot { $PSScriptRoot }

function Get-Manifest {
  $path = Join-Path (Get-RepoRoot) 'embeddings/manifest.json'
  Get-Content -Raw -Path $path | ConvertFrom-Json
}

function Get-ModelOnnxPath($Model, $ModelDir) {
  if (-not $Model.onnx) { return $null }
  Join-Path $ModelDir $Model.onnx
}

function Test-ModelPresent($Model) {
  # Presente = onnx declarado + tokenizer.json (espejo de verify.py).
  if (-not $Model.onnx) { return $false }
  $dir = Join-Path (Get-RepoRoot) "embeddings/models/$($Model.id)"
  $onnx = Get-ModelOnnxPath $Model $dir
  $tok = Join-Path $dir 'tokenizer.json'
  (Test-Path $onnx) -and (Test-Path $tok)
}

function Get-ModelTable($Manifest) {
  foreach ($m in $Manifest.models) {
    $total = ($m.size_onnx_mb ?? 0) + ($m.size_hf_mb ?? 0)
    [pscustomobject]@{
      Id      = $m.id
      Dim     = $m.dim
      TotalMB = $total
      Langs   = ($m.langs -join ',')
      Present = Test-ModelPresent $m
      HasOnnx = (-not [string]::IsNullOrEmpty($m.onnx))
    }
  }
}

function Invoke-DownloadCheck {
  # Verificacion final sin red (contrato EMB-11): debe dar exit 0.
  $dl = Join-Path (Get-RepoRoot) 'embeddings/download.py'
  & python $dl --check
  if ($LASTEXITCODE -ne 0) { throw 'download.py --check fallo (exit $LASTEXITCODE)' }
}

function Read-WithDefault($Prompt, $Default) {
  # Q1: Enter = auto. Todo prompt muestra su default entre corchetes.
  $ans = Read-Host "$Prompt [$Default]"
  if ([string]::IsNullOrWhiteSpace($ans)) { return $Default }
  $ans.Trim()
}

function Confirm-YesNo($Prompt, $DefaultYes = $true) {
  $hint = $DefaultYes ? 'S/n' : 's/N'
  $ans = Read-Host "$Prompt [$hint]"
  if ([string]::IsNullOrWhiteSpace($ans)) { return $DefaultYes }
  's', 'si', 'sí', 'y', 'yes' -contains $ans.Trim().ToLower()
}

function Get-TimeEstimate($TotalMB) {
  # Heuristica honesta ~10 MB/s, marcada como aproximada (Regla 11).
  $sec = [math]::Max(1, [int]($TotalMB / 10))
  $sec -lt 90 ? "≈$sec s (aprox.)" : "≈$([math]::Round($sec / 60)) min (aprox.)"
}

function Show-ModelTable($Manifest) {
  Write-Host ''
  Write-Host 'Modelos (Q2: 3 en disco + resto solo con aviso de tamaño/tiempo):'
  foreach ($row in (Get-ModelTable $Manifest)) {
    $flag = $row.Present ? '[en disco]' : $row.HasOnnx ? "[descarga: $($row.TotalMB) MB, $(Get-TimeEstimate $row.TotalMB)]" : '[GPU-only sin ONNX, no local]'
    $mark = ($row.Id -eq $Manifest.default) ? ' (default)' : ''
    Write-Host ("  {0,-42} {1}d {2,-8} {3}{4}" -f $row.Id, $row.Dim, $row.Langs, $flag, $mark)
  }
  Write-Host ''
}

function Invoke-DownloadModel($Model) {
  $dl = Join-Path (Get-RepoRoot) 'embeddings/download.py'
  & python $dl --only $Model.id
  if ($LASTEXITCODE -ne 0) { throw "descarga de $($Model.id) fallo (exit $LASTEXITCODE)" }
}

function Test-OllamaServer {
  # Q3: solo probe, NUNCA instala nada externo.
  try {
    $r = Invoke-WebRequest 'http://localhost:11434/api/tags' -TimeoutSec 3 -ErrorAction Stop
    $r.StatusCode -eq 200
  } catch { $false }
}

function Test-OpenAiKey {
  # Solo presencia maskeada; la key NUNCA se pide por teclado (PSReadLine
  # history la escribiria a disco) ni se muestra en claro.
  $present = (-not [string]::IsNullOrEmpty($env:VANTADB_OPENAI_API_KEY)) -or
             (-not [string]::IsNullOrEmpty($env:OPENAI_API_KEY))
  $present ? '***set*** (env de sesion)' : 'not set'
}

$script:ortVersion = '1.30.0'
$script:ortZip = 'onnxruntime-win-x64-1.30.0.zip'

function Get-OrtStore {
  # Persistente fuera del repo (Temp/ es efimero, repo-local contamina git).
  $base = $env:LOCALAPPDATA
  if ([string]::IsNullOrEmpty($base)) { $base = Join-Path $HOME '.local/share' }
  Join-Path $base 'VantaDB/onnxruntime'
}

function Get-DllVersion($DllPath) {
  # $null si no existe o sin version legible.
  if (-not (Test-Path $DllPath)) { return $null }
  try {
    $v = [Diagnostics.FileVersionInfo]::GetVersionInfo($DllPath).FileVersion
    if ($v -match '(\d+)\.(\d+)') { return [version]"$($Matches[1]).$($Matches[2])" }
    $null
  } catch { $null }
}

function Find-DllUnder($Dir) {
  if (-not (Test-Path $Dir)) { return $null }
  Get-ChildItem -Recurse -Filter 'onnxruntime.dll' $Dir -ErrorAction SilentlyContinue |
    Select-Object -First 1 -ExpandProperty FullName
}

function Install-OrtStore($FromDll, $AllowDownload) {
  $store = Get-OrtStore
  if ($FromDll) {
    New-Item -ItemType Directory -Force $store | Out-Null
    Copy-Item $FromDll (Join-Path $store 'onnxruntime.dll') -Force
    return (Join-Path $store 'onnxruntime.dll')
  }
  if (-not $AllowDownload) { throw 'ORT >=1.27 no encontrado y descarga no confirmada (sin cambios).' }
  $url = "https://github.com/microsoft/onnxruntime/releases/download/v$($script:ortVersion)/$($script:ortZip)"
  Write-Host "[setup] bajando onnxruntime v$($script:ortVersion) (~80 MB, una vez)..."
  New-Item -ItemType Directory -Force $store | Out-Null
  $zip = Join-Path $store $script:ortZip
  try {
    Invoke-WebRequest $url -OutFile $zip -ErrorAction Stop
    Expand-Archive $zip -DestinationPath (Join-Path $store 'pkg') -Force
    $dll = Find-DllUnder (Join-Path $store 'pkg')
    if (-not $dll) { throw 'zip sin onnxruntime.dll' }
    Copy-Item $dll (Join-Path $store 'onnxruntime.dll') -Force
    Remove-Item $zip -ErrorAction SilentlyContinue
    return (Join-Path $store 'onnxruntime.dll')
  } catch { throw "descarga ORT fallo: $($_.Exception.Message)" }
}

function Ensure-OrtNative($AutoDownload) {
  # Garantiza nativo >=1.27 y deja ORT_DYLIB_PATH en sesion. Orden: store
  # persistente -> ORT_DYLIB_PATH ya seteado -> cache Temp EMB-10 (copia,
  # offline-friendly) -> System32 (casi siempre 1.17.1, se rechaza) -> GitHub.
  $need = [version]'1.27'
  $store = Get-OrtStore
  $dll = Join-Path $store 'onnxruntime.dll'
  if ((Get-DllVersion $dll) -ge $need) {
    Write-Host "[setup] ORT nativo $((Get-DllVersion $dll)) en store persistente."
    $env:ORT_DYLIB_PATH = $store
    return
  }
  if ($env:ORT_DYLIB_PATH) {
    $cur = $env:ORT_DYLIB_PATH
    if (Test-Path $cur -PathType Leaf) { $cur = Split-Path $cur }
    $found = Find-DllUnder $cur
    if ($found -and ((Get-DllVersion $found) -ge $need)) {
      Write-Host "[setup] ORT nativo $((Get-DllVersion $found)) via ORT_DYLIB_PATH existente."
      return
    }
  }
  $cache = Find-DllUnder (Join-Path $env:TEMP 'opencode/ort130')
  if ($cache -and ((Get-DllVersion $cache) -ge $need)) {
    Write-Host '[setup] ORT en cache Temp (efimero) -> copiando a store persistente...'
    $dll = Install-OrtStore $cache $false
    $env:ORT_DYLIB_PATH = $store
    Write-Host "[setup] ORT nativo $((Get-DllVersion $dll)) instalado en $store."
    return
  }
  $sys = Join-Path $env:SystemRoot 'System32/onnxruntime.dll'
  $sysV = Get-DllVersion $sys
  if ($sysV -and ($sysV -lt $need)) {
    Write-Host "[setup] AVISO: System32 trae onnxruntime $sysV (<1.27, ABORTA con ort) -> se ignora, uso dedicado."
  }
  if (-not $AutoDownload) {
    $ok = Confirm-YesNo 'Bajar onnxruntime v1.30.0 oficial (~80 MB, una vez)?'
    if (-not $ok) { throw 'ORT >=1.27 requerido y no confirmado (sin cambios).' }
  }
  $dll = Install-OrtStore $null $true
  $env:ORT_DYLIB_PATH = $store
  Write-Host "[setup] ORT nativo $((Get-DllVersion $dll)) instalado en $store."
  Write-Host '[setup] ORT_DYLIB_PATH seteado en SESION (no persistente: re-ejecuta este script o exportalo vos).'
}

function Get-DefaultDbPath {
  $home = $HOME
  if ([string]::IsNullOrEmpty($home)) { $home = $env:USERPROFILE }
  Join-Path $home '.vantadb'
}

function Write-TextIfChanged($Path, $Content) {
  # Idempotente: no toca el archivo si el contenido es identico.
  # Si existe y cambia -> backup .bak fechado antes de sobrescribir.
  $existing = $null
  if (Test-Path $Path) { $existing = Get-Content -Raw -Path $Path }
  if ($existing -eq $Content) { return 'unchanged' }
  if (Test-Path $Path) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    Copy-Item $Path "$Path.bak-$stamp" -Force
  }
  $dir = Split-Path $Path -Parent
  if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
  Set-Content -Path $Path -Value $Content -Encoding UTF8 -NoNewline
  return ($null -eq $existing) ? 'created' : 'updated-backup'
}

function Get-LauncherPath { Join-Path (Get-RepoRoot) 'vanta-mcp-local.ps1' }

function Get-McpBlock($Client, $Launcher, $Db) {
  $escLauncher = $Launcher -replace '\\', '/'
  $escDb = $Db -replace '\\', '/'
  switch ($Client) {
    'opencode' {
      return @"
{
  "mcp": {
    "vantadb": {
      "type": "local",
      "command": ["pwsh", "-NoProfile", "-File", "$escLauncher", "-DbPath", "$escDb"],
      "enabled": true
    }
  }
}
"@
    }
    'claude' {
      return @"
{
  "mcpServers": {
    "vantadb": {
      "command": "pwsh",
      "args": ["-NoProfile", "-File", "$escLauncher", "-DbPath", "$escDb"],
      "env": { "VANTADB_MEMORY_LIMIT": "512MB" }
    }
  }
}
"@
    }
    'cursor' {
      return @"
{
  "mcpServers": {
    "vantadb": {
      "command": "pwsh",
      "args": ["-NoProfile", "-File", "$escLauncher", "-DbPath", "$escDb"],
      "env": { "VANTADB_MEMORY_LIMIT": "512MB" }
    }
  }
}
"@
    }
    default { throw "cliente '$Client' desconocido (opencode/claude/cursor; codex es manual, ver assets/install/codex-manual.md)" }
  }
}

function Install-McpBlocks($Db) {
  # Escribe bloques MCP personalizados (rutas absolutas launcher+Db) junto a la
  # DB (<Db>/mcp-opencode.json, etc.) — NUNCA a .opencode/ vivo ni sobre las
  # plantillas versionadas skills/vantadb-mcp/assets/install/ (placeholders).
  # Idempotente con backup.
  $launcher = Get-LauncherPath
  foreach ($c in @('opencode', 'claude', 'cursor')) {
    $block = Get-McpBlock $c $launcher $Db
    $out = Join-Path $Db "mcp-$c.json"
    $st = Write-TextIfChanged $out $block
    Write-Host "[setup] MCP $c -> $out ($st)"
  }
  Write-Host '[setup] Plantillas versionadas: skills/vantadb-mcp/assets/install/ (placeholders <REPO>/<DB_PATH>).'
  Write-Host '[setup] Codex: sin formato verificable -> ver skills/vantadb-mcp/assets/install/codex-manual.md (documentado-manual, no bloquea).'
}

$script:agentRuleLine = 'VantaDB es memoria persistente: ante cada mensaje hace memory_recall con el texto verbatim (scope agent, top_k 5) e inyecta prepend_context; al cerrar captura el turno con thread_send (proxy-turns se cura via inbox, nunca auto-promote).'

function Install-AgentRule {
  # Append idempotente de 1 linea a ./AGENTS.md y ./CLAUDE.md del CWD
  # (proyecto TARGET, nunca al repo VantaDB). Crea el archivo si no existe.
  foreach ($name in @('AGENTS.md', 'CLAUDE.md')) {
    $p = Join-Path (Get-Location) $name
    $line = $script:agentRuleLine
    if (Test-Path $p) {
      $content = Get-Content -Raw -Path $p
      if ($content -match [regex]::Escape($line)) {
        Write-Host "[setup] regla ya presente en $name (sin cambios)."
        continue
      }
      Copy-Item -Path $p -Destination "$p.bak" -Force
      Add-Content -Path $p -Value "`n$line`n" -Encoding UTF8
      Write-Host "[setup] regla agregada a $name (backup en $name.bak)."
    } else {
      Set-Content -Path $p -Value "# Agent notes`n`n$line`n" -Encoding UTF8
      Write-Host "[setup] $name creado con regla VantaDB."
    }
  }
}

# --- main ---
if ($Help) {
  Get-Help $PSCommandPath -Detailed
  exit 0
}

$manifest = Get-Manifest
$defaultId = $manifest.default
$model = $manifest.models | Where-Object { $_.id -eq $defaultId }
if (-not $model) { throw "default '$defaultId' no esta en manifest.json" }
$provider = 'local'
$autoDownload = $NonInteractive  # NonInteractive: default sin preguntar.

if (-not $NonInteractive) {
  $provider = (Read-WithDefault 'Provider (local/ollama/openai)' 'local').ToLower()
  if ($provider -notin @('local', 'ollama', 'openai')) {
    Write-Host "[setup] provider '$provider' desconocido -> uso 'local'."
    $provider = 'local'
  }
  if ($provider -eq 'ollama') {
    $up = Test-OllamaServer
    Write-Host ($up ? '[setup] ollama responde en localhost:11434.'
                    : '[setup] AVISO: ollama NO responde en localhost:11434 (requisito). Sigo sin instalar nada; activalo vos e reintenta.')
  }
  if ($provider -eq 'openai') {
    Write-Host ("[setup] OPENAI key: " + (Test-OpenAiKey))
    if ((Test-OpenAiKey) -eq 'not set') {
      Write-Host '[setup] AVISO: sin key en env de sesion (requisito). Seteala vos: $env:VANTADB_OPENAI_API_KEY="..." (nunca la pegues aca ni en archivos).'
    }
  }
  if ($provider -eq 'local') {
    Show-ModelTable $manifest
    $wanted = Read-WithDefault 'Modelo' $defaultId
    $model = $manifest.models | Where-Object { $_.id -eq $wanted }
    if (-not $model) {
      Write-Host "[setup] id '$wanted' no existe -> uso default '$defaultId'."
      $model = $manifest.models | Where-Object { $_.id -eq $defaultId }
    }
    if (-not (Test-ModelPresent $model)) {
      if (-not $model.onnx) {
        Write-Host "[setup] $($model.id): GPU-only sin ONNX ($($model.exception)). No descargable para uso local; elijo default '$defaultId'."
        $model = $manifest.models | Where-Object { $_.id -eq $defaultId }
      } else {
        $total = ($model.size_onnx_mb ?? 0) + ($model.size_hf_mb ?? 0)
        $ok = Confirm-YesNo "Bajar $($model.id) ($total MB, $(Get-TimeEstimate $total))?"
        if (-not $ok) { throw 'cancelado por usuario (sin descarga, sin cambios).' }
        $autoDownload = $true
      }
    }
    $dirDefault = Join-Path (Get-RepoRoot) "embeddings/models/$($model.id)/onnx"
    $dirAns = Read-WithDefault 'Carpeta ONNX del modelo' $dirDefault
    if (-not (Test-Path $dirAns)) {
      Write-Host "[setup] AVISO: '$dirAns' no existe. Uso default del repo."
    } else { $script:onnxOverride = (Resolve-Path $dirAns).Path }
  }
}

$mode = $NonInteractive ? 'defaults, -NonInteractive' : 'interactivo'
Write-Host "[setup] provider=$provider model=$($model.id) ($mode)"
if ($provider -eq 'local' -and -not (Test-ModelPresent $model)) {
  if (-not $autoDownload) { throw "modelo $($model.id) no presente (interactivo cancelado)." }
  Write-Host "[setup] descargando $($model.id)..."
  Invoke-DownloadModel $model
}

$onnxDir = Join-Path (Get-RepoRoot) "embeddings/models/$($model.id)/onnx"
if ($script:onnxOverride) { $onnxDir = $script:onnxOverride }
$env:VANTADB_EMBEDDING_PROVIDER = $provider
if ($provider -eq 'local') {
  $env:VANTADB_LOCAL_MODEL = (Resolve-Path $onnxDir).Path
  Write-Host "[setup] VANTADB_EMBEDDING_PROVIDER=local"
  Write-Host "[setup] VANTADB_LOCAL_MODEL=$env:VANTADB_LOCAL_MODEL"
} else {
  Write-Host "[setup] VANTADB_EMBEDDING_PROVIDER=$provider (requisito marcado arriba; sin modelo local que verificar)"
}

Invoke-DownloadCheck
if ($provider -eq 'local') { Ensure-OrtNative $NonInteractive; Write-Host "[setup] ORT_DYLIB_PATH=$env:ORT_DYLIB_PATH (solo sesion)" }
if ($provider -eq 'local') {
  Write-Host "[setup] OK modelo $($model.id) dim=$($model.dim) langs=$($model.langs -join ',') presente y verificado (--check)."
}

# --- S1: carpeta de base + bloques MCP + regla agente (FIND-104) ---
if ([string]::IsNullOrWhiteSpace($DbPath)) {
  $DbPath = $NonInteractive ? (Get-DefaultDbPath) : (Read-WithDefault 'Carpeta de base VantaDB' (Get-DefaultDbPath))
}
if (-not (Test-Path $DbPath)) {
  New-Item -ItemType Directory -Force $DbPath | Out-Null
  Write-Host "[setup] DB creada: $DbPath"
} else { Write-Host "[setup] DB existente (re-ejecutable, sin romper): $DbPath" }
Install-McpBlocks $DbPath
if ($NonInteractive) {
  Write-Host '[setup] -NonInteractive: omito regla AGENTS.md/CLAUDE.md (ejecuta interactivo para agregarla).'
} else {
  if (Confirm-YesNo 'Agregar linea de regla VantaDB a ./AGENTS.md y ./CLAUDE.md (idempotente)?') { Install-AgentRule }
  else { Write-Host '[setup] regla agente omitida (re-ejecuta para agregarla).' }
}

# --- S2: proxy default-on + TOML minima + guia + resumen (FIND-104, SPEC Q4) ---
$proxyOn = -not $NoProxy
if (-not $NonInteractive -and -not $NoProxy) {
  Write-Host '[setup] Proxy vanta-proxy: proceso extra :8096 que captura turnos a proxy-turns (curaduria a hilos via inbox, nunca auto-promote). Apagado facil: no arranques el proxy o Ctrl+C.'
  $proxyOn = Confirm-YesNo 'Prender proxy por defecto (recomendado, opt-out en 1 paso)?'
}
if ($proxyOn) {
  $proxyToml = Join-Path $DbPath 'vanta-proxy.toml'
  if (-not (Test-Path $proxyToml)) {
    $proxyContent = @'
# vanta-proxy config minima (FIND-104 default-on). Start: vanta-proxy "<esta-ruta>"
# Stop: Ctrl+C (no arrancar = proxy off). Captura turnos a namespace proxy-turns;
# curalos a hilos con thread_send via inbox (nunca auto-promote, ver recall-policy §6).

[server]
host = "127.0.0.1"
port = 8096
rate_limit_per_minute = 60

[upstream]
url = "https://api.anthropic.com"
api_key = ""
forward_timeout_secs = 600
models = []
'@
    Write-TextIfChanged $proxyToml $proxyContent | Out-Null
    Write-Host "[setup] proxy TOML creada: $proxyToml"
  } else { Write-Host "[setup] proxy TOML existente (sin overwrite): $proxyToml" }
  Write-Host '[setup] PROXY base_url por cliente: http://127.0.0.1:8096 (OpenAI-compat: --base-url / baseURL / openai.api_base).'
  Write-Host '[setup] Curaduria: memory_list en proxy-turns (keys {ms}-{seq}) -> propuesta thread_send -> inbox aprueba/rechaza.'
} else { Write-Host '[setup] proxy OFF (opt-out; re-ejecuta sin -NoProxy para prenderlo).' }

Write-Host ''
Write-Host '========== RESUMEN =========='
Write-Host "[setup] provider=$provider model=$($model.id) db=$DbPath proxy=$(($proxyOn) ? 'ON :8096' : 'OFF')"
Write-Host "[setup] MCP bloques: $DbPath/mcp-opencode.json, mcp-claude.json, mcp-cursor.json (+ plantillas skills/vantadb-mcp/assets/install/)"
Write-Host "[setup] Siguiente: apunta tu cliente al bloque MCP + (si proxy ON) base_url http://127.0.0.1:8096 + arranca: vanta-proxy `"$DbPath/vanta-proxy.toml`""
Write-Host '============================='

# --- S4: prueba viva final put->get->search (FIND-104) ---
function Invoke-LiveTest($Db) {
  $cli = Join-Path (Get-RepoRoot) 'target/debug/vanta-cli.exe'
  if (-not (Test-Path $cli)) {
    $cmd = Get-Command 'vanta-cli' -ErrorAction SilentlyContinue
    if ($cmd) { $cli = $cmd.Source }
  }
  if (-not (Test-Path $cli)) { throw 'prueba viva: vanta-cli no encontrado (cargo build --bin vanta-cli).' }
  $ns = 'wizard/selftest'
  $key = 'hello'
  $payload = 'wizard live test memory'
  & $cli --db $Db put --namespace $ns --key $key --payload $payload | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'prueba viva: put fallo.' }
  $got = & $cli --db $Db get --namespace $ns --key $key
  if ($LASTEXITCODE -ne 0) { throw 'prueba viva: get fallo.' }
  if ("$got" -notmatch [regex]::Escape($payload)) { throw 'prueba viva: get no devuelve el payload.' }
  & $cli --db $Db search --namespace $ns --query 'live test' --limit 3 | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'prueba viva: search fallo.' }
  Write-Host '[setup] PRUEBA VIVA verde: put->get->search OK.'
}
if (-not $SkipLiveTest) { Invoke-LiveTest $DbPath }
else { Write-Host '[setup] prueba viva omitida (-SkipLiveTest).' }
