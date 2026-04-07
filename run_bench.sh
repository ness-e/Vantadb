#!/bin/bash
set -e

echo "Installing build prerequisites..."
apt-get update && apt-get install -y clang llvm cmake make g++ libsnappy-dev liblz4-dev libzstd-dev

echo "Compiling benchmark (no-run) unlimited memory..."
export CI=true
cargo bench --bench high_density --no-run

BENCH_BIN=$(ls -t target/release/deps/high_density-* | grep -v '\.d' | grep -v '\.pdb' | grep -v '\.rmeta' | head -n 1)

echo "Benchmark compiled successfully: $BENCH_BIN"

echo "Executing benchmark in strict 512m environment..."
export CONNECTOMEDB_MEMORY_LIMIT=536870912
$BENCH_BIN
echo "Benchmark completed successfully within 512m memory limit!"
