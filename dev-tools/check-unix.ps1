# Solo type-check del target: no enlaza ni ejecuta binarios Linux; Linux CI es el oráculo.
# Cargo sí compila y ejecuta build scripts para el host.
# fakecc permite que build scripts C terminen sin un compilador ni headers cruzados.
$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot
. (Join-Path $PSScriptRoot "gate-common.ps1")

function Find-LlvmTool([string]$name) {
    $command = Get-Command $name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($command) { return $command.Source }
    $fallback = Join-Path "${env:ProgramFiles}\LLVM\bin" "$name.exe"
    if (Test-Path $fallback -PathType Leaf) { return $fallback }
    return $null
}

$clang = Find-LlvmTool "clang"
$ar = Find-LlvmTool "llvm-ar"
if (-not $clang -or -not $ar) {
    Write-Host "check-unix: clang y llvm-ar son necesarios (PATH o Program Files\LLVM\bin)." -ForegroundColor Yellow
    exit 2
}

try {
    $dir = Join-Path $ProjectRoot "target/check-unix"
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    $source = Join-Path $dir "fakecc.rs"
    $compiler = Join-Path $dir "fakecc.exe"
    $empty = Join-Path $dir "empty.c"
    $rust = @'
use std::{env, ffi::OsString, process::{self, Command}};

fn main() {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.iter().any(|arg| arg == "-E") {
        return;
    }
    let mut output = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "-o" || arg == "-Fo" {
            output = args.next().cloned();
            break;
        }
        let text = arg.to_string_lossy();
        if let Some(value) = text.strip_prefix("-o").or_else(|| text.strip_prefix("-Fo")) {
            if !value.is_empty() {
                output = Some(OsString::from(value));
                break;
            }
        }
    }
    let Some(output) = output else {
        eprintln!("fakecc: falta el destino -o/-Fo");
        process::exit(1);
    };
    let status = Command::new(env::var_os("FAKECC_CLANG").expect("FAKECC_CLANG"))
        .arg("--target=x86_64-unknown-linux-gnu")
        .arg("-c")
        .arg(env::var_os("FAKECC_EMPTY").expect("FAKECC_EMPTY"))
        .arg("-o")
        .arg(output)
        .status()
        .expect("no se pudo ejecutar clang");
    process::exit(status.code().unwrap_or(1));
}
'@
    $update = -not (Test-Path $source) -or ((Get-Content -Raw $source) -ne $rust)
    if ($update) { [IO.File]::WriteAllText($source, $rust) }
    if (-not (Test-Path $empty)) { [IO.File]::WriteAllText($empty, "") }
    if ($update -or -not (Test-Path $compiler) -or (Get-Item $source).LastWriteTimeUtc -gt (Get-Item $compiler).LastWriteTimeUtc) {
        & rustc --edition 2021 $source -o $compiler
        if ($LASTEXITCODE -ne 0) { throw "No se pudo compilar fakecc (exit $LASTEXITCODE)." }
    }

    $core = @(Get-CoreFeatures)
    $encrypted = @($core)
    $encrypted[$encrypted.Count - 1] += ",encryption"
    $jobs = Get-AdaptiveJobs
    $cargo = (Get-Command cargo -CommandType Application | Select-Object -First 1).Source
    foreach ($pass in @(@{ Name = "core"; Features = $core }, @{ Name = "encryption"; Features = $encrypted })) {
        Write-Host "check-unix: $($pass.Name) (jobs=$jobs)..."
        $info = [System.Diagnostics.ProcessStartInfo]::new()
        $info.FileName = $cargo
        $info.WorkingDirectory = $ProjectRoot
        $info.UseShellExecute = $false
        foreach ($arg in (@("check", "--locked", "--target", "x86_64-unknown-linux-gnu", "-p", "vantadb", "--tests") + $pass.Features + @("--jobs", "$jobs"))) {
            $info.ArgumentList.Add($arg)
        }
        $info.Environment["CC_x86_64_unknown_linux_gnu"] = $compiler
        $info.Environment["CXX_x86_64_unknown_linux_gnu"] = $compiler
        $info.Environment["AR_x86_64_unknown_linux_gnu"] = $ar
        $info.Environment["FAKECC_CLANG"] = $clang
        $info.Environment["FAKECC_EMPTY"] = $empty
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $process = [Diagnostics.Process]::Start($info)
        $process.WaitForExit()
        $watch.Stop()
        Write-Host ("check-unix: {0} exit={1}, {2:N1}s" -f $pass.Name, $process.ExitCode, $watch.Elapsed.TotalSeconds)
        if ($process.ExitCode -ne 0) { exit 1 }
    }
    exit 0
} catch {
    Write-Error "check-unix: $_" -ErrorAction Continue
    exit 1
}
