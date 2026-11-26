# CLI-01 Build Check Script
# Run this to verify the CLI compiles correctly

Write-Host "=== VantaDB CLI Build Check ===" -ForegroundColor Cyan
Write-Host ""

Write-Host "Running cargo check for vanta-cli..." -ForegroundColor Yellow
$output = cargo check --features cli --bin vanta-cli 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✓ CLI compiles successfully!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "✗ Compilation failed. Errors:" -ForegroundColor Red
    Write-Host $output
}

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Cyan
