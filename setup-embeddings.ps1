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
  [switch]$Help
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
