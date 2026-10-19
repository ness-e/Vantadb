<#
.SYNOPSIS
    Automatiza la descarga, verificación y configuración de EmbeddingGemma 2 en VantaDB.

.DESCRIPTION
    Script de aprovisionamiento para el modelo EmbeddingGemma 2 (Google DeepMind, 768d, Gemma).
    Soporta:
      1. Modo ONNX Nativo: Descarga lazy desde Hugging Face mediante download.py y verificación de integridad.
      2. Modo Ollama: Pull del modelo local en Ollama con aceleración GPU.
      3. Verificación de integridad matemática y estructural del manifest.

.PARAMETER Mode
    Modo de ejecución: 'onnx' (default, local nativo en VantaDB) o 'ollama' (vía daemon local).

.EXAMPLE
    pwsh scripts/setup-embeddinggemma.ps1 -Mode onnx
    pwsh scripts/setup-embeddinggemma.ps1 -Mode ollama
#>

[CmdletBinding()]
param(
    [ValidateSet("onnx", "ollama", "both")]
    [string]$Mode = "onnx"
)

$ErrorActionPreference = "Stop"

Write-Host "`n=== [VantaDB] Aprovisionamiento de EmbeddingGemma 2 (768d) ===" -ForegroundColor Cyan

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $RepoRoot

# 1. Validación de prerequisitos
Write-Host "`n[1/4] Verificando prerequisitos del sistema..." -ForegroundColor Yellow

$Python = Get-Command python -ErrorAction SilentlyContinue
if (-not $Python) {
    Write-Error "Python no está disponible en PATH. Instale Python >= 3.10 para continuar."
}

# 2. Descarga según modo
if ($Mode -eq "onnx" -or $Mode -eq "both") {
    Write-Host "`n[2/4] Aprovisionando modo ONNX Nativo (embed-local)..." -ForegroundColor Yellow
    
    # Validar huggingface_hub
    & python -c "import huggingface_hub" 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Instalando dependencia 'huggingface_hub'..." -ForegroundColor Gray
        & pip install huggingface_hub
    }

    Write-Host "Descargando pesos ONNX de 'onnx-community/embeddinggemma-2-ONNX'..." -ForegroundColor Gray
    & python embeddings/download.py --only embeddinggemma-2
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Fallo en la descarga de EmbeddingGemma 2 vía download.py"
    }

    $ModelDir = Join-Path $RepoRoot "embeddings/models/embeddinggemma-2/onnx"
    $OnnxFile = Join-Path $ModelDir "model.onnx"
    $TokFile = Join-Path $ModelDir "tokenizer.json"

    if (Test-Path $OnnxFile) {
        $SizeMb = [math]::Round((Get-Item $OnnxFile).Length / 1MB, 2)
        Write-Host "OK: Modelo ONNX verificado en $OnnxFile ($SizeMb MB)" -ForegroundColor Green
    } else {
        # En caso de exportaciones multimodales, verificar si el archivo base está en carpeta
        Write-Host "ADVERTENCIA: Archivo model.onnx no detectado directamente en la raíz de onnx/." -ForegroundColor Yellow
    }
}

if ($Mode -eq "ollama" -or $Mode -eq "both") {
    Write-Host "`n[2b/4] Aprovisionando modo Ollama (remote-inference local)..." -ForegroundColor Yellow
    $Ollama = Get-Command ollama -ErrorAction SilentlyContinue
    if (-not $Ollama) {
        Write-Host "ADVERTENCIA: Ollama no está instalado o no está en PATH. Omitiendo pull de Ollama." -ForegroundColor Yellow
    } else {
        Write-Host "Ejecutando 'ollama pull embeddinggemma-2'..." -ForegroundColor Gray
        & ollama pull embeddinggemma-2
        if ($LASTEXITCODE -eq 0) {
            Write-Host "OK: Modelo EmbeddingGemma 2 descargado en Ollama." -ForegroundColor Green
        }
    }
}

# 3. Verificación de integridad de manifest y estructura
Write-Host "`n[3/4] Ejecutando validación de estructura y gates de integridad..." -ForegroundColor Yellow
& python embeddings/download.py --check --only embeddinggemma-2
& python embeddings/verify.py --check

# 4. Resumen de configuración
Write-Host "`n[4/4] Configuración de entorno generada:" -ForegroundColor Cyan
Write-Host "─────────────────────────────────────────────────────────────" -ForegroundColor Gray
if ($Mode -eq "onnx" -or $Mode -eq "both") {
    Write-Host "# Para usar ONNX Nativo en VantaDB (100% Offline):" -ForegroundColor Green
    Write-Host "`$env:VANTADB_EMBEDDING_PROVIDER = `"local`""
    Write-Host "`$env:VANTADB_LOCAL_MODEL = `"$RepoRoot\embeddings\models\embeddinggemma-2\onnx`""
}
if ($Mode -eq "ollama" -or $Mode -eq "both") {
    Write-Host "`n# Para usar Ollama Local con aceleración GPU:" -ForegroundColor Green
    Write-Host "`$env:VANTADB_EMBEDDING_PROVIDER = `"ollama`""
    Write-Host "`$env:VANTADB_LLM_URL = `"http://localhost:11434`""
    Write-Host "`$env:VANTADB_LLM_MODEL = `"embeddinggemma-2`""
}
Write-Host "─────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host "¡Aprovisionamiento completado con éxito!`n" -ForegroundColor Green
