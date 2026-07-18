<#
.SYNOPSIS
    VantaDB Model Proxy — PowerShell wrapper for the campaign executor.

.DESCRIPTION
    Manages the Node.js model proxy lifecycle. Designed to be called by
    .opencode/task-system/harness/harness-executor.ps1 or used directly for campaign tasks.

    Backend providers:
      ds  — DeepSeek V4 (direct, cheapest)
      or  — OpenRouter (multi-provider)
      fw  — Fireworks AI (fastest)
      anthropic — Normal Anthropic (pass-through, no proxy)

    Model mapping (Anthropic → VantaDB):
      claude-opus-4-6/7     → deepseek-v4-pro
      claude-sonnet-4-6     → deepseek-v4-flash
      claude-haiku-4-5      → deepseek-v4-flash

.FUNCTIONS
    Start-VantaProxy [-Backend] [-Port]
    Stop-VantaProxy
    Switch-VantaBackend [-Port] [-Backend]

.EXAMPLE
    $proxy = Start-VantaProxy -Backend ds
    Switch-VantaBackend -Port $proxy.port -Backend or
    Stop-VantaProxy
#>

$ProxyDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ModelProxyJs = Join-Path $ProxyDir "model-proxy.js"
$StartProxyJs = Join-Path $ProxyDir "start-proxy.js"
$TempPortFile = "$env:TEMP\vantadb-proxy-port.txt"

$Providers = @{
    ds = @{
        name = "DeepSeek V4 (direct)"
        url = "https://api.deepseek.com/anthropic"
        keyEnv = "DEEPSEEK_API_KEY"
        opus = "deepseek-v4-pro"
        sonnet = "deepseek-v4-pro"
        haiku = "deepseek-v4-flash"
        subagent = "deepseek-v4-flash"
    }
    or = @{
        name = "OpenRouter"
        url = "https://openrouter.ai/api"
        keyEnv = "OPENROUTER_API_KEY"
        opus = "deepseek/deepseek-v4-pro"
        sonnet = "deepseek/deepseek-v4-pro"
        haiku = "deepseek/deepseek-v4-pro"
        subagent = "deepseek/deepseek-v4-pro"
    }
    fw = @{
        name = "Fireworks AI"
        url = "https://api.fireworks.ai/inference"
        keyEnv = "FIREWORKS_API_KEY"
        opus = "accounts/fireworks/models/deepseek-v4-pro"
        sonnet = "accounts/fireworks/models/deepseek-v4-pro"
        haiku = "accounts/fireworks/models/deepseek-v4-pro"
        subagent = "accounts/fireworks/models/deepseek-v4-pro"
    }
}

function Get-ProviderKey {
    param([string]$KeyEnv)
    $v = $env:$KeyEnv
    if (-not $v) { $v = [Environment]::GetEnvironmentVariable($KeyEnv, "User") }
    return $v
}

