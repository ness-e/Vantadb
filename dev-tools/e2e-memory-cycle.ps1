# MEM-73 — explicit lane for the composed L0 -> L3 cycle in the shipped binary.
#
# Why a lane and not the default gate: the e2e spawns `vantadb-server --mcp`
# plus a scripted LLM endpoint. That is the class of test that poisons a
# default gate when it flakes (FIND-727/739), so it is `#[ignore]` and runs
# only here.
#
# FIND-741: the binary must name the current HEAD. The test itself asserts
# this (a stale artifact answers with the old contract shape and would pass
# without proving anything), so the version check below is a fast pre-flight,
# not the only guard.
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$env:RUST_MIN_STACK = "16777216"

Write-Host "[1/3] build vantadb-server" -ForegroundColor Yellow
cargo build -p vantadb-server --bin vantadb-server
if ($LASTEXITCODE -ne 0) { throw "build failed" }

$bin = Join-Path $ProjectRoot "target/debug/vantadb-server.exe"
if (-not (Test-Path $bin)) { $bin = Join-Path $ProjectRoot "target/debug/vantadb-server" }

$version = & $bin --version
$head = (git rev-parse --short HEAD).Trim()
Write-Host "  binary: $version"
Write-Host "  HEAD:   $head"
if ($version -notmatch [regex]::Escape($head)) {
    throw "stale binary: build it from HEAD ($head) before running the e2e (FIND-741)"
}
$status = git status --porcelain --untracked-files=no
if ($status) {
    # Not fatal: the commit is pinned and the test asserts the post-MCP-43/44
    # contract shape, which a binary predating the wave cannot produce.
    Write-Host "  NOTE: worktree dirty (uncommitted tracked changes at build time)" -ForegroundColor DarkYellow
}

Write-Host "[2/3] run the ignored e2e" -ForegroundColor Yellow
cargo nextest run --profile audit -p vantadb-mcp `
    -E 'test(e2e_l0)' --run-ignored ignored-only --build-jobs 2 --no-capture
if ($LASTEXITCODE -ne 0) { throw "MEM-73 e2e failed" }

Write-Host "[3/3] ok — L0 -> L1 -> L2 -> L3 observed in one real process" -ForegroundColor Green
