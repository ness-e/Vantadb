param (
    [int]$Iterations = 100,
    [switch]$Release,
    [string]$LogPath = "chaos_results.json"
)

# Salida estética de consola (Premium ANSI Colors)
$Esc = [char]27
$Cyan = "$Esc[96m"
$Green = "$Esc[92m"
$Yellow = "$Esc[93m"
$Red = "$Esc[91m"
$Reset = "$Esc[0m"
$Bold = "$Esc[1m"

Write-Host "`n${Bold}${Cyan}======================================================${Reset}"
Write-Host "${Bold}${Cyan}         VANTA DB - CHAOS LOOP TEST RUNNER            ${Reset}"
Write-Host "${Bold}${Cyan}======================================================${Reset}"
Write-Host "Configuración:"
Write-Host "  - Iteraciones: $Iterations"
Write-Host "  - Release Mode: $($Release.IsPresent)"
Write-Host "  - Log de resultados: $LogPath"

$releaseFlag = ""
if ($Release) {
    $releaseFlag = "--release"
}

Write-Host "`n${Yellow}Compilando suite de caos una sola vez...${Reset}"
$buildStart = Get-Date

# Compilar una sola vez y extraer la ruta exacta del ejecutable usando json format
$cargoCmd = "cargo test --test chaos_integrity --features failpoints $releaseFlag --no-run --message-format=json"
$cargoOutput = Invoke-Expression $cargoCmd -ErrorAction Stop

$testExe = ""
foreach ($line in $cargoOutput) {
    if ($line -match '\{.*\}') {
        try {
            $json = $line | ConvertFrom-Json
            if ($json.reason -eq "compiler-artifact" -and $json.target.name -eq "chaos_integrity") {
                $testExe = $json.executable
                break
            }
        } catch {}
    }
}

$buildEnd = Get-Date
$buildDuration = ($buildEnd - $buildStart).TotalSeconds

if (-not $testExe -or -not (Test-Path $testExe)) {
    Write-Host "${Red}CRITICAL: No se pudo localizar el binario de prueba compilado.${Reset}" -ForegroundColor Red
    exit 1
}

Write-Host "${Green}Compilación exitosa en $($buildDuration.ToString("F2"))s.${Reset}"
Write-Host "Ejecutable de prueba: $testExe`n"

$results = @()
$failedCount = 0
$passedCount = 0
$totalDuration = 0

Write-Host "${Yellow}Iniciando loop de ejecución de caos...${Reset}"

for ($i = 1; $i -le $Iterations; $i++) {
    $percent = [int](($i / $Iterations) * 100)
    Write-Progress -Activity "Ejecutando Caos" -Status "Iteración $i de $Iterations" -PercentComplete $percent
    
    $iterStart = Get-Date
    
    # Ejecutamos el ejecutable directamente
    # Para capturar la salida de consola de forma limpia, usamos Start-Process
    $proc = Start-Process -FilePath $testExe -ArgumentList "chaos_integrity_failpoints_certification", "--nocapture" -NoNewWindow -PassThru -Wait
    
    $iterEnd = Get-Date
    $duration = ($iterEnd - $iterStart).TotalMilliseconds
    $totalDuration += $duration
    
    $status = "PASS"
    $color = $Green
    if ($proc.ExitCode -ne 0) {
        $status = "FAIL"
        $color = $Red
        $failedCount++
    } else {
        $passedCount++
    }
    
    $results += @{
        iteration = $i
        status = $status
        exit_code = $proc.ExitCode
        duration_ms = [Math]::Round($duration, 2)
    }
    
    # Imprimir mini log en consola (micro-animación de avance)
    Write-Host "  [Iteración $($i.ToString().PadLeft(4))] Status: ${color}$status${Reset} | Duración: $($duration.ToString("F1")) ms"
}

# Finalizar la barra de progreso
Write-Progress -Activity "Ejecutando Caos" -Completed

$successRatio = ($passedCount / $Iterations) * 100
$avgDuration = $totalDuration / $Iterations

# Escribir reporte JSON
$report = @{
    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
    iterations_configured = $Iterations
    passed = $passedCount
    failed = $failedCount
    success_ratio_percentage = [Math]::Round($successRatio, 2)
    average_duration_ms = [Math]::Round($avgDuration, 2)
    total_duration_ms = [Math]::Round($totalDuration, 2)
    runs = $results
}

$report | ConvertTo-Json -Depth 5 | Out-File -FilePath $LogPath -Encoding utf8

Write-Host "`n${Bold}${Cyan}======================================================${Reset}"
Write-Host "${Bold}${Cyan}               RESUMEN DE CERTIFICACIÓN               ${Reset}"
Write-Host "${Bold}${Cyan}======================================================${Reset}"
Write-Host "Total Iteraciones : $Iterations"
Write-Host "Pases Exitosos    : ${Green}$passedCount${Reset}"
Write-Host "Fallos Detectados : $(if ($failedCount -gt 0) { "${Red}$failedCount${Reset}" } else { "${Green}0${Reset}" })"

if ($failedCount -eq 0) {
    Write-Host "Ratio de Éxito    : ${Bold}${Green}$($successRatio.ToString("F2"))%${Reset}"
    Write-Host "`n${Bold}${Green}🏆 CERTIFICACIÓN APROBADA: Cero fallos bajo caos inyectado.${Reset}`n"
    exit 0
} else {
    Write-Host "Ratio de Éxito    : ${Bold}${Red}$($successRatio.ToString("F2"))%${Reset}"
    Write-Host "`n${Bold}${Red}❌ CERTIFICACIÓN RECHAZADA: Se detectaron fallos de confiabilidad.${Reset}`n"
    exit 1
}
