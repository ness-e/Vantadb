#Requires -Version 5.1
<#
.SYNOPSIS
    Run the VantaDB audit nextest profile with Windows-safe build settings.
#>
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$env:RUST_MIN_STACK = "16777216"
cargo nextest run --profile audit --workspace --build-jobs 2
exit $LASTEXITCODE
