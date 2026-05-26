# VantaDB Smoke Test for Windows (PowerShell)
$ErrorActionPreference = "Stop"

$Env:VANTADB_HOST = "127.0.0.1"
$Env:VANTADB_PORT = "8080"
$Env:VANTADB_STORAGE_PATH = "vantadb_smoke_data"
$Env:RUST_LOG = "info"

$SERVER_PROCESS = $null

function Cleanup-Server {
    Write-Host "[cleanup] Stopping server..."
    if ($SERVER_PROCESS) {
        try {
            Stop-Process -Id $SERVER_PROCESS.Id -Force -ErrorAction SilentlyContinue
        } catch {}
    }
    if (Test-Path $Env:VANTADB_STORAGE_PATH) {
        try {
            Remove-Item -Recurse -Force $Env:VANTADB_STORAGE_PATH -ErrorAction SilentlyContinue
        } catch {}
    }
}

try {
    Write-Host "=== VantaDB Smoke Test (Windows) ==="

    # Clean old data
    if (Test-Path $Env:VANTADB_STORAGE_PATH) {
        Remove-Item -Recurse -Force $Env:VANTADB_STORAGE_PATH
    }

    # 1. Build & Start
    Write-Host "[1/7] Building & starting server..."
    cargo build --release --bin vanta-server

    $SERVER_PROCESS = Start-Process -FilePath "./target/release/vanta-server.exe" -PassThru -NoNewWindow
    Write-Host "       Server PID: $($SERVER_PROCESS.Id)"

    # Wait for server to boot
    Start-Sleep -Seconds 5

    # 2. Verify data directory
    Write-Host "[2/7] Verifying data directory..."
    if (Test-Path $Env:VANTADB_STORAGE_PATH) {
        Write-Host "       Data directory '$($Env:VANTADB_STORAGE_PATH)' exists."
    } else {
        throw "Data directory '$($Env:VANTADB_STORAGE_PATH)' not found!"
    }

    # 3. Health check
    Write-Host "[3/7] Verifying health endpoint..."
    $health = Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"
    if ($health.success -eq $true) {
        Write-Host "       Health check passed."
    } else {
        throw "Health check failed!"
    }

    # 4. Insert data
    Write-Host "[4/7] Inserting node..."
    $query1 = @{ query = '(INSERT :node {:content "smoke test content"})' }
    $body1 = $query1 | ConvertTo-Json
    $response1 = Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v2/query" -Method Post -Body $body1 -ContentType "application/json"
    if ($response1.success -eq $true) {
        Write-Host "       Insert passed."
    } else {
        throw "Insert failed!"
    }

    # 5. Second insert
    Write-Host "[5/7] Second insert..."
    $query2 = @{ query = '(INSERT :node {:content "second smoke entry"})' }
    $body2 = $query2 | ConvertTo-Json
    $response2 = Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v2/query" -Method Post -Body $body2 -ContentType "application/json"
    if ($response2.success -eq $true) {
        Write-Host "       Second insert passed."
    } else {
        throw "Second insert failed!"
    }

    # 6. Restart server
    Write-Host "[6/7] Restarting server..."
    Stop-Process -Id $SERVER_PROCESS.Id -Force
    Start-Sleep -Seconds 2

    $SERVER_PROCESS = Start-Process -FilePath "./target/release/vanta-server.exe" -PassThru -NoNewWindow
    Write-Host "       Restarted with PID: $($SERVER_PROCESS.Id)"
    Start-Sleep -Seconds 5

    # 7. Confirm post-restart
    Write-Host "[7/7] Confirming post-restart health..."
    $health2 = Invoke-RestMethod -Uri "http://127.0.0.1:8080/health"
    if ($health2.success -eq $true) {
        Write-Host "       Post-restart health passed."
    } else {
        throw "Post-restart health failed!"
    }

    Write-Host "=== Smoke Test PASSED ==="
}
catch {
    Write-Host "=== Smoke Test FAILED ==="
    Write-Host $_.Exception.Message
    exit 1
}
finally {
    Cleanup-Server
}
