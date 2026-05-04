$outputFile = 'todo.md'
$scriptName = $MyInvocation.MyCommand.Name

# Directorios a excluir (regex)
$excludedPatterns = @(
    '^\.git$', 'target', 'node_modules', 'venv', '^\.venv$', '__pycache__',
    '^\.idea$', '^\.vscode$', 'dist', 'build', '^\.pytest_cache$', 'vanta_snapshots',
    'tmp', 'datasets', 'test_sdk.*', 'tests_.*', '^\.agents$', '^\.cargo$',
    'vanta-web', 'vantadb-python', 'vantadb_data', 'vanta_python_binding',
    'tests_server_db', 'tests_graph_db', 'tests_vector_db', 'tests_python_api'
)

# Extensiones a incluir (whitelist)
$includedPatterns = @(
    '.rs', '.toml', '.py', '.sh', '.ps1', '.bat', '.md', '.json', '.yml', '.yaml',
    '.sql', '.dockerignore', '.gitignore', '.vanta_profile', '.env', 'Cargo.lock'
)

# --- METADATOS DE GIT ---
$gitBranch = 'N/A'
$gitCommit = 'N/A'
try {
    $gitBranch = (git rev-parse --abbrev-ref HEAD 2>$null)
    $gitCommit = (git rev-parse --short HEAD 2>$null)
} catch {}

# --- CABECERA EN CONSOLA ---
Write-Host ' '
Write-Host '====================================================' -ForegroundColor Cyan
Write-Host ' 🚀 VANTA DB - Professional Code Collector' -ForegroundColor White -BackgroundColor DarkBlue
Write-Host '====================================================' -ForegroundColor Cyan
Write-Host (' 🌿 Rama: ' + $gitBranch + ' | Commit: ' + $gitCommit) -ForegroundColor Gray
Write-Host '====================================================' -ForegroundColor Cyan

