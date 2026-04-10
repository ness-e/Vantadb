#!/bin/bash
set -e

echo "=== System Update and Dependencies ==="
apt-get update
# Install build tools, LLVM/Clang (for RocksDB), and Python
apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake clang libclang-dev \
    python3 python3-pip python3-venv

echo "=== Setting up Python Virtual Environment ==="
python3 -m venv /venv
source /venv/bin/activate

echo "=== Installing Python Test Dependencies ==="
pip install maturin pytest

echo "=== Building NexusDB Python SDK ==="
cd /app/nexusdb-python
# Compile the Rust code into a Python native module (.so)
# We use backtraces to diagnose any unexpected Python crashes
RUST_BACKTRACE=1 maturin develop --release

echo "=== Running Integration Tests ==="
# Execute the complete SDK lifecycle test suite
pytest -v tests/test_sdk.py
