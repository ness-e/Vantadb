$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$env:RUST_MIN_STACK = "16777216"
$pass = 0; $fail = 0

function run($name, [string[]]$cmd) {
    Write-Host "  ${name}..." -ForegroundColor Yellow -NoNewline
    $output = & $cmd[0] $cmd[1..($cmd.Length - 1)] 2>&1
    if ($LASTEXITCODE -eq 0) { Write-Host " ok" -ForegroundColor Green; $Script:pass++ }
    else { Write-Host " FAIL" -ForegroundColor Red; $Script:fail++; Write-Host $output -ForegroundColor Red; throw "step failed" }
}

try {
    $cg = Get-Command "codegraph" -ErrorAction SilentlyContinue
    $hasIndex = Test-Path "$ProjectRoot\.codegraph\codegraph.db"
    if ($cg -and $hasIndex) {
        $changed = git diff --name-only HEAD 2>$null
        if ($changed) {
            $affected = $changed | & "codegraph" affected --stdin --quiet 2>$null
            if ($affected) { Write-Host "  affected: $($affected.Count) files" -ForegroundColor Magenta }
        }
    }

    # Features del core gate: definición canónica compartida (gate-common.ps1) —
    # antes este script omitía roaring y divergía de verify.ps1
    . (Join-Path $PSScriptRoot "gate-common.ps1")
    $feats = Get-CoreFeatures
    run "fmt" ("cargo", "fmt", "--all", "--", "--check")
    run "check" (("cargo", "check", "-p", "vantadb") + $feats + @("-j", "2"))
    run "clippy" (("cargo", "clippy", "-p", "vantadb") + $feats + @("-j", "2", "--", "-D", "warnings"))

    # docs-coverage: solo si cambió código fuente/bindings/docs/api y el script existe (si no, skip silencioso)
    $docsScript = "$ProjectRoot\scripts\validate-docs-coverage.ps1"
    if (Test-Path $docsScript) {
        $docsDiff = git diff --name-only HEAD 2>$null
        $docsTouched = @($docsDiff | Where-Object { $_ -match '(^|/)(src/|[^/]*-python/|vantadb-ts/|vantadb-wasm/|docs/api/)' })
        if ($docsTouched.Count -gt 0) {
            run "docs-coverage" ("pwsh", "-NoProfile", "$docsScript")
        }
    }

    # agents-refs: anti-drift de rutas citadas en AGENTS.md (solo si AGENTS.md cambió)
    $agentsScript = "$ProjectRoot\dev-tools\check-agents-refs.ps1"
    if (Test-Path $agentsScript) {
        $agentsDiff = git diff --name-only HEAD 2>$null
        $agentsTouched = @($agentsDiff | Where-Object { $_ -match '(^|/)AGENTS\.md$' })
        if ($agentsTouched.Count -gt 0) {
            run "agents-refs" ("pwsh", "-NoProfile", "$agentsScript")
        }
    }

    Write-Host "ALL ${pass} PASS" -ForegroundColor Green; exit 0
} catch {
    if ($fail -eq 0) { $Script:fail = 1 }
    Write-Host "${fail} FAIL" -ForegroundColor Red; exit 1
}