# --- INICIALIZACIÓN ---
Set-Content -Path $outputFile -Value '' -Encoding utf8
$basePath = (Get-Location).Path
$files = Get-ChildItem -File -Recurse | Where-Object {
    $relPath = $_.FullName.Substring($basePath.Length).TrimStart('\')
    foreach ($p in $excludedPatterns) { if ($relPath -match ('(^|\\)' + $p + '(\\.*|$)')) { return $false } }
    if ($_.Name -eq $outputFile -or $_.Name -eq $scriptName) { return $false }
    $ext = $_.Extension.ToLower()
    $name = $_.Name.ToLower()
    foreach ($p in $includedPatterns) { if ($ext -eq $p.ToLower() -or $name -eq $p.ToLower()) { return $true } }
    return $false
} | Sort-Object LastWriteTime -Descending

$totalFiles = ($files | Measure-Object).Count
$current = 0
$totalLines = 0
$totalWords = 0
$langStats = @{}

# --- MAPA DE ESTRUCTURA ---
Write-Host '🌳 Generando mapa de estructura...' -ForegroundColor Cyan
$treeText = '## ESTRUCTURA DEL PROYECTO' + [System.Environment]::NewLine + '```text' + [System.Environment]::NewLine
foreach ($f in $files) { $treeText += ($f.FullName.Replace($basePath, '.') + [System.Environment]::NewLine) }
$treeText += '```' + [System.Environment]::NewLine

# --- CONSTRUIR HEADER PARA IA ---
$nl = [System.Environment]::NewLine
$aiHeader = '# VANTA DB - PROJECT CONTEXT SNAPSHOT' + $nl
$aiHeader += 'Generado el: ' + (Get-Date).ToString('yyyy-MM-dd HH:mm:ss') + $nl
$aiHeader += 'Git: Rama [' + $gitBranch + '] | Commit [' + $gitCommit + ']' + $nl + $nl
$aiHeader += '## INSTRUCCIONES PARA LA IA' + $nl
$aiHeader += 'Este documento contiene una consolidacion completa del proyecto VantaDB.' + $nl
$aiHeader += '1. **Contexto**: Este archivo representa el estado actual y veridico del sistema.' + $nl
$aiHeader += '2. **Estructura**: Revisa el mapa de directorios abajo para entender la jerarquia.' + $nl
$aiHeader += '3. **Orden**: Archivos ordenados por RECIENTE (los de arriba son los últimos editados).' + $nl
$aiHeader += '4. **Marcas**: Busca "START OF FILE" y "END OF FILE" para cada seccion.' + $nl + $nl
$aiHeader += $treeText + $nl + ('=' * 80) + $nl

Set-Content -Path $outputFile -Value $aiHeader -Encoding utf8

# --- PROCESAMIENTO ---
Write-Host ('📂 Procesando ' + $totalFiles + ' archivos...') -ForegroundColor Yellow

foreach ($file in $files) {
    $current++
    $relPath = $file.FullName.Substring($basePath.Length).TrimStart('\')
    $ext = $file.Extension.ToLower(); if ($ext -eq '') { $ext = '(sin ext)' }
    $langStats[$ext] = ($langStats[$ext] + 1)
    
    $sizeStr = if ($file.Length -ge 1MB) { ([Math]::Round($file.Length / 1MB, 2).ToString() + ' MB') } else { ([Math]::Round($file.Length / 1KB, 2).ToString() + ' KB') }
    Write-Progress -Activity 'Vanta Collector' -Status ('Procesando: ' + $relPath) -PercentComplete (($current / $totalFiles) * 100)
    
    $sep = '=' * 80
    Add-Content -Path $outputFile -Value ($nl + $sep) -Encoding utf8
    Add-Content -Path $outputFile -Value ('--- START OF FILE: ' + $relPath + ' (Size: ' + $sizeStr + ' | Modified: ' + $file.LastWriteTime.ToString() + ') ---') -Encoding utf8
    Add-Content -Path $outputFile -Value $sep -Encoding utf8
    
    try {
        $content = Get-Content $file.FullName -Raw -ErrorAction Stop
        $totalLines += ($content -split '\r?\n').Count
        $totalWords += ($content -split '\s+').Count
        Add-Content -Path $outputFile -Value $content -Encoding utf8

        Add-Content -Path $outputFile -Value ($nl + $sep) -Encoding utf8
        Add-Content -Path $outputFile -Value ('--- END OF FILE: ' + $relPath + ' ---') -Encoding utf8
        Add-Content -Path $outputFile -Value ($sep + $nl) -Encoding utf8

        Write-Host ('[' + $current + '/' + $totalFiles + '] ') -NoNewline -ForegroundColor Cyan
        Write-Host '✔ ' -NoNewline -ForegroundColor Green
        Write-Host ($relPath + ' ') -NoNewline -ForegroundColor Gray
        Write-Host ('(' + $sizeStr + ')') -ForegroundColor DarkGray
    }
    catch {
        Add-Content -Path $outputFile -Value ('[Error de lectura en ' + $relPath + ']') -Encoding utf8
        Write-Host ('[' + $current + '/' + $totalFiles + '] ✖ ' + $relPath + ' [Error]') -ForegroundColor Red
    }
}

# --- RESUMEN FINAL ---
$estimatedTokens = [Math]::Round($totalWords * 1.35)
Write-Host ' '
Write-Host '✨ ¡Snapshot Completado!' -ForegroundColor Green
Write-Host '------------------------------------------------' -ForegroundColor Gray
Write-Host '📊 Estadísticas por Lenguaje:' -ForegroundColor Cyan
foreach ($kv in $langStats.GetEnumerator()) {
    Write-Host ('   ' + $kv.Key + ': ' + $kv.Value + ' archivos') -ForegroundColor Gray
}
Write-Host '------------------------------------------------' -ForegroundColor Gray
Write-Host '📝 Resumen Global:' -ForegroundColor Cyan
Write-Host '   Total Líneas: ' -NoNewline -ForegroundColor Gray; Write-Host $totalLines -ForegroundColor Yellow
Write-Host '   Total Palabras: ' -NoNewline -ForegroundColor Gray; Write-Host $totalWords -ForegroundColor Yellow
Write-Host '   Tokens Est. (IA): ' -NoNewline -ForegroundColor Gray; Write-Host ('~' + $estimatedTokens) -ForegroundColor Cyan
Write-Host '------------------------------------------------' -ForegroundColor Gray

Add-Content -Path $outputFile -Value ($nl + '# RESUMEN FINAL DE RECOLECCIÓN') -Encoding utf8
Add-Content -Path $outputFile -Value ('Total Archivos: ' + $totalFiles) -Encoding utf8
Add-Content -Path $outputFile -Value ('Total Líneas: ' + $totalLines) -Encoding utf8
Add-Content -Path $outputFile -Value ('Tokens Estimados: ~' + $estimatedTokens) -Encoding utf8
