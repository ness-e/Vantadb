$outputFile = "todo.md"

# Cargar archivos ya procesados si todo.md existe
$alreadyProcessed = @{}
if (Test-Path $outputFile) {
    # Extraer las rutas previamente procesadas leyendo las lneas que empiezan con "Ruta: "
    # Select-String es mucho mas rapido e insensible a memoria que Get-Content directo
    Select-String -Path $outputFile -Pattern "^Ruta:\s*(.+)$" -ErrorAction SilentlyContinue | ForEach-Object {
        $ruta = $_.Matches[0].Groups[1].Value.Trim()
        $alreadyProcessed[$ruta] = $true
    }
    Write-Host "Reanudando... Se encontraron $($alreadyProcessed.Count) archivos ya presentes en todo.md"
}

$excludedDirs = @(".git", "connectome-web", "target", "connectome_snapshots", "tmp")
$excludedExts = @(".exe", ".png", ".jpg", ".rlib", ".rmeta", ".pdb", ".lock", ".pdf")
$scriptName = $MyInvocation.MyCommand.Name

Get-ChildItem -File -Recurse | Where-Object {
    $relPath = $_.FullName.Substring((Get-Location).Path.Length).TrimStart('\')
    
    $exclude = $false
    
    if ($alreadyProcessed.ContainsKey($relPath)) {
        $exclude = $true
    }
    
    # Excluir directorios ignorados
    foreach ($dir in $excludedDirs) {
        if ($relPath -match "(^|\\)$dir\\") {
            $exclude = $true
            break
        }
    }
    
    # Excluir extensiones ignoradas
    if ($excludedExts -contains $_.Extension) {
        $exclude = $true
    }
    
    # Excluir el script actual y el archivo todo.md
    if ($_.Name -eq $outputFile -or ($scriptName -and $_.Name -eq $scriptName)) {
        $exclude = $true
    }
        
    -not $exclude
} | ForEach-Object {
    $relPath = $_.FullName.Substring((Get-Location).Path.Length).TrimStart('\')
    
    Add-Content -Path $outputFile -Value ""
    Add-Content -Path $outputFile -Value "================================================================"
    Add-Content -Path $outputFile -Value "Nombre: $($_.Name)"
    Add-Content -Path $outputFile -Value "Ruta: $relPath"
    Add-Content -Path $outputFile -Value "================================================================"
    Add-Content -Path $outputFile -Value ""
    
    try {
        $content = Get-Content $_.FullName -ErrorAction Stop
        Add-Content -Path $outputFile -Value $content
        Write-Host "Agregado: $relPath"
    }
    catch {
        Add-Content -Path $outputFile -Value "[Contenido no leible o codificacion binaria]"
        Write-Host "Agregado (Error/Binario): $relPath"
    }
}

Write-Host "¡Hecho! El archivo '$outputFile' ha sido actualizado con los archivos faltantes."
