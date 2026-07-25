#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Switch MCP profiles for VantaDB OpenCode.
.DESCRIPTION
    Changes which MCP servers are enabled in opencode.jsonc based on profile.
    Restart OpenCode after switching for changes to take effect.
.PARAMETER Profile
    Profile name: core, design, full
.PARAMETER Status
    Show current MCP status
.PARAMETER List
    List available profiles
.EXAMPLE
    .antigravity/mcp-profiles/switch-profile.ps1 -Profile core
    .antigravity/mcp-profiles/switch-profile.ps1 -Status
#>

param(
    [string]$Profile,
    [switch]$Status,
    [switch]$List
)

$ConfigPath = Join-Path $PSScriptRoot "..\.." "opencode.jsonc"
$ProfilesDir = $PSScriptRoot

# Resolve absolute paths
$ConfigPath = Resolve-Path $ConfigPath -ErrorAction Stop

if ($List) {
    Write-Host "Perfiles disponibles:" -ForegroundColor Cyan
    Get-ChildItem "$ProfilesDir\*.json" | ForEach-Object {
        $name = $_.BaseName
        $content = Get-Content $_ | ConvertFrom-Json
        $props = @($content.mcp.PSObject.Properties)
        $total = $props.Count
        $enabled = ($props | Where-Object { $_.Value.enabled }).Count
        Write-Host "  $name  ($enabled/$total MCPs activos)" -ForegroundColor Yellow
    }
    return
}

if ($Status) {
    Write-Host "Estado actual de MCPs:" -ForegroundColor Cyan
    $config = Get-Content $ConfigPath -Raw | ConvertFrom-Json
    $config.mcp.PSObject.Properties | Sort-Object Name | ForEach-Object {
        if ($_.Value.enabled) {
            Write-Host "  [ON]  $($_.Name)" -ForegroundColor Green
        } else {
            Write-Host "  [OFF] $($_.Name)" -ForegroundColor DarkGray
        }
    }
    return
}

if (-not $Profile) {
    Write-Host "Uso: switch-profile.ps1 -Profile <nombre> | -Status | -List" -ForegroundColor Yellow
    Write-Host "Perfiles: core, design, full" -ForegroundColor Gray
    exit 1
}

$ProfilePath = Join-Path $ProfilesDir "$Profile.json"
if (-not (Test-Path $ProfilePath)) {
    Write-Host "Perfil '$Profile' no encontrado." -ForegroundColor Red
    Write-Host "Disponibles: " -NoNewline
    Get-ChildItem "$ProfilesDir\*.json" | ForEach-Object { Write-Host "$($_.BaseName) " -NoNewline -ForegroundColor Yellow }
    Write-Host ""
    exit 1
}

# Read current config
$config = Get-Content $ConfigPath -Raw | ConvertFrom-Json

# Read profile
$profileConfig = Get-Content $ProfilePath | ConvertFrom-Json

# Apply profile settings
$changed = @()
foreach ($mcpName in $profileConfig.mcp.PSObject.Properties.Name) {
    $targetState = $profileConfig.mcp.$mcpName.enabled
    if ($config.mcp.$mcpName -and $config.mcp.$mcpName.enabled -ne $targetState) {
        $config.mcp.$mcpName.enabled = $targetState
        $changed += $mcpName
    }
}

# Write back with pretty formatting
$json = $config | ConvertTo-Json -Depth 10
Set-Content $ConfigPath -Value $json -Encoding UTF8

if ($changed.Count -eq 0) {
    Write-Host "Perfil '$Profile' ya está activo. Sin cambios." -ForegroundColor Gray
} else {
    Write-Host "Perfil '$Profile' aplicado. MCPs modificados:" -ForegroundColor Green
    foreach ($mcp in $changed) {
        if ($profileConfig.mcp.$mcp.enabled) {
            Write-Host "  $mcp -> [ON] habilitado" -ForegroundColor Yellow
        } else {
            Write-Host "  $mcp -> [OFF] deshabilitado" -ForegroundColor Yellow
        }
    }
    Write-Host ""
    Write-Host "Reinicie Antigravity (o recargue MCPs) para que los cambios surtan efecto." -ForegroundColor Cyan
}