function Start-VantaProxy {
    <#
    .SYNOPSIS
        Launch the VantaDB model proxy.
    .PARAMETER Backend
        Backend to use: ds (default), or, fw, anthropic.
    .PARAMETER Port
        Starting port (default: 3200, auto-increments if busy).
    .OUTPUTS
        Hashtable with port, process, and provider info.
    #>
    param(
        [Alias("b")]
        [string]$Backend = "ds",
        [int]$Port = 3200
    )

    if ($Backend -eq "anthropic") {
        Write-Host "[VANTADB-PROXY] Direct Anthropic mode (no proxy needed)" -ForegroundColor Cyan
        return @{ port = $null; process = $null; mode = "anthropic" }
    }

    $p = $Providers[$Backend]
    if (-not $p) { Write-Host "[VANTADB-PROXY] ERROR: Unknown backend '$Backend'. Valid: ds, or, fw, anthropic" -ForegroundColor Red; return $null }

    $key = Get-ProviderKey $p.keyEnv
    if (-not $key) { Write-Host "[VANTADB-PROXY] ERROR: $($p.keyEnv) not set" -ForegroundColor Red; return $null }

    Write-Host "[VANTADB-PROXY] Starting proxy → $($p.name) on port $Port" -ForegroundColor Cyan

    # Collect all available backend keys for multi-backend support
    $envVars = @()
    foreach ($id in @("ds","or","fw")) {
        $k = Get-ProviderKey $Providers[$id].keyEnv
        if ($k) {
            $envVars += "`$$env:$($Providers[$id].keyEnv)='$k'"
            Set-Item -Path "Env:$($Providers[$id].keyEnv)" -Value $k -ErrorAction SilentlyContinue
        }
    }

    $proxyProc = Start-Process -FilePath "node" -ArgumentList $StartProxyJs, $p.url, $key -PassThru -WindowStyle Hidden -RedirectStandardOutput $TempPortFile

    $tries = 0
    $proxyPort = $null
    while ($tries -lt 30) {
        Start-Sleep -Milliseconds 200
        $tries++
        if (Test-Path $TempPortFile) {
            $content = Get-Content $TempPortFile -ErrorAction SilentlyContinue
            if ($content) { $proxyPort = $content | Select-Object -First 1; break }
        }
    }

    if (-not $proxyPort) {
        Write-Host "[VANTADB-PROXY] ERROR: Proxy failed to start" -ForegroundColor Red
        if ($proxyProc -and -not $proxyProc.HasExited) { Stop-Process -Id $proxyProc.Id -Force }
        return $null
    }

    Write-Host "[VANTADB-PROXY] Proxy running on 127.0.0.1:$proxyPort → $($p.name)" -ForegroundColor Green

    return @{
        port = [int]$proxyPort
        process = $proxyProc
        mode = $Backend
        provider = $p
    }
}

function Stop-VantaProxy {
    <#
    .SYNOPSIS
        Stop the VantaDB model proxy process.
    .PARAMETER Process
        Process object returned by Start-VantaProxy.
    #>
    param(
        [Parameter(Mandatory)]
        $Process
    )

    if (-not $Process) { return }
    if ($Process.process -and -not $Process.process.HasExited) {
        Stop-Process -Id $Process.process.Id -Force -ErrorAction SilentlyContinue
        Write-Host "[VANTADB-PROXY] Proxy stopped (PID $($Process.process.Id))" -ForegroundColor DarkGray
    }
    if (Test-Path $TempPortFile) { Remove-Item $TempPortFile -Force -ErrorAction SilentlyContinue }
}

function Switch-VantaBackend {
    <#
    .SYNOPSIS
        Toggle the running proxy to a different backend mid-session.
    .PARAMETER Port
        Proxy port (from Start-VantaProxy output).
    .PARAMETER Backend
        Target backend: ds, or, fw, anthropic.
    #>
    param(
        [Parameter(Mandatory)]
        [int]$Port,

        [Parameter(Mandatory)]
        [Alias("b")]
        [string]$Backend
    )

    $backendMap = @{ ds = "deepseek"; or = "openrouter"; fw = "fireworks" }
    $realName = $backendMap[$Backend]
    if (-not $realName -and $Backend -ne "anthropic") {
        Write-Host "[VANTADB-PROXY] ERROR: Unknown backend '$Backend'" -ForegroundColor Red
        return $null
    }
    if ($Backend -eq "anthropic") { $realName = "anthropic" }

    try {
        $body = "backend=$realName"
        $resp = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/_proxy/mode" -Method POST -Body $body -ContentType "application/x-www-form-urlencoded" -TimeoutSec 5
        Write-Host "[VANTADB-PROXY] Switched: $($resp.previous) → $($resp.mode)" -ForegroundColor Yellow
        return $resp
    } catch {
        Write-Host "[VANTADB-PROXY] ERROR switching backend: $_" -ForegroundColor Red
        return $null
    }
}
