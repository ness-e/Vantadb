# STRAT-04 slice — build the threads kernel for wasm32 with atomics + shared memory.
#
# Requirements (one-time):
#   rustup toolchain install nightly
#   rustup component add rust-src --toolchain nightly
#   rustup target add wasm32-unknown-unknown
#
# Output:
#   target/wasm32-unknown-unknown/release/vantadb_wasm_threads_kernel.wasm
#
# Consumed by benchmarks/wasm_threads_bench.mjs (Node harness).
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot
try {
  cargo +nightly build --target wasm32-unknown-unknown --release '-Zbuild-std=core'
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
  $wasm = Join-Path $PSScriptRoot "target/wasm32-unknown-unknown/release/vantadb_wasm_threads_kernel.wasm"
  if (-not (Test-Path $wasm)) { throw "wasm artifact not found: $wasm" }
  $size = (Get-Item $wasm).Length
  Write-Host "OK: $wasm ($size bytes)"
} finally {
  Pop-Location
}
