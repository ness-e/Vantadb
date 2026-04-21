$outputFile = "todo.md"
$scriptName = $MyInvocation.MyCommand.Name

# Directorios a excluir (dependencias, temporales, control de versiones)
$excludedDirs = @(
    ".git", 
    "target", 
    "node_modules", 
    "venv", 
    ".venv", 
    "__pycache__", 
    ".idea", 
    ".vscode", 
    "dist", 
    "build", 
    ".pytest_cache", 
    "vanta_snapshots", 
    "tmp",
    "vanta-web",
    "vantadb-python",
    "vantadb_data",
    "tests_server_db",
    "tests_graph_db",
    "tests_vector_db",
    "tests_python_api",
    "datasets"
)

# Extensiones de archivos a excluir (binarios, imagenes, artefactos de compilacion)
$excludedExts = @(
    ".exe", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".woff", ".woff2", 
    ".ttf", ".eot", ".mp4", ".webm", ".zip", ".tar", ".gz", ".7z", 
    ".pdf", ".lock", ".rlib", ".rmeta", ".pdb", ".so", ".dll", ".dylib", 
    ".o", ".obj", ".bin", ".pyc", ".db", ".sqlite"
)

Write-Host "Iniciando generación de $outputFile..."
Write-Host "Excluyendo dependencias y archivos binarios..."

# Reiniciar el archivo de salida
Set-Content -Path $outputFile -Value "" -Encoding utf8

$basePath = (Get-Location).Path
$files = Get-ChildItem -File -Recurse | Where-Object {
    $relPath = $_.FullName.Substring($basePath.Length).TrimStart('\')
    
    $exclude = $false
    
    # Excluir directorios ignorados
    foreach ($dir in $excludedDirs) {
        if ($relPath -match "(^|\\)$dir(\\.*|$)") {
            $exclude = $true
            break
        }
    }
    
    # Excluir extensiones ignoradas
    if ($excludedExts -contains $_.Extension.ToLower()) {
        $exclude = $true
    }
    
    # Excluir el propio script y el archivo de salida
    if ($_.Name -eq $outputFile -or $_.Name -eq $scriptName) {
        $exclude = $true
    }
        
    -not $exclude
}

$totalFiles = ($files | Measure-Object).Count
$current = 0

foreach ($file in $files) {
    $current++
    $relPath = $file.FullName.Substring($basePath.Length).TrimStart('\')
    
    Write-Progress -Activity "Generando $outputFile" -Status "Procesando: $relPath" -PercentComplete (($current / $totalFiles) * 100)
    
    Add-Content -Path $outputFile -Value "" -Encoding utf8
    Add-Content -Path $outputFile -Value "================================================================" -Encoding utf8
    Add-Content -Path $outputFile -Value "Nombre: $($file.Name)" -Encoding utf8
    Add-Content -Path $outputFile -Value "Ruta: $relPath" -Encoding utf8
    Add-Content -Path $outputFile -Value "================================================================" -Encoding utf8
    Add-Content -Path $outputFile -Value "" -Encoding utf8
    
    try {
        $content = Get-Content $file.FullName -Raw -ErrorAction Stop
        Add-Content -Path $outputFile -Value $content -Encoding utf8
        Write-Host "[$current/$totalFiles] Agregado: $relPath"
    }
    catch {
        Add-Content -Path $outputFile -Value "[Contenido no legible o codificación binaria]" -Encoding utf8
        Write-Host "[$current/$totalFiles] Agregado (Error): $relPath" -ForegroundColor Yellow
    }
}

Write-Host "`n¡Hecho! El archivo '$outputFile' ha sido generado con el estado actual del proyecto (excluyendo dependencias)." -ForegroundColor Green
