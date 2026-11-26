# FIND-760 - fmt gate scoped to the files the push CARRIES, not the working tree.
# Called only from .githooks/pre-push (it needs a Windows-visible file list, and
# the hook already requires PowerShell). Why not `cargo fmt --check` on the tree:
# that blocked the push of FIND-152 (2 fmt-clean commits) because unformatted WIP
# from another lane was dirty in the working tree.
#
# Why not call rustfmt from the hook's shell: the shell git uses (Git Bash, WSL)
# does not necessarily have ~/.cargo/bin on PATH - measured here: `command -v
# rustfmt` -> nothing under WSL bash while rustfmt exists at %USERPROFILE%\.cargo.
# PowerShell resolves it the same way verify.ps1 resolves cargo.
#
# FIND-762 - WHICH rustfmt measures is now stated, and it is CI's, not the PATH's.
# Before this, the gate used whatever `rustfmt` the PATH resolved (the repo pin,
# 1.95.0) while the CI `fmt` job measures with the toolchain its rust-setup step
# declares (`nightly`). Two instruments, one green: a push that passed here could
# fail in CI and nothing said so. The rule is the same one FIND-736/FIND-731
# closed elsewhere - measure with the instrument of the contract, or say you are
# not. So: read the toolchain from the workflow (never hardcode it), measure with
# it when installed, and ALWAYS name the instrument in the output. When it is not
# installed the gate falls back to the PATH rustfmt and says so as ADVISORY - it
# never blocks: "you don't have nightly" is not a property of the commit, and
# blocking would freeze every lane (FIND-762 pre-mortem a).
param(
    [Parameter(Mandatory = $true)][string]$ListFile,
    # Probe/override hook: measure with this rustup toolchain instead of the one
    # the CI `fmt` job declares. Empty = read it from ci-rust.yml (the default).
    [string]$Toolchain = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$cargoDir = Join-Path ([Environment]::GetFolderPath("UserProfile")) ".cargo\bin"
if ((Test-Path $cargoDir) -and ($env:PATH -notlike "*$cargoDir*")) { $env:PATH = "$cargoDir;$env:PATH" }

$rustfmt = (Get-Command "rustfmt" -ErrorAction SilentlyContinue).Source
if (-not $rustfmt) {
    # Fail closed: a format gate that cannot measure must not pass.
    Write-Host "fmt-range: ERROR - rustfmt not found (rustup component add rustfmt). Push BLOCKED." -ForegroundColor Red
    exit 2
}

# The toolchain the CI `fmt` job measures with. Read from the workflow so the two
# gates cannot drift by a hardcoded literal (FIND-762): a job key sits at exactly
# two spaces of indent, its rust-setup step declares `toolchain:` further in.
# Returns $null when neither is found - the caller then says so instead of
# pretending the instruments match.
# Ceiling (review L1, measured): this takes the FIRST `toolchain:` line inside the
# `fmt` job, not specifically the one under `uses: rust-setup`. Probed with
# fixtures - it cannot bleed into another job (B/D -> $null), but a `run:` block
# that merely printed `toolchain:` before the real one would win (C). Not
# reachable today: the `fmt` job is one checkout + one rust-setup + one run.
function Get-CiFmtToolchain {
    $wf = Join-Path $ProjectRoot ".github\workflows\ci-rust.yml"
    if (-not (Test-Path -LiteralPath $wf)) { return $null }
    $inFmtJob = $false
    foreach ($line in (Get-Content -LiteralPath $wf)) {
        if ($line -match '^  ([A-Za-z0-9_-]+):\s*$') { $inFmtJob = ($Matches[1] -eq 'fmt'); continue }
        if ($inFmtJob -and $line -match '^\s+toolchain:\s*[''"]?([A-Za-z0-9._-]+)') { return $Matches[1] }
    }
    return $null
}

# The version line, and only that line. A rustup shim on the PATH can emit
# progress on the same stream ("syncing channel updates ... auto-installed"), and
# dumping it into the instrument line makes the one thing that must stay readable
# unreadable (review L4).
function Get-RustfmtVersion([string]$exe) {
    $v = @(& $exe --version 2>&1 | Where-Object { "$_" -match '^rustfmt ' }) | Select-Object -First 1
    if (-not $v) { return "(version unavailable)" }
    return "$v".Trim()
}

$want = if ($Toolchain) { $Toolchain } else { Get-CiFmtToolchain }
$fmtExe = $rustfmt
$measured = Get-RustfmtVersion $rustfmt
# rustfmt may come from a standalone install with no rustup at all (distro
# package, vendored toolchain). Guarding here keeps the new resolution from
# turning "rustup missing" into an unhandled command-not-found that would block
# the push with an unrelated message.
$rustup = (Get-Command "rustup" -ErrorAction SilentlyContinue).Source
if ($want -and -not $rustup) {
    Write-Host "fmt-range: ADVISORY - CI measures fmt with '$want' (ci-rust.yml 'fmt' job) and rustup is not installed here, so it cannot be resolved." -ForegroundColor Yellow
    Write-Host "fmt-range: ADVISORY - measuring with the PATH instrument instead: $measured" -ForegroundColor Yellow
}
if ($want -and $rustup) {
    # Check the INSTALLED list first, and never let rustup resolve a missing one:
    # measured - `rustup which --toolchain 1.50.0 rustfmt` on an absent toolchain
    # *installs it* ("syncing channel updates ... downloading 6 components") and
    # hangs the push on a download instead of falling back in 200 ms. The list is
    # local, offline, and answers the only question that matters here.
    $installed = @(& rustup toolchain list 2>$null | ForEach-Object { ($_ -split '\s+')[0] })
    # StartsWith, not -like: a `$want` coming from -Toolchain could carry wildcard
    # characters, and -like would then match some other installed toolchain.
    $have = @($installed | Where-Object { $_ -eq $want -or $_.StartsWith("$want-") }) | Select-Object -First 1
    if ($have) {
        # `rustup which` on an ALREADY installed toolchain is local and fast, and
        # resolves the binary once - invoking `rustup run` per file would pay the
        # shim for every file in the range, for nothing.
        $resolved = ((& rustup which --toolchain $have rustfmt 2>&1) -join "").Trim()
    } else {
        $resolved = ""
    }
    if ($resolved -and (Test-Path -LiteralPath $resolved)) {
        $fmtExe = $resolved
        $measured = Get-RustfmtVersion $fmtExe
        Write-Host "fmt-range: instrument = $measured (the CI 'fmt' job's toolchain: $want)" -ForegroundColor Cyan
    } else {
        Write-Host "fmt-range: ADVISORY - CI measures fmt with '$want' (ci-rust.yml 'fmt' job) but it is not installed here." -ForegroundColor Yellow
        Write-Host "fmt-range: ADVISORY - measuring with the PATH instrument instead: $measured" -ForegroundColor Yellow
        Write-Host "fmt-range: ADVISORY - a green below does NOT prove CI agrees. Close the gap: rustup toolchain install $want" -ForegroundColor Yellow
    }
} elseif (-not $want) {
    Write-Host "fmt-range: ADVISORY - could not read the CI 'fmt' job's toolchain from .github/workflows/ci-rust.yml;" -ForegroundColor Yellow
    Write-Host "fmt-range: ADVISORY - measuring with the PATH instrument: $measured" -ForegroundColor Yellow
}
if ($want -eq 'nightly' -and $rustup) {
    # The residual gap, named instead of hidden: `toolchain: nightly` in CI is not
    # a pin, so the runner installs the newest nightly on every run while a local
    # nightly ages. Same channel is not the same binary.
    Write-Host "fmt-range: note - CI's 'nightly' is not pinned (it installs the newest nightly per run). If CI's fmt job disagrees with this green: rustup update nightly" -ForegroundColor DarkGray
}

$files = @(Get-Content -LiteralPath $ListFile -Encoding utf8 | Where-Object { $_ -and (Test-Path -LiteralPath $_) })
if ($files.Count -eq 0) {
    Write-Host "fmt-range: no existing .rs files in the range - nothing to check." -ForegroundColor DarkYellow
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
    $out = & $fmtExe --check --edition 2021 --config skip_children=true $f 2>&1
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
Write-Host "fmt-range: ok (range) - measured with $measured." -ForegroundColor Green
exit 0
