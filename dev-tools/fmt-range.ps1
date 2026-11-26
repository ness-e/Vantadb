# FIND-760 — fmt gate scoped to the files the push CARRIES, not the working tree.
# Called only from .githooks/pre-push (it needs a Windows-visible file list, and
# the hook already requires PowerShell). Why not `cargo fmt --check` on the tree:
# that blocked the push of FIND-152 (2 fmt-clean commits) because unformatted WIP
# from another lane was dirty in the working tree.
#
# Why not call rustfmt from the hook's shell: the shell git uses (Git Bash, WSL)
# does not necessarily have ~/.cargo/bin on PATH — measured here: `command -v
# rustfmt` → nothing under WSL bash while rustfmt exists at %USERPROFILE%\.cargo.
# PowerShell resolves it the same way verify.ps1 resolves cargo.
param([Parameter(Mandatory = $true)][string]$ListFile)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$cargoDir = Join-Path ([Environment]::GetFolderPath("UserProfile")) ".cargo\bin"
if ((Test-Path $cargoDir) -and ($env:PATH -notlike "*$cargoDir*")) { $env:PATH = "$cargoDir;$env:PATH" }

$rustfmt = (Get-Command "rustfmt" -ErrorAction SilentlyContinue).Source
if (-not $rustfmt) {
    # Fail closed: a format gate that cannot measure must not pass.
    Write-Host "fmt-range: ERROR — rustfmt not found (rustup component add rustfmt). Push BLOCKED." -ForegroundColor Red
    exit 2
}

$files = @(Get-Content -LiteralPath $ListFile | Where-Object { $_ -and (Test-Path -LiteralPath $_) })
if ($files.Count -eq 0) {
    Write-Host "fmt-range: no existing .rs files in the range — nothing to check." -ForegroundColor DarkYellow
    exit 0
}

Write-Host "fmt-range: $($files.Count) file(s) .rs del rango (no el arbol):"
$bad = 0
foreach ($f in $files) {
    Write-Host "  $f"
    # skip_children: measured, not assumed. rustfmt otherwise follows `mod`
    # declarations and reports diffs in child modules OUTSIDE the range (probe:
    # lib.rs with `pub mod child;` also reported child.rs; with the option, only
    # lib.rs).
    $out = & $rustfmt --check --edition 2021 --config skip_children=true $f 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "fmt-range: FAILED on range file: $f" -ForegroundColor Red
        $out | Select-Object -First 25 | ForEach-Object { Write-Host "    $_" }
        $bad++
    }
}

if ($bad -gt 0) {
    Write-Host "fmt-range: $bad file(s) need formatting. Fix: cargo fmt -p <crate>  (Push BLOCKED.)" -ForegroundColor Red
    exit 1
}
Write-Host "fmt-range: ok (range)." -ForegroundColor Green
exit 0
